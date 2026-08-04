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

#[cfg(test)]
mod testes {
    use super::*;

    fn exemplo() -> Config {
        Config {
            bios: "bios/SCPH1001.BIN".into(),
            escala: 3,
            volume: 42,
            audio_ligado: false,
            filtro_linear: true,
            slot_inicial: 5,
            ..Config::default()
        }
    }

    #[test]
    fn config_vira_toml_legivel_e_volta_igual() {
        let texto = toml::to_string_pretty(&exemplo()).expect("montar TOML");
        assert!(texto.contains("escala = 3"), "TOML foi:\n{texto}");
        assert!(texto.contains("audio_ligado = false"));
        let volta: Config = toml::from_str(&texto).expect("ler TOML");
        assert_eq!(volta, exemplo());
    }

    #[test]
    fn toml_escrito_a_mao_com_chaves_faltando_cai_no_padrao() {
        let volta: Config = toml::from_str("bios = \"b.bin\"\nvolume = 10\n").expect("ler TOML");
        assert_eq!(volta.bios, "b.bin");
        assert_eq!(volta.volume, 10);
        assert_eq!(volta.escala, Config::default().escala);
        assert_eq!(volta.pasta_de_cartoes, Config::default().pasta_de_cartoes);
    }

    #[test]
    fn arquivo_ausente_devolve_padrao_sem_erro() {
        let (c, erro) = carrega(Path::new("nao-existe-mesmo.toml"));
        assert_eq!(c, Config::default());
        assert!(erro.is_none(), "arquivo ausente nao e erro: {erro:?}");
    }

    #[test]
    fn toml_quebrado_devolve_padrao_e_avisa() {
        let caminho = std::env::temp_dir().join("psx-rs-config-quebrado.toml");
        std::fs::write(&caminho, "isto ] nao [ e toml").expect("escrever");
        let (c, erro) = carrega(&caminho);
        assert_eq!(c, Config::default());
        assert!(erro.is_some(), "TOML invalido tem de avisar");
        let _ = std::fs::remove_file(&caminho);
    }
}
