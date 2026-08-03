pub mod adpcm;
pub mod envelope;
pub mod gauss;
pub mod voice;

use voice::Voice;

const RAM_SIZE: usize = 512 * 1024;
const RAM_MASK: u32 = (RAM_SIZE - 1) as u32;

pub const VOICES: usize = 24;
pub const CPU_CYCLES_PER_SAMPLE: u64 = 768;

const REG_VOICE_BASE: u32 = 0x1F80_1C00;
const REG_VOICE_END: u32 = 0x1F80_1D7F;
const REG_MAIN_VOL_L: u32 = 0x1F80_1D80;
const REG_MAIN_VOL_R: u32 = 0x1F80_1D82;
const REG_KON_LO: u32 = 0x1F80_1D88;
const REG_KON_HI: u32 = 0x1F80_1D8A;
const REG_KOFF_LO: u32 = 0x1F80_1D8C;
const REG_KOFF_HI: u32 = 0x1F80_1D8E;
const REG_PMON_LO: u32 = 0x1F80_1D90;
const REG_PMON_HI: u32 = 0x1F80_1D92;
const REG_NON_LO: u32 = 0x1F80_1D94;
const REG_NON_HI: u32 = 0x1F80_1D96;
const REG_EON_LO: u32 = 0x1F80_1D98;
const REG_EON_HI: u32 = 0x1F80_1D9A;
const REG_ENDX_LO: u32 = 0x1F80_1D9C;
const REG_ENDX_HI: u32 = 0x1F80_1D9E;
const REG_IRQ_ADDR: u32 = 0x1F80_1DA4;
const REG_TRANSFER_ADDR: u32 = 0x1F80_1DA6;
const REG_FIFO: u32 = 0x1F80_1DA8;
const REG_CNT: u32 = 0x1F80_1DAA;
const REG_DTC: u32 = 0x1F80_1DAC;
const REG_STAT: u32 = 0x1F80_1DAE;
const REG_CD_VOL_L: u32 = 0x1F80_1DB0;
const REG_CD_VOL_R: u32 = 0x1F80_1DB2;
const REG_EXT_VOL_L: u32 = 0x1F80_1DB4;
const REG_EXT_VOL_R: u32 = 0x1F80_1DB6;

/// Teto do anel de saida em quadros estereo: ~0,2 s a 44,1 kHz. O runner headless
/// nunca drena, e o anel nao pode virar vazamento.
const OUTPUT_CAPACITY: usize = 8192;

#[derive(Debug)]
pub struct Spu {
    ram: Vec<u8>,
    voices: Vec<Voice>,
    cnt: u16,
    dtc: u16,
    stat: u16,
    transfer_addr_reg: u16,
    current_address: u32,
    manual_fifo: Vec<u16>,
    irq_address: u16,
    irq9: bool,
    main_volume_left: u16,
    main_volume_right: u16,
    cd_volume_left: u16,
    cd_volume_right: u16,
    ext_volume_left: u16,
    ext_volume_right: u16,
    kon: u32,
    koff: u32,
    pmon: u32,
    non: u32,
    eon: u32,
    endx: u32,
    noise_level: i16,
    noise_timer: i32,
    cd_left: i16,
    cd_right: i16,
    capture_offset: u32,
    output: Vec<(i16, i16)>,
}

impl Spu {
    pub fn new() -> Self {
        Spu {
            ram: vec![0u8; RAM_SIZE],
            voices: vec![Voice::default(); VOICES],
            cnt: 0,
            dtc: 0,
            stat: 0,
            transfer_addr_reg: 0,
            current_address: 0,
            manual_fifo: Vec::new(),
            irq_address: 0,
            irq9: false,
            main_volume_left: 0,
            main_volume_right: 0,
            cd_volume_left: 0,
            cd_volume_right: 0,
            ext_volume_left: 0,
            ext_volume_right: 0,
            kon: 0,
            koff: 0,
            pmon: 0,
            non: 0,
            eon: 0,
            endx: 0,
            noise_level: 0,
            noise_timer: 0,
            cd_left: 0,
            cd_right: 0,
            capture_offset: 0,
            output: Vec::new(),
        }
    }

    pub fn read16(&self, addr: u32) -> u16 {
        match addr {
            REG_TRANSFER_ADDR => self.transfer_addr_reg,
            REG_CNT => self.cnt,
            REG_DTC => self.dtc,
            REG_STAT => self.stat,
            _ => 0,
        }
    }

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

    pub fn ram_peek16(&self, byte_addr: u32) -> u16 {
        let off = (byte_addr & RAM_MASK) as usize;
        u16::from_le_bytes([self.ram[off], self.ram[off + 1]])
    }

    pub fn dma_push_word(&mut self, word: u32) {
        self.write_ram_halfword((word & 0xFFFF) as u16);
        self.write_ram_halfword((word >> 16) as u16);
    }

    pub fn dma_pop_word(&mut self) -> u32 {
        let lo = self.read_ram_halfword() as u32;
        let hi = self.read_ram_halfword() as u32;
        lo | (hi << 16)
    }

    pub fn set_cd_audio(&mut self, left: i16, right: i16) {
        self.cd_left = left;
        self.cd_right = right;
    }

    pub fn take_irq9(&mut self) -> bool {
        let pending = self.irq9;
        self.irq9 = false;
        pending
    }

    pub fn voice_out(&self, index: usize) -> i16 {
        self.voices.get(index).map(|v| v.out).unwrap_or(0)
    }

    pub fn drain_output(&mut self) -> Vec<(i16, i16)> {
        std::mem::take(&mut self.output)
    }

    pub fn output_len(&self) -> usize {
        self.output.len()
    }

    pub fn tick(&mut self) -> (i16, i16) {
        (0, 0)
    }
}

impl Default for Spu {
    fn default() -> Self {
        Self::new()
    }
}
