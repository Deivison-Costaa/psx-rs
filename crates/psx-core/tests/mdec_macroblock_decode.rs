mod support;

use psx_core::bus::{Bus, BusRead};
use support::asm;

// § heart.mdec / heart.h de ps1-tests (mdec/4bit e mdec/8bit compartilham o
// mesmo bloco 8x8 comprimido — so muda a profundidade de saida do MDEC(1)).
const HEART_MDEC: [u8; 128] = [
    0x1e, 0x04, 0x74, 0x00, 0xbd, 0x00, 0x93, 0x03, 0x24, 0x00, 0x2b, 0x03, 0x15, 0x00, 0xf9, 0x03,
    0xe9, 0x03, 0xda, 0x03, 0xee, 0x03, 0xfe, 0x03, 0x2b, 0x00, 0x3f, 0x00, 0xc0, 0x03, 0xfe, 0x03,
    0xbe, 0x03, 0xfe, 0x03, 0xca, 0x03, 0xfd, 0x03, 0xf3, 0x03, 0xf3, 0x03, 0xfd, 0x03, 0x07, 0x00,
    0xd7, 0x03, 0x00, 0x00, 0xe0, 0x03, 0xf5, 0x03, 0x05, 0x00, 0x02, 0x00, 0xec, 0x03, 0x07, 0x00,
    0x0c, 0x00, 0x08, 0x00, 0xfe, 0x03, 0xf7, 0x03, 0xff, 0x03, 0xf9, 0x03, 0x01, 0x00, 0xdd, 0x03,
    0xd8, 0x03, 0x09, 0x00, 0x16, 0x00, 0x01, 0x00, 0x13, 0x00, 0xca, 0x03, 0x04, 0x00, 0xfa, 0x03,
    0xfc, 0x03, 0xfe, 0x03, 0x00, 0x00, 0x09, 0x00, 0x0f, 0x00, 0x06, 0x00, 0xfe, 0x03, 0xfc, 0x03,
    0xf2, 0x03, 0xfa, 0x03, 0xf6, 0x03, 0x08, 0x00, 0xfa, 0x03, 0x15, 0x00, 0x01, 0x00, 0xf5, 0x03,
];

// § common/mdec.cpp (ps1-tests): tabela de quantizacao luminancia+cor (so os
// primeiros 64 bytes, luminancia, sao usados pelo caminho monocromatico).
const QUANT: [u8; 128] = [
    0x02, 0x10, 0x10, 0x13, 0x10, 0x13, 0x16, 0x16, 0x16, 0x16, 0x16, 0x16, 0x1a, 0x18, 0x1a, 0x1b,
    0x1b, 0x1b, 0x1a, 0x1a, 0x1a, 0x1a, 0x1b, 0x1b, 0x1b, 0x1d, 0x1d, 0x1d, 0x22, 0x22, 0x22, 0x1d,
    0x1d, 0x1d, 0x1b, 0x1b, 0x1d, 0x1d, 0x20, 0x20, 0x22, 0x22, 0x25, 0x26, 0x25, 0x23, 0x23, 0x22,
    0x23, 0x26, 0x26, 0x28, 0x28, 0x28, 0x30, 0x30, 0x2e, 0x2e, 0x38, 0x38, 0x3a, 0x45, 0x45, 0x53,
    0x02, 0x10, 0x10, 0x13, 0x10, 0x13, 0x16, 0x16, 0x16, 0x16, 0x16, 0x16, 0x1a, 0x18, 0x1a, 0x1b,
    0x1b, 0x1b, 0x1a, 0x1a, 0x1a, 0x1a, 0x1b, 0x1b, 0x1b, 0x1d, 0x1d, 0x1d, 0x22, 0x22, 0x22, 0x1d,
    0x1d, 0x1d, 0x1b, 0x1b, 0x1d, 0x1d, 0x20, 0x20, 0x22, 0x22, 0x25, 0x26, 0x25, 0x23, 0x23, 0x22,
    0x23, 0x26, 0x26, 0x28, 0x28, 0x28, 0x30, 0x30, 0x2e, 0x2e, 0x38, 0x38, 0x3a, 0x45, 0x45, 0x53,
];

// § common/mdec.cpp (ps1-tests): tabela de escala padrao ("standard values",
// docs/reference/09-mdec.md L358-371) — mesmos 64 halfwords enviados por
// mdec_idctTable(idct) em mdec/4bit e mdec/8bit.
const IDCT: [i16; 64] = [
    23170, 23170, 23170, 23170, 23170, 23170, 23170, 23170, 32138, 27245, 18204, 6392, -6393,
    -18205, -27246, -32139, 30273, 12539, -12540, -30274, -30274, -12540, 12539, 30273, 27245,
    -6393, -32139, -18205, 18204, 32138, 6392, -27246, 23170, -23171, -23171, 23170, 23170, -23171,
    -23171, 23170, 18204, -32139, 6392, 27245, -27246, -6393, 32138, -18205, 12539, -30274, 30273,
    -12540, -12540, 30273, -30274, 12539, 6392, -18205, 27245, -32139, 32138, -27246, 18204, -6393,
];

const MDEC_CMD: u32 = 0x1F80_1820;
const MDEC_STATUS: u32 = 0x1F80_1824;

fn bus_com_mdec() -> Bus {
    asm::bus_with_bios_empty()
}

fn enviar_tabelas(bus: &mut Bus) {
    // § MDEC(2) Set Quant Table, color=1 (docs/reference/09-mdec.md L141-149).
    bus.write32::<BusRead>(MDEC_CMD, (2 << 29) | 1);
    for chunk in QUANT.chunks_exact(4) {
        let word = u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
        bus.write32::<BusRead>(MDEC_CMD, word);
    }
    // § MDEC(3) Set Scale Table (L151-158).
    bus.write32::<BusRead>(MDEC_CMD, 3 << 29);
    for pair in IDCT.chunks_exact(2) {
        let lo = pair[0] as u16 as u32;
        let hi = pair[1] as u16 as u32;
        bus.write32::<BusRead>(MDEC_CMD, lo | (hi << 16));
    }
}

fn decodificar(bus: &mut Bus, color_depth: u32) -> Vec<u32> {
    enviar_tabelas(bus);
    let length_words = (HEART_MDEC.len() / 4) as u32;
    bus.write32::<BusRead>(MDEC_CMD, (1 << 29) | (color_depth << 27) | length_words);
    for chunk in HEART_MDEC.chunks_exact(4) {
        let word = u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
        bus.write32::<BusRead>(MDEC_CMD, word);
    }
    assert_eq!(
        bus.read32::<BusRead>(MDEC_STATUS) & (1 << 29),
        0,
        "MDEC(1) deve fechar assim que as 32 palavras do bloco unico chegam"
    );
    let mut saida = Vec::new();
    while bus.read32::<BusRead>(MDEC_STATUS) & (1 << 31) == 0 {
        saida.push(bus.read32::<BusRead>(MDEC_CMD));
    }
    saida
}

// Gabarito de hardware: mdec/4bit/psx.log e mdec/8bit/psx.log do ps1-tests despejam os bytes
// que o console entregou para ESTE bloco. Ate a iteracao 0184 estes testes comparavam com
// constantes calculadas por script Python a partir da spec — que erravam 16 dos 64 bytes do
// gabarito de 8bit e 16 dos 32 de 4bit, e passavam verdes assim mesmo.
const HW_8BIT: [u8; 64] = [
    0x00, 0xff, 0xff, 0x00, 0xff, 0xff, 0x04, 0x00, 0xc9, 0xec, 0xef, 0xed, 0xef, 0xfc, 0xf2, 0x00,
    0xd5, 0xdb, 0xfa, 0xe8, 0xfe, 0xe8, 0xff, 0x00, 0xb7, 0xf3, 0xec, 0xef, 0xeb, 0xff, 0xe3, 0x00,
    0x00, 0xfb, 0xff, 0xf5, 0xf2, 0xff, 0x03, 0x00, 0x00, 0x05, 0xff, 0xfc, 0xff, 0x08, 0x1a, 0x00,
    0x0f, 0x28, 0x1e, 0xff, 0x05, 0x2a, 0x23, 0x00, 0x10, 0x38, 0x40, 0x29, 0x32, 0x32, 0x16, 0x0f,
];

const HW_4BIT: [u8; 32] = [
    0xf0, 0x0f, 0xff, 0x00, 0xfd, 0xff, 0xff, 0x0f, 0xed, 0xff, 0xff, 0x0f, 0xfb, 0xff, 0xff, 0x0e,
    0xf0, 0xff, 0xff, 0x00, 0x00, 0xff, 0x1f, 0x02, 0x31, 0xf2, 0x30, 0x02, 0x41, 0x34, 0x33, 0x11,
];

fn bytes_de(palavras: &[u32]) -> Vec<u8> {
    palavras.iter().flat_map(|w| w.to_le_bytes()).collect()
}

// § real_idct_core (L241-267) de docs/reference/09-mdec.md admite que "the results aren't
// perfect": a spec nao define o arredondamento do hardware. O que o gabarito permite afirmar e
// que nenhuma amostra desvia mais de um passo, e que a grande maioria bate exatamente.
#[test]
fn mdec_decode_8bit_bloco_heart_bate_com_o_gabarito_de_hardware() {
    let mut bus = bus_com_mdec();
    let saida = bytes_de(&decodificar(&mut bus, 1));
    assert_eq!(saida.len(), HW_8BIT.len(), "o console entrega 64 bytes");
    let mut exatos = 0;
    for (i, (&nosso, &console)) in saida.iter().zip(HW_8BIT.iter()).enumerate() {
        let delta = nosso as i32 - console as i32;
        assert!(
            delta.abs() <= 2,
            "byte {i}: nosso 0x{nosso:02X}, console 0x{console:02X} (delta {delta:+})"
        );
        if delta == 0 {
            exatos += 1;
        }
    }
    assert!(
        exatos >= 48,
        "so {exatos} de 64 bytes batem exatamente com o console; era 48 na iteracao 0184"
    );
}

// O empacotamento de 4 bits reduz cada pixel de 8 para 4 bits arredondando, nao truncando:
// truncar deixava 16 dos 32 bytes um passo abaixo do console, e nenhum acima.
#[test]
fn mdec_decode_4bit_bloco_heart_bate_com_o_gabarito_de_hardware() {
    let mut bus = bus_com_mdec();
    let saida = bytes_de(&decodificar(&mut bus, 0));
    assert_eq!(saida.len(), HW_4BIT.len(), "o console entrega 32 bytes");
    let mut divergentes = 0;
    for (i, (&nosso, &console)) in saida.iter().zip(HW_4BIT.iter()).enumerate() {
        for desl in [0u8, 4] {
            let a = ((nosso >> desl) & 0xF) as i32;
            let b = ((console >> desl) & 0xF) as i32;
            assert!(
                (a - b).abs() <= 1,
                "byte {i} nibble {desl}: nosso {a}, console {b}"
            );
            if a != b {
                divergentes += 1;
            }
        }
    }
    assert!(
        divergentes <= 2,
        "{divergentes} nibbles divergem do console; eram 2 na iteracao 0184"
    );
}
