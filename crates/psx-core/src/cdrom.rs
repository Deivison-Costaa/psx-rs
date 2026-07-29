#[derive(Debug)]
pub struct Cdrom {
    bank: u8,
}

impl Cdrom {
    pub fn new() -> Self {
        Cdrom { bank: 0 }
    }

    pub fn read8(&self, _offset: u32) -> u8 {
        0
    }

    pub fn write8(&mut self, offset: u32, val: u8) {
        if offset & 0x3 == 0 {
            self.bank = val & 0x3;
        }
    }

    pub fn irq_pending(&self) -> bool {
        false
    }
}

impl Default for Cdrom {
    fn default() -> Self {
        Self::new()
    }
}
