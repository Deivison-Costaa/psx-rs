#[derive(Debug)]
pub struct Dma {
    madr: [u32; 7],
    bcr: [u32; 7],
    chcr: [u32; 7],
    dpcr: u32,
    dicr: u32,
}

impl Dma {
    pub fn new() -> Self {
        let mut chcr = [0u32; 7];
        chcr[6] = 0x0000_0002;
        Dma {
            madr: [0u32; 7],
            bcr: [0u32; 7],
            chcr,
            dpcr: 0x0765_4321,
            dicr: 0,
        }
    }

    pub fn read_madr(&self, ch: usize) -> u32 {
        self.madr[ch] & 0x00FF_FFFF
    }

    pub fn write_madr(&mut self, ch: usize, val: u32) {
        self.madr[ch] = val & 0x00FF_FFFF;
    }

    pub fn read_bcr(&self, ch: usize) -> u32 {
        self.bcr[ch]
    }

    pub fn write_bcr(&mut self, ch: usize, val: u32) {
        self.bcr[ch] = val;
    }

    pub fn read_chcr(&self, ch: usize) -> u32 {
        self.chcr[ch]
    }

    pub fn write_chcr(&mut self, ch: usize, val: u32) {
        if ch == 6 {
            self.chcr[6] = (self.chcr[6] & !0x5100_0000) | (val & 0x5100_0000);
        } else {
            self.chcr[ch] = val;
        }
    }

    pub fn read_dpcr(&self) -> u32 {
        self.dpcr
    }

    pub fn write_dpcr(&mut self, val: u32) {
        self.dpcr = val;
    }

    pub fn read_dicr(&self) -> u32 {
        self.dicr
    }

    pub fn write_dicr(&mut self, val: u32) {
        self.dicr = val;
    }

    pub fn try_execute_otc(&mut self, ram: &mut [u8]) {
        if (self.chcr[6] & ((1 << 24) | (1 << 28))) != ((1 << 24) | (1 << 28)) {
            return;
        }
        let madr = self.madr[6] & 0x00FF_FFFC;
        let bcr = self.bcr[6] & 0xFFFF;
        let count = if bcr == 0 { 0x10000 } else { bcr as usize };
        let mut addr = madr;
        let mut next_val: u32 = 0x00FF_FFFF;
        for _ in 0..count {
            let offset = (addr & 0x1F_FF_FF) as usize;
            if offset + 4 <= ram.len() {
                ram[offset..offset + 4].copy_from_slice(&next_val.to_le_bytes());
            }
            next_val = addr & 0x1F_FFFC;
            addr = addr.wrapping_sub(4);
        }
        self.chcr[6] &= !((1 << 24) | (1 << 28));
    }
}

impl Default for Dma {
    fn default() -> Self {
        Self::new()
    }
}
