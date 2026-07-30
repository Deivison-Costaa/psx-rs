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

    pub fn write_stat_byte(&mut self, offset: u32, byte: u8) {
        let shift = offset * 8;
        let val = (byte as u32) << shift;
        let preserved = !(0xFFu32 << shift);
        self.stat &= val | preserved;
    }

    pub fn write_stat_half(&mut self, offset: u32, val: u16) {
        let shift = offset * 8;
        let val32 = (val as u32) << shift;
        let preserved = !(0xFFFFu32 << shift);
        self.stat &= val32 | preserved;
    }

    pub fn write_mask_byte(&mut self, offset: u32, byte: u8) {
        let shift = offset * 8;
        let mask_byte = ((byte as u32) << shift) & 0x7FF;
        let clear = !(0xFFu32 << shift);
        self.mask = (self.mask & clear) | mask_byte;
    }

    pub fn write_mask_half(&mut self, offset: u32, val: u16) {
        let shift = offset * 8;
        let mask_half = ((val as u32) << shift) & 0x7FF;
        let clear = !(0xFFFFu32 << shift);
        self.mask = (self.mask & clear) | mask_half;
    }

    pub fn read_stat_byte(&self, offset: u32) -> u8 {
        ((self.stat >> (offset * 8)) & 0xFF) as u8
    }

    pub fn read_mask_byte(&self, offset: u32) -> u8 {
        ((self.mask >> (offset * 8)) & 0xFF) as u8
    }
}

impl Default for Irq {
    fn default() -> Self {
        Self::new()
    }
}
