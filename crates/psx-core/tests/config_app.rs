use psx_core::app::config::{Config, ESCALA_MAX, ESCALA_MIN, SLOT_MAX, VOLUME_MAX};

#[test]
fn padrao_e_utilizavel_sem_editar_nada() {
    let c = Config::default();
    assert_eq!(c.escala, 2);
    assert_eq!(c.volume, 100);
    assert!(c.audio_ligado);
    assert_eq!(c.slot_inicial, 0);
    assert_eq!(c.pasta_de_jogos, ".");
    assert_eq!(c.pasta_de_cartoes, "cartoes");
    assert_eq!(c.pasta_de_saves, "saves");
    assert!(c.bios.is_empty(), "BIOS nao tem palpite razoavel");
}

#[test]
fn padrao_ja_esta_ajustado() {
    assert_eq!(Config::default().ajustada(), Config::default());
}

#[test]
fn escala_fora_da_faixa_e_grampeada_nas_pontas() {
    let baixo = Config {
        escala: 0,
        ..Config::default()
    };
    let alto = Config {
        escala: 99,
        ..Config::default()
    };
    assert_eq!(baixo.ajustada().escala, ESCALA_MIN);
    assert_eq!(alto.ajustada().escala, ESCALA_MAX);
}

#[test]
fn volume_e_slot_fora_da_faixa_sao_grampeados() {
    let c = Config {
        volume: 250,
        slot_inicial: 42,
        ..Config::default()
    };
    let a = c.ajustada();
    assert_eq!(a.volume, VOLUME_MAX);
    assert_eq!(a.slot_inicial, SLOT_MAX);
}

#[test]
fn ajustar_devolve_copia_sem_mexer_na_original() {
    let original = Config {
        escala: 99,
        ..Config::default()
    };
    let ajustada = original.ajustada();
    assert_eq!(original.escala, 99, "a original nao pode ser alterada");
    assert_eq!(ajustada.escala, ESCALA_MAX);
}

#[test]
fn pasta_vazia_cai_no_padrao_em_vez_de_virar_raiz() {
    let c = Config {
        pasta_de_jogos: String::new(),
        pasta_de_cartoes: "  ".to_string(),
        pasta_de_saves: String::new(),
        ..Config::default()
    };
    let a = c.ajustada();
    assert_eq!(a.pasta_de_jogos, ".");
    assert_eq!(a.pasta_de_cartoes, "cartoes");
    assert_eq!(a.pasta_de_saves, "saves");
}

#[test]
fn valida_aponta_a_bios_faltando() {
    let problemas = Config::default().valida();
    assert_eq!(problemas.len(), 1, "problemas: {problemas:?}");
    assert!(problemas[0].to_lowercase().contains("bios"));
}

#[test]
fn valida_reclama_de_faixa_antes_de_grampear() {
    let c = Config {
        bios: "bios/SCPH1001.BIN".to_string(),
        escala: 0,
        volume: 200,
        ..Config::default()
    };
    let problemas = c.valida();
    assert_eq!(problemas.len(), 2, "problemas: {problemas:?}");
    assert!(problemas.iter().any(|p| p.contains("escala")));
    assert!(problemas.iter().any(|p| p.contains("volume")));
}

#[test]
fn config_boa_nao_tem_o_que_reclamar() {
    let c = Config {
        bios: "bios/SCPH1001.BIN".to_string(),
        ..Config::default()
    };
    assert!(c.valida().is_empty(), "{:?}", c.valida());
}

#[test]
fn volume_vira_ganho_linear_entre_zero_e_um() {
    let ligado = Config {
        volume: 50,
        ..Config::default()
    };
    assert!((ligado.ganho() - 0.5).abs() < 1e-6);
    assert!((Config::default().ganho() - 1.0).abs() < 1e-6);
}

#[test]
fn audio_desligado_zera_o_ganho_seja_qual_for_o_volume() {
    let c = Config {
        volume: 100,
        audio_ligado: false,
        ..Config::default()
    };
    assert_eq!(c.ganho(), 0.0);
}

#[test]
fn config_sobrevive_a_ida_e_volta_por_serializacao() {
    let c = Config {
        bios: "b.bin".to_string(),
        escala: 4,
        volume: 33,
        audio_ligado: false,
        slot_inicial: 7,
        ..Config::default()
    };
    let bytes = bincode::serde::encode_to_vec(&c, bincode::config::standard()).expect("gravar");
    let (volta, _): (Config, usize) =
        bincode::serde::decode_from_slice(&bytes, bincode::config::standard()).expect("ler");
    assert_eq!(volta, c);
}
