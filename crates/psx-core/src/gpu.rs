#[derive(Debug)]
pub struct Gpu {
    pub stat: u32,
    dma_direction: u8,
    interlace: bool,
}

impl Default for Gpu {
    fn default() -> Self {
        Self::new()
    }
}

impl Gpu {
    pub fn new() -> Self {
        Gpu {
            stat: 0x1480_2000,
            dma_direction: 0,
            interlace: false,
        }
    }

    pub fn read32(&self, offset: u32) -> u32 {
        match offset {
            0x0 => 0,
            0x4 => {
                let bit25 = match self.dma_direction {
                    0 => 0,
                    1 => 1 << 25,
                    2 => ((self.stat >> 28) & 1) << 25,
                    3 => ((self.stat >> 27) & 1) << 25,
                    _ => 0,
                };
                (self.stat & !(1 << 25)) | bit25
            }
            _ => 0,
        }
    }

    pub fn write32(&mut self, offset: u32, val: u32) {
        match offset {
            0x0 => self.write_gp0(val),
            0x4 => self.write_gp1(val),
            _ => {}
        }
    }

    fn write_gp0(&mut self, val: u32) {
        let cmd = (val >> 24) as u8;
        match cmd {
            0x00 | 0x04..=0x1E | 0xE0 | 0xE7..=0xEF => {}
            0xE6 => {
                let param = val & 0xFF_FFFF;
                let mask = (1 << 11) | (1 << 12);
                let bits = (param & 0x3) << 11;
                self.stat = (self.stat & !mask) | bits;
            }
            0xE1 => {
                let param = val & 0xFF_FFFF;
                let mask = (0x7FF) | (1 << 15);
                let bits = (param & 0x7FF) | (((param >> 11) & 1) << 15);
                self.stat = (self.stat & !mask) | bits;
            }
            _ => {}
        }
    }

    fn write_gp1(&mut self, val: u32) {
        let cmd = (val >> 24) as u8;
        match cmd {
            0x00 => {
                self.stat = 0x1480_2000;
                self.dma_direction = 0;
                self.interlace = false;
            }
            0x01 => {}
            0x02 => {
                self.stat &= !(1 << 24);
            }
            0x03 => {
                let bit = val & 1;
                if bit == 0 {
                    self.stat &= !(1 << 23);
                } else {
                    self.stat |= 1 << 23;
                }
            }
            0x04 => {
                self.dma_direction = (val & 0x3) as u8;
                self.stat = (self.stat & !(3 << 29)) | ((val & 0x3) << 29);
            }
            0x08 => {
                let param = val & 0xFF;
                self.interlace = (param >> 5) & 1 != 0;
                let bits: u32 = (param & 0x80) << 7
                    | (param & 0x40) << 10
                    | (param & 0x3F) << 17;
                let mask: u32 = (1 << 14) | (0x7F << 16);
                self.stat = (self.stat & !mask) | bits;
            }
            _ => {}
        }
    }
}
