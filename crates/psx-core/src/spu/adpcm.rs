pub const BLOCK_BYTES: usize = 16;
pub const BLOCK_SAMPLES: usize = 28;

// § Pos/neg Tables (L978) de docs/reference/15-cdrom-format.md, com um filtro a mais.
const POS: [i32; 5] = [0, 60, 115, 98, 122];
const NEG: [i32; 5] = [0, 0, -52, -55, -60];

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Flags {
    pub loop_end: bool,
    pub loop_repeat: bool,
    pub loop_start: bool,
}

impl Flags {
    pub fn from_bits(bits: u8) -> Self {
        Flags {
            loop_end: bits & 0b001 != 0,
            loop_repeat: bits & 0b010 != 0,
            loop_start: bits & 0b100 != 0,
        }
    }
}

/// 16 bytes -> 28 amostras, mais o par (anterior, penultimo) para o proximo bloco.
pub fn decode_block(block: &[u8], prev1: i32, prev2: i32) -> ([i16; BLOCK_SAMPLES], i32, i32) {
    let header = block.first().copied().unwrap_or(0);
    let shift = (header & 0x0F) as u32;
    let filter = usize::from((header >> 4) & 0x07).min(POS.len() - 1);
    let (f0, f1) = (POS[filter], NEG[filter]);

    let mut out = [0i16; BLOCK_SAMPLES];
    let (mut p1, mut p2) = (prev1, prev2);
    for (i, slot) in out.iter_mut().enumerate() {
        let byte = block.get(2 + i / 2).copied().unwrap_or(0);
        let nibble = if i % 2 == 0 { byte & 0x0F } else { byte >> 4 };
        let t = i32::from(nibble) - if nibble >= 8 { 16 } else { 0 };
        let bruto = ((t << 12) >> shift) + ((p1 * f0 + p2 * f1 + 32) >> 6);
        let amostra = bruto.clamp(-0x8000, 0x7FFF);
        *slot = amostra as i16;
        p2 = p1;
        p1 = amostra;
    }
    (out, p1, p2)
}
