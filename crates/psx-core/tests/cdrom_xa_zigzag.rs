use psx_core::cdrom_xa::{self, XaState};

// Tabela da secao "25-point Zigzag Interpolation" de docs/reference/15-cdrom-format.md,
// linha = Index 1..29, coluna = Table1..Table7.
const TABELA: [[i32; 7]; 29] = [
    [0x0000, 0x0000, 0x0000, 0x0000, -0x0001, 0x0002, -0x0005],
    [0x0000, 0x0000, 0x0000, -0x0001, 0x0003, -0x0008, 0x0011],
    [0x0000, 0x0000, -0x0001, 0x0003, -0x0008, 0x0010, -0x0023],
    [0x0000, -0x0002, 0x0003, -0x0008, 0x0011, -0x0023, 0x0046],
    [0x0000, 0x0000, -0x0002, 0x0006, -0x0010, 0x002B, -0x0017],
    [-0x0002, 0x0003, -0x0005, 0x0005, 0x000A, 0x001A, -0x0044],
    [0x000A, -0x0013, 0x001F, -0x001B, 0x006B, -0x00EB, 0x015B],
    [-0x0022, 0x003C, -0x004A, 0x00A6, -0x016D, 0x027B, -0x0347],
    [0x0041, -0x004B, 0x00B3, -0x01A8, 0x0350, -0x0548, 0x080E],
    [-0x0054, 0x00A2, -0x0192, 0x0372, -0x0623, 0x0AFA, -0x1249],
    [0x0034, -0x00E3, 0x02B1, -0x05BF, 0x0BCD, -0x16FA, 0x3C07],
    [0x0009, 0x0132, -0x039E, 0x09B8, -0x1780, 0x53E0, 0x53E0],
    [-0x010A, -0x0043, 0x04F8, -0x11B4, 0x6794, 0x3C07, -0x16FA],
    [0x0400, -0x0267, -0x05A6, 0x74BB, 0x234C, -0x1249, 0x0AFA],
    [-0x0A78, 0x0C9D, 0x7939, 0x0C9D, -0x0A78, 0x080E, -0x0548],
    [0x234C, 0x74BB, -0x05A6, -0x0267, 0x0400, -0x0347, 0x027B],
    [0x6794, -0x11B4, 0x04F8, -0x0043, -0x010A, 0x015B, -0x00EB],
    [-0x1780, 0x09B8, -0x039E, 0x0132, 0x0009, -0x0044, 0x001A],
    [0x0BCD, -0x05BF, 0x02B1, -0x00E3, 0x0034, -0x0017, 0x002B],
    [-0x0623, 0x0372, -0x0192, 0x00A2, -0x0054, 0x0046, -0x0023],
    [0x0350, -0x01A8, 0x00B3, -0x004B, 0x0041, -0x0023, 0x0010],
    [-0x016D, 0x00A6, -0x004A, 0x003C, -0x0022, 0x0011, -0x0008],
    [0x006B, -0x001B, 0x001F, -0x0013, 0x000A, -0x0005, 0x0002],
    [0x000A, 0x0005, -0x0005, 0x0003, -0x0001, 0x0000, 0x0000],
    [-0x0010, 0x0006, -0x0002, 0x0000, 0x0000, 0x0000, 0x0000],
    [0x0011, -0x0008, 0x0003, -0x0002, 0x0001, 0x0000, 0x0000],
    [-0x0008, 0x0003, -0x0001, 0x0000, 0x0000, 0x0000, 0x0000],
    [0x0003, -0x0001, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000],
    [-0x0001, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000],
];

fn coeficiente(indice: usize, tabela: usize) -> i32 {
    if (1..=29).contains(&indice) {
        TABELA[indice - 1][tabela]
    } else {
        0
    }
}

/// -8000h * c / 8000h = -c exato: a resposta ao impulso le a tabela de volta.
fn impulso_em(posicao: usize, total: usize) -> Vec<(i16, i16)> {
    (0..total)
        .map(|i| if i == posicao { (-0x8000, 0) } else { (0, 0) })
        .collect()
}

#[test]
fn cada_seis_amostras_de_37800_viram_sete_de_44100() {
    let mut estado = XaState::default();
    let saida = cdrom_xa::resample_to_44100(&[(0i16, 0i16); 2016], 37800, &mut estado);
    assert_eq!(saida.len(), 2352, "setor estereo: 2016 quadros -> 2352");
    let mut estado = XaState::default();
    let saida = cdrom_xa::resample_to_44100(&[(1, 1); 5], 37800, &mut estado);
    assert!(
        saida.is_empty(),
        "com 5 amostras o contador de seis passos nao fechou"
    );
}

#[test]
fn resposta_ao_impulso_percorre_as_sete_tabelas_da_spec() {
    for k in 0..6 {
        let mut estado = XaState::default();
        let saida = cdrom_xa::resample_to_44100(&impulso_em(k, 36), 37800, &mut estado);
        assert_eq!(saida.len(), 42);
        for grupo in 1..=6 {
            let indice = 6 * grupo - k;
            for tabela in 0..7 {
                let (esq, dir) = saida[(grupo - 1) * 7 + tabela];
                assert_eq!(
                    i32::from(esq),
                    -coeficiente(indice, tabela),
                    "impulso na amostra {k}, grupo {grupo}, Table{}: ringbuf[p-{indice}]",
                    tabela + 1
                );
                assert_eq!(dir, 0, "o canal direito tem anel proprio");
            }
        }
    }
}

#[test]
fn em_18900_cada_amostra_entra_duas_vezes_no_anel() {
    let mut estado = XaState::default();
    let saida = cdrom_xa::resample_to_44100(&impulso_em(0, 3), 18900, &mut estado);
    assert_eq!(saida.len(), 7, "3 amostras a 18900 Hz duram 7 a 44100 Hz");
    for (tabela, quadro) in saida.iter().enumerate() {
        let esperado = -(coeficiente(6, tabela) + coeficiente(5, tabela));
        assert_eq!(i32::from(quadro.0), esperado, "Table{}", tabela + 1);
    }
    let mut estado = XaState::default();
    let mono = cdrom_xa::resample_to_44100(&[(0i16, 0i16); 4032], 18900, &mut estado);
    assert_eq!(
        mono.len(),
        9408,
        "setor mono a 18900 Hz: 4032 amostras -> 9408"
    );
}

#[test]
fn anel_e_contador_atravessam_a_fronteira_do_setor() {
    let entrada: Vec<(i16, i16)> = (0..40)
        .map(|i| ((i * 700 - 9000) as i16, (5000 - i * 311) as i16))
        .collect();
    let mut inteiro = XaState::default();
    let de_uma_vez = cdrom_xa::resample_to_44100(&entrada, 37800, &mut inteiro);
    let mut partido = XaState::default();
    let mut em_partes = cdrom_xa::resample_to_44100(&entrada[..17], 37800, &mut partido);
    em_partes.extend(cdrom_xa::resample_to_44100(
        &entrada[17..],
        37800,
        &mut partido,
    ));
    assert_eq!(de_uma_vez, em_partes);
    assert_eq!(inteiro, partido);
}

#[test]
fn soma_satura_em_7fff_e_menos_8000() {
    let tabela = 2;
    let sinal = |m: usize| coeficiente(30 - m, tabela).signum() as i16;
    let positivo: Vec<(i16, i16)> = (0..30).map(|m| (sinal(m) * 0x7FFF, 0)).collect();
    let negativo: Vec<(i16, i16)> = (0..30).map(|m| (-sinal(m) * 0x7FFF, 0)).collect();
    let mut estado = XaState::default();
    let saida = cdrom_xa::resample_to_44100(&positivo, 37800, &mut estado);
    assert_eq!(
        saida[4 * 7 + tabela].0,
        0x7FFF,
        "soma de |Table3| passa de 8000h"
    );
    let mut estado = XaState::default();
    let saida = cdrom_xa::resample_to_44100(&negativo, 37800, &mut estado);
    assert_eq!(saida[4 * 7 + tabela].0, -0x8000);
}

#[test]
fn taxa_de_cd_da_passa_direto() {
    let entrada = [(1, 2), (3, 4)];
    let mut estado = XaState::default();
    assert_eq!(
        cdrom_xa::resample_to_44100(&entrada, 44100, &mut estado),
        entrada.to_vec()
    );
}
