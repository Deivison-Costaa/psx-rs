#[derive(Debug, Clone)]
pub struct Gte {
    regs: [u32; 64],
}

impl Default for Gte {
    fn default() -> Self {
        Self::new()
    }
}

impl Gte {
    pub fn new() -> Self {
        Gte { regs: [0u32; 64] }
    }

    pub fn read_data(&self, reg: usize) -> u32 {
        if reg < 32 { self.regs[reg] } else { 0 }
    }

    pub fn write_data(&mut self, reg: usize, val: u32) {
        if reg < 32 {
            self.regs[reg] = val;
        }
    }

    pub fn read_ctrl(&self, reg: usize) -> u32 {
        if reg < 32 { self.regs[reg + 32] } else { 0 }
    }

    pub fn write_ctrl(&mut self, reg: usize, val: u32) {
        if reg < 32 {
            self.regs[reg + 32] = val;
        }
    }
}
