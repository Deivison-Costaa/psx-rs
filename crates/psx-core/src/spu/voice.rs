use super::adpcm::BLOCK_SAMPLES;
use super::envelope::Envelope;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Phase {
    #[default]
    Off,
    Attack,
    Decay,
    Sustain,
    Release,
}

#[derive(Debug, Clone)]
pub struct Voice {
    pub volume_left: u16,
    pub volume_right: u16,
    pub pitch: u16,
    pub start_address: u16,
    pub repeat_address: u16,
    pub adsr: u32,
    pub adsr_env: Envelope,
    pub phase: Phase,
    pub sweep_left: Envelope,
    pub sweep_right: Envelope,
    pub current_address: u32,
    pub counter: u32,
    pub block: [i16; BLOCK_SAMPLES],
    pub previous: [i16; 3],
    pub prev1: i32,
    pub prev2: i32,
    pub out: i16,
    pub repeat_latched: bool,
}

impl Default for Voice {
    fn default() -> Self {
        Voice {
            volume_left: 0,
            volume_right: 0,
            pitch: 0,
            start_address: 0,
            repeat_address: 0,
            adsr: 0,
            adsr_env: Envelope::default(),
            phase: Phase::Off,
            sweep_left: Envelope::default(),
            sweep_right: Envelope::default(),
            current_address: 0,
            counter: 0,
            block: [0; BLOCK_SAMPLES],
            previous: [0; 3],
            prev1: 0,
            prev2: 0,
            out: 0,
            repeat_latched: false,
        }
    }
}

impl Voice {
    pub fn sample_index(&self) -> usize {
        0
    }

    pub fn interpolation_index(&self) -> usize {
        0
    }
}
