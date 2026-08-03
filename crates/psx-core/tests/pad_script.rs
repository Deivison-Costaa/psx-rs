use psx_core::pad_script::{DEFAULT_PRESS_STEPS, PadScript, RELEASED};

fn script(specs: &[&str]) -> PadScript {
    let owned: Vec<String> = specs.iter().map(|s| s.to_string()).collect();
    PadScript::parse(&owned).expect("script valido")
}

#[test]
fn sem_aperto_nenhum_todos_os_bits_ficam_soltos() {
    let s = script(&[]);
    assert!(s.is_empty());
    assert_eq!(s.buttons_at(0), RELEASED);
    assert_eq!(s.buttons_at(u64::MAX), RELEASED);
}

#[test]
fn start_apertado_zera_o_bit_3_so_dentro_da_janela() {
    let s = script(&["start@1000:500"]);
    assert_eq!(
        s.buttons_at(999),
        RELEASED,
        "antes do passo o botao esta solto"
    );
    assert_eq!(
        s.buttons_at(1000),
        RELEASED & !(1 << 3),
        "no passo inicial ja vale"
    );
    assert_eq!(
        s.buttons_at(1499),
        RELEASED & !(1 << 3),
        "ultimo passo da janela"
    );
    assert_eq!(
        s.buttons_at(1500),
        RELEASED,
        "a janela e semiaberta: solta no fim"
    );
}

#[test]
fn duracao_omitida_usa_o_padrao() {
    let s = script(&["cross@10"]);
    assert_eq!(
        s.buttons_at(10 + DEFAULT_PRESS_STEPS - 1),
        RELEASED & !(1 << 14)
    );
    assert_eq!(s.buttons_at(10 + DEFAULT_PRESS_STEPS), RELEASED);
}

#[test]
fn apertos_sobrepostos_zeram_os_dois_bits() {
    let s = script(&["l1@100:200", "r1@150:200"]);
    assert_eq!(s.buttons_at(120), RELEASED & !(1 << 10));
    assert_eq!(s.buttons_at(160), RELEASED & !(1 << 10) & !(1 << 11));
    assert_eq!(s.buttons_at(320), RELEASED & !(1 << 11));
    assert_eq!(s.buttons_at(360), RELEASED);
}

// § Standard Controllers (L618) de docs/reference/10-controllers-memcards.md da a ordem dos
// dezesseis bits do halfword 1, com 0=Pressed.
#[test]
fn os_dezesseis_nomes_batem_com_a_ordem_de_bits_da_spec() {
    let esperado = [
        ("select", 0),
        ("l3", 1),
        ("r3", 2),
        ("start", 3),
        ("up", 4),
        ("right", 5),
        ("down", 6),
        ("left", 7),
        ("l2", 8),
        ("r2", 9),
        ("l1", 10),
        ("r1", 11),
        ("triangle", 12),
        ("circle", 13),
        ("cross", 14),
        ("square", 15),
    ];
    for (nome, bit) in esperado {
        let s = script(&[&format!("{nome}@0:1")]);
        assert_eq!(
            s.buttons_at(0),
            RELEASED & !(1u16 << bit),
            "{nome} deveria zerar o bit {bit}"
        );
    }
}

#[test]
fn o_nome_do_botao_nao_depende_de_caixa() {
    assert_eq!(
        script(&["START@0:1"]).buttons_at(0),
        script(&["start@0:1"]).buttons_at(0)
    );
}

#[test]
fn especificacao_malformada_e_erro_com_o_texto_ofensor() {
    for ruim in [
        "start",
        "start@",
        "@100",
        "start@abc",
        "botao@100",
        "start@100:0",
        "start@1:x",
    ] {
        let erro = PadScript::parse(&[ruim.to_string()]).expect_err(ruim);
        assert!(
            erro.contains(ruim),
            "a mensagem deve citar '{ruim}', veio '{erro}'"
        );
    }
}

#[test]
fn a_janela_de_um_aperto_nunca_e_vazia() {
    let s = script(&["down@7:1"]);
    assert_eq!(s.buttons_at(7), RELEASED & !(1 << 6));
    assert_eq!(s.buttons_at(8), RELEASED);
}
