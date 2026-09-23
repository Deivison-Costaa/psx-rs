use std::cell::RefCell;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};

use psx_core::app::library::{self, Identidade};
use psx_core::cdrom_bin_cue::{DiscLayout, TrackType, parse_cue};
use psx_core::disc_image::{DiscImage, RAW_SECTOR_BYTES, RawSector};

fn bytes_por_setor(layout: &DiscLayout) -> u64 {
    match layout.tracks.first().map(|t| &t.track_type) {
        Some(TrackType::Mode1_2048) => 2048,
        _ => 2352,
    }
}

fn le_cue(cue: &Path) -> Result<DiscLayout, String> {
    let texto = std::fs::read_to_string(cue)
        .map_err(|e| format!("nao foi possivel ler o CUE '{}': {e}", cue.display()))?;
    let layout = parse_cue(&texto);
    if layout.arquivos_em_ordem().is_empty() {
        return Err(format!("CUE sem FILE: '{}'", cue.display()));
    }
    Ok(layout)
}

/// Abre a imagem sem copia-la para a memoria: cada setor e lido do arquivo quando o
/// drive pede.
pub fn carrega(cue: &Path) -> Result<(DiscLayout, Box<dyn DiscImage>), String> {
    let mut layout = le_cue(cue)?;
    let pasta = cue.parent().unwrap_or_else(|| Path::new("."));
    let imagem = DiscoEmArquivo::abre(pasta, &mut layout)?;
    Ok((layout, Box::new(imagem)))
}

#[derive(Debug)]
struct Faixa {
    arquivo: RefCell<File>,
    primeiro: u32,
    setores: u32,
}

/// Imagem lida do arquivo setor a setor: so o setor pedido passa pela memoria.
#[derive(Debug)]
pub struct DiscoEmArquivo {
    faixas: Vec<Faixa>,
}

impl DiscoEmArquivo {
    /// Abre os arquivos do CUE na ordem das trilhas e converte os INDEX 01 em LBA absoluto.
    pub fn abre(pasta: &Path, layout: &mut DiscLayout) -> Result<Self, String> {
        let mut faixas = Vec::new();
        let mut primeiro = 0u32;
        for nome in layout.arquivos_em_ordem() {
            let caminho = pasta.join(&nome);
            let arquivo = File::open(&caminho)
                .map_err(|e| format!("nao foi possivel abrir '{}': {e}", caminho.display()))?;
            let bytes = arquivo
                .metadata()
                .map_err(|e| format!("nao foi possivel medir '{}': {e}", caminho.display()))?
                .len();
            let setores = u32::try_from(bytes / RAW_SECTOR_BYTES as u64).unwrap_or(u32::MAX);
            faixas.push(Faixa {
                arquivo: RefCell::new(arquivo),
                primeiro,
                setores,
            });
            primeiro = primeiro.saturating_add(setores);
        }
        let setores: Vec<u32> = faixas.iter().map(|f| f.setores).collect();
        layout.atribui_lbas_absolutos(&setores);
        Ok(Self { faixas })
    }
}

impl DiscImage for DiscoEmArquivo {
    fn sector_count(&self) -> u32 {
        self.faixas.iter().map(|f| f.setores).sum()
    }

    fn read_sector(&self, lba: u32) -> Option<RawSector> {
        let faixa = self
            .faixas
            .iter()
            .find(|f| lba >= f.primeiro && lba - f.primeiro < f.setores)?;
        let deslocamento = u64::from(lba - faixa.primeiro) * RAW_SECTOR_BYTES as u64;
        let mut arquivo = faixa.arquivo.try_borrow_mut().ok()?;
        arquivo.seek(SeekFrom::Start(deslocamento)).ok()?;
        let mut setor = [0u8; RAW_SECTOR_BYTES];
        arquivo.read_exact(&mut setor).ok()?;
        Some(setor)
    }
}

/// Identifica o disco lendo so os setores que o ISO 9660 pede, por seek — varrer uma
/// biblioteca nao pode custar a leitura de centenas de MB por jogo.
pub fn identifica(cue: &Path) -> Result<Identidade, String> {
    let layout = le_cue(cue)?;
    let pasta = cue.parent().unwrap_or_else(|| Path::new("."));
    let primeiro = layout
        .arquivos_em_ordem()
        .first()
        .map(|a| pasta.join(a))
        .ok_or_else(|| "CUE sem trilha de dados".to_string())?;
    let mut arquivo = File::open(&primeiro)
        .map_err(|e| format!("nao foi possivel abrir '{}': {e}", primeiro.display()))?;
    let passo = bytes_por_setor(&layout);

    Ok(library::identifica(|lba| {
        let mut bruto = vec![0u8; passo as usize];
        arquivo.seek(SeekFrom::Start(u64::from(lba) * passo)).ok()?;
        arquivo.read_exact(&mut bruto).ok()?;
        library::dados_do_setor(&bruto).map(<[u8]>::to_vec)
    }))
}

fn e_cue(caminho: &Path) -> bool {
    caminho
        .extension()
        .is_some_and(|e| e.eq_ignore_ascii_case("cue"))
}

fn cues_ate(pasta: &Path, niveis: u8, fora: &mut Vec<PathBuf>) {
    let Ok(entradas) = std::fs::read_dir(pasta) else {
        return;
    };
    for caminho in entradas.filter_map(Result::ok).map(|e| e.path()) {
        if caminho.is_dir() {
            if niveis > 1 {
                cues_ate(&caminho, niveis - 1, fora);
            }
        } else if e_cue(&caminho) {
            fora.push(caminho.canonicalize().unwrap_or(caminho));
        }
    }
}

/// CUEs das pastas dadas e de uma subpasta abaixo: o rip comum poe cada disco na
/// propria pasta (`Jogo (Disc 2)/Jogo (Disc 2).cue`).
pub fn lista_cues(raizes: &[&Path]) -> Vec<PathBuf> {
    let mut fora = Vec::new();
    for raiz in raizes {
        cues_ate(raiz, 2, &mut fora);
    }
    fora.sort();
    fora.dedup();
    fora
}

#[cfg(test)]
mod tests {
    use super::*;
    use psx_core::cdrom_bin_cue::parse_cue;

    fn pasta(tag: &str) -> std::path::PathBuf {
        let dir =
            std::env::temp_dir().join(format!("psx-disco-desktop-{tag}-{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("cria pasta temporaria");
        dir
    }

    fn arquivo(dir: &Path, nome: &str, setores: u8, marca: u8) {
        let mut bytes = Vec::new();
        for i in 0..setores {
            bytes.extend(std::iter::repeat_n(marca.wrapping_add(i), RAW_SECTOR_BYTES));
        }
        std::fs::write(dir.join(nome), bytes).expect("grava trilha");
    }

    const CUE: &str = "FILE \"a.bin\" BINARY\n  TRACK 01 MODE2/2352\n    INDEX 01 00:00:00\n\
                       FILE \"b.bin\" BINARY\n  TRACK 02 AUDIO\n    INDEX 00 00:00:00\n    INDEX 01 00:00:01\n";

    #[test]
    fn le_setores_atravessando_arquivos_de_trilha() {
        let dir = pasta("multi");
        arquivo(&dir, "a.bin", 3, 0x10);
        arquivo(&dir, "b.bin", 2, 0x80);
        let mut layout = parse_cue(CUE);
        let disco = DiscoEmArquivo::abre(&dir, &mut layout).expect("abre imagem");
        assert_eq!(disco.sector_count(), 5);
        assert_eq!(layout.tracks[1].start_lba, 4);
        let lidos: Vec<Option<u8>> = (0..6).map(|l| disco.read_sector(l).map(|s| s[0])).collect();
        assert_eq!(
            lidos,
            vec![
                Some(0x10),
                Some(0x11),
                Some(0x12),
                Some(0x80),
                Some(0x81),
                None
            ]
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn arquivo_de_trilha_ausente_vira_erro() {
        let dir = pasta("ausente");
        let mut layout = parse_cue(CUE);
        assert!(DiscoEmArquivo::abre(&dir, &mut layout).is_err());
        let _ = std::fs::remove_dir_all(&dir);
    }
}
