const MODE_MASK: u16 = 0x00FF;
const CTRL_ACK: u16 = 1 << 4;
const CTRL_RESET: u16 = 1 << 6;

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct Sio1 {
    mode: u16,
    ctrl: u16,
    misc: u16,
    baud: u16,
}

impl Sio1 {
    pub fn read16(&self, phys: u32) -> u16 {
        match phys & 0xE {
            0x8 => self.mode,
            0xA => self.ctrl,
            0xC => self.misc,
            0xE => self.baud,
            _ => 0,
        }
    }

    pub fn write16(&mut self, phys: u32, val: u16) {
        match phys & 0xE {
            0x8 => self.mode = val & MODE_MASK,
            0xA if val & CTRL_RESET != 0 => *self = Self::default(),
            0xA => self.ctrl = val & !CTRL_ACK,
            0xC => self.misc = val,
            0xE => self.baud = val,
            _ => {}
        }
    }

    pub fn write8(&mut self, phys: u32, val: u8) {
        let shift = (phys & 1) * 8;
        let old = self.read16(phys);
        let new = (old & !(0xFF << shift)) | (u16::from(val) << shift);
        self.write16(phys, new);
    }
}
