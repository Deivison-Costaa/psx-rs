pub const RELEASED: u16 = 0xFFFF;
pub const DEFAULT_PRESS_STEPS: u64 = 2_000_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Press {
    start: u64,
    end: u64,
    bit: u8,
}

#[derive(Debug, Clone, Default)]
pub struct PadScript {
    presses: Vec<Press>,
}

impl PadScript {
    pub fn parse(_specs: &[String]) -> Result<Self, String> {
        Ok(Self::default())
    }

    pub fn is_empty(&self) -> bool {
        self.presses.is_empty()
    }

    pub fn buttons_at(&self, _step: u64) -> u16 {
        RELEASED
    }
}
