use std::path::{Path, PathBuf};

use psx_core::app::config::Config;
use psx_core::app::pastas::{Ambiente, Copia, Pastas, config_migrada, migracoes};

fn ambiente(home: &str) -> Ambiente {
    Ambiente {
        home: Some(home.to_string()),
        ..Ambiente::default()
    }
}

fn atual() -> PathBuf {
    PathBuf::from("/tmp/de/onde/abriu")
}

#[test]
fn sem_xdg_usa_config_e_local_share_da_home() {
    let p = Pastas::padrao(&ambiente("/home/ana"), &atual());
    assert_eq!(
        p.arquivo_de_config(),
        Path::new("/home/ana/.config/psx-rs/psx-rs.toml")
    );
    assert_eq!(
        p.cartoes_padrao(),
        Path::new("/home/ana/.local/share/psx-rs/cartoes")
    );
    assert_eq!(
        p.saves_padrao(),
        Path::new("/home/ana/.local/share/psx-rs/saves")
    );
    assert_eq!(
        p.perfil_de_controle(),
        Path::new("/home/ana/.config/psx-rs/controles.txt"),
        "o perfil de controle mora junto da config, nao na pasta de onde o app abriu"
    );
}

#[test]
fn xdg_absoluto_vence_a_home() {
    let amb = Ambiente {
        home: Some("/home/ana".into()),
        xdg_config_home: Some("/cfg".into()),
        xdg_data_home: Some("/dados".into()),
    };
    let p = Pastas::padrao(&amb, &atual());
    assert_eq!(p.arquivo_de_config(), Path::new("/cfg/psx-rs/psx-rs.toml"));
    assert_eq!(p.cartoes_padrao(), Path::new("/dados/psx-rs/cartoes"));
}

#[test]
fn xdg_relativo_ou_vazio_e_ignorado_como_manda_a_spec() {
    let amb = Ambiente {
        home: Some("/home/ana".into()),
        xdg_config_home: Some("relativo".into()),
        xdg_data_home: Some("  ".into()),
    };
    let p = Pastas::padrao(&amb, &atual());
    assert_eq!(
        p.arquivo_de_config(),
        Path::new("/home/ana/.config/psx-rs/psx-rs.toml")
    );
    assert_eq!(
        p.saves_padrao(),
        Path::new("/home/ana/.local/share/psx-rs/saves")
    );
}

#[test]
fn sem_home_nem_xdg_cai_na_pasta_atual() {
    let p = Pastas::padrao(&Ambiente::default(), &atual());
    assert_eq!(p.arquivo_de_config(), atual().join("psx-rs.toml"));
    assert_eq!(p.cartoes_padrao(), atual().join("cartoes"));
}

#[test]
fn config_explicita_sobrepoe_e_ancora_os_relativos_na_pasta_dela() {
    let p = Pastas::padrao(&ambiente("/home/ana"), &atual())
        .com_config_em(Path::new("outra/meu.toml"), &atual());
    assert_eq!(p.arquivo_de_config(), atual().join("outra/meu.toml"));
    assert_eq!(p.resolve("jogos"), atual().join("outra/jogos"));
    assert_eq!(
        p.cartoes_padrao(),
        Path::new("/home/ana/.local/share/psx-rs/cartoes"),
        "--config troca so a config; os dados continuam na pasta de dados"
    );
}

#[test]
fn relativo_da_config_e_relativo_a_pasta_da_config() {
    let p = Pastas::padrao(&ambiente("/home/ana"), &atual());
    assert_eq!(
        p.resolve("bios/SCPH1001.BIN"),
        Path::new("/home/ana/.config/psx-rs/bios/SCPH1001.BIN")
    );
    assert_eq!(p.resolve("/abs/x"), Path::new("/abs/x"));
    assert_eq!(p.resolve("~/roms"), Path::new("/home/ana/roms"));
}

#[test]
fn efetiva_resolve_todas_as_pastas_e_vazio_vira_padrao() {
    let p = Pastas::padrao(&ambiente("/home/ana"), &atual());
    let c = Config {
        bios: "bios.bin".into(),
        pasta_de_jogos: "/roms".into(),
        pasta_de_cartoes: String::new(),
        pasta_de_saves: "meus-saves".into(),
        ..Config::default()
    };
    let e = p.efetiva(&c);
    assert_eq!(e.bios, "/home/ana/.config/psx-rs/bios.bin");
    assert_eq!(e.pasta_de_jogos, "/roms");
    assert_eq!(e.pasta_de_cartoes, "/home/ana/.local/share/psx-rs/cartoes");
    assert_eq!(e.pasta_de_saves, "/home/ana/.config/psx-rs/meus-saves");
}

#[test]
fn bios_vazia_continua_vazia_para_a_validacao_avisar() {
    let p = Pastas::padrao(&ambiente("/home/ana"), &atual());
    assert_eq!(p.efetiva(&Config::default()).bios, "");
}

#[test]
fn pasta_de_jogos_vazia_vira_a_home() {
    let p = Pastas::padrao(&ambiente("/home/ana"), &atual());
    assert_eq!(p.efetiva(&Config::default()).pasta_de_jogos, "/home/ana");
}

#[test]
fn migra_o_que_existe_na_pasta_atual_e_falta_no_destino() {
    let p = Pastas::padrao(&ambiente("/home/ana"), &atual());
    let existe = |c: &Path| {
        c == atual().join("psx-rs.toml")
            || c == atual().join("cartoes")
            || c == atual().join("controles.txt")
            || c == Path::new("/home/ana/.local/share/psx-rs/saves")
            || c == atual().join("saves")
    };
    let copias = migracoes(&p, &atual(), existe);
    assert_eq!(
        copias,
        vec![
            Copia {
                origem: atual().join("psx-rs.toml"),
                destino: PathBuf::from("/home/ana/.config/psx-rs/psx-rs.toml"),
            },
            Copia {
                origem: atual().join("controles.txt"),
                destino: PathBuf::from("/home/ana/.config/psx-rs/controles.txt"),
            },
            Copia {
                origem: atual().join("cartoes"),
                destino: PathBuf::from("/home/ana/.local/share/psx-rs/cartoes"),
            },
        ],
        "saves/ ja existe no destino: nao pode ser sobrescrito"
    );
}

#[test]
fn nada_a_migrar_quando_a_pasta_atual_ja_e_o_destino() {
    let p = Pastas::padrao(&Ambiente::default(), &atual());
    assert!(migracoes(&p, &atual(), |_| true).is_empty());
}

#[test]
fn config_migrada_ancora_relativos_na_pasta_de_origem() {
    let velha = Config {
        bios: "bios/SCPH1001.BIN".into(),
        pasta_de_jogos: "roms".into(),
        pasta_de_cartoes: "cartoes".into(),
        pasta_de_saves: "/abs/saves".into(),
        ..Config::default()
    };
    let nova = config_migrada(&velha, &atual());
    assert_eq!(
        nova.bios,
        atual().join("bios/SCPH1001.BIN").to_string_lossy()
    );
    assert_eq!(nova.pasta_de_jogos, atual().join("roms").to_string_lossy());
    assert_eq!(
        nova.pasta_de_cartoes, "",
        "o 'cartoes' antigo foi copiado para a pasta de dados: vira o padrao"
    );
    assert_eq!(nova.pasta_de_saves, "/abs/saves");
}

#[test]
fn padrao_das_pastas_de_dados_e_vazio() {
    let c = Config::default();
    assert_eq!(c.pasta_de_cartoes, "");
    assert_eq!(c.pasta_de_saves, "");
    assert_eq!(c.pasta_de_jogos, "");
}
