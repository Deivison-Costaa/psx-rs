use crate::gpu::Gpu;

const SCRATCHPAD_SIZE: usize = 1024;

#[derive(Debug, Clone)]
pub struct Ram {
    data: Vec<u8>,
}

impl Ram {
    pub fn new() -> Self {
        Ram {
            data: vec![0u8; 0x200_000],
        }
    }
}

impl Default for Ram {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
struct Scratchpad {
    data: [u8; SCRATCHPAD_SIZE],
}

impl Scratchpad {
    fn new() -> Self {
        Scratchpad {
            data: [0u8; SCRATCHPAD_SIZE],
        }
    }

    fn read32(&self, offset: usize) -> u32 {
        u32::from_le_bytes([
            self.data[offset],
            self.data[offset + 1],
            self.data[offset + 2],
            self.data[offset + 3],
        ])
    }

    fn write32(&mut self, offset: usize, val: u32) {
        self.data[offset..offset + 4].copy_from_slice(&val.to_le_bytes());
    }
}

#[derive(Debug, Clone)]
struct MemCtrl {
    regs: [u32; 10],
}

impl MemCtrl {
    fn new() -> Self {
        MemCtrl { regs: [0u32; 10] }
    }

    fn read32(&self, phys: u32) -> u32 {
        self.regs[Self::index(phys)]
    }

    fn write32(&mut self, phys: u32, val: u32) {
        self.regs[Self::index(phys)] = val;
    }

    fn index(phys: u32) -> usize {
        match phys {
            0x1F80_1060 => 9,
            _ => ((phys - 0x1F80_1000) / 4) as usize,
        }
    }
}

#[derive(Debug, Clone)]
struct Bcc(u32);

impl Bcc {
    fn new() -> Self {
        Bcc(0)
    }
}

pub struct BusRead;
pub struct BusWrite;

pub trait MemoryOp {
    const READ: bool;
    const WRITE: bool;
}

impl MemoryOp for BusRead {
    const READ: bool = true;
    const WRITE: bool = false;
}

impl MemoryOp for BusWrite {
    const READ: bool = false;
    const WRITE: bool = true;
}

#[derive(Debug)]
pub struct Bus {
    ram: Ram,
    bios: Bios,
    gpu: Gpu,
    scratchpad: Scratchpad,
    mem_ctrl: MemCtrl,
    bcc: Bcc,
    tty_buffer: Vec<u8>,
}

impl Bus {
    pub fn gpu(&self) -> &Gpu {
        &self.gpu
    }

    pub fn new(ram: Ram, bios: Bios) -> Self {
        Bus {
            ram,
            bios,
            gpu: Gpu::new(),
            scratchpad: Scratchpad::new(),
            mem_ctrl: MemCtrl::new(),
            bcc: Bcc::new(),
            tty_buffer: Vec::new(),
        }
    }

    pub fn tty_push(&mut self, byte: u8) {
        self.tty_buffer.push(byte);
    }

    pub fn take_tty(&mut self) -> Vec<u8> {
        std::mem::take(&mut self.tty_buffer)
    }

    fn kseg(addr: u32) -> u8 {
        (addr >> 29) as u8
    }

    fn ram_offset(&self, addr: u32) -> usize {
        let phys = Self::to_physical(addr);
        (phys & 0x1F_FF_FF) as usize
    }

    fn region_read32(&self, phys: u32, kseg: u8) -> Option<u32> {
        match phys {
            0x1F80_0000..=0x1F80_03FF => {
                if kseg == 0b101 {
                    return Some(0);
                }
                let offset = (phys - 0x1F80_0000) as usize;
                Some(self.scratchpad.read32(offset))
            }
            0x1F80_1000..=0x1F80_1023 => Some(self.mem_ctrl.read32(phys)),
            0x1F80_1060 => Some(self.mem_ctrl.read32(phys)),
            0xFFFE_0130 => Some(self.bcc.0),
            0x1F80_1810 | 0x1F80_1814 => Some(self.gpu.read32(phys - 0x1F80_1810)),
            0x1F80_1024..=0x1F80_105F | 0x1F80_1061..=0x1F80_1FFF => Some(0),
            _ => None,
        }
    }

    fn region_write32(&mut self, phys: u32, kseg: u8, val: u32) -> bool {
        match phys {
            0x1F80_0000..=0x1F80_03FF => {
                if kseg == 0b101 {
                    return true;
                }
                let offset = (phys - 0x1F80_0000) as usize;
                self.scratchpad.write32(offset, val);
                true
            }
            0x1F80_1000..=0x1F80_1023 => {
                self.mem_ctrl.write32(phys, val);
                true
            }
            0x1F80_1060 => {
                self.mem_ctrl.write32(phys, val);
                true
            }
            0xFFFE_0130 => {
                self.bcc.0 = val;
                true
            }
            0x1F80_1810 | 0x1F80_1814 => {
                self.gpu.write32(phys - 0x1F80_1810, val);
                true
            }
            0x1F80_1024..=0x1F80_105F | 0x1F80_1061..=0x1F80_1FFF => true,
            _ => false,
        }
    }

    fn region_read_byte(&self, phys: u32, kseg: u8, offset: u32) -> Option<u8> {
        match phys {
            0x1F80_0000..=0x1F80_03FF => {
                if kseg == 0b101 {
                    return Some(0);
                }
                Some(self.scratchpad.data[(phys - 0x1F80_0000 + offset) as usize])
            }
            0x1F80_1000..=0x1F80_1023 => {
                let val = self.mem_ctrl.read32(phys);
                let byte_index = (phys & 3) + offset;
                Some(((val >> (byte_index * 8)) & 0xFF) as u8)
            }
            0x1F80_1060..=0x1F80_1063 => {
                let val = self.mem_ctrl.read32(0x1F80_1060);
                let byte_index = (phys - 0x1F80_1060) + offset;
                Some(((val >> (byte_index * 8)) & 0xFF) as u8)
            }
            0xFFFE_0130..=0xFFFE_0133 => {
                let byte_index = (phys - 0xFFFE_0130) + offset;
                Some(((self.bcc.0 >> (byte_index * 8)) & 0xFF) as u8)
            }
            0x1F80_1810..=0x1F80_1817 => {
                let base = phys & !3;
                let val = self.gpu.peek32(base - 0x1F80_1810);
                let byte = ((val >> ((phys & 3) * 8)) & 0xFF) as u8;
                Some(byte)
            }
            0x1F80_1024..=0x1F80_105F | 0x1F80_1064..=0x1F80_1FFF => Some(0),
            _ => None,
        }
    }

    fn region_write_byte(&mut self, phys: u32, kseg: u8, offset: u32, val: u8) -> bool {
        match phys {
            0x1F80_0000..=0x1F80_03FF => {
                if kseg == 0b101 {
                    return true;
                }
                self.scratchpad.data[(phys - 0x1F80_0000 + offset) as usize] = val;
                true
            }
            0x1F80_1000..=0x1F80_1023 | 0x1F80_1060 | 0xFFFE_0130 => true,
            0x1F80_1810..=0x1F80_1817 => true, // Registradores da GPU sao de 32 bits; escrita de byte e descartada
            0x1F80_1024..=0x1F80_105F | 0x1F80_1061..=0x1F80_1FFF => true,
            _ => false,
        }
    }

    pub fn read32<Op: MemoryOp>(&self, addr: u32) -> u32 {
        let phys = Self::to_physical(addr);
        if (0x1FC0_0000..0x1FC0_0000 + 0x80000).contains(&phys) {
            return self.bios.read32((phys - 0x1FC0_0000) as usize);
        }
        if let Some(val) = self.region_read32(phys, Self::kseg(addr)) {
            return val;
        }
        let idx = self.ram_offset(addr);
        u32::from_le_bytes([
            self.ram.data[idx],
            self.ram.data[idx + 1],
            self.ram.data[idx + 2],
            self.ram.data[idx + 3],
        ])
    }

    pub fn write32<Op: MemoryOp>(&mut self, addr: u32, val: u32) {
        let phys = Self::to_physical(addr);
        if self.region_write32(phys, Self::kseg(addr), val) {
            return;
        }
        let idx = self.ram_offset(addr);
        let bytes = val.to_le_bytes();
        self.ram.data[idx] = bytes[0];
        self.ram.data[idx + 1] = bytes[1];
        self.ram.data[idx + 2] = bytes[2];
        self.ram.data[idx + 3] = bytes[3];
    }

    pub fn read8<Op: MemoryOp>(&self, addr: u32) -> u8 {
        let phys = Self::to_physical(addr);
        if (0x1FC0_0000..0x1FC0_0000 + 0x80000).contains(&phys) {
            return self.bios.raw()[(phys - 0x1FC0_0000) as usize];
        }
        if let Some(val) = self.region_read_byte(phys, Self::kseg(addr), 0) {
            return val;
        }
        self.ram.data[self.ram_offset(addr)]
    }

    pub fn read16<Op: MemoryOp>(&self, addr: u32) -> u16 {
        let phys = Self::to_physical(addr);
        if (0x1FC0_0000..0x1FC0_0000 + 0x80000).contains(&phys) {
            let offset = (phys - 0x1FC0_0000) as usize;
            return u16::from_le_bytes([self.bios.raw()[offset], self.bios.raw()[offset + 1]]);
        }
        if let (Some(lo), Some(hi)) = (
            self.region_read_byte(phys, Self::kseg(addr), 0),
            self.region_read_byte(phys, Self::kseg(addr), 1),
        ) {
            return u16::from_le_bytes([lo, hi]);
        }
        u16::from_le_bytes([
            self.ram.data[self.ram_offset(addr)],
            self.ram.data[self.ram_offset(addr.wrapping_add(1))],
        ])
    }

    pub fn write8<Op: MemoryOp>(&mut self, addr: u32, val: u8) {
        let phys = Self::to_physical(addr);
        if self.region_write_byte(phys, Self::kseg(addr), 0, val) {
            return;
        }
        let idx = self.ram_offset(addr);
        self.ram.data[idx] = val;
    }

    pub fn write16<Op: MemoryOp>(&mut self, addr: u32, val: u16) {
        let phys = Self::to_physical(addr);
        if self.region_write_byte(phys, Self::kseg(addr), 0, val as u8)
            && self.region_write_byte(phys, Self::kseg(addr), 1, (val >> 8) as u8)
        {
            return;
        }
        let idx = self.ram_offset(addr);
        let bytes = val.to_le_bytes();
        self.ram.data[idx] = bytes[0];
        self.ram.data[idx + 1] = bytes[1];
    }

    fn to_physical(addr: u32) -> u32 {
        match addr >> 29 {
            0b010 => addr & 0x1FFF_FFFF, // 0x4000_0000..0x5FFF_FFFF
            0b100 => addr & 0x1FFF_FFFF,
            0b101 => addr & 0x1FFF_FFFF,
            _ => addr,
        }
    }
}

#[derive(Debug)]
pub struct Bios {
    data: Vec<u8>,
}

#[derive(Debug)]
pub enum BiosError {
    WrongSize { got: usize, expected: usize },
}

impl std::fmt::Display for BiosError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BiosError::WrongSize { got, expected } => {
                write!(f, "BIOS size {got} does not match expected {expected}")
            }
        }
    }
}

impl std::error::Error for BiosError {}

impl Bios {
    pub fn from_bytes(bytes: Vec<u8>) -> Result<Bios, BiosError> {
        if bytes.len() != 0x80000 {
            return Err(BiosError::WrongSize {
                got: bytes.len(),
                expected: 0x80000,
            });
        }
        Ok(Bios { data: bytes })
    }

    pub fn size(&self) -> usize {
        self.data.len()
    }

    pub fn raw(&self) -> &[u8] {
        &self.data
    }

    pub fn read32(&self, offset: usize) -> u32 {
        u32::from_le_bytes([
            self.data[offset],
            self.data[offset + 1],
            self.data[offset + 2],
            self.data[offset + 3],
        ])
    }
}
