use psx_core::spu::Spu;
use psx_core::spu::envelope::{Envelope, Rate};
use psx_core::spu::gauss;

const V0: u32 = 0x1F80_1C00;
const V1: u32 = 0x1F80_1C10;
const MAIN_L: u32 = 0x1F80_1D80;
const MAIN_R: u32 = 0x1F80_1D82;
const KON_LO: u32 = 0x1F80_1D88;
const PMON_LO: u32 = 0x1F80_1D90;
const ENDX_LO: u32 = 0x1F80_1D9C;
const TRANSFER_ADDR: u32 = 0x1F80_1DA6;
const FIFO: u32 = 0x1F80_1DA8;
const CNT: u32 = 0x1F80_1DAA;

/// ADSR que sobe em tres ciclos e depois trava: ataque linear shift 0 passo 0,
/// nivel de sustain 0Fh (8000h, acima do teto) e taxa de sustain com todos os
/// bits em um, que nunca avanca.
const ADSR_LO_TRAVA_NO_TETO: u16 = 0x000F;
const ADSR_HI_TRAVA_NO_TETO: u16 = 0x1FC0;

fn bloco(shift_filtro: u8, flags: u8, nibble: u8) -> [u8; 16] {
    let mut b = [0u8; 16];
    b[0] = shift_filtro;
    b[1] = flags;
    for byte in b.iter_mut().skip(2) {
        *byte = (nibble & 0x0F) | ((nibble & 0x0F) << 4);
    }
    b
}

fn carrega(spu: &mut Spu, endereco_em_oitavos: u16, bytes: &[u8]) {
    spu.write16(TRANSFER_ADDR, endereco_em_oitavos);
    for par in bytes.chunks(2) {
        spu.write16(FIFO, u16::from_le_bytes([par[0], par[1]]));
    }
    spu.write16(CNT, 1 << 4);
    spu.write16(CNT, 0);
}

/// Voz tocando um bloco de amostras constantes, com envoltoria travada no teto.
fn voz_constante(spu: &mut Spu, base: u32, oitavos: u16, nibble: u8, pitch: u16, bit: u32) {
    // Flags 111b: loop-start marca o repeat no proprio bloco e End+Repeat volta para
    // ele, entao o tom e constante e o ENDX acende no fim de cada volta.
    carrega(spu, oitavos, &bloco(0x00, 0b111, nibble));
    spu.write16(base + 6, oitavos);
    spu.write16(base + 4, pitch);
    spu.write16(base + 8, ADSR_LO_TRAVA_NO_TETO);
    spu.write16(base + 0x0A, ADSR_HI_TRAVA_NO_TETO);
    spu.write16(KON_LO, (1 << bit) as u16);
}

fn envoltoria(nivel: i32, taxa: Rate, ciclos: usize) -> Vec<i32> {
    let mut env = Envelope {
        level: nivel,
        counter: 0,
    };
    (0..ciclos)
        .map(|_| {
            env.tick(taxa);
            env.level
        })
        .collect()
}

fn linear(step: u8, shift: u8, decrescente: bool) -> Rate {
    Rate {
        step,
        shift,
        exponential: false,
        decreasing: decrescente,
        phase_negative: false,
    }
}

#[test]
fn gauss_com_janela_constante_devolve_quase_a_amostra() {
    assert_eq!(
        gauss::interpolate(0, 16384, 16384, 16384, 16384),
        16318,
        "as quatro entradas somam 7F80h, nao 8000h — a saida fica 0,4% abaixo"
    );
    assert_eq!(gauss::interpolate(128, 16384, 16384, 16384, 16384), 16319);
}

#[test]
fn gauss_da_entrada_zero_pesa_a_amostra_nova_com_menos_um() {
    assert_eq!(gauss::interpolate(0, 0, 0, 0, 32767), -1);
}

#[test]
fn envoltoria_ataque_linear_shift_0_sobe_14336_por_ciclo() {
    assert_eq!(
        envoltoria(0, linear(0, 0, false), 4),
        vec![14336, 28672, 32767, 32767],
        "AdsrStep = (7-0) SHL 11 = 3800h e a subida satura em 7FFFh"
    );
}

#[test]
fn envoltoria_passo_3_e_shift_1_valem_um_quarto_do_passo_0_shift_0() {
    assert_eq!(
        envoltoria(0, linear(3, 1, false), 4),
        vec![4096, 8192, 12288, 16384]
    );
}

#[test]
fn envoltoria_decay_exponencial_cai_pela_metade() {
    let exp_dec = Rate {
        step: 0,
        shift: 0,
        exponential: true,
        decreasing: true,
        phase_negative: false,
    };
    assert_eq!(
        envoltoria(0x7FFF, exp_dec, 5),
        vec![16383, 8191, 4095, 2047, 1023],
        "AdsrStep = AdsrStep * AdsrLevel / 8000h"
    );
}

#[test]
fn envoltoria_ataque_exponencial_desacelera_acima_de_6000h() {
    let exp_inc = Rate {
        step: 0,
        shift: 0,
        exponential: true,
        decreasing: false,
        phase_negative: false,
    };
    assert_eq!(
        envoltoria(0x6001, exp_inc, 3),
        vec![28161, 31745, 32767],
        "com shift < 10 o passo e dividido por 4 acima de 6000h"
    );
    let exp_inc_shift2 = Rate {
        shift: 2,
        ..exp_inc
    };
    assert_eq!(
        envoltoria(0x6000, exp_inc_shift2, 1),
        vec![0x6000 + 3584],
        "em 6000h exato a condicao e level > 6000h, entao vale o passo cheio"
    );
    assert_eq!(
        envoltoria(0x6001, exp_inc_shift2, 1),
        vec![0x6001 + 896],
        "um degrau acima o passo cai a um quarto"
    );
}

#[test]
fn envoltoria_com_taxa_toda_em_um_nunca_avanca() {
    assert_eq!(
        envoltoria(0x4000, linear(3, 31, true), 64),
        vec![0x4000; 64],
        "StepValue | (ShiftValue SHL 2) = 7Fh: o contador nao recebe o minimo de 1"
    );
}

#[test]
fn envoltoria_decrescente_para_em_zero() {
    assert_eq!(envoltoria(0x4000, linear(0, 0, true), 2), vec![0, 0]);
}

#[test]
fn contador_de_pitch_gasta_28_ciclos_por_bloco_em_1000h() {
    let mut spu = Spu::new();
    carrega(&mut spu, 0x200, &bloco(0x00, 0b001, 7));
    spu.write16(V0 + 6, 0x0200);
    spu.write16(V0 + 4, 0x1000);
    spu.write16(V0 + 8, ADSR_LO_TRAVA_NO_TETO);
    spu.write16(V0 + 0x0A, ADSR_HI_TRAVA_NO_TETO);
    spu.write16(KON_LO, 1);
    for _ in 0..27 {
        spu.tick();
    }
    assert_eq!(spu.read16(ENDX_LO) & 1, 0, "o bloco tem 28 amostras");
    spu.tick();
    assert_eq!(spu.read16(ENDX_LO) & 1, 1);
}

#[test]
fn pitch_acima_de_3fffh_e_limitado_a_4000h() {
    for pitch in [0x4000u16, 0xFFFF] {
        let mut spu = Spu::new();
        carrega(&mut spu, 0x200, &bloco(0x00, 0b001, 7));
        spu.write16(V0 + 6, 0x0200);
        spu.write16(V0 + 4, pitch);
        spu.write16(V0 + 8, ADSR_LO_TRAVA_NO_TETO);
        spu.write16(V0 + 0x0A, ADSR_HI_TRAVA_NO_TETO);
        spu.write16(KON_LO, 1);
        for _ in 0..6 {
            spu.tick();
        }
        assert_eq!(spu.read16(ENDX_LO) & 1, 0, "pitch {pitch:04X}");
        spu.tick();
        assert_eq!(
            spu.read16(ENDX_LO) & 1,
            1,
            "IF Step>3FFFh then Step=4000h: 4 amostras por ciclo, 7 ciclos por bloco"
        );
    }
}

#[test]
fn mixer_aplica_volume_da_voz_e_depois_o_volume_principal() {
    let mut spu = Spu::new();
    voz_constante(&mut spu, V0, 0x200, 7, 0x1000, 0);
    spu.write16(V0, 0x3FFF);
    spu.write16(V0 + 2, 0x3FFF);
    spu.write16(MAIN_L, 0x3FFF);
    spu.write16(MAIN_R, 0x3FFF);
    spu.write16(CNT, 0xC000);
    let mut saida = (0, 0);
    for _ in 0..8 {
        saida = spu.tick();
    }
    assert_eq!(
        spu.voice_out(0),
        28558,
        "amostra 7000h interpolada (28559) vezes a envoltoria 7FFFh"
    );
    assert_eq!(
        saida,
        (28554, 28554),
        "volume fixo 3FFFh vale 7FFEh; cada etapa e (x * vol) SAR 15"
    );
}

#[test]
fn volume_fixo_e_o_dobro_do_campo_de_14_bits() {
    let mut spu = Spu::new();
    voz_constante(&mut spu, V0, 0x200, 7, 0x1000, 0);
    spu.write16(V0, 0x2000);
    spu.write16(V0 + 2, 0x3FFF);
    spu.write16(MAIN_L, 0x3FFF);
    spu.write16(MAIN_R, 0x3FFF);
    spu.write16(CNT, 0xC000);
    let mut saida = (0, 0);
    for _ in 0..8 {
        saida = spu.tick();
    }
    assert_eq!(saida.0, 14278, "2000h -> 4000h, metade de 7FFEh");
    assert_eq!(saida.1, 28554);
}

#[test]
fn spucnt_com_bit14_zerado_silencia_a_saida() {
    let mut spu = Spu::new();
    voz_constante(&mut spu, V0, 0x200, 7, 0x1000, 0);
    spu.write16(V0, 0x3FFF);
    spu.write16(V0 + 2, 0x3FFF);
    spu.write16(MAIN_L, 0x3FFF);
    spu.write16(MAIN_R, 0x3FFF);
    spu.write16(CNT, 0x8000);
    let mut saida = (0, 0);
    for _ in 0..8 {
        saida = spu.tick();
    }
    assert_eq!(
        saida,
        (0, 0),
        "§ 1F801DAAh (L659): bit14 em zero e Mute SPU"
    );
    assert_ne!(
        spu.voice_out(0),
        0,
        "a voz continua rodando por baixo do mudo"
    );
}

#[test]
fn pmon_usa_a_amplitude_da_voz_anterior_como_fator_de_passo() {
    let mut spu = Spu::new();
    voz_constante(&mut spu, V0, 0x200, 0xC, 0x1000, 0);
    voz_constante(&mut spu, V1, 0x300, 7, 0x1000, 1);
    spu.write16(PMON_LO, 0b10);
    for _ in 0..28 {
        spu.tick();
    }
    assert_eq!(
        spu.read16(ENDX_LO) & 0b1,
        0b1,
        "a voz 0 nao e modulada e fecha o bloco em 28 ciclos"
    );
    assert_eq!(
        spu.read16(ENDX_LO) & 0b10,
        0,
        "Factor = VxOUTX(0)+8000h: com a voz 0 em -4000h o passo da voz 1 cai pela metade"
    );
    for _ in 0..72 {
        spu.tick();
    }
    assert_eq!(spu.read16(ENDX_LO) & 0b10, 0b10);
}

#[test]
fn pmon_nao_afeta_a_voz_zero() {
    let mut spu = Spu::new();
    carrega(&mut spu, 0x200, &bloco(0x00, 0b001, 7));
    spu.write16(V0 + 6, 0x0200);
    spu.write16(V0 + 4, 0x1000);
    spu.write16(V0 + 8, ADSR_LO_TRAVA_NO_TETO);
    spu.write16(V0 + 0x0A, ADSR_HI_TRAVA_NO_TETO);
    spu.write16(PMON_LO, 0b1);
    spu.write16(KON_LO, 1);
    for _ in 0..28 {
        spu.tick();
    }
    assert_eq!(
        spu.read16(ENDX_LO) & 1,
        1,
        "§ 1F801D90h (L281): o bit 0 do PMON nao tem voz anterior"
    );
}

#[test]
fn cada_ciclo_empilha_um_quadro_estereo_no_anel_de_saida() {
    let mut spu = Spu::new();
    voz_constante(&mut spu, V0, 0x200, 7, 0x1000, 0);
    spu.write16(V0, 0x3FFF);
    spu.write16(V0 + 2, 0x3FFF);
    spu.write16(MAIN_L, 0x3FFF);
    spu.write16(MAIN_R, 0x3FFF);
    spu.write16(CNT, 0xC000);
    for _ in 0..10 {
        spu.tick();
    }
    let quadros = spu.drain_output();
    assert_eq!(quadros.len(), 10);
    assert_eq!(quadros[9], (28554, 28554));
    assert_eq!(spu.output_len(), 0, "drenar esvazia o anel");
}
