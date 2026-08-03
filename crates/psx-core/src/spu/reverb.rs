pub const REG_COUNT: usize = 32;
pub const RAM_END: u32 = 0x8_0000;

const D_APF1: usize = 0x00;
const D_APF2: usize = 0x01;
const V_IIR: usize = 0x02;
const V_COMB1: usize = 0x03;
const V_COMB2: usize = 0x04;
const V_COMB3: usize = 0x05;
const V_COMB4: usize = 0x06;
const V_WALL: usize = 0x07;
const V_APF1: usize = 0x08;
const V_APF2: usize = 0x09;
const M_LSAME: usize = 0x0A;
const M_RSAME: usize = 0x0B;
const M_LCOMB1: usize = 0x0C;
const M_RCOMB1: usize = 0x0D;
const M_LCOMB2: usize = 0x0E;
const M_RCOMB2: usize = 0x0F;
const D_LSAME: usize = 0x10;
const D_RSAME: usize = 0x11;
const M_LDIFF: usize = 0x12;
const M_RDIFF: usize = 0x13;
const M_LCOMB3: usize = 0x14;
const M_RCOMB3: usize = 0x15;
const M_LCOMB4: usize = 0x16;
const M_RCOMB4: usize = 0x17;
const D_LDIFF: usize = 0x18;
const D_RDIFF: usize = 0x19;
const M_LAPF1: usize = 0x1A;
const M_RAPF1: usize = 0x1B;
const M_LAPF2: usize = 0x1C;
const M_RAPF2: usize = 0x1D;
const V_LIN: usize = 0x1E;
const V_RIN: usize = 0x1F;

#[derive(Debug, Clone)]
pub struct Reverb {
    pub regs: [u16; REG_COUNT],
    pub vlout: u16,
    pub vrout: u16,
    pub mbase: u16,
    pub current: u32,
}

impl Default for Reverb {
    fn default() -> Self {
        Reverb {
            regs: [0; REG_COUNT],
            vlout: 0,
            vrout: 0,
            mbase: 0,
            current: 0,
        }
    }
}

impl Reverb {
    pub fn set_mbase(&mut self, val: u16) {
        self.mbase = val;
        self.current = u32::from(val) * 8;
    }

    pub fn advance(&mut self) {
        let base = u32::from(self.mbase) * 8;
        let proximo = (self.current + 2) & (RAM_END - 2);
        self.current = proximo.max(base);
    }

    pub fn run(&mut self, _ram: &mut [u8], _lin: i32, _rin: i32, _escrever: bool) -> (i32, i32) {
        (0, 0)
    }
}
