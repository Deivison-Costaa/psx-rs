use psx_core::app::input_map::{
    Alvo, Comando, Entrada, Estilo, Navegador, Perfil, REPETE_APOS, REPETE_CADA, SOLTO, Sentido,
    primeira_nova,
};
use psx_core::pad_script::button_bit;

fn botao(nome: &str) -> Alvo {
    Alvo::botao(nome).expect("botao conhecido")
}

fn apertado(palavra: u16, nome: &str) -> bool {
    palavra & (1u16 << button_bit(nome).expect("botao")) == 0
}

#[test]
fn entradas_de_um_alvo_listam_botao_e_eixo() {
    let p = Perfil::padrao();
    let cima = p.entradas_de(botao("up"));
    assert!(cima.contains(&Entrada::DpadCima));
    assert!(cima.contains(&Entrada::EixoPositivo(1)));
    assert_eq!(p.entradas_de(botao("cross")), vec![Entrada::Sul]);
    assert_eq!(p.entradas_de(Alvo::Analog), vec![Entrada::Modo]);
}

#[test]
fn associar_botao_do_controle_substitui_so_o_botao_do_mesmo_tipo() {
    let p = Perfil::padrao().associa_controle(botao("up"), Entrada::Norte);
    let cima = p.entradas_de(botao("up"));
    assert!(cima.contains(&Entrada::Norte));
    assert!(!cima.contains(&Entrada::DpadCima), "o botao antigo sai");
    assert!(cima.contains(&Entrada::EixoPositivo(1)), "o eixo fica");
    assert!(
        p.entradas_de(botao("triangle")).is_empty(),
        "Norte saiu do triangulo: uma entrada so aciona um botao"
    );
    assert!(apertado(p.palavra(&[Entrada::Norte]), "up"));
}

#[test]
fn associar_eixo_a_um_botao_funciona_no_modo_digital() {
    let p = Perfil::padrao().associa_controle(botao("l2"), Entrada::EixoPositivo(3));
    assert!(apertado(p.palavra(&[Entrada::EixoPositivo(3)]), "l2"));
    assert!(
        p.entradas_de(botao("l2")).contains(&Entrada::L2),
        "o L2 fisico e de outro tipo e continua"
    );
}

#[test]
fn analog_pode_ir_para_outro_botao_e_sai_do_botao_antigo() {
    let p = Perfil::padrao().associa_controle(Alvo::Analog, Entrada::Select);
    assert_eq!(p.analog(), Some(Entrada::Select));
    assert!(p.entradas_de(botao("select")).is_empty());
    assert_eq!(p.palavra(&[Entrada::Select]), SOLTO);
    let de_volta = p.associa_controle(botao("select"), Entrada::Select);
    assert_eq!(de_volta.analog(), None, "Select nao pode ser Analog e Select ao mesmo tempo");
}

#[test]
fn limpar_controle_tira_todas_as_entradas_do_alvo() {
    let p = Perfil::padrao().limpa_controle(botao("up"));
    assert!(p.entradas_de(botao("up")).is_empty());
    assert_eq!(Perfil::padrao().limpa_controle(Alvo::Analog).analog(), None);
}

#[test]
fn perfil_pronto_troca_o_controle_e_mantem_o_teclado() {
    let meu = Perfil::padrao().com_teclado(
        Perfil::padrao()
            .teclado()
            .associa(botao("cross"), "X"),
    );
    let trocado = meu.controle_de(&Perfil::faces_trocadas());
    assert_eq!(trocado.teclado(), meu.teclado());
    assert!(apertado(trocado.palavra(&[Entrada::Sul]), "circle"));
}

#[test]
fn captura_pega_so_o_que_foi_apertado_agora() {
    assert_eq!(primeira_nova(&[], &[]), None);
    assert_eq!(
        primeira_nova(&[Entrada::Sul], &[Entrada::Sul]),
        None,
        "segurado desde antes nao conta"
    );
    assert_eq!(
        primeira_nova(&[Entrada::Sul], &[Entrada::Sul, Entrada::L1]),
        Some(Entrada::L1)
    );
    assert_eq!(
        primeira_nova(&[], &[Entrada::EixoNegativo(0), Entrada::Start]),
        Some(Entrada::Start),
        "botao ganha de eixo: analogico solto oscila e roubaria a captura"
    );
    assert_eq!(
        primeira_nova(&[], &[Entrada::EixoNegativo(2)]),
        Some(Entrada::EixoNegativo(2))
    );
}

#[test]
fn estilo_do_controle_vem_do_nome_ou_do_fabricante() {
    assert_eq!(
        Estilo::detecta("Sony Interactive Entertainment Wireless Controller", None),
        Estilo::PlayStation
    );
    assert_eq!(Estilo::detecta("qualquer", Some(0x054c)), Estilo::PlayStation);
    assert_eq!(Estilo::detecta("Nintendo Switch Pro Controller", None), Estilo::Nintendo);
    assert_eq!(Estilo::detecta("Xbox Wireless Controller", None), Estilo::Xbox);
    assert_eq!(Estilo::detecta("Generic USB Joystick", None), Estilo::Xbox);
}

#[test]
fn nomes_amigaveis_das_entradas_fisicas() {
    assert_eq!(Entrada::Sul.rotulo(Estilo::Xbox), "A");
    assert_eq!(Entrada::Leste.rotulo(Estilo::Xbox), "B");
    assert_eq!(Entrada::Leste.rotulo(Estilo::Nintendo), "A");
    assert!(Entrada::Sul.rotulo(Estilo::PlayStation).contains("Cruz"));
    assert!(Entrada::Leste.rotulo(Estilo::PlayStation).contains("Círculo"));
    assert_eq!(
        Entrada::EixoNegativo(0).rotulo(Estilo::Xbox),
        "Analógico esquerdo ←"
    );
    assert_eq!(
        Entrada::EixoPositivo(1).rotulo(Estilo::Xbox),
        "Analógico esquerdo ↑"
    );
    assert_eq!(
        Entrada::EixoPositivo(2).rotulo(Estilo::Xbox),
        "Analógico direito →"
    );
    assert_eq!(Entrada::DpadCima.rotulo(Estilo::Xbox), "Direcional ↑");
    assert!(!Entrada::Modo.rotulo(Estilo::Xbox).contains("modo"));
}

fn passo(n: &Navegador, agora: &[Entrada], t: f64) -> (Navegador, Vec<Comando>) {
    n.passo(&Perfil::padrao(), agora, t)
}

#[test]
fn circulo_volta_e_xis_ativa_so_na_borda() {
    let n = Navegador::default();
    let (n, c) = passo(&n, &[Entrada::Leste], 0.0);
    assert_eq!(c, vec![Comando::Voltar]);
    let (n, c) = passo(&n, &[Entrada::Leste], 0.1);
    assert!(c.is_empty(), "segurar nao repete Voltar");
    let (_, c) = passo(&n, &[Entrada::Sul], 0.2);
    assert_eq!(c, vec![Comando::Ativar]);
}

#[test]
fn faces_trocadas_trocam_tambem_quem_confirma_no_menu() {
    let (_, c) = Navegador::default().passo(&Perfil::faces_trocadas(), &[Entrada::Leste], 0.0);
    assert_eq!(c, vec![Comando::Ativar]);
}

#[test]
fn direcional_move_o_foco_e_repete_se_segurado() {
    let n = Navegador::default();
    let (n, c) = passo(&n, &[Entrada::DpadBaixo], 0.0);
    assert_eq!(c, vec![Comando::Mover(Sentido::Baixo)]);
    let (n, c) = passo(&n, &[Entrada::DpadBaixo], REPETE_APOS / 2.0);
    assert!(c.is_empty(), "antes do atraso nao repete");
    let (n, c) = passo(&n, &[Entrada::DpadBaixo], REPETE_APOS + 0.001);
    assert_eq!(c, vec![Comando::Mover(Sentido::Baixo)]);
    let (n, c) = passo(&n, &[Entrada::DpadBaixo], REPETE_APOS + REPETE_CADA / 2.0);
    assert!(c.is_empty());
    let (n, c) = passo(&n, &[Entrada::DpadBaixo], REPETE_APOS + REPETE_CADA + 0.002);
    assert_eq!(c, vec![Comando::Mover(Sentido::Baixo)]);
    let (n, c) = passo(&n, &[], 5.0);
    assert!(c.is_empty());
    let (_, c) = passo(&n, &[Entrada::DpadBaixo], 5.01);
    assert_eq!(c, vec![Comando::Mover(Sentido::Baixo)], "soltou e apertou de novo");
}

#[test]
fn analogico_esquerdo_tambem_navega() {
    let (_, c) = passo(&Navegador::default(), &[Entrada::EixoPositivo(1)], 0.0);
    assert_eq!(c, vec![Comando::Mover(Sentido::Cima)]);
}

#[test]
fn estado_antigo_nao_gera_borda_falsa() {
    let n = Navegador::default().com_antes(&[Entrada::Leste]);
    let (_, c) = passo(&n, &[Entrada::Leste], 0.0);
    assert!(c.is_empty(), "entrou no menu com ○ segurado: nao volta sozinho");
}
