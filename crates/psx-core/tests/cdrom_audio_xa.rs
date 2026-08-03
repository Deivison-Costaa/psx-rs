use psx_core::cdrom_xa::{self, CDDA_FRAMES, OUTPUT_HZ, RAW_SECTOR_BYTES, XaState};

/// Monta um setor de 2352 bytes com os 18 grupos preenchidos pelo mesmo byte de
/// nibbles e o mesmo par de shift/filtro no bloco 0.
fn setor(par_baixo: u8, par_alto: u8, byte_de_nibbles: u8) -> Vec<u8> {
    let mut s = vec![0u8; RAW_SECTOR_BYTES];
    for grupo in 0..18 {
        let base = 12 + 4 + 8 + grupo * 128;
        for blk in 0..4 {
            s[base + 4 + blk * 2] = par_baixo;
            s[base + 5 + blk * 2] = par_alto;
            for j in 0..28 {
                s[base + 16 + blk + j * 4] = byte_de_nibbles;
            }
        }
    }
    s
}

fn dados(s: &[u8]) -> &[u8] {
    &s[24..]
}

#[test]
fn xa_desloca_o_nibble_por_12_menos_o_campo_de_shift() {
    let s = setor(0x08, 0x00, 0x37);
    let (amostras, old, older) = cdrom_xa::decode_28_nibbles(dados(&s), 0, 0, 0, 0);
    assert_eq!(
        &amostras[..4],
        &[112, 112, 112, 112],
        "shift = 12 - (par AND 0Fh) = 4, filtro 0: s = 7 SHL 4"
    );
    assert_eq!((old, older), (112, 112));
}

#[test]
fn xa_nibble_alto_usa_o_proprio_par_de_shift_e_filtro() {
    let s = setor(0x08, 0x00, 0x37);
    let (amostras, _, _) = cdrom_xa::decode_28_nibbles(dados(&s), 0, 1, 0, 0);
    assert_eq!(
        &amostras[..4],
        &[12288, 12288, 12288, 12288],
        "o par do nibble alto e src[4+blk*2+1] = 00h, entao shift = 12"
    );
}

#[test]
fn xa_filtro_2_soma_115_do_anterior_e_menos_52_do_penultimo() {
    let s = setor(0x28, 0x00, 0x37);
    let (amostras, _, _) = cdrom_xa::decode_28_nibbles(dados(&s), 0, 0, 0, 0);
    assert_eq!(
        &amostras[..6],
        &[112, 313, 583, 905, 1264, 1648],
        "mesma tabela pos/neg do SPU-ADPCM"
    );
}

#[test]
fn xa_estereo_alterna_nibble_baixo_a_esquerda_e_alto_a_direita() {
    let s = setor(0x08, 0x00, 0x37);
    let mut estado = XaState::default();
    let quadros = cdrom_xa::decode_sector(&s, true, &mut estado);
    assert_eq!(
        quadros.len(),
        18 * 4 * 28,
        "18 grupos x 4 blocos x 28 amostras por canal"
    );
    assert_eq!(quadros[0], (112, 12288));
    assert_eq!(quadros[27], (112, 12288));
}

#[test]
fn xa_mono_poe_os_dois_nibbles_em_sequencia_no_mesmo_canal() {
    let s = setor(0x08, 0x00, 0x37);
    let mut estado = XaState::default();
    let quadros = cdrom_xa::decode_sector(&s, false, &mut estado);
    assert_eq!(
        quadros.len(),
        18 * 4 * 56,
        "mono da 2x28 amostras por bloco"
    );
    assert_eq!(quadros[0], (112, 112), "mono duplica o canal");
    assert_eq!(quadros[28], (12288, 12288));
}

#[test]
fn setor_cdda_vira_588_quadros_estereo_de_16_bits() {
    let mut raw = vec![0u8; RAW_SECTOR_BYTES];
    raw[0..4].copy_from_slice(&[0x34, 0x12, 0x78, 0x56]);
    raw[RAW_SECTOR_BYTES - 4..].copy_from_slice(&[0x00, 0x80, 0xFF, 0x7F]);
    let quadros = cdrom_xa::cdda_frames(&raw);
    assert_eq!(quadros.len(), CDDA_FRAMES);
    assert_eq!(quadros.len(), 588);
    assert_eq!(quadros[0], (0x1234, 0x5678), "little-endian, L antes de R");
    assert_eq!(quadros[587], (-32768, 32767));
}

#[test]
fn xa_carrega_o_historico_de_um_bloco_para_o_seguinte() {
    let s = setor(0x28, 0x28, 0x37);
    let mut estado = XaState::default();
    let quadros = cdrom_xa::decode_sector(&s, true, &mut estado);
    assert_ne!(
        estado,
        XaState::default(),
        "old/older sobrevivem ao fim do setor para o proximo"
    );
    let primeiro = quadros[0].0;
    let mut estado2 = estado;
    let seguintes = cdrom_xa::decode_sector(&s, true, &mut estado2);
    assert_ne!(
        seguintes[0].0, primeiro,
        "com historico diferente de zero a primeira amostra do setor seguinte muda"
    );
}

#[test]
fn reamostragem_de_37800_para_44100_estica_sete_por_seis() {
    let entrada: Vec<(i16, i16)> = (0..6).map(|i| (i as i16, -(i as i16))).collect();
    let saida = cdrom_xa::resample_to_44100(&entrada, 37800);
    assert_eq!(
        saida.len(),
        7,
        "6 quadros a 37800 Hz duram 7 quadros a 44100"
    );
    assert_eq!(saida[0], (0, 0));
    assert_eq!(saida[6], (5, -5));
    assert_eq!(
        cdrom_xa::resample_to_44100(&entrada, OUTPUT_HZ),
        entrada,
        "na taxa de saida nao ha reamostragem"
    );
}

#[test]
fn subcabecalho_diz_taxa_estereo_e_se_o_setor_e_de_audio() {
    assert_eq!(cdrom_xa::xa_sample_rate(0x00), 37800);
    assert_eq!(cdrom_xa::xa_sample_rate(0x04), 18900);
    assert!(cdrom_xa::xa_is_stereo(0x01));
    assert!(!cdrom_xa::xa_is_stereo(0x00));

    let mut cru = vec![0u8; RAW_SECTOR_BYTES];
    assert!(!cdrom_xa::is_xa_audio_sector(&cru), "modo 1 nao e audio XA");
    cru[0x0F] = 0x02;
    assert!(!cdrom_xa::is_xa_audio_sector(&cru), "submode sem o bit2");
    cru[0x12] = 0x04;
    assert!(cdrom_xa::is_xa_audio_sector(&cru));
}
