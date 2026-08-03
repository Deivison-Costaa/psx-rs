mod support;

use psx_core::bus::{Bus, BusRead};
use support::asm;

// Gabarito de hardware real: mdec/step-by-step-log do ps1-tests alimenta o MDEC a mao com
// 256 palavras e registra cada leitura de 1F801820h no psx.log. ENTRADA sao as linhas
// "MDEC_DATA <- 0x........" e SAIDA as "MDEC_DATA -> 0x........", na ordem em que o console
// as produziu. Nao e derivado de nenhuma implementacao nossa (R1).

const ENTRADA: [u32; 256] = [
    0xF8000400, 0xF8000400, 0xF8000600, 0x07ED07FE, 0x03FB0024, 0x041D0031, 0x00230006, 0x00040035,
    0x000803B9, 0x07E6007F, 0x03D103F6, 0x03E303F2, 0x00050043, 0x03F903A8, 0x03FF0392, 0x04120056,
    0x00250002, 0x001E001A, 0x0011000A, 0x03C003F5, 0x03CB03F8, 0x03B90001, 0x000803FA, 0x03FB03E8,
    0x03F003F2, 0x001003F7, 0x000203D9, 0x000603C9, 0x000303ED, 0x03F70003, 0x000703FD, 0x000C0008,
    0x03F7000C, 0x0118078E, 0x03640044, 0x0015002B, 0x03FD0015, 0x03DD03CD, 0x00170009, 0x00020040,
    0x000E0010, 0x000F0001, 0x03D1002D, 0x03FB0007, 0x03DB03E2, 0x000D0005, 0x00060001, 0x03FF0028,
    0x0004000A, 0x00020014, 0x03F60005, 0x03F70009, 0x00090023, 0x001A0003, 0x03F60007, 0x000D0010,
    0x03F4000C, 0x03FD03F4, 0x00190003, 0x000503F3, 0x000103F8, 0x001F03F8, 0x03FD03FE, 0x000103FA,
    0x000103FC, 0x0384078E, 0x03640044, 0x036E03F0, 0x03E50060, 0x03DD03EF, 0x03D80009, 0x00110054,
    0x00530390, 0x03F103F2, 0x00340014, 0x03FB0007, 0x000C001D, 0x000D0012, 0x03D4000D, 0x03FB003F,
    0x03F703F6, 0x002003DD, 0x03F603FB, 0x03F203F6, 0x000903F0, 0x03EC0004, 0x03FD000A, 0x000803EF,
    0x03FC03FB, 0x03E703F2, 0x00140003, 0x000E0004, 0x03EC03F2, 0x03FF03FD, 0x0008000F, 0x03FF03FE,
    0x00020004, 0xF8000400, 0xF8000400, 0xF8000600, 0x043206BF, 0x070603F2, 0x07FC07BE, 0x04BF0814,
    0x0405042B, 0x03E50410, 0x07B50FF1, 0x07FD07E9, 0x041007ED, 0x0C080420, 0x040F0802, 0x07E807EF,
    0x040D13F8, 0x0BFA040B, 0xFE000000, 0x006E043E, 0x03B60066, 0x03CA0008, 0x03E4007F, 0x001603EA,
    0x000C03DD, 0x002A03B3, 0x00040361, 0x03DD0005, 0x000D03FF, 0x03E8002A, 0x03FD0012, 0x003303FA,
    0x03DE03F8, 0x03E803D5, 0x03DC03BB, 0x03BC03BF, 0x000D03F7, 0x00100003, 0x03EE03D7, 0x000403EB,
    0x00140408, 0x00060015, 0x03E9000B, 0x000E03F8, 0x002403FB, 0x000403FE, 0x000B000A, 0x03F60003,
    0x03E203F4, 0x000303F6, 0xFE000002, 0x000D072F, 0x000103F7, 0x02D4000B, 0x03CF03ED, 0x001E0004,
    0x03FE03C2, 0x03ED001F, 0x0004008F, 0x03F9003E, 0x03F903E6, 0x03C9002B, 0x001E03F8, 0x03CC0003,
    0x03B70003, 0x0401000E, 0x000D000B, 0x03F903F7, 0x03FD0010, 0x000D000E, 0x03FF001A, 0x000D0014,
    0x03F40006, 0x03EE03FE, 0x03FF000C, 0x00230005, 0x03FE03FD, 0x03F703FD, 0x03FE03ED, 0x03FF03F8,
    0x03F403FA, 0x03FB03F8, 0xFE0003FD, 0x0061046B, 0x04D803D5, 0x080C07E3, 0x07B407C6, 0x13F90413,
    0x04140421, 0x07F2042E, 0x03EB1804, 0x07F407F6, 0x180607DD, 0x040B0407, 0x07F913FD, 0xFE000802,
    0x004D0454, 0x07FF0320, 0x084007E9, 0x13DE140F, 0x18151FF5, 0xFE006C00, 0xF8000731, 0xF8000457,
    0xF8000674, 0xF80007A5, 0x00860795, 0x045903BA, 0x081407D8, 0x07E107E8, 0x13F5041B, 0x0408040D,
    0x07ED0413, 0x03F81807, 0x07FB07FC, 0x180207F2, 0x04050403, 0x07FD13FF, 0xFE000801, 0x036507AC,
    0x079A03C8, 0x0810042E, 0x0424041B, 0x13F807E1, 0x07F607F1, 0x041607EA, 0x000A1805, 0x04050405,
    0x1BFD0411, 0x07FB07FC, 0x04031002, 0xFE000BFF, 0xF8000588, 0xF80004CB, 0xF8000600, 0xF80005FC,
    0xFE00FE00, 0xFE00FE00, 0xFE00FE00, 0xFE00FE00, 0xFE00FE00, 0xFE00FE00, 0xFE00FE00, 0xFE00FE00,
    0xFE00FE00, 0xFE00FE00, 0xFE00FE00, 0xFE00FE00, 0xFE00FE00, 0xFE00FE00, 0xFE00FE00, 0xFE00FE00,
    0xFE00FE00, 0xFE00FE00, 0xFE00FE00, 0xFE00FE00, 0xFE00FE00, 0xFE00FE00, 0xFE00FE00, 0xFE00FE00,
];

const SAIDA: [u32; 512] = [
    0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000,
    0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000,
    0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000,
    0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000,
    0x7BDE0000, 0x77BD6F7B, 0x739C7BDE, 0x00007BDE, 0x00007FFF, 0x18C60842, 0x10841084, 0x7FFF0000,
    0x00007FFF, 0x00007FFF, 0x739C0421, 0x7FFF0000, 0x00007FFF, 0x1CE70000, 0x14A518C6, 0x7FFF0000,
    0x00007FFF, 0x7FFF0000, 0x77BD77BD, 0x7FFF0000, 0x00007FFF, 0x6F7B7BDE, 0x00007FFF, 0x7FFF0000,
    0x00007FFF, 0x14A51084, 0x000018C6, 0x7FFF0000, 0x7BDE0000, 0x7FFF6739, 0x6F7B7FFF, 0x00007BDE,
    0x00007FFF, 0x0C631084, 0x08420842, 0x08420C63, 0x739C7FFF, 0x00007FFF, 0x00001084, 0x04210000,
    0x7BDE7FFF, 0x6B5A7BDE, 0x00007BDE, 0x00000842, 0x6F7B739C, 0x7BDE739C, 0x631877BD, 0x00006F7B,
    0x77BD77BD, 0x6B5A77BD, 0x00007FFF, 0x00000842, 0x77BD7FFF, 0x00007FFF, 0x00001084, 0x00000000,
    0x08427FFF, 0x084218C6, 0x04210421, 0x08420842, 0x10842529, 0x10841084, 0x08421084, 0x08420842,
    0x0C630842, 0x084214A5, 0x08421084, 0x00007FFF, 0x00000000, 0x000018C6, 0x7FFF7FFF, 0x00007FFF,
    0x00000000, 0x67397FFF, 0x7FFF7BDE, 0x00007FFF, 0x63185294, 0x7BDE7FFF, 0x77BD6F7B, 0x00007FFF,
    0x00000000, 0x67397FFF, 0x7FFF77BD, 0x00007FFF, 0x00000000, 0x000018C6, 0x7BDE7FFF, 0x00007FFF,
    0x08420421, 0x04211084, 0x0C6318C6, 0x00007FFF, 0x10840421, 0x0C6318C6, 0x18C60842, 0x04212108,
    0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000,
    0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000,
    0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000,
    0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000,
    0x00000421, 0x7FFF0000, 0x00007FFF, 0x04210000, 0x00000421, 0x7FFF0000, 0x00007FFF, 0x04210000,
    0x00000000, 0x7FFF0000, 0x00007FFF, 0x00000000, 0x00000421, 0x7FFF0000, 0x00007FFF, 0x04210000,
    0x00000421, 0x7FFF0000, 0x00007FFF, 0x04210000, 0x00000842, 0x2D6B0000, 0x00002D6B, 0x08420000,
    0x00000C63, 0x7FFF0000, 0x00007FFF, 0x0C630000, 0x00000842, 0x21080000, 0x00002108, 0x08420000,
    0x7FFF0000, 0x77BD77BD, 0x7FFF739C, 0x00000000, 0x7FFF6739, 0x00000C63, 0x7FFF7FFF, 0x00007FFF,
    0x7FFF739C, 0x77BD0000, 0x7FFF6318, 0x00007FFF, 0x7FFF6F7B, 0x7FFF7BDE, 0x7FFF0000, 0x00007FFF,
    0x7BDE77BD, 0x00007FFF, 0x7FFF0000, 0x00007FFF, 0x7FFF6F7B, 0x10840842, 0x7FFF0000, 0x00007FFF,
    0x7FFF0000, 0x6F7B6F7B, 0x7FFF6739, 0x00000842, 0x25290000, 0x0C632D6B, 0x210814A5, 0x000014A5,
    0x00000000, 0x7FFF0000, 0x00007FFF, 0x04210000, 0x00000000, 0x7FFF7FFF, 0x00007FFF, 0x00000000,
    0x00000000, 0x7FFF0000, 0x00007FFF, 0x00000000, 0x00000421, 0x7FFF0000, 0x00007FFF, 0x04210000,
    0x00000421, 0x7FFF0000, 0x00007FFF, 0x04210000, 0x00000000, 0x7FFF0000, 0x00007FFF, 0x00000000,
    0x6F7B0000, 0x7FFF6739, 0x63187FFF, 0x00006B5A, 0x00000842, 0x1CE70421, 0x04212108, 0x04210000,
    0x145B145B, 0x143C143C, 0x141D141D, 0x1C1F1C1F, 0x145B145B, 0x143C143C, 0x141D141D, 0x1C1F1C1F,
    0x185A185A, 0x183B183B, 0x181E181E, 0x1C1F1C1F, 0x185A185A, 0x183B183B, 0x181E181E, 0x1C1F1C1F,
    0x105B105B, 0x103D103D, 0x103D103D, 0x141F141F, 0x105B105B, 0x103D103D, 0x103D103D, 0x141F141F,
    0x143C143C, 0x143D143D, 0x143C143C, 0x1C1F1C1F, 0x143C143C, 0x143D143D, 0x143C143C, 0x1C1F1C1F,
    0x13E013E0, 0x17A517A5, 0x17851785, 0x1B671B67, 0x13E013E0, 0x17A517A5, 0x17851785, 0x1B671B67,
    0x13E013E0, 0x1BA41BA4, 0x1B861B86, 0x1B671B67, 0x13E013E0, 0x1BA41BA4, 0x1B861B86, 0x1B671B67,
    0x0BE00BE0, 0x13A413A4, 0x13A513A5, 0x13861386, 0x0BE00BE0, 0x13A413A4, 0x13A513A5, 0x13861386,
    0x13E013E0, 0x17861786, 0x17851785, 0x17861786, 0x13E013E0, 0x17861786, 0x17851785, 0x17861786,
    0x68046804, 0x68036803, 0x68046804, 0x70017001, 0x68046804, 0x68036803, 0x68046804, 0x70017001,
    0x70047004, 0x70037003, 0x70037003, 0x74207420, 0x70047004, 0x70037003, 0x70037003, 0x74207420,
    0x68056805, 0x68046804, 0x68026802, 0x70207020, 0x68056805, 0x68046804, 0x68026802, 0x70207020,
    0x68056805, 0x68046804, 0x68036803, 0x70407040, 0x68056805, 0x68046804, 0x68036803, 0x70407040,
    0x645E645E, 0x6C9A6C9A, 0x6C7B6C7B, 0x6C9A6C9A, 0x645E645E, 0x6C9A6C9A, 0x6C7B6C7B, 0x6C9A6C9A,
    0x6C3F6C3F, 0x707C707C, 0x707B707B, 0x709A709A, 0x6C3F6C3F, 0x707C707C, 0x707B707B, 0x709A709A,
    0x645E645E, 0x687C687C, 0x689A689A, 0x68B968B9, 0x645E645E, 0x687C687C, 0x689A689A, 0x68B968B9,
    0x643F643F, 0x687B687B, 0x689B689B, 0x68996899, 0x643F643F, 0x687B687B, 0x689B689B, 0x68996899,
    0x17FF17FF, 0x17FF17FF, 0x13FF13FF, 0x03DF03DF, 0x17FF17FF, 0x17FF17FF, 0x13FF13FF, 0x03DF03DF,
    0x17FF17FF, 0x17FF17FF, 0x0FFF0FFF, 0x03DF03DF, 0x17FF17FF, 0x17FF17FF, 0x0FFF0FFF, 0x03DF03DF,
    0x13FF13FF, 0x0FFF0FFF, 0x0FFF0FFF, 0x03DF03DF, 0x13FF13FF, 0x0FFF0FFF, 0x0FFF0FFF, 0x03DF03DF,
    0x17FF17FF, 0x17FF17FF, 0x13FF13FF, 0x03DF03DF, 0x17FF17FF, 0x17FF17FF, 0x13FF13FF, 0x03DF03DF,
    0x7FE07FE0, 0x7FE37FE3, 0x7BC47BC4, 0x7BC47BC4, 0x7FE07FE0, 0x7FE37FE3, 0x7BC47BC4, 0x7BC47BC4,
    0x7FE07FE0, 0x7FE37FE3, 0x7BC47BC4, 0x77C477C4, 0x7FE07FE0, 0x7FE37FE3, 0x7BC47BC4, 0x77C477C4,
    0x7FE07FE0, 0x7FE37FE3, 0x7BE37BE3, 0x77E377E3, 0x7FE07FE0, 0x7FE37FE3, 0x7BE37BE3, 0x77E377E3,
    0x7FE07FE0, 0x7BC47BC4, 0x7BE37BE3, 0x7BC37BC3, 0x7FE07FE0, 0x7BC47BC4, 0x7BE37BE3, 0x7BC37BC3,
    0x00010001, 0x00200020, 0x00010001, 0x00010001, 0x00010001, 0x00200020, 0x00010001, 0x00010001,
    0x00010001, 0x00000000, 0x00000000, 0x00200020, 0x00010001, 0x00000000, 0x00000000, 0x00200020,
    0x00210021, 0x00200020, 0x00200020, 0x00200020, 0x00210021, 0x00200020, 0x00200020, 0x00200020,
    0x00010001, 0x00200020, 0x00200020, 0x00200020, 0x00010001, 0x00200020, 0x00200020, 0x00200020,
    0x7FFD7FFD, 0x7FFC7FFC, 0x7FFD7FFD, 0x7FFD7FFD, 0x7FFD7FFD, 0x7FFC7FFC, 0x7FFD7FFD, 0x7FFD7FFD,
    0x7FFE7FFE, 0x7FFE7FFE, 0x7FFE7FFE, 0x7FFD7FFD, 0x7FFE7FFE, 0x7FFE7FFE, 0x7FFE7FFE, 0x7FFD7FFD,
    0x7FFD7FFD, 0x7FFD7FFD, 0x7FFD7FFD, 0x7FFC7FFC, 0x7FFD7FFD, 0x7FFD7FFD, 0x7FFD7FFD, 0x7FFC7FFC,
    0x7FFE7FFE, 0x7FFD7FFD, 0x7FFD7FFD, 0x7FFC7FFC, 0x7FFE7FFE, 0x7FFD7FFD, 0x7FFD7FFD, 0x7FFC7FFC,
];

// § common/mdec.cpp (ps1-tests): mesma tabela de quantizacao de mdec/4bit e mdec/8bit,
// enviada com color=1 (64 bytes de luminancia seguidos de 64 de cor).
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

// § common/mdec.cpp (ps1-tests): tabela de escala padrao, os mesmos 64 halfwords de
// mdec_idctTable().
const IDCT: [i16; 64] = [
    23170, 23170, 23170, 23170, 23170, 23170, 23170, 23170, 32138, 27245, 18204, 6392, -6393,
    -18205, -27246, -32139, 30273, 12539, -12540, -30274, -30274, -12540, 12539, 30273, 27245,
    -6393, -32139, -18205, 18204, 32138, 6392, -27246, 23170, -23171, -23171, 23170, 23170, -23171,
    -23171, 23170, 18204, -32139, 6392, 27245, -27246, -6393, 32138, -18205, 12539, -30274, 30273,
    -12540, -12540, 30273, -30274, 12539, 6392, -18205, 27245, -32139, 32138, -27246, 18204, -6393,
];

const MDEC_CMD: u32 = 0x1F80_1820;
const MDEC_STATUS: u32 = 0x1F80_1824;

fn enviar_tabelas(bus: &mut Bus) {
    bus.write32::<BusRead>(MDEC_CMD, (2 << 29) | 1);
    for chunk in QUANT.chunks_exact(4) {
        bus.write32::<BusRead>(
            MDEC_CMD,
            u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]),
        );
    }
    bus.write32::<BusRead>(MDEC_CMD, 3 << 29);
    for pair in IDCT.chunks_exact(2) {
        let lo = pair[0] as u16 as u32;
        let hi = pair[1] as u16 as u32;
        bus.write32::<BusRead>(MDEC_CMD, lo | (hi << 16));
    }
}

/// Alimenta as 256 palavras do gabarito e drena a fifo de saida. `profundidade` vai nos
/// bits 28-27 do MDEC(1) (2=24bpp, 3=15bpp).
fn decodificar(profundidade: u32) -> Vec<u32> {
    let mut bus = asm::bus_with_bios_empty();
    enviar_tabelas(&mut bus);
    bus.write32::<BusRead>(
        MDEC_CMD,
        (1 << 29) | (profundidade << 27) | ENTRADA.len() as u32,
    );
    for &palavra in ENTRADA.iter() {
        bus.write32::<BusRead>(MDEC_CMD, palavra);
    }
    let mut saida = Vec::new();
    while bus.read32::<BusRead>(MDEC_STATUS) & (1 << 31) == 0 {
        saida.push(bus.read32::<BusRead>(MDEC_CMD));
        assert!(
            saida.len() <= SAIDA.len() * 4,
            "a fifo nao esvazia: o decodificador esta produzindo mais do que o console"
        );
    }
    saida
}

fn canais(p: u16) -> [i32; 3] {
    [
        (p & 0x1F) as i32,
        ((p >> 5) & 0x1F) as i32,
        ((p >> 10) & 0x1F) as i32,
    ]
}

fn pixels(palavras: &[u32]) -> Vec<u16> {
    palavras
        .iter()
        .flat_map(|w| [*w as u16, (*w >> 16) as u16])
        .collect()
}

// O console emite quatro macroblocos completos com as 256 palavras deste gabarito, e para: o
// resto dos dados nao fecha um quinto macrobloco, e um macrobloco incompleto nao sai.
//
// § real_idct_core (L241-267) de docs/reference/09-mdec.md diz que o arredondamento exato do
// hardware nao e conhecido ("the results aren't perfect"). O que o gabarito permite exigir e
// que nenhum canal de nenhum pixel desvie mais de um passo de 5 bits, e que a esmagadora
// maioria das palavras bata byte a byte.
#[test]
fn mdec_15bpp_reproduz_o_gabarito_de_hardware_dentro_de_um_passo() {
    let saida = decodificar(3);
    assert_eq!(
        saida.len(),
        SAIDA.len(),
        "o console entregou {} palavras e nos {}",
        SAIDA.len(),
        saida.len()
    );
    for (i, (&nosso, &console)) in pixels(&saida).iter().zip(pixels(&SAIDA).iter()).enumerate() {
        for (c, (a, b)) in canais(nosso).iter().zip(canais(console).iter()).enumerate() {
            assert!(
                (a - b).abs() <= 1,
                "pixel {i} canal {c}: nosso {a}, console {b} (0x{nosso:04X} vs 0x{console:04X})"
            );
        }
    }
    let exatas = saida
        .iter()
        .zip(SAIDA.iter())
        .filter(|(a, b)| a == b)
        .count();
    assert!(
        exatas >= 477,
        "so {exatas} de {} palavras batem exatamente; eram 477 na iteracao 0184",
        SAIDA.len()
    );
}

// O caminho 24bpp nao tem gabarito palavra a palavra no ps1-tests, mas partilha o yuv_to_rgb
// com o de 15bpp: reduzir cada canal de 8 para 5 bits com a mesma regra do modo 15bpp tem de
// devolver os pixels que o console produziu. Isso prende a ordem dos pixels e a dos canais; so
// os 3 bits baixos de cada canal ficam sem oraculo.
#[test]
fn mdec_24bpp_reduzido_a_15_bits_bate_com_o_gabarito_de_hardware() {
    let saida = decodificar(2);
    assert_eq!(
        saida.len(),
        SAIDA.len() * 3 / 2,
        "24bpp gasta 3 bytes por pixel contra 2 do 15bpp"
    );
    let bytes: Vec<u8> = saida.iter().flat_map(|w| w.to_le_bytes()).collect();
    for (i, &console) in pixels(&SAIDA).iter().enumerate() {
        let (r, g, b) = (bytes[i * 3], bytes[i * 3 + 1], bytes[i * 3 + 2]);
        let reduz = |v: u8| (((v as u32) + 4) >> 3).min(31) as u16;
        let nosso = reduz(r) | (reduz(g) << 5) | (reduz(b) << 10);
        for (c, (a, e)) in canais(nosso).iter().zip(canais(console).iter()).enumerate() {
            assert!(
                (a - e).abs() <= 1,
                "pixel {i} canal {c}: 24bpp deu ({r:02X},{g:02X},{b:02X}) -> {a}, console {e}"
            );
        }
    }
}

// Bit25 do MDEC(1) manda o bit15 de cada pixel de 15bpp. O gabarito acima roda com
// setBit15=0; com bit15=1 os mesmos pixels tem de sair com o bit alto ligado, e nada mais
// pode mudar. § MDEC(1) bit25 (L134) de docs/reference/09-mdec.md.
#[test]
fn mdec_15bpp_bit15_liga_o_bit_alto_sem_mexer_na_cor() {
    let sem = decodificar(3);
    let mut bus = asm::bus_with_bios_empty();
    enviar_tabelas(&mut bus);
    bus.write32::<BusRead>(
        MDEC_CMD,
        (1 << 29) | (3 << 27) | (1 << 25) | ENTRADA.len() as u32,
    );
    for &palavra in ENTRADA.iter() {
        bus.write32::<BusRead>(MDEC_CMD, palavra);
    }
    let mut com = Vec::new();
    while bus.read32::<BusRead>(MDEC_STATUS) & (1 << 31) == 0 {
        com.push(bus.read32::<BusRead>(MDEC_CMD));
        if com.len() > sem.len() {
            break;
        }
    }
    assert_eq!(com.len(), sem.len());
    for (i, (&a, &b)) in com.iter().zip(sem.iter()).enumerate() {
        assert_eq!(
            a,
            b | 0x8000_8000,
            "palavra {i} com setBit15=1 tem de ser a de setBit15=0 com bit15 de cada pixel"
        );
    }
}

const D1_MADR: u32 = 0x1F80_1090;
const D1_BCR: u32 = 0x1F80_1094;
const D1_CHCR: u32 = 0x1F80_1098;
const DPCR: u32 = 0x1F80_10F0;
const CHCR_MDEC_OUT: u32 = 0x0100_0200;

// § MDEC Data/Response Register (L74-78) de docs/reference/09-mdec.md: o registrador entrega
// quatro bitmaps 8x8 em sequencia, e "usually, the data is received via DMA1, which is doing
// the re-ordering automatically". § Colored Macroblocks (L376-388) diz como os quatro Y ladrilham
// o 16x16: Y1 em cima a esquerda, Y2 em cima a direita, Y3 embaixo a esquerda, Y4 a direita.
// Sem esse reordenamento o quadro sai em faixas de 16x4 — foi exatamente o que a VRAM do Rayman
// mostrou na iteracao 0184.
#[test]
fn dma1_reordena_os_quatro_blocos_8x8_em_macroblocos_16x16() {
    let mut bus = asm::bus_with_bios_empty();
    enviar_tabelas(&mut bus);
    bus.write32::<BusRead>(MDEC_CMD, (1 << 29) | (3 << 27) | ENTRADA.len() as u32);
    for &palavra in ENTRADA.iter() {
        bus.write32::<BusRead>(MDEC_CMD, palavra);
    }

    let dst: u32 = 0x0001_0000;
    bus.write32::<BusRead>(DPCR, 0x0765_4321 | (1 << 7));
    bus.write32::<BusRead>(D1_MADR, dst);
    bus.write32::<BusRead>(D1_BCR, 0x0004_0080); // 4 macroblocos de 128 palavras
    bus.write32::<BusRead>(D1_CHCR, CHCR_MDEC_OUT);
    assert_eq!(
        bus.read32::<BusRead>(D1_CHCR) & (1 << 24),
        0,
        "as 512 palavras pedidas existem: o canal tem de fechar"
    );

    // A referencia aqui e a NOSSA propria saida pelo registrador, nao a do console: o que este
    // teste prende e a permutacao, e os valores ja estao presos ao hardware pelo teste de cima.
    let ordem_registrador = pixels(&decodificar(3));
    for macro_i in 0..4usize {
        for y in 0..16usize {
            for x in 0..16usize {
                let quad = usize::from(x >= 8) + 2 * usize::from(y >= 8);
                let dentro = (y % 8) * 8 + (x % 8);
                let esperado = ordem_registrador[macro_i * 256 + quad * 64 + dentro];
                let raster = macro_i * 256 + y * 16 + x;
                let palavra = bus.read32::<BusRead>(dst + (raster as u32 / 2) * 4);
                let obtido = if raster % 2 == 0 {
                    palavra as u16
                } else {
                    (palavra >> 16) as u16
                };
                assert_eq!(
                    obtido,
                    esperado,
                    "macrobloco {macro_i} pixel ({x},{y}) devia vir do bloco Y{} posicao {dentro}",
                    quad + 1
                );
            }
        }
    }
}

// § MDEC(2) - Set Quant Table(s) (L141-149) de docs/reference/09-mdec.md: com bit0=1 vem uma
// SEGUNDA tabela de 64 bytes, usada por Cb e Cr. No gabarito do ps1-tests as duas metades sao
// iguais, entao ignorar a de cor passa despercebido — trocar so a segunda metade tem de mudar
// o quadro.
#[test]
fn tabela_de_quantizacao_de_cor_e_usada_por_cr_e_cb() {
    let referencia = decodificar(3);

    let mut bus = asm::bus_with_bios_empty();
    bus.write32::<BusRead>(MDEC_CMD, (2 << 29) | 1);
    let mut tabela = QUANT;
    for b in tabela[64..].iter_mut() {
        *b = b.saturating_mul(2);
    }
    for chunk in tabela.chunks_exact(4) {
        bus.write32::<BusRead>(
            MDEC_CMD,
            u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]),
        );
    }
    bus.write32::<BusRead>(MDEC_CMD, 3 << 29);
    for pair in IDCT.chunks_exact(2) {
        bus.write32::<BusRead>(
            MDEC_CMD,
            (pair[0] as u16 as u32) | ((pair[1] as u16 as u32) << 16),
        );
    }
    bus.write32::<BusRead>(MDEC_CMD, (1 << 29) | (3 << 27) | ENTRADA.len() as u32);
    for &palavra in ENTRADA.iter() {
        bus.write32::<BusRead>(MDEC_CMD, palavra);
    }
    let mut saida = Vec::new();
    while bus.read32::<BusRead>(MDEC_STATUS) & (1 << 31) == 0 {
        saida.push(bus.read32::<BusRead>(MDEC_CMD));
        if saida.len() > referencia.len() {
            break;
        }
    }
    assert_eq!(saida.len(), referencia.len());
    assert_ne!(
        saida, referencia,
        "dobrar a tabela de cor tem de mudar o croma; se nao muda, Cr e Cb estao lendo a \
         tabela de luminancia"
    );
}

// O DMA1 so pode levar macroblocos INTEIROS: metade de um macrobloco reordenado e lixo, e o
// pedaco que sobra na fifo desalinha o proximo. Com meio macrobloco a mais na fifo do que o
// pedido cabe, o canal leva o que fecha e fica armado para o resto.
#[test]
fn dma1_nao_leva_macrobloco_pela_metade() {
    let mut bus = asm::bus_with_bios_empty();
    enviar_tabelas(&mut bus);
    bus.write32::<BusRead>(MDEC_CMD, (1 << 29) | (3 << 27) | ENTRADA.len() as u32);
    let mut entregues = 0;
    for &palavra in ENTRADA.iter() {
        bus.write32::<BusRead>(MDEC_CMD, palavra);
        entregues += 1;
        if bus.mdec().output_len() >= 192 {
            break;
        }
    }
    assert_eq!(
        bus.mdec().output_len(),
        192,
        "pre-condicao: um macrobloco e meio na fifo apos {entregues} palavras"
    );

    let dst: u32 = 0x0002_0000;
    bus.write32::<BusRead>(DPCR, 0x0765_4321 | (1 << 7));
    bus.write32::<BusRead>(D1_MADR, dst);
    bus.write32::<BusRead>(D1_BCR, 0x0001_00C0); // 192 palavras
    bus.write32::<BusRead>(D1_CHCR, CHCR_MDEC_OUT);

    assert_eq!(
        bus.read32::<BusRead>(D1_CHCR) & (1 << 24),
        1 << 24,
        "so um macrobloco inteiro cabe: o canal continua em andamento"
    );
    assert_eq!(
        bus.mdec().output_len(),
        64,
        "os 64 words do macrobloco incompleto continuam na fifo"
    );
    assert_eq!(
        bus.read32::<BusRead>(dst + 128 * 4),
        0,
        "nada pode ter sido escrito depois do primeiro macrobloco"
    );
}
