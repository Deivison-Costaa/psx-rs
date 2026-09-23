use psx_core::app::input_map::Eixos;
use psx_core::app::troca::{
    CICLOS_DE_PORTA_ABERTA, PortaAberta, apertou_agora, forca_de_vibracao, ordena_candidatos,
    titulo_base,
};
use psx_core::dualshock::{Rumble, Sticks};

fn nomes(lista: &[&str]) -> Vec<String> {
    lista.iter().map(|s| s.to_string()).collect()
}

#[test]
fn titulo_base_tira_o_sufixo_de_disco() {
    assert_eq!(
        titulo_base("Final Fantasy IX (USA) (Disc 2)"),
        "Final Fantasy IX (USA)"
    );
    assert_eq!(
        titulo_base("Metal Gear Solid (USA) (Disc 1) (Rev 1)"),
        "Metal Gear Solid (USA)"
    );
    assert_eq!(titulo_base("Jogo [Disc2]"), "Jogo");
    assert_eq!(
        titulo_base("Crash Bandicoot (USA)"),
        "Crash Bandicoot (USA)"
    );
}

#[test]
fn titulo_base_ignora_caixa_da_marca() {
    assert_eq!(titulo_base("Jogo (DISC 3)"), "Jogo");
    assert_eq!(titulo_base("Jogo (CD2)"), "Jogo");
}

#[test]
fn candidatos_do_mesmo_jogo_vem_primeiro() {
    let lista = nomes(&[
        "Crash Bandicoot (USA)",
        "Final Fantasy IX (USA) (Disc 3)",
        "Tekken 3 (USA)",
        "Final Fantasy IX (USA) (Disc 1)",
        "Final Fantasy IX (USA) (Disc 2)",
    ]);
    let ordem = ordena_candidatos("Final Fantasy IX (USA) (Disc 1)", &lista);
    let indices: Vec<usize> = ordem.iter().map(|c| c.indice).collect();
    assert_eq!(indices, vec![3, 4, 1, 0, 2]);
    assert!(ordem[..3].iter().all(|c| c.mesmo_jogo));
    assert!(ordem[3..].iter().all(|c| !c.mesmo_jogo));
}

#[test]
fn candidato_atual_e_marcado() {
    let lista = nomes(&["A (Disc 1)", "A (Disc 2)"]);
    let ordem = ordena_candidatos("A (Disc 1)", &lista);
    assert!(ordem[0].atual);
    assert!(!ordem[1].atual);
}

#[test]
fn lista_vazia_nao_quebra() {
    assert!(ordena_candidatos("A", &[]).is_empty());
}

#[test]
fn titulo_vazio_nao_casa_com_ninguem() {
    let lista = nomes(&["(Disc 1)", "B"]);
    let ordem = ordena_candidatos("(Disc 2)", &lista);
    assert!(ordem.iter().all(|c| !c.mesmo_jogo));
}

#[test]
fn porta_fecha_depois_de_um_segundo_emulado() {
    let porta = PortaAberta::desde(1_000);
    assert!(!porta.deve_fechar(1_000));
    assert!(!porta.deve_fechar(1_000 + CICLOS_DE_PORTA_ABERTA - 1));
    assert!(porta.deve_fechar(1_000 + CICLOS_DE_PORTA_ABERTA));
    assert_eq!(CICLOS_DE_PORTA_ABERTA, 33_868_800);
}

#[test]
fn porta_no_fim_do_contador_nao_estoura() {
    let porta = PortaAberta::desde(u64::MAX - 1);
    assert!(porta.deve_fechar(u64::MAX));
}

#[test]
fn vibracao_escala_os_dois_motores() {
    assert_eq!(forca_de_vibracao(Rumble::default()), (0, 0));
    assert_eq!(
        forca_de_vibracao(Rumble {
            small: true,
            large: 0xFF
        }),
        (u16::MAX, u16::MAX)
    );
    assert_eq!(
        forca_de_vibracao(Rumble {
            small: false,
            large: 0x40
        }),
        (0x4040, 0)
    );
}

#[test]
fn botao_analog_so_conta_na_borda_de_subida() {
    assert!(apertou_agora(false, true));
    assert!(!apertou_agora(true, true));
    assert!(!apertou_agora(true, false));
    assert!(!apertou_agora(false, false));
}

#[test]
fn eixos_do_controle_invertem_y() {
    let e = Eixos::do_controle((1.0, 1.0), (-1.0, -1.0));
    assert_eq!(
        e.sticks(),
        Sticks {
            right_x: 0x00,
            right_y: 0xFF,
            left_x: 0xFF,
            left_y: 0x00,
        }
    );
}

#[test]
fn eixos_do_controle_em_repouso_ficam_no_centro() {
    let e = Eixos::do_controle((0.03, -0.05), (0.0, 0.07));
    assert_eq!(e.sticks(), Sticks::CENTERED);
}
