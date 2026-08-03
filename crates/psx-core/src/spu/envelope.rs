#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Rate {
    pub step: u8,
    pub shift: u8,
    pub exponential: bool,
    pub decreasing: bool,
    pub phase_negative: bool,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Envelope {
    pub level: i32,
    pub counter: i32,
}

impl Envelope {
    pub fn tick(&mut self, _rate: Rate) {}
}
