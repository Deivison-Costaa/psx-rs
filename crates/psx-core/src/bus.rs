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
}

impl Bus {
    pub fn new(ram: Ram, bios: Bios) -> Self {
        Bus { ram, bios }
    }

    fn ram_offset(&self, addr: u32) -> usize {
        let phys = Self::to_physical(addr);
        (phys & 0x1F_FF_FF) as usize
    }

    pub fn read32<Op: MemoryOp>(&self, addr: u32) -> u32 {
        let phys = Self::to_physical(addr);
        if (0x1FC0_0000..0x1FC0_0000 + 0x80000).contains(&phys) {
            let offset = (phys - 0x1FC0_0000) as usize;
            return self.bios.read32(offset);
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
        let idx = self.ram_offset(addr);
        let bytes = val.to_le_bytes();
        self.ram.data[idx] = bytes[0];
        self.ram.data[idx + 1] = bytes[1];
        self.ram.data[idx + 2] = bytes[2];
        self.ram.data[idx + 3] = bytes[3];
    }

    fn read_byte(&self, addr: u32) -> u8 {
        let phys = Self::to_physical(addr);
        if (0x1FC0_0000..0x1FC0_0000 + 0x80000).contains(&phys) {
            return self.bios.raw()[(phys - 0x1FC0_0000) as usize];
        }
        self.ram.data[self.ram_offset(addr)]
    }

    pub fn read8<Op: MemoryOp>(&self, addr: u32) -> u8 {
        self.read_byte(addr)
    }

    pub fn read16<Op: MemoryOp>(&self, addr: u32) -> u16 {
        u16::from_le_bytes([self.read_byte(addr), self.read_byte(addr.wrapping_add(1))])
    }

    pub fn write8<Op: MemoryOp>(&mut self, addr: u32, val: u8) {
        let idx = self.ram_offset(addr);
        self.ram.data[idx] = val;
    }

    pub fn write16<Op: MemoryOp>(&mut self, addr: u32, val: u16) {
        let idx = self.ram_offset(addr);
        let bytes = val.to_le_bytes();
        self.ram.data[idx] = bytes[0];
        self.ram.data[idx + 1] = bytes[1];
    }

    fn to_physical(addr: u32) -> u32 {
        match addr >> 29 {
            0b010 => addr & 0x1FFF_FFFF, // KUSEG: 0x0000_0000..0x1FFF_FFFF
            0b100 => addr & 0x1FFF_FFFF, // KSEG0: 0x8000_0000..0x9FFF_FFFF
            0b101 => addr & 0x1FFF_FFFF, // KSEG1: 0xA000_0000..0xBFFF_FFFF
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
