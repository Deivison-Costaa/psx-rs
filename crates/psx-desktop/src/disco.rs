use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

use psx_core::app::library::{self, Identidade};
use psx_core::cdrom_bin_cue::{DiscLayout, TrackType, parse_cue};

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

/// Le a imagem inteira, na ordem das trilhas, e converte os INDEX 01 em LBA absoluto.
/// Rip por trilha tem um arquivo por trilha, cada um comecando no proprio LBA 0.
pub fn carrega(cue: &Path) -> Result<(DiscLayout, Vec<u8>), String> {
    let mut layout = le_cue(cue)?;
    let pasta = cue.parent().unwrap_or_else(|| Path::new("."));
    let mut setores = Vec::new();
    let mut bin = Vec::new();
    for arquivo in layout.arquivos_em_ordem() {
        let caminho = pasta.join(&arquivo);
        let dados = std::fs::read(&caminho)
            .map_err(|e| format!("nao foi possivel ler '{}': {e}", caminho.display()))?;
        setores.push((dados.len() / 2352) as u32);
        bin.extend_from_slice(&dados);
    }
    layout.atribui_lbas_absolutos(&setores);
    Ok((layout, bin))
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
