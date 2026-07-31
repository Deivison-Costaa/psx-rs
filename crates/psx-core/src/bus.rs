use crate::cdrom::Cdrom;
use crate::cdrom_bin_cue::DiscLayout;
use crate::dma::Dma;
use crate::gpu::Gpu;
use crate::gte::Gte;
use crate::irq::Irq;
use crate::scheduler::{EventId, ScheduleKey, Scheduler};
use crate::sio::Sio;
use crate::timers::Timers;

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

const VBLANK_ENTER: u32 = 0;
const VBLANK_EXIT: u32 = 1;

#[derive(Debug)]
pub struct Bus {
    ram: Ram,
    bios: Bios,
    gpu: Gpu,
    gte: Gte,
    irq: Irq,
    dma: Dma,
    cdrom: Cdrom,
    disc_layout: Option<DiscLayout>,
    disc_bin: Option<Vec<u8>>,
    timers: Timers,
    sio: Sio,
    scratchpad: Scratchpad,
    mem_ctrl: MemCtrl,
    bcc: Bcc,
    tty_buffer: Vec<u8>,
    scheduler: Scheduler,
    total_cycles: u64,
}

impl Bus {
    pub fn gpu(&self) -> &Gpu {
        &self.gpu
    }

    pub fn gpu_mut(&mut self) -> &mut Gpu {
        &mut self.gpu
    }

    pub fn gte(&self) -> &Gte {
        &self.gte
    }

    pub fn gte_mut(&mut self) -> &mut Gte {
        &mut self.gte
    }

    pub fn new(ram: Ram, bios: Bios) -> Self {
        let mut gpu = Gpu::new();
        let frame = gpu.frame_cycles();
        let total_sl = if gpu.video_mode() { 314u64 } else { 263u64 };
        let y1 = gpu.display_range_y1() as u64;
        let y2 = gpu.display_range_y2() as u64;

        let vblank_enter_offset = frame * y2 / total_sl;
        let vblank_exit_offset = frame * y1 / total_sl;

        let mut scheduler = Scheduler::new();
        scheduler.schedule(ScheduleKey::new(vblank_exit_offset), EventId(VBLANK_EXIT));
        scheduler.schedule(ScheduleKey::new(vblank_enter_offset), EventId(VBLANK_ENTER));

        // Raster starts at scanline 0, which is in top blanking (0 < y1)
        gpu.enter_vblank();

        Bus {
            ram,
            bios,
            gpu,
            gte: Gte::new(),
            irq: Irq::new(),
            dma: Dma::new(),
            cdrom: Cdrom::new(),
            disc_layout: None,
            disc_bin: None,
            timers: Timers::new(),
            sio: Sio::new(),
            scratchpad: Scratchpad::new(),
            mem_ctrl: MemCtrl::new(),
            bcc: Bcc::new(),
            tty_buffer: Vec::new(),
            scheduler,
            total_cycles: 0,
        }
    }

    pub fn irq(&self) -> &Irq {
        &self.irq
    }

    pub fn irq_mut(&mut self) -> &mut Irq {
        &mut self.irq
    }

    pub fn cdrom(&self) -> &Cdrom {
        &self.cdrom
    }

    pub fn cdrom_mut(&mut self) -> &mut Cdrom {
        &mut self.cdrom
    }

    pub fn inject_disc(&mut self, layout: DiscLayout, bin: Vec<u8>) {
        self.disc_layout = Some(layout);
        self.disc_bin = Some(bin);
    }

    pub fn timers_mut(&mut self) -> &mut Timers {
        &mut self.timers
    }

    pub fn sio_mut(&mut self) -> &mut Sio {
        &mut self.sio
    }

    fn service_sio_irq(&mut self) {
        if self.sio.take_irq7() {
            self.irq.raise(7);
        }
    }

    fn service_cdrom_irq(&mut self) {
        if self.cdrom.take_irq2_edge() {
            self.irq.raise(2);
        }
    }

    pub fn total_cycles(&self) -> u64 {
        self.total_cycles
    }

    pub fn tick_timers(&mut self, cycles: u32) {
        self.total_cycles += cycles as u64;

        let frame = self.gpu.frame_cycles();
        while let Some(EventId(id)) = self.scheduler.advance_to(self.total_cycles) {
            match id {
                VBLANK_ENTER => {
                    self.gpu.enter_vblank();
                    self.irq.raise(0);
                    self.scheduler.schedule(
                        ScheduleKey::new(self.total_cycles + frame),
                        EventId(VBLANK_ENTER),
                    );
                }
                VBLANK_EXIT => {
                    self.gpu.exit_vblank();
                    self.gpu.toggle_odd_line();
                    self.scheduler.schedule(
                        ScheduleKey::new(self.total_cycles + frame),
                        EventId(VBLANK_EXIT),
                    );
                }
                _ => {}
            }
        }

        let hb = self.gpu.hblank_active();
        let vb = self.gpu.vblank_active();
        for base in &[0x1F80_1100u32, 0x1F80_1110, 0x1F80_1120] {
            if let Some(bit) = self.timers.tick(*base, cycles, hb, vb) {
                self.irq.raise(bit);
            }
        }
    }

    pub fn scheduler_pending_count(&self) -> usize {
        self.scheduler.pending_events().len()
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
            0x1F80_1070 => Some(self.irq.read_stat()),
            0x1F80_1074 => Some(self.irq.read_mask()),
            0x1F80_1080..=0x1F80_10EC => {
                let offset = phys - 0x1F80_1080;
                let ch = (offset / 0x10) as usize;
                match offset % 0x10 {
                    0x0 => Some(self.dma.read_madr(ch)),
                    0x4 => Some(self.dma.read_bcr(ch)),
                    0x8 => Some(self.dma.read_chcr(ch)),
                    _ => Some(0),
                }
            }
            0x1F80_10F0 => Some(self.dma.read_dpcr()),
            0x1F80_10F4 => Some(self.dma.read_dicr()),
            0xFFFE_0130 => Some(self.bcc.0),
            0x1F80_1100..=0x1F80_112F => Some(self.timers.read32(phys)),
            0x1F80_1810 | 0x1F80_1814 => Some(self.gpu.read32(phys - 0x1F80_1810)),
            0x1F80_1800..=0x1F80_1803 => {
                let b0 = self.cdrom.read8(0) as u32;
                let b1 = self.cdrom.read8(1) as u32;
                let b2 = self.cdrom.read8(2) as u32;
                let b3 = self.cdrom.read8(3) as u32;
                Some(b0 | (b1 << 8) | (b2 << 16) | (b3 << 24))
            }
            0x1F80_1024..=0x1F80_103F
            | 0x1F80_1041..=0x1F80_1043
            | 0x1F80_1045..=0x1F80_105F
            | 0x1F80_1061..=0x1F80_10FF
            | 0x1F80_1130..=0x1F80_1FFF => Some(0),
            0x1F80_1040 => Some(self.sio.read_data()),
            0x1F80_1044 => Some(self.sio.read_stat()),
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
            0x1F80_1070 => {
                self.irq.write_stat(val);
                true
            }
            0x1F80_1074 => {
                self.irq.write_mask(val);
                true
            }
            0x1F80_1080..=0x1F80_10EC => {
                let offset = phys - 0x1F80_1080;
                let ch = (offset / 0x10) as usize;
                match offset % 0x10 {
                    0x0 => self.dma.write_madr(ch, val),
                    0x4 => self.dma.write_bcr(ch, val),
                    0x8 => {
                        self.dma.write_chcr(ch, val);
                        match ch {
                            2 => self.dma.try_execute_dma2(&mut self.ram.data, &mut self.gpu),
                            3 => self.dma.try_execute_dma3(&mut self.ram.data, &self.cdrom),
                            6 => self.dma.try_execute_otc(&mut self.ram.data),
                            _ => {}
                        }
                    }
                    _ => {}
                }
                true
            }
            0x1F80_10F0 => {
                self.dma.write_dpcr(val);
                true
            }
            0x1F80_10F4 => {
                self.dma.write_dicr(val);
                true
            }
            0xFFFE_0130 => {
                self.bcc.0 = val;
                true
            }
            0x1F80_1100..=0x1F80_112F => {
                self.timers.write32(phys, val);
                true
            }
            0x1F80_1810 | 0x1F80_1814 => {
                self.gpu.write32(phys - 0x1F80_1810, val);
                true
            }
            0x1F80_1800..=0x1F80_1803 => {
                let disc_layout = self.disc_layout.as_ref();
                let disc_bin = self.disc_bin.as_deref();
                self.cdrom.write8(0, val as u8, disc_layout, disc_bin);
                self.cdrom
                    .write8(1, (val >> 8) as u8, disc_layout, disc_bin);
                self.cdrom
                    .write8(2, (val >> 16) as u8, disc_layout, disc_bin);
                self.cdrom
                    .write8(3, (val >> 24) as u8, disc_layout, disc_bin);
                self.service_cdrom_irq();
                true
            }
            0x1F80_1024..=0x1F80_103F
            | 0x1F80_1041..=0x1F80_105F
            | 0x1F80_1061..=0x1F80_10FF
            | 0x1F80_1130..=0x1F80_1FFF => true,
            0x1F80_1040 => {
                self.sio.write_data(val);
                self.service_sio_irq();
                true
            }
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
                let byte_index = (phys & 3) + offset;
                Some(((val >> (byte_index * 8)) & 0xFF) as u8)
            }
            0x1F80_1800..=0x1F80_1803 => {
                let reg = (phys - 0x1F80_1800 + offset) & 0x3;
                Some(self.cdrom.read8(reg))
            }
            0x1F80_1024..=0x1F80_103F | 0x1F80_1041..=0x1F80_1043 | 0x1F80_1064..=0x1F80_1FFF => {
                Some(0)
            }
            0x1F80_1040 => Some(self.sio.read_byte(phys + offset)),
            0x1F80_1044..=0x1F80_104F => Some(self.sio.read_byte(phys + offset)),
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
            0x1F80_1800..=0x1F80_1803 => {
                let reg = (phys - 0x1F80_1800 + offset) & 0x3;
                self.cdrom.write8(
                    reg,
                    val,
                    self.disc_layout.as_ref(),
                    self.disc_bin.as_deref(),
                );
                self.service_cdrom_irq();
                true
            }
            0x1F80_1024..=0x1F80_103F | 0x1F80_1041..=0x1F80_1043 | 0x1F80_1061..=0x1F80_1FFF => {
                true
            }
            0x1F80_1040 | 0x1F80_1044..=0x1F80_104F => {
                self.sio.write_byte(phys + offset, val);
                self.service_sio_irq();
                true
            }
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
        match phys {
            0x1F80_1070..=0x1F80_1073 => return self.irq.read_stat_byte(phys - 0x1F80_1070),
            0x1F80_1074..=0x1F80_1077 => return self.irq.read_mask_byte(phys - 0x1F80_1074),
            _ => {}
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
        match phys {
            0x1F80_1070 => return (self.irq.read_stat() & 0xFFFF) as u16,
            0x1F80_1072 => return ((self.irq.read_stat() >> 16) & 0xFFFF) as u16,
            0x1F80_1074 => return (self.irq.read_mask() & 0xFFFF) as u16,
            0x1F80_1076 => return ((self.irq.read_mask() >> 16) & 0xFFFF) as u16,
            _ => {}
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
        match phys {
            0x1F80_1070..=0x1F80_1073 => {
                self.irq.write_stat_byte(phys - 0x1F80_1070, val);
                return;
            }
            0x1F80_1074..=0x1F80_1077 => {
                self.irq.write_mask_byte(phys - 0x1F80_1074, val);
                return;
            }
            _ => {}
        }
        if self.region_write_byte(phys, Self::kseg(addr), 0, val) {
            return;
        }
        let idx = self.ram_offset(addr);
        self.ram.data[idx] = val;
    }

    pub fn write16<Op: MemoryOp>(&mut self, addr: u32, val: u16) {
        let phys = Self::to_physical(addr);
        match phys {
            0x1F80_1070 | 0x1F80_1072 => {
                self.irq.write_stat_half(phys - 0x1F80_1070, val);
                return;
            }
            0x1F80_1074 | 0x1F80_1076 => {
                self.irq.write_mask_half(phys - 0x1F80_1074, val);
                return;
            }
            _ => {}
        }
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

    pub fn load_cycles(addr: u32) -> u32 {
        match Self::to_physical(addr) {
            0x1F80_0000..=0x1F80_03FF => 1,
            0x1F80_1000..=0x1F80_2FFF => 5,
            0x1FC0_0000..=0x1FC7_FFFF => 27,
            _ => 7,
        }
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
