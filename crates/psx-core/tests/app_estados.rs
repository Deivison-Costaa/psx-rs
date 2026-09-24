use psx_core::app::estados::{
    MINIATURA_ALTURA, MINIATURA_LARGURA, data_hora, deslocamento_de, miniatura, nome_da_miniatura,
    nome_do_estado,
};

#[test]
fn nome_do_estado_e_compativel_com_o_formato_antigo() {
    assert_eq!(nome_do_estado("SCUS-94900", 3), "SCUS-94900-3.state");
    assert_eq!(nome_da_miniatura("SCUS-94900", 3), "SCUS-94900-3.png");
}

#[test]
fn serial_com_barra_nao_escapa_da_pasta_de_saves() {
    assert_eq!(nome_do_estado("../../x", 0), "x-0.state");
}

#[test]
fn miniatura_tem_tamanho_fixo_e_amostra_o_quadro() {
    let (l, a) = (320usize, 240usize);
    let mut rgba = vec![0u8; l * a * 4];
    for y in 0..a {
        for x in 0..l {
            let i = (y * l + x) * 4;
            rgba[i] = if x < l / 2 { 255 } else { 0 };
            rgba[i + 2] = if y < a / 2 { 255 } else { 0 };
            rgba[i + 3] = 255;
        }
    }
    let m = miniatura(&rgba, l, a).expect("miniatura");
    assert_eq!(m.len(), MINIATURA_LARGURA * MINIATURA_ALTURA * 4);
    let px = |x: usize, y: usize| {
        let i = (y * MINIATURA_LARGURA + x) * 4;
        (m[i], m[i + 2], m[i + 3])
    };
    assert_eq!(px(0, 0), (255, 255, 255));
    assert_eq!(px(MINIATURA_LARGURA - 1, 0), (0, 255, 255));
    assert_eq!(px(0, MINIATURA_ALTURA - 1), (255, 0, 255));
}

#[test]
fn miniatura_forca_alfa_opaco() {
    let m = miniatura(&[10, 20, 30, 0], 1, 1).expect("1x1");
    assert!(m.chunks(4).all(|p| p == [10, 20, 30, 255]));
}

#[test]
fn quadro_vazio_ou_incoerente_nao_gera_miniatura() {
    assert!(miniatura(&[], 0, 0).is_none());
    assert!(miniatura(&[0; 8], 4, 4).is_none());
}

#[test]
fn data_hora_no_formato_brasileiro() {
    assert_eq!(data_hora(0), "01/01/1970 00:00");
    assert_eq!(data_hora(1_790_150_700), "23/09/2026 08:05");
    assert_eq!(data_hora(951_782_400), "29/02/2000 00:00");
}

#[test]
fn deslocamento_do_fuso_vem_do_date_z() {
    assert_eq!(deslocamento_de("-0300\n"), Some(-3 * 3600));
    assert_eq!(deslocamento_de("+0530"), Some(5 * 3600 + 30 * 60));
    assert_eq!(deslocamento_de("lixo"), None);
}
