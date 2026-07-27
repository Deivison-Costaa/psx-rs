use crate::bus::{Bus, BusRead};

#[derive(Debug, Clone)]
pub struct Cpu {
    pub regs: [u32; 32],
    pub pc: u32,
}

impl Cpu {
    pub fn new() -> Self {
        Cpu {
            regs: [0u32; 32],
            pc: 0xBFC0_0000,
        }
    }

    pub fn step(&mut self, bus: &mut Bus) {
        let instr = bus.read32::<BusRead>(self.pc);
        self.pc = self.pc.wrapping_add(4);
        let primary = instr >> 26;
        match primary {
            0x0F => self.lui(instr),
            0x0D => self.ori(instr),
            0x2B => self.sw(instr, bus),
            _ => unimplemented!("opcode primary={:02X} nao implementado", primary),
        }
    }

    fn lui(&mut self, instr: u32) {
        let rt = ((instr >> 16) & 0x1F) as usize;
        let imm = instr & 0xFFFF;
        self.set_reg(rt, imm << 16);
    }

    fn ori(&mut self, instr: u32) {
        let rs = ((instr >> 21) & 0x1F) as usize;
        let rt = ((instr >> 16) & 0x1F) as usize;
        let imm = instr & 0xFFFF;
        let val = self.reg(rs) | imm;
        self.set_reg(rt, val);
    }

    fn sw(&mut self, instr: u32, bus: &mut Bus) {
        let rs = ((instr >> 21) & 0x1F) as usize;
        let rt = ((instr >> 16) & 0x1F) as usize;
        let imm = instr & 0xFFFF;
        let addr = self.reg(rs).wrapping_add(imm);
        let val = self.reg(rt);
        bus.write32::<BusRead>(addr, val);
    }

    fn reg(&self, idx: usize) -> u32 {
        self.regs[idx]
    }

    fn set_reg(&mut self, idx: usize, val: u32) {
        if idx == 0 {
            return;
        }
        self.regs[idx] = val;
    }
}

impl Default for Cpu {
    fn default() -> Self {
        Self::new()
    }
}
