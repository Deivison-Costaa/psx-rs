use psx_core::spu::Spu;
use psx_core::spu::reverb::{RAM_END, Reverb};

const V0: u32 = 0x1F80_1C00;
const CURRENT_ADSR: u32 = 0x1F80_1C0C;
const MAIN_L: u32 = 0x1F80_1D80;
const MAIN_R: u32 = 0x1F80_1D82;
const VLOUT: u32 = 0x1F80_1D84;
const KON_LO: u32 = 0x1F80_1D88;
const NON_LO: u32 = 0x1F80_1D94;
const EON_LO: u32 = 0x1F80_1D98;
const MBASE: u32 = 0x1F80_1DA2;
const TRANSFER_ADDR: u32 = 0x1F80_1DA6;
const FIFO: u32 = 0x1F80_1DA8;
const CNT: u32 = 0x1F80_1DAA;
const CD_VOL_L: u32 = 0x1F80_1DB0;
const CD_VOL_R: u32 = 0x1F80_1DB2;
const REV: u32 = 0x1F80_1DC0;

const V_IIR: u32 = REV + 0x02 * 2;
const V_LIN: u32 = REV + 0x1E * 2;
const M_LSAME: u32 = REV + 0x0A * 2;
const M_LAPF2: u32 = REV + 0x1C * 2;
const D_APF2: u32 = REV + 2;

/// Area de reverb em F000h (byte 78000h), com espaco de 8000h bytes.
const BASE_EM_OITAVOS: u16 = 0xF000;
const BASE_EM_BYTES: u32 = 0x7_8000;

fn escreve_ram(spu: &mut Spu, oitavos: u16, valores: &[u16]) {
    spu.write16(TRANSFER_ADDR, oitavos);
    for v in valores {
        spu.write16(FIFO, *v);
    }
    spu.write16(CNT, 1 << 4);
    spu.write16(CNT, 0);
}

fn bloco_constante(nibble: u8, flags: u8) -> [u8; 16] {
    let mut b = [0u8; 16];
    b[1] = flags;
    for byte in b.iter_mut().skip(2) {
        *byte = (nibble & 0x0F) | ((nibble & 0x0F) << 4);
    }
    b
}

fn carrega_bloco(spu: &mut Spu, oitavos: u16, bytes: &[u8]) {
    spu.write16(TRANSFER_ADDR, oitavos);
    for par in bytes.chunks(2) {
        spu.write16(FIFO, u16::from_le_bytes([par[0], par[1]]));
    }
    spu.write16(CNT, 1 << 4);
    spu.write16(CNT, 0);
}

#[test]
fn os_32_registradores_de_reverb_leem_de_volta() {
    let mut spu = Spu::new();
    for i in 0..32u32 {
        spu.write16(REV + i * 2, 0x1000 + i as u16);
    }
    for i in 0..32u32 {
        assert_eq!(spu.read16(REV + i * 2), 0x1000 + i as u16, "rev{i:02X}");
    }
    spu.write16(VLOUT, 0x4321);
    assert_eq!(spu.read16(VLOUT), 0x4321);
}

#[test]
fn mbase_reposiciona_o_endereco_corrente() {
    let mut r = Reverb::default();
    r.set_mbase(BASE_EM_OITAVOS);
    assert_eq!(
        r.current, BASE_EM_BYTES,
        "escrever mBASE tambem move o endereco corrente para la"
    );
}

#[test]
fn endereco_corrente_anda_de_duas_em_duas_e_da_a_volta_no_topo() {
    let mut r = Reverb::default();
    // Area minima: FFFEh -> 7FFF0h, oito meias-palavras ate o teto fixo de 7FFFEh.
    r.set_mbase(0xFFFE);
    assert_eq!(r.current, RAM_END - 0x10);
    for esperado in 1..8u32 {
        r.advance();
        assert_eq!(r.current, RAM_END - 0x10 + esperado * 2);
    }
    r.advance();
    assert_eq!(
        r.current,
        RAM_END - 0x10,
        "BufferAddress = MAX(mBASE, (BufferAddress+2) AND 7FFFEh)"
    );
}

#[test]
fn reflexao_do_mesmo_lado_grava_lin_vezes_viir_no_buffer() {
    let mut spu = Spu::new();
    spu.write16(MBASE, BASE_EM_OITAVOS);
    spu.write16(V_LIN, 0x7FFF);
    spu.write16(V_IIR, 0x7FFF);
    spu.write16(M_LSAME, 0x20);
    spu.write16(CD_VOL_L, 0x7FFF);
    spu.set_cd_audio(0x4000, 0);
    spu.write16(CNT, 0xC085);
    spu.tick();
    assert_eq!(
        spu.ram_peek16(BASE_EM_BYTES + 0x100),
        16381,
        "Lin = 7FFFh*4000h/8000h = 16382; [mLSAME] = 16382*7FFFh/8000h"
    );
}

#[test]
fn bit7_do_spucnt_corta_a_escrita_no_buffer_mas_nao_a_leitura() {
    let mut spu = Spu::new();
    escreve_ram(&mut spu, 0xF008, &[0x1234]);
    spu.write16(MBASE, BASE_EM_OITAVOS);
    spu.write16(V_LIN, 0x7FFF);
    spu.write16(V_IIR, 0x7FFF);
    spu.write16(M_LSAME, 0x20);
    spu.write16(M_LAPF2, 0x10);
    spu.write16(D_APF2, 0x08);
    spu.write16(VLOUT, 0x7FFF);
    spu.write16(MAIN_L, 0x3FFF);
    spu.write16(CD_VOL_L, 0x7FFF);
    spu.set_cd_audio(0x4000, 0);
    spu.write16(CNT, 0xC005);
    let (esquerda, _) = spu.tick();
    assert_eq!(
        spu.ram_peek16(BASE_EM_BYTES + 0x100),
        0,
        "com o bit7 zerado o SPU para de escrever no buffer de reverb"
    );
    assert_ne!(esquerda, 0, "mas continua lendo: Lout = [mLAPF2-dAPF2]");
}

#[test]
fn apf2_com_volume_zero_devolve_o_que_esta_no_buffer_vezes_vlout() {
    let mut spu = Spu::new();
    escreve_ram(&mut spu, 0xF008, &[0x1234]);
    spu.write16(MBASE, BASE_EM_OITAVOS);
    spu.write16(M_LAPF2, 0x10);
    spu.write16(D_APF2, 0x08);
    spu.write16(VLOUT, 0x7FFF);
    spu.write16(MAIN_L, 0x3FFF);
    spu.write16(CNT, 0xC080);
    let (esquerda, direita) = spu.tick();
    assert_eq!(
        esquerda, 4658,
        "vAPF2=0 -> Lout = [78040h] = 4660; depois vLOUT e o volume principal"
    );
    assert_eq!(direita, 0, "vROUT ficou em zero");
}

#[test]
fn ruido_avanca_como_lfsr_de_paridade() {
    let mut spu = Spu::new();
    // Shift 0Fh recarrega o temporizador com 4, entao o passo 4 anda todo ciclo.
    spu.write16(CNT, 0xC000 | (0x0F << 10));
    let esperado = [
        1i16, 3, 7, 15, 31, 63, 127, 255, 511, 1023, 2047, 4094, 8189, 16378, 32756, -24,
    ];
    let medido: Vec<i16> = esperado
        .iter()
        .map(|_| {
            spu.tick();
            spu.noise_level()
        })
        .collect();
    assert_eq!(
        medido,
        esperado.to_vec(),
        "ParityBit = Bit15 xor Bit12 xor Bit11 xor Bit10 xor 1"
    );
}

#[test]
fn passo_do_ruido_vem_dos_bits_9_8_do_spucnt() {
    // Shift 0Dh recarrega com 16: o passo (4 ou 7) decide de quantos em quantos ciclos
    // o LFSR anda. E a unica coisa que os bits 9-8 controlam.
    let mut lento = Spu::new();
    lento.write16(CNT, 0xC000 | (0x0D << 10));
    let mut rapido = Spu::new();
    rapido.write16(CNT, 0xC000 | (0x0D << 10) | (3 << 8));
    let colhe = |spu: &mut Spu| -> Vec<i16> {
        (0..10)
            .map(|_| {
                spu.tick();
                spu.noise_level()
            })
            .collect()
    };
    assert_eq!(colhe(&mut lento), vec![1, 1, 1, 1, 3, 3, 3, 3, 7, 7]);
    assert_eq!(colhe(&mut rapido), vec![1, 1, 3, 3, 7, 7, 15, 15, 15, 31]);
}

#[test]
fn non_troca_a_amostra_da_voz_pelo_ruido() {
    let mut spu = Spu::new();
    carrega_bloco(&mut spu, 0x200, &bloco_constante(7, 0b111));
    spu.write16(V0 + 6, 0x0200);
    spu.write16(V0 + 4, 0x1000);
    spu.write16(NON_LO, 1);
    spu.write16(CNT, 0xC000 | (0x0F << 10));
    spu.write16(CURRENT_ADSR, 0x7FFF);
    for _ in 0..15 {
        spu.tick();
    }
    assert_eq!(spu.noise_level(), 32756);
    assert_eq!(
        spu.voice_out(0),
        32755,
        "com NON a voz entrega o nivel do ruido, nao a amostra ADPCM de 7000h"
    );
}

#[test]
fn eon_manda_a_saida_da_voz_para_o_reverb() {
    let mut medidas = Vec::new();
    for eon in [0u16, 1] {
        let mut spu = Spu::new();
        carrega_bloco(&mut spu, 0x200, &bloco_constante(7, 0b111));
        spu.write16(V0 + 6, 0x0200);
        spu.write16(V0 + 4, 0x1000);
        spu.write16(V0, 0x3FFF);
        spu.write16(V0 + 8, 0x000F);
        spu.write16(V0 + 0x0A, 0x1FC0);
        spu.write16(MBASE, BASE_EM_OITAVOS);
        spu.write16(V_LIN, 0x7FFF);
        spu.write16(V_IIR, 0x7FFF);
        spu.write16(M_LSAME, 0x20);
        spu.write16(EON_LO, eon);
        spu.write16(CNT, 0xC080);
        spu.write16(KON_LO, 1);
        for _ in 0..8 {
            spu.tick();
        }
        medidas.push(spu.ram_peek16(BASE_EM_BYTES + 0x106));
    }
    assert_eq!(medidas[0], 0, "sem EON a voz nao alimenta o reverb");
    assert_ne!(
        medidas[1], 0,
        "§ 1F801D98h (L924): EON manda a voz ao reverb"
    );
}

#[test]
fn volume_de_cd_entra_no_mixer_so_com_o_bit0_do_spucnt() {
    let mut resultados = Vec::new();
    for cnt in [0xC000u16, 0xC001] {
        let mut spu = Spu::new();
        spu.write16(CD_VOL_L, 0x7FFF);
        spu.write16(CD_VOL_R, 0x7FFF);
        spu.write16(MAIN_L, 0x3FFF);
        spu.write16(MAIN_R, 0x3FFF);
        spu.set_cd_audio(0x4000, 0x2000);
        spu.write16(CNT, cnt);
        resultados.push(spu.tick());
    }
    assert_eq!(resultados[0], (0, 0), "bit0 zerado: CD fora do mixer");
    assert_eq!(
        resultados[1],
        (16382, 8190),
        "§ 1F801DB0h (L498): volume de CD e signed de 16 bits, sem o meio-passo do canal"
    );
}
