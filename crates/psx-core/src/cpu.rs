use crate::bus::{Bus, BusRead};

#[derive(Debug, Clone)]
pub struct Cpu {
    pub regs: [u32; 32],
    pub pc: u32,
    load_delay: Option<(usize, u32)>,
    branch_target: Option<u32>,
}

impl Cpu {
    pub fn new() -> Self {
        Cpu {
            regs: [0u32; 32],
            pc: 0xBFC0_0000,
            load_delay: None,
            branch_target: None,
        }
    }

    pub fn step(&mut self, bus: &mut Bus) {
        let instr = bus.read32::<BusRead>(self.pc);
        if let Some(target) = self.branch_target.take() {
            self.pc = target;
        } else {
            self.pc = self.pc.wrapping_add(4);
        }
        let new_load = self.execute(instr, bus);
        if let Some((reg, val)) = self.load_delay.take() {
            self.set_reg(reg, val);
        }
        if let Some((reg, val)) = new_load {
            if reg != 0 {
                self.load_delay = Some((reg, val));
            }
        }
    }

    fn execute(&mut self, instr: u32, bus: &mut Bus) -> Option<(usize, u32)> {
        let primary = instr >> 26;
        match primary {
            0x00 => {
                self.special(instr);
                None
            }
            0x01 => {
                self.bcondz(instr);
                None
            }
            0x02 => {
                self.j(instr);
                None
            }
            0x03 => {
                self.jal(instr);
                None
            }
            0x04 => {
                self.beq(instr);
                None
            }
            0x05 => {
                self.bne(instr);
                None
            }
            0x06 => {
                self.blez(instr);
                None
            }
            0x07 => {
                self.bgtz(instr);
                None
            }
            0x08 => {
                self.addi(instr);
                None
            }
            0x09 => {
                self.addiu(instr);
                None
            }
            0x0A => {
                self.slti(instr);
                None
            }
            0x0B => {
                self.sltiu(instr);
                None
            }
            0x0C => {
                self.andi(instr);
                None
            }
            0x0D => {
                self.ori(instr);
                None
            }
            0x0E => {
                self.xori(instr);
                None
            }
            0x0F => {
                self.lui(instr);
                None
            }
            0x20 => Some(self.lb(instr, bus)),
            0x21 => Some(self.lh(instr, bus)),
            0x23 => Some(self.lw(instr, bus)),
            0x24 => Some(self.lbu(instr, bus)),
            0x25 => Some(self.lhu(instr, bus)),
            0x28 => {
                self.sb(instr, bus);
                None
            }
            0x29 => {
                self.sh(instr, bus);
                None
            }
            0x2B => {
                self.sw(instr, bus);
                None
            }
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
            0x08 => {
                self.jr(instr);
            }
            0x09 => {
                self.jalr(instr);
            }
            _ => unimplemented!("secondary opcode={:02X} nao implementado", secondary),
        }
    }

    fn j(&mut self, instr: u32) {
        let target = instr & 0x03FF_FFFF;
        self.branch_target = Some((self.pc & 0xF000_0000) | (target << 2));
    }

    fn jal(&mut self, instr: u32) {
        let target = instr & 0x03FF_FFFF;
        self.set_reg(31, self.pc + 4);
        self.branch_target = Some((self.pc & 0xF000_0000) | (target << 2));
    }

    fn jr(&mut self, instr: u32) {
        let rs = ((instr >> 21) & 0x1F) as usize;
        self.branch_target = Some(self.reg(rs));
    }

    fn jalr(&mut self, instr: u32) {
        let rs = ((instr >> 21) & 0x1F) as usize;
        let rd = ((instr >> 11) & 0x1F) as usize;
        let target = self.reg(rs);
        self.set_reg(rd, self.pc + 4);
        self.branch_target = Some(target);
    }

    fn branch_taken(&mut self, offset: u32) {
        self.branch_target = Some(self.pc.wrapping_add(offset << 2));
    }

    fn beq(&mut self, instr: u32) {
        let rs = ((instr >> 21) & 0x1F) as usize;
        let rt = ((instr >> 16) & 0x1F) as usize;
        let imm = Self::sign_extend_imm(instr);
        if self.reg(rs) == self.reg(rt) {
            self.branch_taken(imm);
        }
    }

    fn bne(&mut self, instr: u32) {
        let rs = ((instr >> 21) & 0x1F) as usize;
        let rt = ((instr >> 16) & 0x1F) as usize;
        let imm = Self::sign_extend_imm(instr);
        if self.reg(rs) != self.reg(rt) {
            self.branch_taken(imm);
        }
    }

    fn blez(&mut self, instr: u32) {
        let rs = ((instr >> 21) & 0x1F) as usize;
        let imm = Self::sign_extend_imm(instr);
        if (self.reg(rs) as i32) <= 0 {
            self.branch_taken(imm);
        }
    }

    fn bgtz(&mut self, instr: u32) {
        let rs = ((instr >> 21) & 0x1F) as usize;
        let imm = Self::sign_extend_imm(instr);
        if (self.reg(rs) as i32) > 0 {
            self.branch_taken(imm);
        }
    }

    fn bcondz(&mut self, instr: u32) {
        let rs = ((instr >> 21) & 0x1F) as usize;
        let rt = ((instr >> 16) & 0x1F) as usize;
        let imm = Self::sign_extend_imm(instr);
        let rs_val = self.reg(rs) as i32;
        let link = rt >= 16;
        let cond = match rt & 0x1F {
            0x00 => rs_val < 0,
            0x01 => rs_val >= 0,
            0x10 => rs_val < 0,
            0x11 => rs_val >= 0,
            _ => return,
        };
        if link {
            self.set_reg(31, self.pc + 4);
        }
        if cond {
            self.branch_taken(imm);
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

    fn lb(&self, instr: u32, bus: &Bus) -> (usize, u32) {
        let rt = ((instr >> 16) & 0x1F) as usize;
        let rs = ((instr >> 21) & 0x1F) as usize;
        let imm = Self::sign_extend_imm(instr);
        let addr = self.reg(rs).wrapping_add(imm);
        let val = bus.read8::<BusRead>(addr) as i8 as u32;
        (rt, val)
    }

    fn lbu(&self, instr: u32, bus: &Bus) -> (usize, u32) {
        let rt = ((instr >> 16) & 0x1F) as usize;
        let rs = ((instr >> 21) & 0x1F) as usize;
        let imm = Self::sign_extend_imm(instr);
        let addr = self.reg(rs).wrapping_add(imm);
        let val = bus.read8::<BusRead>(addr) as u32;
        (rt, val)
    }

    fn lh(&self, instr: u32, bus: &Bus) -> (usize, u32) {
        let rt = ((instr >> 16) & 0x1F) as usize;
        let rs = ((instr >> 21) & 0x1F) as usize;
        let imm = Self::sign_extend_imm(instr);
        let addr = self.reg(rs).wrapping_add(imm);
        let val = bus.read16::<BusRead>(addr) as i16 as u32;
        (rt, val)
    }

    fn lhu(&self, instr: u32, bus: &Bus) -> (usize, u32) {
        let rt = ((instr >> 16) & 0x1F) as usize;
        let rs = ((instr >> 21) & 0x1F) as usize;
        let imm = Self::sign_extend_imm(instr);
        let addr = self.reg(rs).wrapping_add(imm);
        let val = bus.read16::<BusRead>(addr) as u32;
        (rt, val)
    }

    fn lw(&self, instr: u32, bus: &Bus) -> (usize, u32) {
        let rt = ((instr >> 16) & 0x1F) as usize;
        let rs = ((instr >> 21) & 0x1F) as usize;
        let imm = Self::sign_extend_imm(instr);
        let addr = self.reg(rs).wrapping_add(imm);
        let val = bus.read32::<BusRead>(addr);
        (rt, val)
    }

    fn sb(&mut self, instr: u32, bus: &mut Bus) {
        let rt = ((instr >> 16) & 0x1F) as usize;
        let rs = ((instr >> 21) & 0x1F) as usize;
        let imm = Self::sign_extend_imm(instr);
        let addr = self.reg(rs).wrapping_add(imm);
        let val = self.reg(rt) as u8;
        bus.write8::<BusRead>(addr, val);
    }

    fn sh(&mut self, instr: u32, bus: &mut Bus) {
        let rt = ((instr >> 16) & 0x1F) as usize;
        let rs = ((instr >> 21) & 0x1F) as usize;
        let imm = Self::sign_extend_imm(instr);
        let addr = self.reg(rs).wrapping_add(imm);
        let val = self.reg(rt) as u16;
        bus.write16::<BusRead>(addr, val);
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
