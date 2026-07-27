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

    pub fn read32(&self, offset: usize) -> u32 {
        u32::from_le_bytes([
            self.data[offset],
            self.data[offset + 1],
            self.data[offset + 2],
            self.data[offset + 3],
        ])
    }
}
