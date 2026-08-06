use crate::cdrom::Cdrom;
use crate::cdrom_bin_cue::DiscLayout;
use crate::dma::Dma;
use crate::gpu::Gpu;
use crate::gte::Gte;
use crate::irq::Irq;
use crate::mdec::Mdec;
use crate::scheduler::{EventId, ScheduleKey, Scheduler};
use crate::sio::Sio;
use crate::spu::{self, Spu};
use crate::timers::Timers;

const SCRATCHPAD_SIZE: usize = 1024;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
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

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub(crate) struct Scratchpad {
    #[serde(with = "crate::serde_grande")]
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

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub(crate) struct MemCtrl {
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

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub(crate) struct Bcc(u32);

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
const CDROM_RESPONSE: u32 = 2;
const CDROM_SECOND: u32 = 3;
const SIO_ACK: u32 = 4;
const SPU_TICK: u32 = 5;

// Atraso do /ACK depois do ULTIMO pulso de SCK. § Address byte (01h) being sent (L379-386) de
// docs/reference/10-controllers-memcards.md: o driver do kernel ignora pulsos nos primeiros
// 2-3 us (100 ciclos) e desiste em 100 us. O tempo do byte em si vem do baud (Sio::transfer_cycles):
// entregar o /ACK antes disso faz o kernel apaga-lo na limpeza de IRQ7 que ele so faz depois de
// mandar o byte (§ Emulation Note, L316-320).
const SIO_ACK_DELAY_CYCLES: u64 = 338;

#[derive(Debug)]
pub struct Bus {
    pub(crate) ram: Ram,
    bios: Bios,
    pub(crate) gpu: Gpu,
    pub(crate) gte: Gte,
    pub(crate) irq: Irq,
    pub(crate) dma: Dma,
    pub(crate) cdrom: Cdrom,
    disc_layout: Option<DiscLayout>,
    disc_bin: Option<Vec<u8>>,
    pub(crate) timers: Timers,
    pub(crate) sio: Sio,
    pub(crate) mdec: Mdec,
    pub(crate) spu: Spu,
    pub(crate) scratchpad: Scratchpad,
    pub(crate) mem_ctrl: MemCtrl,
    pub(crate) bcc: Bcc,
    pub(crate) tty_buffer: Vec<u8>,
    pub(crate) scheduler: Scheduler,
    pub(crate) total_cycles: u64,
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
        scheduler.schedule(
            ScheduleKey::new(spu::CPU_CYCLES_PER_SAMPLE),
            EventId(SPU_TICK),
        );

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
            mdec: Mdec::new(),
            spu: Spu::new(),
            scratchpad: Scratchpad::new(),
            mem_ctrl: MemCtrl::new(),
            bcc: Bcc::new(),
            tty_buffer: Vec::new(),
            scheduler,
            total_cycles: 0,
        }
    }

    pub fn tem_disco(&self) -> bool {
        self.disc_bin.is_some()
    }

    pub fn bios_size(&self) -> usize {
        self.bios.size()
    }

    pub fn spu(&self) -> &Spu {
        &self.spu
    }

    pub fn spu_mut(&mut self) -> &mut Spu {
        &mut self.spu
    }

    pub fn drain_audio(&mut self) -> Vec<(i16, i16)> {
        self.spu.drain_output()
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

    pub fn sio(&self) -> &Sio {
        &self.sio
    }

    pub fn sio_mut(&mut self) -> &mut Sio {
        &mut self.sio
    }

    pub fn mdec(&self) -> &Mdec {
        &self.mdec
    }

    fn service_sio_irq(&mut self) {
        if self.sio.take_irq7() {
            self.irq.raise(7);
        }
    }

    fn service_spu_irq(&mut self) {
        if self.spu.take_irq9() {
            self.irq.raise(9);
        }
    }

    fn schedule_sio_ack(&mut self) {
        if self.sio.take_ack_request() {
            self.scheduler.cancel(EventId(SIO_ACK));
            self.scheduler.schedule(
                ScheduleKey::new(
                    self.total_cycles + self.sio.transfer_cycles() + SIO_ACK_DELAY_CYCLES,
                ),
                EventId(SIO_ACK),
            );
        }
    }

    fn service_dma_irq(&mut self) {
        if self.dma.take_irq3_edge() {
            self.irq.raise(3);
        }
    }

    fn service_cdrom_irq(&mut self) {
        if self.cdrom.take_irq2_edge() {
            let ints = self.cdrom.intsts();
            let cmd = self.cdrom.pending_cmd();
            eprintln!(
                "# CDROM_IRQ2 step={} total_cycles={} intsts=0x{:02X} pending_cmd=0x{:02X}",
                self.total_cycles,
                self.total_cycles,
                ints,
                cmd.unwrap_or(0xFF)
            );
            self.irq.raise(2);
        }
    }

    fn schedule_cdrom_response(&mut self) {
        if let Some(cmd) = self.cdrom.take_issued_command() {
            self.scheduler.cancel(EventId(CDROM_RESPONSE));
            let prazo = self.total_cycles + Cdrom::first_response_cycles(cmd);
            self.scheduler
                .schedule(ScheduleKey::new(prazo), EventId(CDROM_RESPONSE));
        }
    }

    fn schedule_cdrom_second(&mut self) {
        if self.cdrom.take_second_request() {
            self.scheduler.cancel(EventId(CDROM_SECOND));
            let prazo = self.total_cycles + self.cdrom.second_response_cycles();
            self.scheduler
                .schedule(ScheduleKey::new(prazo), EventId(CDROM_SECOND));
        }
    }

    pub fn total_cycles(&self) -> u64 {
        self.total_cycles
    }

    pub fn tick_timers(&mut self, cycles: u32) {
        self.total_cycles += cycles as u64;

        self.timers.update_gpu_timing(
            self.gpu.cycles_per_pix(),
            self.gpu.video_cycles_per_scanline(),
        );

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
                SPU_TICK => {
                    let (cd_l, cd_r) = self.cdrom.take_audio_frame().unwrap_or((0, 0));
                    self.spu.set_cd_audio(cd_l, cd_r);
                    self.spu.tick();
                    if self.spu.take_irq9() {
                        self.irq.raise(9);
                    }
                    self.scheduler.schedule(
                        ScheduleKey::new(self.total_cycles + spu::CPU_CYCLES_PER_SAMPLE),
                        EventId(SPU_TICK),
                    );
                }
                SIO_ACK => {
                    self.sio.deliver_ack();
                    if self.sio.take_irq7() {
                        self.irq.raise(7);
                    }
                }
                CDROM_RESPONSE => {
                    if self.cdrom.deliver_first() {
                        self.scheduler.cancel(EventId(CDROM_SECOND));
                    }
                    if self.cdrom.take_irq2_edge() {
                        let ints = self.cdrom.intsts();
                        let cmd = self.cdrom.pending_cmd();
                        eprintln!(
                            "# CDROM_RESPONSE_IRQ2 step={} total_cycles={} intsts=0x{:02X} pending_cmd=0x{:02X}",
                            self.total_cycles,
                            self.total_cycles,
                            ints,
                            cmd.unwrap_or(0xFF)
                        );
                        self.irq.raise(2);
                    }
                }
                CDROM_SECOND => {
                    self.cdrom
                        .deliver_second_now(self.disc_layout.as_ref(), self.disc_bin.as_deref());
                    if self.cdrom.take_irq2_edge() {
                        let ints = self.cdrom.intsts();
                        eprintln!(
                            "# CDROM_SECOND_IRQ2 total_cycles={} intsts=0x{:02X}",
                            self.total_cycles, ints
                        );
                        self.irq.raise(2);
                    }
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
            0x1F80_1080..=0x1F80_10EC | 0x1F80_10F0 | 0x1F80_10F4 => self.dma_register_value(phys),
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
            0x1F80_1820 | 0x1F80_1824 => Some(self.mdec.read32(phys - 0x1F80_1820)),
            0x1F80_1C00..=0x1F80_1E7F => Some(
                u32::from(self.spu.read16(phys)) | (u32::from(self.spu.read16(phys + 2)) << 16),
            ),
            0x1F80_1024..=0x1F80_103F
            | 0x1F80_1041..=0x1F80_1043
            | 0x1F80_1045..=0x1F80_105F
            | 0x1F80_1061..=0x1F80_10FF
            | 0x1F80_1130..=0x1F80_1FFF => Some(0),
            0x1F80_1040 => Some(self.sio.read_data()),
            0x1F80_1044 => Some(self.sio.read_stat()),
            0x1F80_2000..=0x1F80_3FFF => Some(0xFFFF_FFFF),
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
                            0 => {
                                self.dma.try_execute_dma0(&self.ram.data, &mut self.mdec);
                                self.dma.try_execute_dma1(&mut self.ram.data, &self.mdec);
                            }
                            1 => self.dma.try_execute_dma1(&mut self.ram.data, &self.mdec),
                            2 => self.dma.try_execute_dma2(&mut self.ram.data, &mut self.gpu),
                            3 => self.dma.try_execute_dma3(&mut self.ram.data, &self.cdrom),
                            4 => self.dma.try_execute_dma4(&mut self.ram.data, &mut self.spu),
                            6 => self.dma.try_execute_otc(&mut self.ram.data),
                            _ => {}
                        }
                    }
                    _ => {}
                }
                self.service_dma_irq();
                true
            }
            0x1F80_10F0 => {
                self.dma.write_dpcr(val);
                self.dma.try_execute_dma0(&self.ram.data, &mut self.mdec);
                self.dma.try_execute_dma1(&mut self.ram.data, &self.mdec);
                self.dma.try_execute_dma2(&mut self.ram.data, &mut self.gpu);
                self.dma.try_execute_dma3(&mut self.ram.data, &self.cdrom);
                self.dma.try_execute_dma4(&mut self.ram.data, &mut self.spu);
                self.dma.try_execute_otc(&mut self.ram.data);
                self.service_dma_irq();
                true
            }
            0x1F80_10F4 => {
                self.dma.write_dicr(val);
                self.service_dma_irq();
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
            0x1F80_1820 | 0x1F80_1824 => {
                self.mdec.write32(phys - 0x1F80_1820, val);
                // Um DMA1 ja armado espera saida do MDEC: alimentar o MDEC pela CPU tem de
                // retomar o canal, como o DREQ faria no console.
                self.dma.try_execute_dma1(&mut self.ram.data, &self.mdec);
                self.service_dma_irq();
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
                self.schedule_cdrom_response();
                self.schedule_cdrom_second();
                if self.cdrom.take_second_dirty() {
                    self.scheduler.cancel(EventId(CDROM_SECOND));
                }
                self.service_cdrom_irq();
                true
            }
            0x1F80_1C00..=0x1F80_1E7F => {
                self.spu.write16(phys, val as u16);
                self.spu.write16(phys + 2, (val >> 16) as u16);
                self.service_spu_irq();
                true
            }
            0x1F80_1044..=0x1F80_104F => {
                let bytes = val.to_le_bytes();
                self.sio.write_byte(phys, bytes[0]);
                self.sio.write_byte(phys + 1, bytes[1]);
                self.sio.write_byte(phys + 2, bytes[2]);
                self.sio.write_byte(phys + 3, bytes[3]);
                self.schedule_sio_ack();
                self.service_sio_irq();
                true
            }
            0x1F80_1024..=0x1F80_103F
            | 0x1F80_1041..=0x1F80_105F
            | 0x1F80_1061..=0x1F80_10FF
            | 0x1F80_1130..=0x1F80_1FFF => true,
            0x1F80_1040 => {
                self.sio.write_data(val);
                self.schedule_sio_ack();
                self.service_sio_irq();
                true
            }
            0x1F80_2000..=0x1F80_3FFF => true,
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
                let byte_index = ((phys & 3) + offset) & 3;
                Some(((val >> (byte_index * 8)) & 0xFF) as u8)
            }
            0x1F80_1800..=0x1F80_1803 => {
                let reg = (phys - 0x1F80_1800 + offset) & 0x3;
                Some(self.cdrom.read8(reg))
            }
            0x1F80_1100..=0x1F80_112F => {
                let base = phys & !3;
                let val = self.timers.peek32(base);
                let byte_index = ((phys & 3) + offset) & 3;
                Some(((val >> (byte_index * 8)) & 0xFF) as u8)
            }
            0x1F80_1C00..=0x1F80_1E7F => {
                let base = phys & !1;
                let val = self.spu.read16(base);
                let byte_index = ((phys & 1) + offset) & 1;
                Some(((val >> (byte_index * 8)) & 0xFF) as u8)
            }
            0x1F80_1080..=0x1F80_10EC | 0x1F80_10F0 | 0x1F80_10F4 => {
                let base = phys & !3;
                let val = self.dma_register_value(base).unwrap_or(0);
                let byte_index = (phys & 3) + offset;
                Some(((val >> (byte_index * 8)) & 0xFF) as u8)
            }
            0x1F80_1024..=0x1F80_103F | 0x1F80_1041..=0x1F80_1043 | 0x1F80_1064..=0x1F80_1FFF => {
                Some(0)
            }
            0x1F80_1040 => Some(self.sio.read_byte(phys + offset)),
            0x1F80_1044..=0x1F80_104F => Some(self.sio.read_byte(phys + offset)),
            0x1F80_2000..=0x1F80_3FFF => Some(0xFF),
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
                self.schedule_cdrom_response();
                self.schedule_cdrom_second();
                if self.cdrom.take_second_dirty() {
                    self.scheduler.cancel(EventId(CDROM_SECOND));
                }
                self.service_cdrom_irq();
                true
            }
            0x1F80_1100..=0x1F80_112F => {
                let base = phys & !3;
                let byte_index = ((phys & 3) + offset) & 3;
                let shift = byte_index * 8;
                let atual = self.timers.peek32(base);
                let novo = (atual & !(0xFFu32 << shift)) | ((val as u32) << shift);
                self.timers.write32(base, novo);
                true
            }
            0x1F80_1C00..=0x1F80_1E7F => {
                let base = phys & !1;
                let old = self.spu.read16(base);
                let byte_index = ((phys & 1) + offset) & 1;
                let deslocamento = byte_index * 8;
                let novo = (old & !(0xFFu16 << deslocamento)) | ((val as u16) << deslocamento);
                self.spu.write16(base, novo);
                true
            }
            0x1F80_1024..=0x1F80_103F | 0x1F80_1041..=0x1F80_1043 | 0x1F80_1061..=0x1F80_1FFF => {
                true
            }
            0x1F80_1040 | 0x1F80_1044..=0x1F80_104F => {
                self.sio.write_byte(phys + offset, val);
                self.schedule_sio_ack();
                self.service_sio_irq();
                true
            }
            0x1F80_2000..=0x1F80_3FFF => true,
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
            0x1F80_1C00..=0x1F80_1E7F => return self.spu.read16(phys),
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
            0x1F80_1C00..=0x1F80_1E7F => {
                // Rota direta (nao via par de region_write_byte): o registrador
                // de fifo manual (1F801DA8h) e escrita-apenas e um push por
                // meia-palavra — compor a partir de dois bytes o duplicaria.
                self.spu.write16(phys, val);
                self.service_spu_irq();
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

    /// § Caution - 8/16-bit writes to certain IO registers (L309) de docs/reference/02-cpu.md:
    /// o registrador de DMA nao decodifica os byte-enables do barramento, entao um `sb`/`sh`
    /// carrega os 32 bits inteiros de `rt` como se fosse um `sw` alinhado.
    fn e_registrador_dma_de_32_bits(phys: u32) -> bool {
        matches!(phys, 0x1F80_1080..=0x1F80_10EC | 0x1F80_10F0 | 0x1F80_10F4)
    }

    /// Mesma decodificacao de endereco de `region_read32` para o banco de DMA, reaproveitada
    /// por `region_read_byte` — leitura de byte/halfword tem de refletir o registrador de
    /// verdade, nao um zero fixo.
    fn dma_register_value(&self, phys: u32) -> Option<u32> {
        match phys {
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
            _ => None,
        }
    }

    pub fn write8_gpr_completo<Op: MemoryOp>(&mut self, addr: u32, gpr: u32) {
        let phys = Self::to_physical(addr);
        if Self::e_registrador_dma_de_32_bits(phys) {
            self.write32::<Op>(addr & !0x3, gpr);
            return;
        }
        self.write8::<Op>(addr, gpr as u8);
    }

    pub fn write16_gpr_completo<Op: MemoryOp>(&mut self, addr: u32, gpr: u32) {
        let phys = Self::to_physical(addr);
        if Self::e_registrador_dma_de_32_bits(phys) {
            self.write32::<Op>(addr & !0x3, gpr);
            return;
        }
        self.write16::<Op>(addr, gpr as u16);
    }

    pub fn load_cycles(addr: u32) -> u32 {
        match Self::to_physical(addr) {
            0x1F80_0000..=0x1F80_03FF => 1,
            0x1F80_1000..=0x1F80_2FFF => 5,
            0x1FC0_0000..=0x1FC7_FFFF => 27,
            _ => 7,
        }
    }

    /// § Scratchpad (L114, L137-140) de docs/reference/01-memory-map.md: "the scratchpad is
    /// NOT executable... a bus error will still occur". A spec local e omissa sobre QUAIS
    /// blocos de I/O tambem faltam ao buscar instrucao — so cita "Unused Memory Regions"
    /// genericamente (§ Memory Exceptions, L156-160). O gabarito de hardware real
    /// (ps1-tests/cpu/code-in-io/psx.log) e o oraculo aqui: testCodeInInterrupts e
    /// testCodeInMDEC lancam (06h), mas testCodeInDMA0/DMAControl/SPU NAO lancam — medido
    /// por instrumentacao na 0172, nao adivinhado. So os dois blocos comprovados entram.
    pub fn fetch_causa_bus_error(addr: u32) -> bool {
        matches!(
            Self::to_physical(addr),
            0x1F80_0000..=0x1F80_03FF          // Scratchpad
            | 0x1F80_1070..=0x1F80_1077        // Interrupt Control (I_STAT/I_MASK)
            | 0x1F80_1820..=0x1F80_1827        // MDEC Registers
        )
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
