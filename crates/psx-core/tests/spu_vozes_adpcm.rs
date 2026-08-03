use psx_core::spu::Spu;
use psx_core::spu::adpcm::{self, BLOCK_SAMPLES, Flags};

const V0: u32 = 0x1F80_1C00;
const KON_LO: u32 = 0x1F80_1D88;
const KOFF_LO: u32 = 0x1F80_1D8C;
const ENDX_LO: u32 = 0x1F80_1D9C;
const TRANSFER_ADDR: u32 = 0x1F80_1DA6;
const FIFO: u32 = 0x1F80_1DA8;
const CNT: u32 = 0x1F80_1DAA;
const STAT: u32 = 0x1F80_1DAE;

const CURRENT_ADSR: u32 = 0x1F80_1C0C;

/// Bloco de 16 bytes: cabecalho (shift/filtro), flags e 14 bytes de nibbles.
fn bloco(shift_filtro: u8, flags: u8, nibbles: [u8; 28]) -> [u8; 16] {
    let mut b = [0u8; 16];
    b[0] = shift_filtro;
    b[1] = flags;
    for i in 0..14 {
        b[2 + i] = (nibbles[i * 2] & 0x0F) | ((nibbles[i * 2 + 1] & 0x0F) << 4);
    }
    b
}

fn repetido(n: u8) -> [u8; 28] {
    [n; 28]
}

/// Carrega bytes na RAM do SPU pela escrita manual (DA6h -> DA8h -> CNT modo 1),
/// que e o caminho que o jogo usa.
fn carrega(spu: &mut Spu, endereco_em_oitavos: u16, bytes: &[u8]) {
    spu.write16(TRANSFER_ADDR, endereco_em_oitavos);
    for par in bytes.chunks(2) {
        let hi = if par.len() > 1 { par[1] } else { 0 };
        spu.write16(FIFO, u16::from_le_bytes([par[0], hi]));
    }
    spu.write16(CNT, 1 << 4);
    spu.write16(CNT, 0);
}

#[test]
fn adpcm_filtro_0_shift_0_devolve_o_nibble_deslocado_12_bits() {
    let b = bloco(0x00, 0, repetido(7));
    let (amostras, _, _) = adpcm::decode_block(&b, 0, 0);
    assert_eq!(
        amostras.to_vec(),
        vec![28672i16; BLOCK_SAMPLES],
        "filtro 0 nao soma historico: s = (t SHL 12) SAR shift, com t=+7 e shift=0"
    );
}

#[test]
fn adpcm_nibble_de_8_a_f_e_negativo() {
    let mut n = [0u8; 28];
    for (i, v) in n.iter_mut().enumerate() {
        *v = if i % 2 == 0 { 7 } else { 9 };
    }
    let (amostras, _, _) = adpcm::decode_block(&bloco(0x00, 0, n), 0, 0);
    assert_eq!(&amostras[..4], &[28672, -28672, 28672, -28672]);
}

#[test]
fn adpcm_filtro_2_soma_115_do_anterior_e_menos_52_do_penultimo() {
    let (amostras, prev1, prev2) = adpcm::decode_block(&bloco(0x24, 0, repetido(7)), 0, 0);
    assert_eq!(
        &amostras[..6],
        &[1792, 5012, 9342, 14506, 20267, 26423],
        "s = (t SHL 12 SAR 4) + ((old*115 + older*(-52) + 32) SAR 6)"
    );
    assert_eq!(prev1 as i16, amostras[BLOCK_SAMPLES - 1]);
    assert_eq!(prev2 as i16, amostras[BLOCK_SAMPLES - 2]);
}

#[test]
fn adpcm_satura_o_resultado_em_16_bits_com_sinal() {
    let (amostras, _, _) = adpcm::decode_block(&bloco(0x20, 0, repetido(7)), 0, 0);
    assert_eq!(amostras[0], 28672);
    assert_eq!(
        amostras[1], 32767,
        "MinMax(s, -8000h, +7FFFh) — 28672 + 115*28672/64 estoura"
    );
}

#[test]
fn adpcm_le_os_tres_bits_de_flag_do_segundo_byte() {
    assert_eq!(Flags::from_bits(0), Flags::default());
    let f = Flags::from_bits(0b111);
    assert!(f.loop_end && f.loop_repeat && f.loop_start);
    let so_start = Flags::from_bits(0b100);
    assert!(so_start.loop_start && !so_start.loop_end && !so_start.loop_repeat);
}

#[test]
fn voz_le_de_volta_os_oito_registradores_que_foram_escritos() {
    let mut spu = Spu::new();
    let escritos: [(u32, u16); 6] = [
        (V0, 0x1234),
        (V0 + 2, 0x2345),
        (V0 + 4, 0x0800),
        (V0 + 6, 0x0200),
        (V0 + 8, 0x00FF),
        (V0 + 0x0E, 0x0201),
    ];
    for (a, v) in escritos {
        spu.write16(a, v);
    }
    for (a, v) in escritos {
        assert_eq!(spu.read16(a), v, "registrador {a:08X}");
    }
    // Voz 5 tem base 1F801C50h e nao pode espelhar a voz 0.
    spu.write16(0x1F80_1C50, 0x5555);
    assert_eq!(spu.read16(0x1F80_1C50), 0x5555);
    assert_eq!(spu.read16(V0), 0x1234);
}

#[test]
fn key_on_copia_o_start_address_e_zera_a_envoltoria() {
    let mut spu = Spu::new();
    carrega(&mut spu, 0x200, &bloco(0x00, 0, repetido(7)));
    spu.write16(V0 + 6, 0x0200);
    spu.write16(CURRENT_ADSR, 0x4000);
    spu.write16(KON_LO, 1);
    assert_eq!(
        spu.read16(CURRENT_ADSR),
        0,
        "§ 1F801D88h (L599): key on inicializa o volume ADSR em zero"
    );
    spu.write16(V0 + 4, 0x1000);
    for _ in 0..4 {
        spu.tick();
    }
    assert_ne!(
        spu.voice_out(0),
        0,
        "com o endereco corrente carregado do start address a voz tem de produzir amostra"
    );
}

#[test]
fn flag_loop_start_copia_o_endereco_corrente_para_o_repeat() {
    let mut spu = Spu::new();
    carrega(&mut spu, 0x200, &bloco(0x00, 0b100, repetido(7)));
    spu.write16(V0 + 6, 0x0200);
    spu.write16(V0 + 4, 0x1000);
    spu.write16(V0 + 8, 0x00FF);
    spu.write16(KON_LO, 1);
    spu.tick();
    assert_eq!(
        spu.read16(V0 + 0x0E),
        0x0200,
        "§ 1F801C0Eh+N*10h (L203): loop-start copia o endereco corrente para o repeat"
    );
}

#[test]
fn flag_loop_end_liga_o_endx_e_salta_para_o_repeat() {
    let mut spu = Spu::new();
    carrega(&mut spu, 0x200, &bloco(0x00, 0b011, repetido(7)));
    carrega(&mut spu, 0x300, &bloco(0x00, 0, repetido(1)));
    spu.write16(V0 + 6, 0x0200);
    spu.write16(V0 + 0x0E, 0x0300);
    spu.write16(V0 + 4, 0x1000);
    spu.write16(V0 + 8, 0x00FF);
    spu.write16(KON_LO, 1);
    for _ in 0..40 {
        spu.tick();
    }
    assert_eq!(
        spu.read16(ENDX_LO) & 1,
        1,
        "§ 1F801D9Ch (L615): o bit e SET ao alcancar loop-end no cabecalho"
    );
}

#[test]
fn loop_end_sem_repeat_forca_release_com_nivel_zero() {
    let mut spu = Spu::new();
    carrega(&mut spu, 0x200, &bloco(0x00, 0b001, repetido(7)));
    spu.write16(V0 + 6, 0x0200);
    spu.write16(V0 + 4, 0x1000);
    spu.write16(V0 + 8, 0x000F);
    spu.write16(V0 + 0x0A, 0x1FC0);
    spu.write16(KON_LO, 1);
    for _ in 0..10 {
        spu.tick();
    }
    assert_eq!(
        spu.read16(CURRENT_ADSR),
        0x7FFF,
        "antes do fim do bloco a envoltoria esta travada no teto"
    );
    for _ in 0..30 {
        spu.tick();
    }
    assert_eq!(
        spu.read16(CURRENT_ADSR),
        0,
        "Code 1 = End+Mute: forca release e zera o nivel da envoltoria"
    );
}

#[test]
fn key_on_limpa_o_bit_de_endx_da_voz() {
    let mut spu = Spu::new();
    carrega(&mut spu, 0x200, &bloco(0x00, 0b011, repetido(7)));
    spu.write16(V0 + 6, 0x0200);
    spu.write16(V0 + 0x0E, 0x0200);
    spu.write16(V0 + 4, 0x1000);
    spu.write16(V0 + 8, 0x00FF);
    spu.write16(KON_LO, 1);
    for _ in 0..40 {
        spu.tick();
    }
    assert_eq!(spu.read16(ENDX_LO) & 1, 1);
    spu.write16(KON_LO, 1);
    assert_eq!(
        spu.read16(ENDX_LO) & 1,
        0,
        "§ 1F801D9Ch (L615): os bits sao CLEARED ao ligar o KEY ON correspondente"
    );
}

#[test]
fn key_off_leva_a_voz_para_release() {
    let mut spu = Spu::new();
    carrega(&mut spu, 0x200, &bloco(0x00, 0b100, repetido(7)));
    spu.write16(V0 + 6, 0x0200);
    spu.write16(V0 + 4, 0x1000);
    // Ataque linear rapido, release linear shift 0 (queda instantanea).
    spu.write16(V0 + 8, 0x0000);
    spu.write16(V0 + 0x0A, 0x0000);
    spu.write16(KON_LO, 1);
    for _ in 0..3 {
        spu.tick();
    }
    assert_eq!(
        spu.read16(CURRENT_ADSR),
        0x7FFF,
        "ataque linear com shift 0 satura em tres ciclos"
    );
    spu.write16(KOFF_LO, 1);
    spu.tick();
    assert_eq!(
        spu.read16(CURRENT_ADSR),
        0x3FFF,
        "release linear tira 4000h por ciclo"
    );
    spu.tick();
    assert_eq!(
        spu.read16(CURRENT_ADSR),
        0,
        "e para em zero, sem passar para negativo"
    );
}

#[test]
fn spustat_espelha_os_bits_5_0_do_spucnt() {
    let mut spu = Spu::new();
    spu.write16(CNT, 0b0000_0000_0011_0101);
    assert_eq!(
        spu.read16(STAT) & 0x3F,
        0b11_0101,
        "§ 1F801DAEh (L678): bits 5-0 do SPUSTAT sao os bits 5-0 do SPUCNT"
    );
    spu.write16(CNT, 0);
    assert_eq!(spu.read16(STAT) & 0x3F, 0);
}

#[test]
fn escrita_manual_leva_as_amostras_para_a_ram_do_spu() {
    let mut spu = Spu::new();
    let b = bloco(0x0C, 0b100, repetido(3));
    carrega(&mut spu, 0x200, &b);
    assert_eq!(spu.ram_peek16(0x1000), u16::from_le_bytes([b[0], b[1]]));
    assert_eq!(spu.ram_peek16(0x1002), u16::from_le_bytes([b[2], b[3]]));
}
