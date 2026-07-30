#[derive(Debug)]
pub struct Gte {
    pub regs: [u32; 64],
}

impl Default for Gte {
    fn default() -> Self {
        Gte { regs: [0u32; 64] }
    }
}

impl Gte {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn read_data(&self, rd: usize) -> u32 {
        self.regs[rd]
    }

    pub fn write_data(&mut self, rd: usize, val: u32) {
        self.regs[rd] = val;
    }

    pub fn read_control(&self, rd: usize) -> u32 {
        let idx = 32 + rd;
        let val = self.regs[idx];
        if is_standalone_s16_control(rd) {
            (val as i16 as i32) as u32
        } else {
            val
        }
    }

    pub fn write_control(&mut self, rd: usize, val: u32) {
        let idx = 32 + rd;
        self.regs[idx] = val;
    }
}

fn is_standalone_s16_control(rd: usize) -> bool {
    matches!(rd, 4 | 12 | 20 | 27 | 29 | 30)
}
