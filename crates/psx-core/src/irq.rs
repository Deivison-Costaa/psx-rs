#[derive(Debug)]
pub struct Irq {
    stat: u32,
    mask: u32,
}

impl Irq {
    pub fn new() -> Self {
        Irq { stat: 0, mask: 0 }
    }

    pub fn pending(&self) -> bool {
        (self.stat & self.mask & 0x7FF) != 0
    }

    pub fn read_stat(&self) -> u32 {
        self.stat & 0x7FF
    }

    pub fn write_stat(&mut self, val: u32) {
        self.stat &= val | !0x7FF;
    }

    pub fn read_mask(&self) -> u32 {
        self.mask & 0x7FF
    }

    pub fn write_mask(&mut self, val: u32) {
        self.mask = val & 0x7FF;
    }

    pub fn raise(&mut self, bit: u32) {
        self.stat |= 1 << bit;
    }
}

impl Default for Irq {
    fn default() -> Self {
        Self::new()
    }
}
