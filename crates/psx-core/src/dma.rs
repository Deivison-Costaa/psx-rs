use crate::cdrom::Cdrom;
use crate::gpu::Gpu;

#[derive(Debug)]
pub struct Dma {
    madr: [u32; 7],
    bcr: [u32; 7],
    chcr: [u32; 7],
    dpcr: u32,
    dicr: u32,
    irq_line: bool,
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
            irq_line: false,
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
        let flags = self.dicr & !(val & 0x7F00_0000);
        self.dicr = (val & 0x00FF_807F) | (flags & 0x7F00_0000);
        self.recalc_master_flag();
    }

    fn recalc_master_flag(&mut self) {
        let bus_error = self.dicr & (1 << 15) != 0;
        let master_enable = self.dicr & (1 << 23) != 0;
        let algum_flag = self.dicr & 0x7F00_0000 != 0;
        if bus_error || (master_enable && algum_flag) {
            self.dicr |= 1 << 31;
        } else {
            self.dicr &= !(1 << 31);
        }
    }

    fn signal_completion(&mut self, ch: usize) {
        let mascara = self.dicr & (1 << (16 + ch)) != 0;
        let master_enable = self.dicr & (1 << 23) != 0;
        if mascara && master_enable {
            self.dicr |= 1 << (24 + ch);
            self.recalc_master_flag();
        }
    }

    pub fn irq3_pending(&self) -> bool {
        self.dicr & (1 << 31) != 0
    }

    pub fn take_irq3_edge(&mut self) -> bool {
        let nivel = self.irq3_pending();
        let borda = nivel && !self.irq_line;
        self.irq_line = nivel;
        borda
    }

    pub fn try_execute_otc(&mut self, ram: &mut [u8]) {
        if self.dpcr & (1 << 27) == 0 {
            return;
        }
        if (self.chcr[6] & ((1 << 24) | (1 << 28))) != ((1 << 24) | (1 << 28)) {
            return;
        }
        let madr = self.madr[6] & 0x00FF_FFFC;
        let bcr = self.bcr[6] & 0xFFFF;
        let count = if bcr == 0 { 0x10000 } else { bcr as usize };
        let mut addr = madr;
        for i in 0..count {
            let offset = (addr & 0x1F_FF_FF) as usize;
            if offset + 4 <= ram.len() {
                let val = if i == count - 1 {
                    0x00FF_FFFF
                } else {
                    addr.wrapping_sub(4) & 0x00FF_FFFC
                };
                ram[offset..offset + 4].copy_from_slice(&val.to_le_bytes());
            }
            addr = addr.wrapping_sub(4);
        }
        self.chcr[6] &= !((1 << 24) | (1 << 28));
        self.signal_completion(6);
    }

    pub fn try_execute_dma3(&mut self, ram: &mut [u8], cdrom: &Cdrom) {
        if self.dpcr & (1 << 15) == 0 {
            return;
        }
        if (self.chcr[3] & ((1 << 24) | (1 << 28))) != ((1 << 24) | (1 << 28)) {
            return;
        }
        if !cdrom.drqsts_active() {
            return;
        }
        let bcr = self.bcr[3];
        let word_count = if (bcr & 0xFFFF) == 0 {
            0x10000
        } else {
            (bcr & 0xFFFF) as usize
        };
        let mut addr = self.madr[3] & 0x00FF_FFFC;

        for _ in 0..word_count {
            let b0 = cdrom.read8(2) as u32;
            let b1 = cdrom.read8(2) as u32;
            let b2 = cdrom.read8(2) as u32;
            let b3 = cdrom.read8(2) as u32;
            let word = b0 | (b1 << 8) | (b2 << 16) | (b3 << 24);
            let offset = (addr & 0x1F_FF_FF) as usize;
            if offset + 4 <= ram.len() {
                ram[offset..offset + 4].copy_from_slice(&word.to_le_bytes());
            }
            addr = addr.wrapping_add(4);
        }
        self.madr[3] = (self.madr[3] & !0x00FF_FFFF) | (addr & 0x00FF_FFFF);
        self.chcr[3] &= !((1 << 24) | (1 << 28));
        self.signal_completion(3);
    }

    pub fn try_execute_dma2(&mut self, ram: &mut [u8], gpu: &mut Gpu) {
        if self.dpcr & (1 << 11) == 0 {
            return;
        }
        if self.chcr[2] & (1 << 24) == 0 {
            return;
        }
        let sync_mode = (self.chcr[2] >> 9) & 3;
        match sync_mode {
            1 => self.execute_block(ram, gpu),
            2 => self.execute_linked_list(ram, gpu),
            _ => {}
        }
    }

    fn execute_block(&mut self, ram: &mut [u8], gpu: &mut Gpu) {
        let bcr = self.bcr[2];
        let bs = if (bcr & 0xFFFF) == 0 {
            0x10000
        } else {
            (bcr & 0xFFFF) as usize
        };
        let ba = if ((bcr >> 16) & 0xFFFF) == 0 {
            0x10000
        } else {
            ((bcr >> 16) & 0xFFFF) as usize
        };
        let step: i32 = if self.chcr[2] & 2 != 0 { -4 } else { 4 };
        let mut addr = self.madr[2] & 0x00FF_FFFC;

        for _ in 0..ba {
            for _ in 0..bs {
                let offset = (addr & 0x1F_FF_FF) as usize;
                if offset + 4 <= ram.len() {
                    let word = u32::from_le_bytes([
                        ram[offset],
                        ram[offset + 1],
                        ram[offset + 2],
                        ram[offset + 3],
                    ]);
                    gpu.write32(0, word);
                }
                addr = if step < 0 {
                    addr.wrapping_sub(4)
                } else {
                    addr.wrapping_add(4)
                };
            }
        }
        self.madr[2] = (self.madr[2] & !0x00FF_FFFF) | (addr & 0x00FF_FFFF);
        self.chcr[2] &= !(1 << 24);
        self.signal_completion(2);
    }

    fn execute_linked_list(&mut self, ram: &mut [u8], gpu: &mut Gpu) {
        let mut addr = self.madr[2] & 0x00FF_FFFC;
        let mut node_count = 0;

        loop {
            node_count += 1;
            if node_count > 4096 {
                break;
            }
            let offset = (addr & 0x1F_FF_FF) as usize;
            if offset + 4 > ram.len() {
                break;
            }
            let header = u32::from_le_bytes([
                ram[offset],
                ram[offset + 1],
                ram[offset + 2],
                ram[offset + 3],
            ]);
            let next_addr = header & 0x00FF_FFFF;
            let word_count = (header >> 24) as usize;

            let mut data_addr = addr.wrapping_add(4);
            for _ in 0..word_count {
                let doff = (data_addr & 0x1F_FF_FF) as usize;
                if doff + 4 <= ram.len() {
                    let word = u32::from_le_bytes([
                        ram[doff],
                        ram[doff + 1],
                        ram[doff + 2],
                        ram[doff + 3],
                    ]);
                    gpu.write32(0, word);
                }
                data_addr = data_addr.wrapping_add(4);
            }

            if next_addr == 0x00FF_FFFF || (next_addr & 0x0080_0000) != 0 {
                self.madr[2] = (self.madr[2] & !0x00FF_FFFF) | next_addr;
                break;
            }

            addr = next_addr & 0x00FF_FFFC;
        }
        self.chcr[2] &= !(1 << 24);
        self.signal_completion(2);
    }
}

impl Default for Dma {
    fn default() -> Self {
        Self::new()
    }
}
