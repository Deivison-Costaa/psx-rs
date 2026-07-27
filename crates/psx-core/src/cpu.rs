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
            0x00 => self.special(instr),
            0x08 => self.addi(instr),
            0x09 => self.addiu(instr),
            0x0A => self.slti(instr),
            0x0B => self.sltiu(instr),
            0x0C => self.andi(instr),
            0x0D => self.ori(instr),
            0x0E => self.xori(instr),
            0x0F => self.lui(instr),
            0x2B => self.sw(instr, bus),
            _ => unimplemented!("opcode primary={:02X} nao implementado", primary),
        }
    }

    fn special(&mut self, instr: u32) {
        let secondary = instr & 0x3F;
        let rs = ((instr >> 21) & 0x1F) as usize;
        let rt = ((instr >> 16) & 0x1F) as usize;
        let rd = ((instr >> 11) & 0x1F) as usize;
        let sa = ((instr >> 6) & 0x1F) as usize;
        match secondary {
            0x00 => {
                let val = self.reg(rt) << sa;
                self.set_reg(rd, val);
            }
            0x02 => {
                let val = self.reg(rt) >> sa;
                self.set_reg(rd, val);
            }
            0x03 => {
                let val = (self.reg(rt) as i32 >> sa) as u32;
                self.set_reg(rd, val);
            }
            0x04 => {
                let shift = self.reg(rs) & 0x1F;
                let val = self.reg(rt) << shift;
                self.set_reg(rd, val);
            }
            0x06 => {
                let shift = self.reg(rs) & 0x1F;
                let val = self.reg(rt) >> shift;
                self.set_reg(rd, val);
            }
            0x07 => {
                let shift = self.reg(rs) & 0x1F;
                let val = (self.reg(rt) as i32 >> shift) as u32;
                self.set_reg(rd, val);
            }
            0x21 => {
                let val = self.reg(rs).wrapping_add(self.reg(rt));
                self.set_reg(rd, val);
            }
            0x23 => {
                let val = self.reg(rs).wrapping_sub(self.reg(rt));
                self.set_reg(rd, val);
            }
            0x24 => {
                let val = self.reg(rs) & self.reg(rt);
                self.set_reg(rd, val);
            }
            0x25 => {
                let val = self.reg(rs) | self.reg(rt);
                self.set_reg(rd, val);
            }
            0x26 => {
                let val = self.reg(rs) ^ self.reg(rt);
                self.set_reg(rd, val);
            }
            0x27 => {
                let val = !(self.reg(rs) | self.reg(rt));
                self.set_reg(rd, val);
            }
            0x2A => {
                let val = (self.reg(rs) as i32) < (self.reg(rt) as i32);
                self.set_reg(rd, val as u32);
            }
            0x2B => {
                let val = self.reg(rs) < self.reg(rt);
                self.set_reg(rd, val as u32);
            }
            _ => unimplemented!("secondary opcode={:02X} nao implementado", secondary),
        }
    }

    fn sign_extend_imm(instr: u32) -> u32 {
        (instr & 0xFFFF) as u16 as i16 as u32
    }

    fn addiu(&mut self, instr: u32) {
        let rs = ((instr >> 21) & 0x1F) as usize;
        let rt = ((instr >> 16) & 0x1F) as usize;
        let imm = Self::sign_extend_imm(instr);
        let val = self.reg(rs).wrapping_add(imm);
        self.set_reg(rt, val);
    }

    fn addi(&mut self, instr: u32) {
        let rs = ((instr >> 21) & 0x1F) as usize;
        let rt = ((instr >> 16) & 0x1F) as usize;
        let imm = Self::sign_extend_imm(instr);
        let val = self.reg(rs).wrapping_add(imm);
        self.set_reg(rt, val);
    }

    fn slti(&mut self, instr: u32) {
        let rs = ((instr >> 21) & 0x1F) as usize;
        let rt = ((instr >> 16) & 0x1F) as usize;
        let imm = Self::sign_extend_imm(instr);
        let val = (self.reg(rs) as i32) < (imm as i32);
        self.set_reg(rt, val as u32);
    }

    fn sltiu(&mut self, instr: u32) {
        let rs = ((instr >> 21) & 0x1F) as usize;
        let rt = ((instr >> 16) & 0x1F) as usize;
        let imm = Self::sign_extend_imm(instr);
        let val = self.reg(rs) < imm;
        self.set_reg(rt, val as u32);
    }

    fn andi(&mut self, instr: u32) {
        let rs = ((instr >> 21) & 0x1F) as usize;
        let rt = ((instr >> 16) & 0x1F) as usize;
        let imm = instr & 0xFFFF;
        let val = self.reg(rs) & imm;
        self.set_reg(rt, val);
    }

    fn xori(&mut self, instr: u32) {
        let rs = ((instr >> 21) & 0x1F) as usize;
        let rt = ((instr >> 16) & 0x1F) as usize;
        let imm = instr & 0xFFFF;
        let val = self.reg(rs) ^ imm;
        self.set_reg(rt, val);
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
        let imm = Self::sign_extend_imm(instr);
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
