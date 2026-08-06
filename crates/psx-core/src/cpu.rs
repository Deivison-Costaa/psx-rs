use crate::bus::{Bus, BusRead, BusWrite};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Cpu {
    pub regs: [u32; 32],
    pub cop0: [u32; 32],
    pub pc: u32,
    pub hi: u32,
    pub lo: u32,
    load_delay: Option<(usize, u32)>,
    branch_target: Option<u32>,
    delay_slot_pending: bool,
    branch_taken: bool,
    pending_exception: Option<u8>,
    exception_badvaddr: Option<u32>,
    pending_ce: Option<u8>,
    load_extra_cycles: u32,
    written_gpr: Option<usize>,
    pub irq_handler_entries: u64,
}

impl Cpu {
    pub fn sr(&self) -> u32 {
        self.cop0[12]
    }

    pub fn set_sr(&mut self, val: u32) {
        self.cop0[12] = val;
    }

    pub fn new() -> Self {
        let mut cop0 = [0u32; 32];
        cop0[15] = 0x0000_0002;
        Cpu {
            regs: [0u32; 32],
            cop0,
            pc: 0xBFC0_0000,
            hi: 0,
            lo: 0,
            load_delay: None,
            branch_target: None,
            delay_slot_pending: false,
            branch_taken: false,
            pending_exception: None,
            exception_badvaddr: None,
            pending_ce: None,
            load_extra_cycles: 0,
            written_gpr: None,
            irq_handler_entries: 0,
        }
    }

    fn raise_exception(&mut self, exc_code: u8, bad_vaddr: Option<u32>) {
        self.pending_exception = Some(exc_code);
        self.exception_badvaddr = bad_vaddr;
    }

    fn enter_exception(
        &mut self,
        exc_code: u8,
        bad_vaddr: Option<u32>,
        instr_pc: u32,
        in_delay_slot: bool,
        was_taken: bool,
        ce: Option<u8>,
    ) {
        let sr = self.cop0[12];
        let ie_ku_shifted = ((sr & 0x3) << 2) | ((sr & 0xC) << 2);
        self.cop0[12] = (sr & !0x3F) | ie_ku_shifted;

        let mut cause = (exc_code as u32) << 2;
        if in_delay_slot {
            cause |= 1 << 31;
            if was_taken {
                cause |= 1 << 30;
            }
        }
        // § cop0r13 - CAUSE, campo CE bits 28-29 (02-cpu.md L681): numero do coprocessador
        // da excecao Coprocessor Unusable. So mexe nesses bits quando a excecao e CpU;
        // outros tipos de excecao deixam o CE anterior como estava.
        let ce_mask = match ce {
            Some(unit) => {
                cause |= (unit as u32 & 0x3) << 28;
                0x3000_0000
            }
            None => 0,
        };
        self.cop0[13] = (self.cop0[13] & !(0xC000_007C | ce_mask)) | cause;

        if in_delay_slot {
            self.cop0[14] = instr_pc.wrapping_sub(4);
        } else {
            self.cop0[14] = instr_pc;
        }

        if let Some(vaddr) = bad_vaddr {
            self.cop0[8] = vaddr;
        }

        self.pc = if exc_code == 0x09 {
            0x8000_0040
        } else {
            0x8000_0080
        };

        self.branch_target = None;
        self.delay_slot_pending = false;
        self.branch_taken = false;
        if let Some((reg, val)) = self.load_delay.take() {
            self.set_reg(reg, val);
        }
    }

    pub fn step(&mut self, bus: &mut Bus) {
        self.cop0[13] = if bus.irq().pending() {
            self.cop0[13] | (1 << 10)
        } else {
            self.cop0[13] & !(1 << 10)
        };

        let sr = self.cop0[12];
        if (sr & 0x1) != 0 && (sr & (1 << 10)) != 0 && (self.cop0[13] & (1 << 10)) != 0 {
            if let Some((reg, val)) = self.load_delay.take() {
                self.set_reg(reg, val);
            }
            let ie_ku_shifted = ((sr & 0x3) << 2) | ((sr & 0xC) << 2);
            self.cop0[12] = (sr & !0x3F) | ie_ku_shifted;
            self.cop0[13] &= !0xC000_007C;
            if self.delay_slot_pending {
                self.cop0[13] |= 1 << 31;
                if self.branch_taken {
                    self.cop0[13] |= 1 << 30;
                }
                self.cop0[14] = self.pc.wrapping_sub(4);
            } else {
                self.cop0[14] = self.pc;
            }
            self.branch_target = None;
            self.delay_slot_pending = false;
            self.branch_taken = false;
            self.pc = 0x8000_0080;
            self.irq_handler_entries = self.irq_handler_entries.wrapping_add(1);
            return;
        }

        let instr_pc = self.pc;
        if instr_pc & 3 != 0 {
            self.enter_exception(0x04, Some(instr_pc), instr_pc, false, false, None);
            return;
        }
        if Bus::fetch_causa_bus_error(instr_pc) {
            self.enter_exception(0x06, Some(instr_pc), instr_pc, false, false, None);
            return;
        }
        let instr = bus.read32::<BusRead>(instr_pc);

        let phys = instr_pc & 0x1FFF_FFFF;
        if phys == 0xA0 || phys == 0xB0 {
            let fn_idx = self.reg_with_pending(9);
            let stubbed = bus.read32::<BusRead>(phys) == Self::JR_RA;
            match (fn_idx, phys) {
                (0x3C, 0xA0) | (0x3D, 0xB0) => {
                    bus.tty_push((self.reg_with_pending(4) & 0xFF) as u8);
                }
                (0x3E, 0xA0) | (0x3F, 0xB0) if stubbed => {
                    let src = self.reg_with_pending(4);
                    if src == 0 {
                        for &b in b"<NULL>" {
                            bus.tty_push(b);
                        }
                    } else {
                        for offset in 0..1_048_576u32 {
                            let byte = bus.read8::<BusRead>(src.wrapping_add(offset));
                            if byte == 0 {
                                break;
                            }
                            bus.tty_push(byte);
                        }
                    }
                }
                (0x3F, 0xA0) if stubbed => self.do_printf(bus),
                _ => {}
            }
        }

        let in_delay_slot = self.delay_slot_pending;
        let was_taken = self.branch_taken;
        self.delay_slot_pending = false;
        self.branch_taken = false;

        if let Some(target) = self.branch_target.take() {
            self.pc = target;
        } else {
            self.pc = self.pc.wrapping_add(4);
        }

        self.written_gpr = None;
        let new_load = self.execute(instr, bus);

        if let Some(exc_code) = self.pending_exception.take() {
            let bad_vaddr = self.exception_badvaddr.take();
            let ce = self.pending_ce.take();
            self.enter_exception(exc_code, bad_vaddr, instr_pc, in_delay_slot, was_taken, ce);
            return;
        }

        if let Some((reg, val)) = self.load_delay.take() {
            let replaced_by_load = new_load.is_some_and(|(new_reg, _)| new_reg == reg);
            if self.written_gpr != Some(reg) && !replaced_by_load {
                self.set_reg(reg, val);
            }
        }
        if let Some((reg, val)) = new_load {
            if reg != 0 {
                self.load_delay = Some((reg, val));
            }
        }
        bus.tick_timers(1 + std::mem::take(&mut self.load_extra_cycles));
    }

    fn execute(&mut self, instr: u32, bus: &mut Bus) -> Option<(usize, u32)> {
        let primary = instr >> 26;
        if (0x20..=0x26).contains(&primary) {
            let rs = ((instr >> 21) & 0x1F) as usize;
            let addr = self.reg(rs).wrapping_add(Self::sign_extend_imm(instr));
            self.load_extra_cycles = Bus::load_cycles(addr) - 1;
        }
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
            0x22 => Some(self.lwl(instr, bus)),
            0x23 => Some(self.lw(instr, bus)),
            0x24 => Some(self.lbu(instr, bus)),
            0x25 => Some(self.lhu(instr, bus)),
            0x26 => Some(self.lwr(instr, bus)),
            0x28 => {
                self.sb(instr, bus);
                None
            }
            0x29 => {
                self.sh(instr, bus);
                None
            }
            0x2A => {
                self.swl(instr, bus);
                None
            }
            0x2B => {
                self.sw(instr, bus);
                None
            }
            0x2E => {
                self.swr(instr, bus);
                None
            }
            0x10 | 0x11 | 0x12 | 0x13 | 0x30 | 0x31 | 0x32 | 0x33 | 0x38 | 0x39 | 0x3A | 0x3B => {
                let unit = primary & 3;
                if !self.cop_usable(unit, primary & 0x20 == 0) {
                    self.pending_ce = Some(unit as u8);
                    self.raise_exception(0x0B, None);
                    return None;
                }
                match primary {
                    0x10 => self.cop0_op(instr),
                    0x12 => self.cop2_op(instr, bus),
                    0x32 => self.lwc2_op(instr, bus),
                    0x3A => self.swc2_op(instr, bus),
                    _ => None,
                }
            }
            _ => {
                self.raise_exception(0x0A, None);
                None
            }
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
            0x10 => self.mfhi(instr),
            0x11 => self.mthi(instr),
            0x12 => self.mflo(instr),
            0x13 => self.mtlo(instr),
            0x0C => self.raise_exception(0x08, None),
            0x0D => self.raise_exception(0x09, None),
            0x18 => self.mult(instr),
            0x19 => self.multu(instr),
            0x1A => self.div(instr),
            0x1B => self.divu(instr),
            0x20 => {
                let a = self.reg(rs) as i32;
                let b = self.reg(rt) as i32;
                if let Some(result) = a.checked_add(b) {
                    self.set_reg(rd, result as u32);
                } else {
                    self.raise_exception(0x0C, None);
                }
            }
            0x22 => {
                let a = self.reg(rs) as i32;
                let b = self.reg(rt) as i32;
                if let Some(result) = a.checked_sub(b) {
                    self.set_reg(rd, result as u32);
                } else {
                    self.raise_exception(0x0C, None);
                }
            }
            _ => self.raise_exception(0x0A, None),
        }
    }

    fn j(&mut self, instr: u32) {
        self.delay_slot_pending = true;
        self.branch_taken = true;
        let target = instr & 0x03FF_FFFF;
        self.branch_target = Some((self.pc & 0xF000_0000) | (target << 2));
    }

    fn jal(&mut self, instr: u32) {
        self.delay_slot_pending = true;
        self.branch_taken = true;
        let target = instr & 0x03FF_FFFF;
        self.set_reg(31, self.pc.wrapping_add(4));
        self.branch_target = Some((self.pc & 0xF000_0000) | (target << 2));
    }

    fn jr(&mut self, instr: u32) {
        self.delay_slot_pending = true;
        self.branch_taken = true;
        let rs = ((instr >> 21) & 0x1F) as usize;
        self.branch_target = Some(self.reg(rs));
    }

    fn jalr(&mut self, instr: u32) {
        self.delay_slot_pending = true;
        self.branch_taken = true;
        let rs = ((instr >> 21) & 0x1F) as usize;
        let rd = ((instr >> 11) & 0x1F) as usize;
        let target = self.reg(rs);
        self.set_reg(rd, self.pc.wrapping_add(4));
        self.branch_target = Some(target);
    }

    fn mfhi(&mut self, instr: u32) {
        let rd = ((instr >> 11) & 0x1F) as usize;
        self.set_reg(rd, self.hi);
    }

    fn mthi(&mut self, instr: u32) {
        let rs = ((instr >> 21) & 0x1F) as usize;
        self.hi = self.reg(rs);
    }

    fn mflo(&mut self, instr: u32) {
        let rd = ((instr >> 11) & 0x1F) as usize;
        self.set_reg(rd, self.lo);
    }

    fn mtlo(&mut self, instr: u32) {
        let rs = ((instr >> 21) & 0x1F) as usize;
        self.lo = self.reg(rs);
    }

    fn mult(&mut self, instr: u32) {
        let rs = ((instr >> 21) & 0x1F) as usize;
        let rt = ((instr >> 16) & 0x1F) as usize;
        let a = self.reg(rs) as i32 as i64;
        let b = self.reg(rt) as i32 as i64;
        let prod = a.wrapping_mul(b);
        self.lo = prod as u32;
        self.hi = (prod >> 32) as u32;
    }

    fn multu(&mut self, instr: u32) {
        let rs = ((instr >> 21) & 0x1F) as usize;
        let rt = ((instr >> 16) & 0x1F) as usize;
        let prod = (self.reg(rs) as u64).wrapping_mul(self.reg(rt) as u64);
        self.lo = prod as u32;
        self.hi = (prod >> 32) as u32;
    }

    fn div(&mut self, instr: u32) {
        let rs = ((instr >> 21) & 0x1F) as usize;
        let rt = ((instr >> 16) & 0x1F) as usize;
        let rs_val = self.reg(rs) as i32;
        let rt_val = self.reg(rt) as i32;
        if rt_val == 0 {
            self.lo = if rs_val >= 0 { 0xFFFF_FFFF } else { 1 };
            self.hi = self.reg(rs);
        } else if rs_val == i32::MIN && rt_val == -1 {
            self.lo = 0x8000_0000;
            self.hi = 0;
        } else {
            self.lo = (rs_val / rt_val) as u32;
            self.hi = (rs_val % rt_val) as u32;
        }
    }

    fn divu(&mut self, instr: u32) {
        let rs = ((instr >> 21) & 0x1F) as usize;
        let rt = ((instr >> 16) & 0x1F) as usize;
        let rs_val = self.reg(rs);
        let rt_val = self.reg(rt);
        match (rs_val.checked_div(rt_val), rs_val.checked_rem(rt_val)) {
            (Some(quot), Some(rem)) => {
                self.lo = quot;
                self.hi = rem;
            }
            _ => {
                self.lo = 0xFFFF_FFFF;
                self.hi = rs_val;
            }
        }
    }

    fn branch_taken(&mut self, offset: u32) {
        self.branch_target = Some(self.pc.wrapping_add(offset << 2));
    }

    fn beq(&mut self, instr: u32) {
        self.delay_slot_pending = true;
        self.branch_taken = false;
        let rs = ((instr >> 21) & 0x1F) as usize;
        let rt = ((instr >> 16) & 0x1F) as usize;
        let imm = Self::sign_extend_imm(instr);
        if self.reg(rs) == self.reg(rt) {
            self.branch_taken = true;
            self.branch_taken(imm);
        }
    }

    fn bne(&mut self, instr: u32) {
        self.delay_slot_pending = true;
        self.branch_taken = false;
        let rs = ((instr >> 21) & 0x1F) as usize;
        let rt = ((instr >> 16) & 0x1F) as usize;
        let imm = Self::sign_extend_imm(instr);
        if self.reg(rs) != self.reg(rt) {
            self.branch_taken = true;
            self.branch_taken(imm);
        }
    }

    fn blez(&mut self, instr: u32) {
        self.delay_slot_pending = true;
        self.branch_taken = false;
        let rs = ((instr >> 21) & 0x1F) as usize;
        let imm = Self::sign_extend_imm(instr);
        if (self.reg(rs) as i32) <= 0 {
            self.branch_taken = true;
            self.branch_taken(imm);
        }
    }

    fn bgtz(&mut self, instr: u32) {
        self.delay_slot_pending = true;
        self.branch_taken = false;
        let rs = ((instr >> 21) & 0x1F) as usize;
        let imm = Self::sign_extend_imm(instr);
        if (self.reg(rs) as i32) > 0 {
            self.branch_taken = true;
            self.branch_taken(imm);
        }
    }

    fn bcondz(&mut self, instr: u32) {
        self.delay_slot_pending = true;
        self.branch_taken = false;
        let rs = ((instr >> 21) & 0x1F) as usize;
        let rt = ((instr >> 16) & 0x1F) as usize;
        let imm = Self::sign_extend_imm(instr);
        let rs_val = self.reg(rs) as i32;
        let link = rt == 0x10 || rt == 0x11;
        let cond = if rt & 0x01 == 0 {
            rs_val < 0
        } else {
            rs_val >= 0
        };
        if link {
            self.set_reg(31, self.pc.wrapping_add(4));
        }
        if cond {
            self.branch_taken = true;
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
        let a = self.reg(rs) as i32;
        let b = imm as i32;
        if let Some(result) = a.checked_add(b) {
            self.set_reg(rt, result as u32);
        } else {
            self.raise_exception(0x0C, None);
        }
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

    fn is_isc(&self) -> bool {
        (self.cop0[12] >> 16) & 1 != 0
    }

    fn sw(&mut self, instr: u32, bus: &mut Bus) {
        let rs = ((instr >> 21) & 0x1F) as usize;
        let rt = ((instr >> 16) & 0x1F) as usize;
        let imm = Self::sign_extend_imm(instr);
        let addr = self.reg(rs).wrapping_add(imm);
        if addr & 3 != 0 {
            self.raise_exception(0x05, Some(addr));
            return;
        }
        if self.is_isc() {
            return;
        }
        let val = self.reg(rt);
        bus.write32::<BusRead>(addr, val);
    }

    fn lb(&mut self, instr: u32, bus: &Bus) -> (usize, u32) {
        let rt = ((instr >> 16) & 0x1F) as usize;
        let rs = ((instr >> 21) & 0x1F) as usize;
        let imm = Self::sign_extend_imm(instr);
        let addr = self.reg(rs).wrapping_add(imm);
        let val = bus.read8::<BusRead>(addr) as i8 as u32;
        (rt, val)
    }

    fn lbu(&mut self, instr: u32, bus: &Bus) -> (usize, u32) {
        let rt = ((instr >> 16) & 0x1F) as usize;
        let rs = ((instr >> 21) & 0x1F) as usize;
        let imm = Self::sign_extend_imm(instr);
        let addr = self.reg(rs).wrapping_add(imm);
        let val = bus.read8::<BusRead>(addr) as u32;
        (rt, val)
    }

    fn lh(&mut self, instr: u32, bus: &Bus) -> (usize, u32) {
        let rt = ((instr >> 16) & 0x1F) as usize;
        let rs = ((instr >> 21) & 0x1F) as usize;
        let imm = Self::sign_extend_imm(instr);
        let addr = self.reg(rs).wrapping_add(imm);
        if addr & 1 != 0 {
            self.raise_exception(0x04, Some(addr));
            return (rt, 0);
        }
        let val = bus.read16::<BusRead>(addr) as i16 as u32;
        (rt, val)
    }

    fn lhu(&mut self, instr: u32, bus: &Bus) -> (usize, u32) {
        let rt = ((instr >> 16) & 0x1F) as usize;
        let rs = ((instr >> 21) & 0x1F) as usize;
        let imm = Self::sign_extend_imm(instr);
        let addr = self.reg(rs).wrapping_add(imm);
        if addr & 1 != 0 {
            self.raise_exception(0x04, Some(addr));
            return (rt, 0);
        }
        let val = bus.read16::<BusRead>(addr) as u32;
        (rt, val)
    }

    fn lw(&mut self, instr: u32, bus: &Bus) -> (usize, u32) {
        let rt = ((instr >> 16) & 0x1F) as usize;
        let rs = ((instr >> 21) & 0x1F) as usize;
        let imm = Self::sign_extend_imm(instr);
        let addr = self.reg(rs).wrapping_add(imm);
        if addr & 3 != 0 {
            self.raise_exception(0x04, Some(addr));
            return (rt, 0);
        }
        let val = bus.read32::<BusRead>(addr);
        (rt, val)
    }

    fn sb(&mut self, instr: u32, bus: &mut Bus) {
        let rs = ((instr >> 21) & 0x1F) as usize;
        let rt = ((instr >> 16) & 0x1F) as usize;
        let imm = Self::sign_extend_imm(instr);
        let addr = self.reg(rs).wrapping_add(imm);
        if self.is_isc() {
            return;
        }
        let val = self.reg(rt);
        bus.write8_gpr_completo::<BusRead>(addr, val);
    }

    fn sh(&mut self, instr: u32, bus: &mut Bus) {
        let rs = ((instr >> 21) & 0x1F) as usize;
        let rt = ((instr >> 16) & 0x1F) as usize;
        let imm = Self::sign_extend_imm(instr);
        let addr = self.reg(rs).wrapping_add(imm);
        if addr & 1 != 0 {
            self.raise_exception(0x05, Some(addr));
            return;
        }
        if self.is_isc() {
            return;
        }
        let val = self.reg(rt);
        bus.write16_gpr_completo::<BusRead>(addr, val);
    }

    fn lwl(&mut self, instr: u32, bus: &Bus) -> (usize, u32) {
        let rt = ((instr >> 16) & 0x1F) as usize;
        let rs = ((instr >> 21) & 0x1F) as usize;
        let imm = Self::sign_extend_imm(instr);
        let addr = self.reg(rs).wrapping_add(imm);
        let aligned = addr & !3;
        let offset = (addr & 3) as usize;
        let word = bus.read32::<BusRead>(aligned);
        let old = self.reg_with_pending(rt);
        let val = match offset {
            0 => (old & 0x00FF_FFFF) | ((word & 0xFF) << 24),
            1 => (old & 0x0000_FFFF) | ((word & 0xFFFF) << 16),
            2 => (old & 0x0000_00FF) | ((word & 0x00FF_FFFF) << 8),
            _ => word,
        };
        (rt, val)
    }

    fn lwr(&mut self, instr: u32, bus: &Bus) -> (usize, u32) {
        let rt = ((instr >> 16) & 0x1F) as usize;
        let rs = ((instr >> 21) & 0x1F) as usize;
        let imm = Self::sign_extend_imm(instr);
        let addr = self.reg(rs).wrapping_add(imm);
        let aligned = addr & !3;
        let offset = (addr & 3) as usize;
        let word = bus.read32::<BusRead>(aligned);
        let old = self.reg_with_pending(rt);
        let val = match offset {
            0 => word,
            1 => (old & 0xFF00_0000) | (word >> 8),
            2 => (old & 0xFFFF_0000) | (word >> 16),
            _ => (old & 0xFFFF_FF00) | (word >> 24),
        };
        (rt, val)
    }

    fn swl(&mut self, instr: u32, bus: &mut Bus) {
        let rs = ((instr >> 21) & 0x1F) as usize;
        let rt = ((instr >> 16) & 0x1F) as usize;
        let imm = Self::sign_extend_imm(instr);
        let addr = self.reg(rs).wrapping_add(imm);
        let aligned = addr & !3;
        let offset = (addr & 3) as usize;
        if self.is_isc() {
            return;
        }
        let word = bus.read32::<BusRead>(aligned);
        let val = self.reg(rt);
        let merged = match offset {
            0 => (word & 0xFFFF_FF00) | ((val >> 24) & 0xFF),
            1 => (word & 0xFFFF_0000) | ((val >> 16) & 0xFFFF),
            2 => (word & 0xFF00_0000) | ((val >> 8) & 0x00FF_FFFF),
            _ => val,
        };
        bus.write32::<BusRead>(aligned, merged);
    }

    fn swr(&mut self, instr: u32, bus: &mut Bus) {
        let rs = ((instr >> 21) & 0x1F) as usize;
        let rt = ((instr >> 16) & 0x1F) as usize;
        let imm = Self::sign_extend_imm(instr);
        let addr = self.reg(rs).wrapping_add(imm);
        let aligned = addr & !3;
        let offset = (addr & 3) as usize;
        if self.is_isc() {
            return;
        }
        let word = bus.read32::<BusRead>(aligned);
        let val = self.reg(rt);
        let merged = match offset {
            0 => val,
            1 => (word & 0x0000_00FF) | ((val & 0x00FF_FFFF) << 8),
            2 => (word & 0x0000_FFFF) | ((val & 0xFFFF) << 16),
            _ => (word & 0x00FF_FFFF) | ((val & 0xFF) << 24),
        };
        bus.write32::<BusRead>(aligned, merged);
    }

    fn reg_with_pending(&self, idx: usize) -> u32 {
        if let Some((reg, val)) = self.load_delay {
            if reg == idx {
                return val;
            }
        }
        self.regs[idx]
    }

    fn reg(&self, idx: usize) -> u32 {
        self.regs[idx]
    }

    fn cop0_op(&mut self, instr: u32) -> Option<(usize, u32)> {
        let co = (instr >> 21) & 0x1F;
        match co {
            0x00 => {
                let rt = ((instr >> 16) & 0x1F) as usize;
                let rd = ((instr >> 11) & 0x1F) as usize;
                let val = self.cop0_read(rd);
                Some((rt, val))
            }
            0x04 => {
                let rt = ((instr >> 16) & 0x1F) as usize;
                let rd = ((instr >> 11) & 0x1F) as usize;
                let val = self.regs[rt];
                self.cop0_write(rd, val);
                None
            }
            0x10..=0x1F => {
                let cop0cmd = instr & 0x3F;
                match cop0cmd {
                    0x10 => {
                        let sr = self.cop0[12];
                        let iec_kuc = (sr >> 2) & 0x3;
                        let iep_kup = (sr >> 4) & 0x3;
                        self.cop0[12] = (sr & !0xF) | iec_kuc | (iep_kup << 2);
                        None
                    }
                    0x01 | 0x02 | 0x06 | 0x08 => {
                        self.raise_exception(0x0A, None);
                        None
                    }
                    _ => None,
                }
            }
            _ => None,
        }
    }

    fn cop0_write(&mut self, reg: usize, val: u32) {
        if reg == 15 {
            return;
        }
        if reg == 13 {
            self.cop0[13] = (self.cop0[13] & !0x300) | (val & 0x300);
            return;
        }
        self.cop0[reg] = val;
    }

    fn cop0_read(&self, reg: usize) -> u32 {
        self.cop0[reg]
    }

    fn cop2_op(&mut self, instr: u32, bus: &mut Bus) -> Option<(usize, u32)> {
        let co = (instr >> 21) & 0x1F;
        match co {
            0x00 => {
                let rt = ((instr >> 16) & 0x1F) as usize;
                let rd = ((instr >> 11) & 0x1F) as usize;
                let val = bus.gte().read_data(rd);
                Some((rt, val))
            }
            0x02 => {
                let rt = ((instr >> 16) & 0x1F) as usize;
                let rd = ((instr >> 11) & 0x1F) as usize;
                let val = bus.gte().read_control(rd);
                Some((rt, val))
            }
            0x04 => {
                let rt = ((instr >> 16) & 0x1F) as usize;
                let rd = ((instr >> 11) & 0x1F) as usize;
                let val = self.regs[rt];
                bus.gte_mut().write_data(rd, val);
                None
            }
            0x06 => {
                let rt = ((instr >> 16) & 0x1F) as usize;
                let rd = ((instr >> 11) & 0x1F) as usize;
                let val = self.regs[rt];
                bus.gte_mut().write_control(rd, val);
                None
            }
            0x10..=0x1F => {
                let func = instr & 0x01FF_FFFF;
                bus.gte_mut().execute_command(func);
                None
            }
            _ => None,
        }
    }

    fn lwc2_op(&mut self, instr: u32, bus: &mut Bus) -> Option<(usize, u32)> {
        let rs = ((instr >> 21) & 0x1F) as usize;
        let rt = ((instr >> 16) & 0x1F) as usize;
        let imm = (instr & 0xFFFF) as u16 as i16 as i32 as u32;
        let addr = self.regs[rs].wrapping_add(imm);
        let val = bus.read32::<BusRead>(addr);
        bus.gte_mut().write_data(rt, val);
        None
    }

    fn swc2_op(&mut self, instr: u32, bus: &mut Bus) -> Option<(usize, u32)> {
        let rs = ((instr >> 21) & 0x1F) as usize;
        let rt = ((instr >> 16) & 0x1F) as usize;
        let imm = (instr & 0xFFFF) as u16 as i16 as i32 as u32;
        let addr = self.regs[rs].wrapping_add(imm);
        let val = bus.gte().read_data(rt);
        bus.write32::<BusWrite>(addr, val);
        None
    }

    fn set_reg(&mut self, idx: usize, val: u32) {
        if idx == 0 {
            return;
        }
        self.written_gpr = Some(idx);
        self.regs[idx] = val;
    }

    const JR_RA: u32 = (31 << 21) | 0x08;

    fn cop_usable(&self, unit: u32, is_cop_op: bool) -> bool {
        let sr = self.cop0[12];
        if sr & (1 << (28 + unit)) != 0 {
            return true;
        }
        unit == 0 && is_cop_op && sr & (1 << 1) == 0
    }

    fn do_printf(&self, bus: &mut Bus) {
        let fmt = self.reg_with_pending(4);
        let sp = self.reg_with_pending(29);
        let regs = [
            self.reg_with_pending(5),
            self.reg_with_pending(6),
            self.reg_with_pending(7),
        ];
        let mut arg_idx = 0u32;
        let mut i = 0u32;

        loop {
            if i >= 1_048_576 {
                break;
            }
            let byte = bus.read8::<BusRead>(fmt.wrapping_add(i));
            i = i.wrapping_add(1);
            if byte == 0 {
                break;
            }
            if byte != b'%' {
                bus.tty_push(byte);
                continue;
            }

            let (spec, zero_pad, width) = self.parse_printf_spec(fmt, &mut i, bus);

            match spec {
                0 => {
                    bus.tty_push(b'%');
                    break;
                }
                b'%' => bus.tty_push(b'%'),
                b'c' => {
                    let val = self.next_printf_arg(&regs, sp, &mut arg_idx, bus);
                    bus.tty_push((val & 0xFF) as u8);
                }
                b's' => {
                    let ptr = self.next_printf_arg(&regs, sp, &mut arg_idx, bus);
                    for off in 0..1_048_576u32 {
                        let b = bus.read8::<BusRead>(ptr.wrapping_add(off));
                        if b == 0 {
                            break;
                        }
                        bus.tty_push(b);
                    }
                }
                b'd' | b'i' => {
                    let val = self.next_printf_arg(&regs, sp, &mut arg_idx, bus) as i32;
                    emit_signed(bus, val, zero_pad, width);
                }
                b'u' => {
                    let val = self.next_printf_arg(&regs, sp, &mut arg_idx, bus);
                    emit_unsigned(bus, val, zero_pad, width);
                }
                b'x' => {
                    let val = self.next_printf_arg(&regs, sp, &mut arg_idx, bus);
                    emit_hex(bus, val, false, zero_pad, width);
                }
                b'X' => {
                    let val = self.next_printf_arg(&regs, sp, &mut arg_idx, bus);
                    emit_hex(bus, val, true, zero_pad, width);
                }
                _ => {
                    bus.tty_push(b'%');
                    bus.tty_push(spec);
                }
            }
        }
    }

    fn parse_printf_spec(&self, fmt: u32, i: &mut u32, bus: &Bus) -> (u8, bool, usize) {
        let first = bus.read8::<BusRead>(fmt.wrapping_add(*i));
        *i = i.wrapping_add(1);
        if first == 0 {
            return (0, false, 0);
        }

        let mut zero_pad = false;
        let mut ch = first;
        let mut width = 0usize;

        if ch == b'0' {
            zero_pad = true;
            if *i >= 1_048_576 {
                return (b'0', false, 0);
            }
            ch = bus.read8::<BusRead>(fmt.wrapping_add(*i));
            *i = i.wrapping_add(1);
            if ch == 0 {
                return (0, zero_pad, 0);
            }
        }

        while ch.is_ascii_digit() {
            width = width
                .saturating_mul(10)
                .saturating_add((ch - b'0') as usize);
            if *i >= 1_048_576 {
                return (ch, zero_pad, width);
            }
            ch = bus.read8::<BusRead>(fmt.wrapping_add(*i));
            *i = i.wrapping_add(1);
            if ch == 0 {
                return (0, zero_pad, width);
            }
        }

        (ch, zero_pad, width)
    }

    fn next_printf_arg(&self, regs: &[u32; 3], sp: u32, idx: &mut u32, bus: &Bus) -> u32 {
        let i = *idx;
        *idx += 1;
        if i < 3 {
            regs[i as usize]
        } else {
            let addr = sp.wrapping_add(0x10).wrapping_add((i - 3) * 4);
            bus.read32::<BusRead>(addr)
        }
    }
}

fn emit_signed(bus: &mut Bus, val: i32, zero_pad: bool, width: usize) {
    let negative = val < 0;
    let abs = (val as i64).unsigned_abs();
    let body = format!("{}", abs);
    emit_padded_signed(bus, &body, zero_pad, width, negative);
}

fn emit_unsigned(bus: &mut Bus, val: u32, zero_pad: bool, width: usize) {
    let s = format!("{}", val);
    emit_padded_signed(bus, &s, zero_pad, width, false);
}

fn emit_hex(bus: &mut Bus, val: u32, upper: bool, zero_pad: bool, width: usize) {
    let s = if upper {
        format!("{:X}", val)
    } else {
        format!("{:x}", val)
    };
    emit_padded_signed(bus, &s, zero_pad, width, false);
}

fn emit_padded_signed(bus: &mut Bus, body: &str, zero_pad: bool, width: usize, negative: bool) {
    let body_len = body.len();
    let sign_len = if negative { 1 } else { 0 };
    let total_len = body_len + sign_len;

    let pad_char = if zero_pad { b'0' } else { b' ' };

    if zero_pad && negative {
        bus.tty_push(b'-');
        for _ in 0..(width.saturating_sub(total_len)) {
            bus.tty_push(pad_char);
        }
    } else {
        for _ in 0..(width.saturating_sub(total_len)) {
            bus.tty_push(pad_char);
        }
        if negative {
            bus.tty_push(b'-');
        }
    }

    for b in body.bytes() {
        bus.tty_push(b);
    }
}

impl Default for Cpu {
    fn default() -> Self {
        Self::new()
    }
}
