pub mod adpcm;
pub mod envelope;
pub mod gauss;
pub mod reverb;
pub mod voice;

use reverb::Reverb;
use voice::{Voice, Volume};

const RAM_SIZE: usize = 512 * 1024;
const RAM_MASK: u32 = (RAM_SIZE - 1) as u32;

pub const VOICES: usize = 24;
/// 33.868.800 Hz / 44.100 Hz. § Unstable and Delayed I/O (L179) de
/// docs/reference/08-spu.md fala em 300h ciclos por amostra.
pub const CPU_CYCLES_PER_SAMPLE: u64 = 768;

const REG_VOICE_BASE: u32 = 0x1F80_1C00;
const REG_VOICE_END: u32 = 0x1F80_1D7F;
const REG_MAIN_VOL_L: u32 = 0x1F80_1D80;
const REG_MAIN_VOL_R: u32 = 0x1F80_1D82;
const REG_VLOUT: u32 = 0x1F80_1D84;
const REG_VROUT: u32 = 0x1F80_1D86;
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
const REG_MBASE: u32 = 0x1F80_1DA2;
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
const REG_CURRENT_MAIN_L: u32 = 0x1F80_1DB8;
const REG_CURRENT_MAIN_R: u32 = 0x1F80_1DBA;
const REG_REVERB_BASE: u32 = 0x1F80_1DC0;
const REG_REVERB_END: u32 = 0x1F80_1DFF;
const REG_VOICE_INTERNAL: u32 = 0x1F80_1E00;
const REG_VOICE_INTERNAL_END: u32 = 0x1F80_1E5F;

const CAPTURE_HALF: u32 = 0x200;
const CAPTURE_SIZE: u32 = 0x400;

/// Teto do anel de saida em quadros estereo: ~0,2 s a 44,1 kHz. O runner headless
/// nunca drena, e o anel nao pode virar vazamento.
const OUTPUT_CAPACITY: usize = 8192;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Spu {
    ram: Vec<u8>,
    voices: Vec<Voice>,
    cnt: u16,
    dtc: u16,
    transfer_addr_reg: u16,
    current_address: u32,
    manual_fifo: Vec<u16>,
    irq_address: u16,
    irq9: bool,
    main_volume_left: Volume,
    main_volume_right: Volume,
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
    reverb: Reverb,
    reverb_fase: bool,
    output: Vec<(i16, i16)>,
}

impl Spu {
    pub fn new() -> Self {
        Spu {
            ram: vec![0u8; RAM_SIZE],
            voices: vec![Voice::default(); VOICES],
            cnt: 0,
            dtc: 0,
            transfer_addr_reg: 0,
            current_address: 0,
            manual_fifo: Vec::new(),
            irq_address: 0,
            irq9: false,
            main_volume_left: Volume::default(),
            main_volume_right: Volume::default(),
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
            reverb: Reverb::default(),
            reverb_fase: false,
            output: Vec::new(),
        }
    }

    fn voice_reg(addr: u32) -> (usize, u32) {
        let offset = addr - REG_VOICE_BASE;
        ((offset / 0x10) as usize, offset % 0x10)
    }

    pub fn read16(&self, addr: u32) -> u16 {
        match addr {
            REG_VOICE_BASE..=REG_VOICE_END => {
                let (i, reg) = Self::voice_reg(addr);
                let v = &self.voices[i];
                match reg {
                    0x0 => v.volume_left.raw,
                    0x2 => v.volume_right.raw,
                    0x4 => v.pitch,
                    0x6 => v.start_address,
                    0x8 => v.adsr as u16,
                    0xA => (v.adsr >> 16) as u16,
                    0xC => v.adsr_env.level as u16,
                    _ => v.repeat_address,
                }
            }
            REG_VOICE_INTERNAL..=REG_VOICE_INTERNAL_END => {
                let offset = addr - REG_VOICE_INTERNAL;
                let v = &self.voices[(offset / 4) as usize];
                if offset % 4 == 0 {
                    v.volume_left.level() as u16
                } else {
                    v.volume_right.level() as u16
                }
            }
            REG_MAIN_VOL_L => self.main_volume_left.raw,
            REG_MAIN_VOL_R => self.main_volume_right.raw,
            REG_VLOUT => self.reverb.vlout,
            REG_VROUT => self.reverb.vrout,
            REG_MBASE => self.reverb.mbase,
            REG_REVERB_BASE..=REG_REVERB_END => {
                self.reverb.regs[((addr - REG_REVERB_BASE) / 2) as usize]
            }
            REG_KON_LO => self.kon as u16,
            REG_KON_HI => (self.kon >> 16) as u16,
            REG_KOFF_LO => self.koff as u16,
            REG_KOFF_HI => (self.koff >> 16) as u16,
            REG_PMON_LO => self.pmon as u16,
            REG_PMON_HI => (self.pmon >> 16) as u16,
            REG_NON_LO => self.non as u16,
            REG_NON_HI => (self.non >> 16) as u16,
            REG_EON_LO => self.eon as u16,
            REG_EON_HI => (self.eon >> 16) as u16,
            REG_ENDX_LO => self.endx as u16,
            REG_ENDX_HI => (self.endx >> 16) as u16,
            REG_IRQ_ADDR => self.irq_address,
            REG_TRANSFER_ADDR => self.transfer_addr_reg,
            REG_CNT => self.cnt,
            REG_DTC => self.dtc,
            REG_STAT => self.status(),
            REG_CD_VOL_L => self.cd_volume_left,
            REG_CD_VOL_R => self.cd_volume_right,
            REG_EXT_VOL_L => self.ext_volume_left,
            REG_EXT_VOL_R => self.ext_volume_right,
            REG_CURRENT_MAIN_L => self.main_volume_left.level() as u16,
            REG_CURRENT_MAIN_R => self.main_volume_right.level() as u16,
            _ => 0,
        }
    }

    // § 1F801DAEh - SPU Status Register (L678) de docs/reference/08-spu.md: bits 5-0
    // espelham o SPUCNT, bit6 e a flag de IRQ9 e bit11 diz em qual metade do buffer
    // de captura a escrita esta.
    fn status(&self) -> u16 {
        let mut s = self.cnt & 0x3F;
        if self.irq9 {
            s |= 1 << 6;
        }
        if self.transfer_mode() >= 2 {
            s |= 1 << 7;
        }
        if self.capture_offset >= CAPTURE_HALF {
            s |= 1 << 11;
        }
        s
    }

    pub fn write16(&mut self, addr: u32, val: u16) {
        match addr {
            REG_VOICE_BASE..=REG_VOICE_END => {
                let (i, reg) = Self::voice_reg(addr);
                let v = &mut self.voices[i];
                match reg {
                    0x0 => v.volume_left.write(val),
                    0x2 => v.volume_right.write(val),
                    0x4 => v.pitch = val,
                    0x6 => v.start_address = val,
                    0x8 => v.adsr = (v.adsr & 0xFFFF_0000) | u32::from(val),
                    0xA => v.adsr = (v.adsr & 0x0000_FFFF) | (u32::from(val) << 16),
                    0xC => v.adsr_env.level = i32::from(val as i16),
                    _ => v.repeat_address = val,
                }
            }
            REG_MAIN_VOL_L => self.main_volume_left.write(val),
            REG_MAIN_VOL_R => self.main_volume_right.write(val),
            REG_VLOUT => self.reverb.vlout = val,
            REG_VROUT => self.reverb.vrout = val,
            REG_MBASE => self.reverb.set_mbase(val),
            REG_REVERB_BASE..=REG_REVERB_END => {
                self.reverb.regs[((addr - REG_REVERB_BASE) / 2) as usize] = val
            }
            REG_KON_LO => {
                self.kon = (self.kon & 0xFFFF_0000) | u32::from(val);
                self.apply_key_on(u32::from(val));
            }
            REG_KON_HI => {
                self.kon = (self.kon & 0xFFFF) | (u32::from(val) << 16);
                self.apply_key_on(u32::from(val) << 16);
            }
            REG_KOFF_LO => {
                self.koff = (self.koff & 0xFFFF_0000) | u32::from(val);
                self.apply_key_off(u32::from(val));
            }
            REG_KOFF_HI => {
                self.koff = (self.koff & 0xFFFF) | (u32::from(val) << 16);
                self.apply_key_off(u32::from(val) << 16);
            }
            REG_PMON_LO => self.pmon = (self.pmon & 0xFFFF_0000) | u32::from(val),
            REG_PMON_HI => self.pmon = (self.pmon & 0xFFFF) | (u32::from(val) << 16),
            REG_NON_LO => self.non = (self.non & 0xFFFF_0000) | u32::from(val),
            REG_NON_HI => self.non = (self.non & 0xFFFF) | (u32::from(val) << 16),
            REG_EON_LO => self.eon = (self.eon & 0xFFFF_0000) | u32::from(val),
            REG_EON_HI => self.eon = (self.eon & 0xFFFF) | (u32::from(val) << 16),
            REG_ENDX_LO => self.endx = (self.endx & 0xFFFF_0000) | u32::from(val),
            REG_ENDX_HI => self.endx = (self.endx & 0xFFFF) | (u32::from(val) << 16),
            REG_IRQ_ADDR => self.irq_address = val,
            REG_TRANSFER_ADDR => {
                self.transfer_addr_reg = val;
                self.current_address = (u32::from(val)).wrapping_mul(8) & RAM_MASK;
            }
            REG_FIFO => {
                if self.manual_fifo.len() < 32 {
                    self.manual_fifo.push(val);
                }
            }
            REG_CNT => {
                self.cnt = val;
                if val & (1 << 6) == 0 {
                    self.irq9 = false;
                }
                if self.transfer_mode() == 1 {
                    self.flush_manual_write();
                }
            }
            REG_DTC => self.dtc = val,
            REG_CD_VOL_L => self.cd_volume_left = val,
            REG_CD_VOL_R => self.cd_volume_right = val,
            REG_EXT_VOL_L => self.ext_volume_left = val,
            REG_EXT_VOL_R => self.ext_volume_right = val,
            _ => {}
        }
    }

    fn apply_key_on(&mut self, mascara: u32) {
        let irq = self.irq_byte_address();
        let mut disparou = false;
        {
            let Spu { ram, voices, .. } = self;
            for (i, voz) in voices.iter_mut().enumerate() {
                if mascara & (1 << i) != 0 {
                    disparou |= voz.key_on(ram, irq);
                }
            }
        }
        self.endx &= !mascara;
        if disparou && self.irq_enabled() {
            self.irq9 = true;
        }
    }

    fn apply_key_off(&mut self, mascara: u32) {
        for (i, voz) in self.voices.iter_mut().enumerate() {
            if mascara & (1 << i) != 0 {
                voz.key_off();
            }
        }
    }

    fn transfer_mode(&self) -> u16 {
        (self.cnt >> 4) & 0b11
    }

    fn irq_byte_address(&self) -> u32 {
        (u32::from(self.irq_address) * 8) & RAM_MASK
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
        if self.current_address == self.irq_byte_address() && self.irq_enabled() {
            self.irq9 = true;
        }
        self.current_address = self.current_address.wrapping_add(2) & RAM_MASK;
    }

    fn read_ram_halfword(&mut self) -> u16 {
        let off = (self.current_address & RAM_MASK) as usize;
        let half = u16::from_le_bytes([self.ram[off], self.ram[off + 1]]);
        if self.current_address == self.irq_byte_address() && self.irq_enabled() {
            self.irq9 = true;
        }
        self.current_address = self.current_address.wrapping_add(2) & RAM_MASK;
        half
    }

    fn irq_enabled(&self) -> bool {
        self.cnt & (1 << 6) != 0 && self.cnt & (1 << 15) != 0
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
        let lo = u32::from(self.read_ram_halfword());
        let hi = u32::from(self.read_ram_halfword());
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

    pub fn noise_level(&self) -> i16 {
        self.noise_level
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

    // § SPU Noise Generator (L633) de docs/reference/08-spu.md.
    fn step_noise(&mut self) {
        let shift = u32::from((self.cnt >> 10) & 0x0F);
        let passo = 4 + i32::from((self.cnt >> 8) & 0x3);
        self.noise_timer -= passo;
        if self.noise_timer < 0 {
            let n = self.noise_level as u16;
            let paridade = ((n >> 15) ^ (n >> 12) ^ (n >> 11) ^ (n >> 10) ^ 1) & 1;
            self.noise_level = ((n << 1) | paridade) as i16;
            let recarga = 0x20000i32 >> shift;
            self.noise_timer += recarga;
            if self.noise_timer < 0 {
                self.noise_timer += recarga;
            }
        }
    }

    /// Um quadro de 44,1 kHz: 24 vozes, entrada de CD, volume principal.
    pub fn tick(&mut self) -> (i16, i16) {
        self.step_noise();
        let ruido = self.noise_level;
        let irq_addr = self.irq_byte_address();
        let irq_ligada = self.irq_enabled();

        let (mut esquerda, mut direita) = (0i32, 0i32);
        let (mut rev_esq, mut rev_dir) = (0i32, 0i32);
        let mut anterior = 0i16;
        let mut pedidos_de_irq = false;
        let mut fim = 0u32;

        for i in 0..VOICES {
            let bit = 1u32 << i;
            let modulador = if self.pmon & bit != 0 && i > 0 {
                Some(anterior)
            } else {
                None
            };
            let usa_ruido = if self.non & bit != 0 {
                Some(ruido)
            } else {
                None
            };
            let Spu { ram, voices, .. } = self;
            let voz = &mut voices[i];
            let passo = voz.step(ram, usa_ruido, modulador, irq_addr);
            anterior = passo.out;
            pedidos_de_irq |= passo.irq;
            if passo.reached_end {
                fim |= bit;
            }
            let vl = (i32::from(passo.out) * voz.volume_left.level()) >> 15;
            let vr = (i32::from(passo.out) * voz.volume_right.level()) >> 15;
            esquerda += vl;
            direita += vr;
            if self.eon & bit != 0 {
                rev_esq += vl;
                rev_dir += vr;
            }
        }

        self.endx |= fim;
        if pedidos_de_irq && irq_ligada {
            self.irq9 = true;
        }

        if self.cnt & 1 != 0 {
            let cd_e = (i32::from(self.cd_left) * volume_de_16_bits(self.cd_volume_left)) >> 15;
            let cd_d = (i32::from(self.cd_right) * volume_de_16_bits(self.cd_volume_right)) >> 15;
            esquerda += cd_e;
            direita += cd_d;
            if self.cnt & (1 << 2) != 0 {
                rev_esq += cd_e;
                rev_dir += cd_d;
            }
        }

        let (rl, rr) = self.run_reverb(rev_esq, rev_dir);
        esquerda += rl;
        direita += rr;

        self.capture();
        self.main_volume_left.tick();
        self.main_volume_right.tick();

        let habilitado = self.cnt & (1 << 15) != 0 && self.cnt & (1 << 14) != 0;
        let quadro = if habilitado {
            (
                aplica_volume(esquerda, self.main_volume_left.level()),
                aplica_volume(direita, self.main_volume_right.level()),
            )
        } else {
            (0, 0)
        };

        if self.output.len() < OUTPUT_CAPACITY {
            self.output.push(quadro);
        }
        quadro
    }

    // § Reverb Formula (L947) de docs/reference/08-spu.md: a unidade roda a 22050 Hz,
    // metade do mixer. O bit7 do SPUCNT so corta a ESCRITA no buffer; a leitura continua.
    fn run_reverb(&mut self, lin: i32, rin: i32) -> (i32, i32) {
        self.reverb_fase = !self.reverb_fase;
        if !self.reverb_fase {
            return (0, 0);
        }
        let escrever = self.cnt & (1 << 7) != 0;
        let Spu { ram, reverb, .. } = self;
        let saida = reverb.run(ram, lin, rin, escrever);
        reverb.advance();
        saida
    }

    // § SPU Memory layout (L140) de docs/reference/08-spu.md: os 4 KiB iniciais
    // guardam CD esquerdo/direito e as vozes 1 e 3 depois do ADSR.
    fn capture(&mut self) {
        let off = self.capture_offset;
        let cd_l = self.cd_left;
        let cd_r = self.cd_right;
        let v1 = self.voice_out(1);
        let v3 = self.voice_out(3);
        for (base, amostra) in [(0u32, cd_l), (0x400, cd_r), (0x800, v1), (0xC00, v3)] {
            let endereco = base + off;
            let bytes = amostra.to_le_bytes();
            self.ram[endereco as usize] = bytes[0];
            self.ram[endereco as usize + 1] = bytes[1];
        }
        self.capture_offset = (off + 2) % CAPTURE_SIZE;
    }
}

fn volume_de_16_bits(raw: u16) -> i32 {
    i32::from(raw as i16)
}

fn aplica_volume(amostra: i32, volume: i32) -> i16 {
    ((amostra.clamp(-0x8000, 0x7FFF) * volume) >> 15).clamp(-0x8000, 0x7FFF) as i16
}

impl Default for Spu {
    fn default() -> Self {
        Self::new()
    }
}
