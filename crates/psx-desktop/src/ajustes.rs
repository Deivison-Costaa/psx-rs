use std::path::{Path, PathBuf};

use psx_core::app::config::Config;

pub const ARQUIVO: &str = "psx-rs.toml";

/// Configuração ausente ou ilegível não é erro: o app abre no padrão e a tela de ajustes
/// grava o arquivo na primeira vez que o usuário mexer em algo.
pub fn carrega(caminho: &Path) -> (Config, Option<String>) {
    let Ok(texto) = std::fs::read_to_string(caminho) else {
        return (Config::default(), None);
    };
    match toml::from_str::<Config>(&texto) {
        Ok(c) => (c.ajustada(), None),
        Err(e) => (
            Config::default(),
            Some(format!("'{}' invalido: {e}", caminho.display())),
        ),
    }
}

pub fn grava(caminho: &Path, config: &Config) -> Result<(), String> {
    let texto = toml::to_string_pretty(config).map_err(|e| format!("montando TOML: {e}"))?;
    if let Some(pai) = caminho.parent() {
        if !pai.as_os_str().is_empty() {
            std::fs::create_dir_all(pai)
                .map_err(|e| format!("criando '{}': {e}", pai.display()))?;
        }
    }
    std::fs::write(caminho, texto).map_err(|e| format!("gravando '{}': {e}", caminho.display()))
}

pub fn caminho_padrao() -> PathBuf {
    PathBuf::from(ARQUIVO)
}
