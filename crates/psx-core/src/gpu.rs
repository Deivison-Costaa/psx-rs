#[derive(Debug)]
pub struct Gpu {
    pub stat: u32,
}

impl Gpu {
    pub fn new() -> Self {
        Gpu {
            stat: 0x1480_2000,
        }
    }

    pub fn read32(&self, offset: u32) -> u32 {
        match offset {
            0x0 => 0,
            0x4 => self.stat,
            _ => 0,
        }
    }

    pub fn write32(&mut self, offset: u32, val: u32) {
        let _ = (offset, val);
    }
}
