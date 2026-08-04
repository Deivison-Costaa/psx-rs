use std::path::{Path, PathBuf};

use psx_core::app::library::Identidade;

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

    pub fn detalhe(&self) -> String {
        let rotulo = self.identidade.rotulo.as_deref().unwrap_or("-");
        format!(
            "{} · {} · {}",
            self.serial(),
            self.identidade.regiao.nome(),
            rotulo
        )
    }
}

fn titulo_do_arquivo(cue: &Path) -> String {
    cue.file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| cue.display().to_string())
}

/// Varre a pasta (um nivel) atras de `.cue`. Disco ilegivel entra na lista mesmo assim,
/// com identidade vazia: esconder o jogo faria o usuario procurar um bug onde ha um CUE
/// quebrado.
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
