pub const BLOCK_BYTES: usize = 16;
pub const BLOCK_SAMPLES: usize = 28;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Flags {
    pub loop_end: bool,
    pub loop_repeat: bool,
    pub loop_start: bool,
}

impl Flags {
    pub fn from_bits(_bits: u8) -> Self {
        Flags::default()
    }
}

pub fn decode_block(_block: &[u8], prev1: i32, prev2: i32) -> ([i16; BLOCK_SAMPLES], i32, i32) {
    ([0i16; BLOCK_SAMPLES], prev1, prev2)
}
