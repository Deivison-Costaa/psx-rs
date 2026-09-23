use psx_core::app::input_map::{Eixos, Entrada, Perfil, SOLTO, direcao, eixo_para_byte};
use psx_core::dualshock::Sticks;
use psx_core::pad_script::button_bit;

fn bit(nome: &str) -> u16 {
    1u16 << button_bit(nome).expect("botao conhecido")
}

#[test]
fn eixo_nos_extremos_e_no_centro() {
    assert_eq!(eixo_para_byte(-1.0), 0x00, "00h = esquerda/cima");
    assert_eq!(eixo_para_byte(1.0), 0xFF, "FFh = direita/baixo");
    assert_eq!(eixo_para_byte(0.0), 0x80, "80h = centro");
}

#[test]
fn eixo_dentro_da_zona_morta_fica_no_centro() {
    assert_eq!(eixo_para_byte(0.05), 0x80);
    assert_eq!(eixo_para_byte(-0.05), 0x80);
}

#[test]
fn eixo_fora_da_faixa_ou_invalido_e_contido() {
    assert_eq!(eixo_para_byte(3.0), 0xFF);
    assert_eq!(eixo_para_byte(-3.0), 0x00);
    assert_eq!(eixo_para_byte(f32::NAN), 0x80);
}

#[test]
fn eixo_e_monotonico() {
    let mut anterior = 0u8;
    for i in -100..=100 {
        let atual = eixo_para_byte(i as f32 / 100.0);
        assert!(atual >= anterior, "{i}: {atual} < {anterior}");
        anterior = atual;
    }
}

#[test]
fn eixos_viram_sticks_na_ordem_certa() {
    let e = Eixos {
        esquerdo_x: -1.0,
        esquerdo_y: 1.0,
        direito_x: 1.0,
        direito_y: -1.0,
    };
    assert_eq!(
        e.sticks(),
        Sticks {
            right_x: 0xFF,
            right_y: 0x00,
            left_x: 0x00,
            left_y: 0xFF,
        }
    );
    assert_eq!(Eixos::default().sticks(), Sticks::CENTERED);
}

#[test]
fn unir_teclado_e_controle_fica_com_o_mais_inclinado() {
    let controle = Eixos {
        esquerdo_x: 0.3,
        direito_y: -0.9,
        ..Eixos::default()
    };
    let teclado = Eixos {
        esquerdo_x: -1.0,
        direito_y: 0.2,
        ..Eixos::default()
    };
    let unido = controle.une(&teclado);
    assert_eq!(unido.esquerdo_x, -1.0);
    assert_eq!(unido.direito_y, -0.9);
    assert_eq!(unido.esquerdo_y, 0.0);
}

#[test]
fn direcao_digital_se_anula_com_as_duas_teclas() {
    assert_eq!(direcao(true, false), -1.0);
    assert_eq!(direcao(false, true), 1.0);
    assert_eq!(direcao(true, true), 0.0);
    assert_eq!(direcao(false, false), 0.0);
}

#[test]
fn no_modo_analogico_o_analogico_esquerdo_nao_aperta_o_direcional() {
    let p = Perfil::padrao();
    let entradas = [Entrada::EixoNegativo(0), Entrada::Sul];

    let digital = p.palavra_no_modo(&entradas, false);
    assert_eq!(digital, SOLTO & !bit("left") & !bit("cross"));

    let analogico = p.palavra_no_modo(&entradas, true);
    assert_eq!(analogico, SOLTO & !bit("cross"), "so o X continua apertado");
}

#[test]
fn palavra_sem_modo_continua_tratando_eixo_como_direcional() {
    let p = Perfil::padrao();
    assert_eq!(p.palavra(&[Entrada::EixoPositivo(1)]), SOLTO & !bit("up"));
}
