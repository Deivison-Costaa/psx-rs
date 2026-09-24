use std::path::{Path, PathBuf};

use psx_core::app::library::{self, Identidade};

use crate::disco;

#[derive(Debug, Clone)]
pub struct Jogo {
    pub cue: PathBuf,
    pub titulo: String,
    pub identidade: Identidade,
}

impl Jogo {
    pub fn serial(&self) -> &str {
        self.identidade.serial.as_deref().unwrap_or("sem serial")
    }

    pub fn detalhe(&self, segundos: u64) -> String {
        library::detalhe(
            self.identidade.regiao,
            self.identidade.serial.as_deref(),
            segundos,
        )
    }
}

fn titulo_do_arquivo(cue: &Path) -> String {
    cue.file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| cue.display().to_string())
}

/// Disco ilegivel entra na lista com identidade vazia: esconder o jogo faria o usuario
/// procurar um bug onde ha um CUE quebrado.
pub fn varre(pasta: &Path) -> Vec<Jogo> {
    let Ok(entradas) = std::fs::read_dir(pasta) else {
        return Vec::new();
    };
    let mut jogos: Vec<Jogo> = entradas
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|e| e.eq_ignore_ascii_case("cue")))
        .map(|cue| {
            let identidade = disco::identifica(&cue).unwrap_or_default();
            Jogo {
                titulo: titulo_do_arquivo(&cue),
                cue,
                identidade,
            }
        })
        .collect();
    jogos.sort_by_key(|j| j.titulo.to_lowercase());
    jogos
}
