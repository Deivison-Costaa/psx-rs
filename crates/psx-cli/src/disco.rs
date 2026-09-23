use std::cell::RefCell;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

use psx_core::cdrom_bin_cue::DiscLayout;
use psx_core::disc_image::{DiscImage, RAW_SECTOR_BYTES, RawSector};

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

#[cfg(test)]
mod tests {
    use super::*;
    use psx_core::cdrom_bin_cue::parse_cue;

    fn pasta(tag: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("psx-disco-cli-{tag}-{}", std::process::id()));
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
