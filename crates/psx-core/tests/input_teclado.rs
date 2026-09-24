use psx_core::app::input_map::{Alvo, Entrada, LINHAS, Perfil, SOLTO, Sentido, Teclado};
use psx_core::pad_script::button_bit;

fn bit(nome: &str) -> u16 {
    1u16 << button_bit(nome).expect("botao conhecido")
}

fn apertado(palavra: u16, nome: &str) -> bool {
    palavra & bit(nome) == 0
}

fn botao(nome: &str) -> Alvo {
    Alvo::botao(nome).expect("botao conhecido")
}

const ESQ_CIMA: Alvo = Alvo::Analogico {
    direito: false,
    sentido: Sentido::Cima,
};

#[test]
fn teclado_padrao_e_o_layout_que_era_fixo_no_codigo() {
    let t = Teclado::padrao();
    let esperado = [
        ("up", "Up"),
        ("down", "Down"),
        ("left", "Left"),
        ("right", "Right"),
        ("cross", "Z"),
        ("circle", "Space"),
        ("square", "A"),
        ("triangle", "S"),
        ("start", "Enter"),
        ("select", "Tab"),
        ("l1", "D"),
        ("r1", "F"),
        ("l2", "E"),
        ("r2", "R"),
    ];
    for (nome, tecla) in esperado {
        assert_eq!(t.tecla_de(botao(nome)), Some(tecla), "{nome}");
    }
    assert_eq!(t.tecla_de(ESQ_CIMA), Some("I"));
    assert_eq!(t.tecla_de(Alvo::Analog), Some("F3"));
    assert!(t.conflitos().is_empty(), "padrao sem conflito");
}

#[test]
fn tecla_apertada_vira_bit_do_botao() {
    let t = Teclado::padrao();
    assert_eq!(t.palavra(&[]), SOLTO);
    let palavra = t.palavra(&["Z", "Enter"]);
    assert!(apertado(palavra, "cross"));
    assert!(apertado(palavra, "start"));
    assert_eq!(palavra, SOLTO & !bit("cross") & !bit("start"));
}

#[test]
fn teclas_do_analogico_viram_eixos_e_nao_botoes() {
    let t = Teclado::padrao();
    assert_eq!(t.palavra(&["I", "L"]), SOLTO);
    let e = t.eixos(&["I", "L"]);
    assert_eq!(e.esquerdo_y, -1.0, "cima = Y negativo na convencao do PS1");
    assert_eq!(e.esquerdo_x, 1.0);
    assert_eq!(e.direito_x, 0.0);
    let ambos = t.eixos(&["J", "L"]);
    assert_eq!(ambos.esquerdo_x, 0.0, "opostas se anulam");
}

#[test]
fn associar_troca_a_tecla_e_devolve_teclado_novo() {
    let antigo = Teclado::padrao();
    let novo = antigo.associa(botao("cross"), "X");
    assert_eq!(antigo.tecla_de(botao("cross")), Some("Z"));
    assert_eq!(novo.tecla_de(botao("cross")), Some("X"));
    assert!(apertado(novo.palavra(&["X"]), "cross"));
    assert_eq!(novo.palavra(&["Z"]), SOLTO, "a tecla antiga deixa de valer");
}

#[test]
fn analogico_direito_pode_ganhar_tecla() {
    let alvo = Alvo::Analogico {
        direito: true,
        sentido: Sentido::Esquerda,
    };
    let t = Teclado::padrao().associa(alvo, "Num4");
    assert_eq!(t.eixos(&["Num4"]).direito_x, -1.0);
}

#[test]
fn mesma_tecla_em_dois_botoes_e_conflito_e_aciona_os_dois() {
    let t = Teclado::padrao().associa(botao("cross"), "Space");
    assert_eq!(t.conflitos(), vec!["Space".to_string()]);
    let palavra = t.palavra(&["Space"]);
    assert!(apertado(palavra, "cross") && apertado(palavra, "circle"));
    assert_eq!(t.alvos_da("Space").len(), 2);
}

#[test]
fn limpar_tira_so_o_alvo_pedido() {
    let t = Teclado::padrao().limpa(botao("cross"));
    assert_eq!(t.tecla_de(botao("cross")), None);
    assert_eq!(t.tecla_de(botao("circle")), Some("Space"));
}

#[test]
fn linhas_da_tela_cobrem_todos_os_botoes_do_pad() {
    for nome in [
        "cross", "circle", "square", "triangle", "l1", "l2", "r1", "r2", "l3", "r3", "select",
        "start", "up", "down", "left", "right",
    ] {
        assert!(LINHAS.contains(&botao(nome)), "{nome} falta na tela");
    }
    assert!(LINHAS.contains(&Alvo::Analog));
    assert_eq!(
        LINHAS
            .iter()
            .filter(|a| matches!(a, Alvo::Analogico { .. }))
            .count(),
        8
    );
    assert_eq!(LINHAS[0], botao("cross"), "a tela comeca pelo ✖");
}

#[test]
fn alvo_vai_e_volta_pela_chave_de_texto() {
    for alvo in LINHAS {
        assert_eq!(Alvo::de_chave(&alvo.chave()), Some(alvo), "{alvo:?}");
    }
    assert_eq!(Alvo::de_chave("turbo"), None);
}

#[test]
fn rotulos_tem_simbolo_e_acento() {
    assert!(botao("cross").rotulo().contains('✖'));
    assert!(botao("triangle").rotulo().contains("Triângulo"));
    assert!(ESQ_CIMA.rotulo().contains("Analógico esquerdo"));
    assert!(Alvo::Analog.rotulo().contains("Analog"));
}

#[test]
fn teclado_vai_para_o_texto_junto_com_o_controle() {
    let p = Perfil::padrao().com_teclado(Teclado::padrao().associa(botao("cross"), "X"));
    let texto = p.para_texto();
    assert!(texto.contains("teclado.X = cross"), "texto foi:\n{texto}");
    assert!(
        texto.contains("sul = cross"),
        "controle continua no arquivo"
    );
    let volta = Perfil::de_texto("Do arquivo", &texto);
    assert_eq!(volta.teclado(), p.teclado());
    assert_eq!(volta.ligacoes().len(), p.ligacoes().len());
    assert_eq!(volta.analog(), p.analog());
}

#[test]
fn teclado_esvaziado_continua_vazio_depois_de_gravar() {
    let p = Perfil::padrao().com_teclado(Teclado::vazio());
    let volta = Perfil::de_texto("x", &p.para_texto());
    assert_eq!(volta.teclado(), &Teclado::vazio());
}

#[test]
fn arquivo_no_formato_antigo_ganha_teclado_padrao_e_analog_no_home() {
    let antigo = "dpad-cima = up\nsul = cross\nleste = circle\n";
    let p = Perfil::de_texto("Do arquivo", antigo);
    assert_eq!(p.teclado(), &Teclado::padrao());
    assert_eq!(p.analog(), Some(Entrada::Modo));
    assert_eq!(p.nome_do_botao(Entrada::Sul), Some("cross"));
}

#[test]
fn conflito_no_teclado_sobrevive_ao_texto() {
    let t = Teclado::padrao().associa(botao("cross"), "Space");
    let p = Perfil::padrao().com_teclado(t.clone());
    let volta = Perfil::de_texto("x", &p.para_texto());
    assert_eq!(volta.teclado().conflitos(), t.conflitos());
}
