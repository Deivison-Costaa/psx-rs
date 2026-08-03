const RAM_SIZE: usize = 512 * 1024;
const RAM_MASK: u32 = (RAM_SIZE - 1) as u32;

const REG_TRANSFER_ADDR: u32 = 0x1F80_1DA6;
const REG_FIFO: u32 = 0x1F80_1DA8;
const REG_CNT: u32 = 0x1F80_1DAA;
const REG_DTC: u32 = 0x1F80_1DAC;
const REG_STAT: u32 = 0x1F80_1DAE;

/// SPU RAM (512 KiB) e transferencia manual/DMA. § docs/reference/08-spu.md
/// L659-824. Vozes/ADPCM/ADSR/mixer/reverb ficam para os itens 7.1-7.4.
#[derive(Debug)]
pub struct Spu {
    ram: Vec<u8>,
    cnt: u16,
    dtc: u16,
    transfer_addr_reg: u16,
    current_address: u32,
    manual_fifo: Vec<u16>,
}

impl Spu {
    pub fn new() -> Self {
        Spu {
            ram: vec![0u8; RAM_SIZE],
            cnt: 0,
            dtc: 0,
            transfer_addr_reg: 0,
            current_address: 0,
            manual_fifo: Vec::new(),
        }
    }

    // § docs/reference/08-spu.md L659-724 (registros) e L673-674/L687 (bits 5-0
    // de STAT espelham CNT; no nosso modelo atomico Busy/bit10 e sempre 0).
    pub fn read16(&self, addr: u32) -> u16 {
        match addr {
            REG_TRANSFER_ADDR => self.transfer_addr_reg,
            REG_CNT => self.cnt,
            REG_DTC => self.dtc,
            REG_STAT => self.cnt & 0x3F,
            _ => 0,
        }
    }

    // § 1F801DA6h (L702-710): current address interna = valor*8. § 1F801DA8h
    // (L712-716): fifo de escrita manual (32 halfwords). § SPU RAM Manual Write
    // (L741-753): mudar CNT para o modo 1 descarrega a fifo na RAM do SPU.
    pub fn write16(&mut self, addr: u32, val: u16) {
        match addr {
            REG_TRANSFER_ADDR => {
                self.transfer_addr_reg = val;
                self.current_address = (val as u32).wrapping_mul(8) & RAM_MASK;
            }
            REG_FIFO => {
                if self.manual_fifo.len() < 32 {
                    self.manual_fifo.push(val);
                }
            }
            REG_CNT => {
                self.cnt = val;
                if Self::transfer_mode(val) == 1 {
                    self.flush_manual_write();
                }
            }
            REG_DTC => self.dtc = val,
            _ => {}
        }
    }

    fn transfer_mode(cnt: u16) -> u16 {
        (cnt >> 4) & 0b11
    }

    fn flush_manual_write(&mut self) {
        let queued = std::mem::take(&mut self.manual_fifo);
        for half in queued {
            self.write_ram_halfword(half);
        }
    }

    fn write_ram_halfword(&mut self, half: u16) {
        let off = (self.current_address & RAM_MASK) as usize;
        self.ram[off] = (half & 0xFF) as u8;
        self.ram[off + 1] = (half >> 8) as u8;
        self.current_address = self.current_address.wrapping_add(2) & RAM_MASK;
    }

    fn read_ram_halfword(&mut self) -> u16 {
        let off = (self.current_address & RAM_MASK) as usize;
        let half = u16::from_le_bytes([self.ram[off], self.ram[off + 1]]);
        self.current_address = self.current_address.wrapping_add(2) & RAM_MASK;
        half
    }

    // § SPU RAM DMA-Write/-Read (L755-773): DMA4 le/escreve direto na RAM do
    // SPU a partir do endereco corrente, sem passar pela fifo manual (DA8h).
    pub fn dma_push_word(&mut self, word: u32) {
        self.write_ram_halfword((word & 0xFFFF) as u16);
        self.write_ram_halfword((word >> 16) as u16);
    }

    pub fn dma_pop_word(&mut self) -> u32 {
        let lo = self.read_ram_halfword() as u32;
        let hi = self.read_ram_halfword() as u32;
        lo | (hi << 16)
    }
}

impl Default for Spu {
    fn default() -> Self {
        Self::new()
    }
}
