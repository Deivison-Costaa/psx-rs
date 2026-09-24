use std::path::{Path, PathBuf};

use psx_core::app::config::Config;
use psx_core::app::pastas::{Ambiente, Pastas, config_migrada, migracoes};

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
            Some(format!("'{}' inválido: {e}", caminho.display())),
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

fn variavel(nome: &str) -> Option<String> {
    std::env::var(nome).ok()
}

pub fn pasta_atual() -> PathBuf {
    std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
}

#[cfg(not(windows))]
fn ambiente() -> Ambiente {
    Ambiente {
        home: variavel("HOME"),
        xdg_config_home: variavel("XDG_CONFIG_HOME"),
        xdg_data_home: variavel("XDG_DATA_HOME"),
    }
}

/// No Windows config e dados ficam juntos em `%APPDATA%\psx-rs`.
#[cfg(windows)]
fn ambiente() -> Ambiente {
    Ambiente {
        home: variavel("USERPROFILE"),
        xdg_config_home: variavel("APPDATA"),
        xdg_data_home: variavel("APPDATA"),
    }
}

/// Pastas XDG (`~/.config/psx-rs`, `~/.local/share/psx-rs`); `--config` troca so o arquivo.
pub fn pastas(config: Option<PathBuf>, atual: &Path) -> Pastas {
    let ambiente = ambiente();
    let padrao = Pastas::padrao(&ambiente, atual);
    match config {
        Some(arquivo) => padrao.com_config_em(&arquivo, atual),
        None => padrao,
    }
}

fn copia_arvore(origem: &Path, destino: &Path) -> std::io::Result<()> {
    if origem.is_dir() {
        std::fs::create_dir_all(destino)?;
        for entrada in std::fs::read_dir(origem)? {
            let entrada = entrada?;
            copia_arvore(&entrada.path(), &destino.join(entrada.file_name()))?;
        }
        return Ok(());
    }
    if let Some(pai) = destino.parent() {
        std::fs::create_dir_all(pai)?;
    }
    std::fs::copy(origem, destino).map(|_| ())
}

fn copia_config(origem: &Path, destino: &Path, pasta: &Path) -> Result<(), String> {
    let (config, erro) = carrega(origem);
    if let Some(e) = erro {
        return Err(e);
    }
    grava(destino, &config_migrada(&config, pasta))
}

/// Copia (nao move) o layout antigo da pasta atual para as pastas novas, uma vez: depois
/// da copia o destino existe e `migracoes` nao devolve mais nada.
pub fn migra(pastas: &Pastas, atual: &Path) -> Option<String> {
    let copias = migracoes(pastas, atual, Path::exists);
    if copias.is_empty() {
        return None;
    }
    let mut copiados = Vec::new();
    let mut falhas = Vec::new();
    let mut destinos: Vec<String> = Vec::new();
    for copia in copias {
        let resultado = if copia.destino == pastas.arquivo_de_config() {
            copia_config(&copia.origem, &copia.destino, atual)
        } else {
            copia_arvore(&copia.origem, &copia.destino).map_err(|e| e.to_string())
        };
        let nome = copia
            .origem
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();
        match resultado {
            Ok(()) => copiados.push(nome),
            Err(e) => falhas.push(format!("{nome}: {e}")),
        }
        let pasta = copia.destino.parent().map(|p| p.display().to_string());
        if let Some(p) = pasta.filter(|p| !destinos.contains(p)) {
            destinos.push(p);
        }
    }
    let mut aviso = format!(
        "Copiei {} da pasta atual para {} (os originais ficaram onde estavam).",
        copiados.join(", "),
        destinos.join(" e ")
    );
    if !falhas.is_empty() {
        aviso.push_str(&format!(" Falhou: {}", falhas.join("; ")));
    }
    Some(aviso)
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

    fn pasta_temporaria(tag: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("psx-rs-migra-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("cria pasta temporaria");
        dir
    }

    fn pastas_de_teste(raiz: &Path) -> Pastas {
        let ambiente = Ambiente {
            home: Some(raiz.join("casa").to_string_lossy().to_string()),
            ..Ambiente::default()
        };
        Pastas::padrao(&ambiente, &raiz.join("velha"))
    }

    #[test]
    fn migra_copia_config_cartoes_e_perfil_e_avisa_uma_vez() {
        let raiz = pasta_temporaria("copia");
        let velha = raiz.join("velha");
        std::fs::create_dir_all(velha.join("cartoes")).expect("cartoes");
        std::fs::write(velha.join("cartoes/SCUS-94900.mcd"), b"MC").expect("cartao");
        std::fs::write(velha.join("controles.txt"), "sul = cross\n").expect("perfil");
        std::fs::write(
            velha.join("psx-rs.toml"),
            "bios = \"b.bin\"\npasta_de_cartoes = \"cartoes\"\n",
        )
        .expect("config");
        let pastas = pastas_de_teste(&raiz);

        let aviso = migra(&pastas, &velha).expect("primeira vez avisa");
        assert!(aviso.contains("Copiei"), "{aviso}");
        assert!(pastas.cartoes_padrao().join("SCUS-94900.mcd").exists());
        assert!(pastas.perfil_de_controle().exists());
        assert!(
            velha.join("cartoes/SCUS-94900.mcd").exists(),
            "copia, nao move"
        );
        let (nova, _) = carrega(&pastas.arquivo_de_config());
        assert_eq!(nova.bios, velha.join("b.bin").to_string_lossy());
        assert_eq!(nova.pasta_de_cartoes, "");
        assert!(migra(&pastas, &velha).is_none(), "segunda vez nao repete");
        let _ = std::fs::remove_dir_all(&raiz);
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
