use std::cell::Cell;

use crate::cdrom_bin_cue::DiscLayout;

const PAUSE_READING_CYCLES: u64 = 0x021_181C;
const PAUSE_IDLE_CYCLES: u64 = 0x1DF2;
const STOP_MOTOR_CYCLES: u64 = 0x0D3_8ACA;
const STOP_STOPPED_CYCLES: u64 = 0x1D7B;

#[derive(Debug)]
pub struct Cdrom {
    bank: Cell<u8>,
    param_count: Cell<u8>,
    param_buf: Cell<[u8; 16]>,
    result_count: Cell<u8>,
    result_head: Cell<u8>,
    result_buf: Cell<[u8; 16]>,
    intsts: Cell<u8>,
    intmsk: Cell<u8>,
    busy: Cell<bool>,
    pending_second: Cell<u8>,
    second_request: Cell<bool>,
    disc_inserted: Cell<bool>,
    motor_on: Cell<bool>,
    seeking: Cell<bool>,
    reading: Cell<bool>,
    shell_open: Cell<bool>,
    seek_min: Cell<u8>,
    seek_sec: Cell<u8>,
    seek_sect: Cell<u8>,
    data_buffer: Cell<[u8; 2048]>,
    data_pos: Cell<usize>,
    read_mode: Cell<u8>,
    hchpctl: Cell<u8>,
    irq_line: Cell<bool>,
    pending_cmd: Cell<Option<u8>>,
    pending_params: Cell<[u8; 16]>,
    pending_param_count: Cell<u8>,
    issued_cmd: Cell<Option<u8>>,
    int2_pending: Cell<bool>,
    int1_pending: Cell<bool>,
    second_cycles: Cell<u64>,
    read_pos_mm: Cell<u8>,
    read_pos_ss: Cell<u8>,
    read_pos_ff: Cell<u8>,
    mode: Cell<u8>,
    second_dirty: Cell<bool>,
    playing: Cell<bool>,
    play_track: Cell<u8>,
}

impl Cdrom {
    pub fn new() -> Self {
        Cdrom {
            bank: Cell::new(0),
            param_count: Cell::new(0),
            param_buf: Cell::new([0u8; 16]),
            result_count: Cell::new(0),
            result_head: Cell::new(0),
            result_buf: Cell::new([0u8; 16]),
            intsts: Cell::new(0),
            intmsk: Cell::new(0),
            busy: Cell::new(false),
            pending_second: Cell::new(0),
            second_request: Cell::new(false),
            disc_inserted: Cell::new(false),
            motor_on: Cell::new(false),
            seeking: Cell::new(false),
            reading: Cell::new(false),
            shell_open: Cell::new(false),
            seek_min: Cell::new(0),
            seek_sec: Cell::new(0),
            seek_sect: Cell::new(0),
            data_buffer: Cell::new([0u8; 2048]),
            data_pos: Cell::new(0),
            read_mode: Cell::new(0),
            hchpctl: Cell::new(0),
            irq_line: Cell::new(false),
            pending_cmd: Cell::new(None),
            pending_params: Cell::new([0u8; 16]),
            pending_param_count: Cell::new(0),
            issued_cmd: Cell::new(None),
            int2_pending: Cell::new(false),
            int1_pending: Cell::new(false),
            second_cycles: Cell::new(0x4A00),
            read_pos_mm: Cell::new(0),
            read_pos_ss: Cell::new(0),
            read_pos_ff: Cell::new(0),
            mode: Cell::new(0),
            second_dirty: Cell::new(false),
            playing: Cell::new(false),
            play_track: Cell::new(0),
        }
    }

    pub fn insert_disc(&self) {
        self.disc_inserted.set(true);
        self.motor_on.set(true);
    }

    fn stat_byte(&self) -> u8 {
        let mut s = 0u8;
        if self.playing.get() {
            s |= 1 << 7;
        }
        if self.seeking.get() {
            s |= 1 << 6;
        }
        if self.reading.get() {
            s |= 1 << 5;
        }
        if self.shell_open.get() {
            s |= 1 << 4;
        }
        if self.motor_on.get() {
            s |= 1 << 1;
        }
        s
    }

    fn param_push(&self, val: u8) {
        let count = self.param_count.get() as usize;
        if count < 16 {
            let mut buf = self.param_buf.get();
            buf[count] = val;
            self.param_buf.set(buf);
            self.param_count.set((count + 1) as u8);
        }
    }

    fn param_pop(&self) -> u8 {
        let count = self.param_count.get() as usize;
        if count == 0 {
            return 0;
        }
        let buf = self.param_buf.get();
        let val = buf[0];
        let mut new_buf = [0u8; 16];
        new_buf[..count - 1].copy_from_slice(&buf[1..count]);
        self.param_buf.set(new_buf);
        self.param_count.set((count - 1) as u8);
        val
    }

    fn param_is_empty(&self) -> bool {
        self.param_count.get() == 0
    }

    fn param_len(&self) -> u8 {
        self.param_count.get()
    }

    fn param_clear(&self) {
        self.param_count.set(0);
    }

    fn result_push(&self, val: u8) {
        let count = self.result_count.get() as usize;
        if count < 16 {
            let mut buf = self.result_buf.get();
            buf[count] = val;
            self.result_buf.set(buf);
            self.result_count.set((count + 1) as u8);
        }
    }

    fn result_pop(&self) -> u8 {
        let count = self.result_count.get() as usize;
        if count == 0 {
            return 0;
        }
        let head = self.result_head.get() as usize;
        let buf = self.result_buf.get();
        let val = buf[head];
        self.result_head.set(((head + 1) & 0xF) as u8);
        self.result_count.set((count - 1) as u8);
        val
    }

    fn result_is_empty(&self) -> bool {
        self.result_count.get() == 0
    }

    fn result_clear(&self) {
        self.result_count.set(0);
        self.result_head.set(0);
    }

    fn hsts(&self) -> u8 {
        let mut s = self.bank.get() & 0x3;
        if self.param_is_empty() {
            s |= 1 << 3;
        }
        if self.param_len() < 16 {
            s |= 1 << 4;
        }
        if !self.result_is_empty() {
            s |= 1 << 5;
        }
        if self.data_pos.get() < 2048
            && self.read_mode.get() != 0
            && (self.hchpctl.get() & 0x80) != 0
        {
            s |= 1 << 6;
        }
        if self.busy.get() {
            s |= 1 << 7;
        }
        s
    }

    fn set_bank(&self, val: u8) {
        self.bank.set(val & 0x3);
    }

    fn latch_command(&self, cmd: u8) {
        self.pending_cmd.set(Some(cmd));
        self.pending_params.set(self.param_buf.get());
        self.pending_param_count.set(self.param_count.get());
        self.issued_cmd.set(Some(cmd));
        if self.intsts.get() == 0 && !self.int2_pending.get() && !self.int1_pending.get() {
            self.second_dirty.set(true);
        }
    }

    pub fn take_issued_command(&self) -> Option<u8> {
        self.issued_cmd.take()
    }

    pub fn first_response_cycles(cmd: u8) -> u64 {
        match cmd {
            0x0A | 0x1E => 0x1_3CCE,
            _ => 0xC4E1,
        }
    }

    pub fn second_response_cycles_for(cmd: u8) -> u64 {
        match cmd {
            0x06 => 0x4_A00,
            0x07 => 0x04_A00,
            0x0A | 0x1E => 0x4_A00,
            0x12 => 0x04_A00,
            0x15 | 0x16 => 0x04_A00,
            0x1A => 0x4_A00,
            0x1B => 0x4_A00,
            0x1D => 0x04_A00,
            _ => 0x4_A00,
        }
    }

    pub fn second_response_cycles(&self) -> u64 {
        self.second_cycles.get()
    }

    pub fn deliver_first(&self) -> bool {
        let blocked =
            self.intsts.get() != 0 && (self.int2_pending.get() || self.int1_pending.get());
        if blocked {
            return false;
        }
        let cmd = match self.pending_cmd.take() {
            Some(cmd) => cmd,
            None => return false,
        };
        self.param_buf.set(self.pending_params.get());
        self.param_count.set(self.pending_param_count.get());
        self.send_command(cmd);
        self.second_dirty.set(true);
        true
    }

    fn send_command(&self, cmd: u8) {
        self.busy.set(true);
        self.result_clear();
        match cmd {
            0x02 => {
                let mm = self.param_pop();
                let ss = self.param_pop();
                let ff = self.param_pop();
                self.param_clear();
                let bcd_ok = ss < 0x60 && (ss & 0x0F) < 0x0A && ff < 0x75 && (ff & 0x0F) < 0x0A;
                if !bcd_ok {
                    self.result_push(self.stat_byte() | 0x01);
                    self.result_push(0x10);
                    self.intsts.set(5);
                } else if !self.disc_inserted.get() {
                    self.result_push(self.stat_byte() | 0x01);
                    self.result_push(0x80);
                    self.intsts.set(5);
                } else {
                    self.seek_min.set(mm);
                    self.seek_sec.set(ss);
                    self.seek_sect.set(ff);
                    self.result_push(self.stat_byte());
                    self.intsts.set(3);
                }
                self.busy.set(false);
            }
            0x03 => {
                if !self.disc_inserted.get() {
                    self.result_push(self.stat_byte() | 0x01);
                    self.result_push(0x80);
                    self.intsts.set(5);
                    self.busy.set(false);
                } else {
                    self.playing.set(true);
                    self.motor_on.set(true);
                    self.play_track.set(0);
                    self.read_pos_mm.set(self.seek_min.get());
                    self.read_pos_ss.set(self.seek_sec.get());
                    self.read_pos_ff.set(self.seek_sect.get());
                    self.result_push(self.stat_byte());
                    self.intsts.set(3);
                    if self.mode.get() & 0x04 != 0 {
                        self.int1_pending.set(true);
                        self.pending_second.set(6);
                        self.second_cycles
                            .set(Self::second_response_cycles_for(0x03));
                    } else {
                        self.busy.set(false);
                    }
                }
            }
            0x06 => {
                if !self.disc_inserted.get() {
                    self.result_push(self.stat_byte() | 0x01);
                    self.result_push(0x80);
                    self.intsts.set(5);
                    self.busy.set(false);
                } else {
                    self.reading.set(true);
                    self.read_mode.set(1);
                    self.read_pos_mm.set(self.seek_min.get());
                    self.read_pos_ss.set(self.seek_sec.get());
                    self.read_pos_ff.set(self.seek_sect.get());
                    self.result_push(self.stat_byte());
                    self.intsts.set(3);
                    self.int1_pending.set(true);
                    self.pending_second.set(5);
                    self.second_cycles
                        .set(Self::second_response_cycles_for(0x06));
                }
            }
            0x08 => {
                self.int1_pending.set(false);
                let stat = self.stat_byte();
                self.result_push(stat);
                self.intsts.set(3);
                self.int2_pending.set(true);
                self.pending_second.set(4);
                let timing = if self.motor_on.get() {
                    STOP_MOTOR_CYCLES
                } else {
                    STOP_STOPPED_CYCLES
                };
                self.second_cycles.set(timing);
                self.motor_on.set(false);
                self.playing.set(false);
            }
            0x09 => {
                self.int1_pending.set(false);
                let stat = self.stat_byte();
                self.result_push(stat);
                self.intsts.set(3);
                self.int2_pending.set(true);
                self.read_mode.set(0);
                self.playing.set(false);
                self.pending_second.set(4);
                let timing = if self.reading.get() {
                    PAUSE_READING_CYCLES
                } else {
                    PAUSE_IDLE_CYCLES
                };
                self.second_cycles.set(timing);
            }
            0x0A => {
                if self.int2_pending.get() && self.pending_second.get() == 1 {
                    self.busy.set(false);
                    return;
                }
                if self.disc_inserted.get() {
                    self.motor_on.set(true);
                }
                self.result_push(self.stat_byte());
                self.intsts.set(3);
                self.int2_pending.set(true);
                self.pending_second.set(1);
                self.second_cycles
                    .set(Self::second_response_cycles_for(0x0A));
            }
            0x0E => {
                if !self.param_is_empty() {
                    self.mode.set(self.param_pop());
                }
                self.param_clear();
                self.result_push(self.stat_byte());
                self.intsts.set(3);
                self.busy.set(false);
            }
            0x1E => {
                self.result_push(self.stat_byte());
                self.intsts.set(3);
                self.int2_pending.set(true);
                self.pending_second.set(1);
                self.second_cycles
                    .set(Self::second_response_cycles_for(0x1E));
            }
            0x15 => {
                if !self.disc_inserted.get() {
                    self.result_push(self.stat_byte() | 0x01);
                    self.result_push(0x80);
                    self.intsts.set(5);
                    self.busy.set(false);
                } else {
                    self.seeking.set(true);
                    self.result_push(self.stat_byte());
                    self.intsts.set(3);
                    self.int2_pending.set(true);
                    self.pending_second.set(3);
                    self.second_cycles
                        .set(Self::second_response_cycles_for(0x15));
                }
            }
            0x19 => {
                self.intsts.set(3);
                let sub = self.param_pop();
                match sub {
                    0x20 => {
                        self.result_push(0x97);
                        self.result_push(0x01);
                        self.result_push(0x10);
                        self.result_push(0xC2);
                    }
                    0x21 => {
                        self.result_push(0x01);
                    }
                    _ => {
                        self.result_push(self.stat_byte());
                    }
                }
                self.param_clear();
                self.busy.set(false);
            }
            0x1A => {
                self.result_push(self.stat_byte());
                self.intsts.set(3);
                self.int2_pending.set(true);
                self.pending_second.set(2);
                self.second_cycles
                    .set(Self::second_response_cycles_for(0x1A));
            }
            0x1B => {
                if !self.disc_inserted.get() {
                    self.result_push(self.stat_byte() | 0x01);
                    self.result_push(0x80);
                    self.intsts.set(5);
                    self.busy.set(false);
                } else {
                    self.reading.set(true);
                    self.read_mode.set(2);
                    self.read_pos_mm.set(self.seek_min.get());
                    self.read_pos_ss.set(self.seek_sec.get());
                    self.read_pos_ff.set(self.seek_sect.get());
                    self.result_push(self.stat_byte());
                    self.intsts.set(3);
                    self.int1_pending.set(true);
                    self.pending_second.set(5);
                    self.second_cycles
                        .set(Self::second_response_cycles_for(0x1B));
                }
            }
            _ => {
                self.result_push(self.stat_byte());
                self.intsts.set(3);
                self.busy.set(false);
            }
        }
    }

    pub fn read8(&self, offset: u32) -> u8 {
        match offset & 0x3 {
            0 => self.hsts(),
            1 => self.result_pop(),
            2 => {
                let buf = self.data_buffer.get();
                let pos = self.data_pos.get();
                if pos < 2048 {
                    let val = buf[pos];
                    self.data_pos.set(pos + 1);
                    val
                } else {
                    0
                }
            }
            3 => {
                let base = self.intsts.get() & 0x7;
                if self.bank.get() == 1 || self.bank.get() == 3 {
                    base | 0xE0
                } else {
                    self.intmsk.get() | 0xE0
                }
            }
            _ => 0,
        }
    }

    pub fn write8(
        &self,
        offset: u32,
        val: u8,
        _disc_layout: Option<&DiscLayout>,
        _disc_bin: Option<&[u8]>,
    ) {
        match offset & 0x3 {
            0 => self.set_bank(val),
            1 if self.bank.get() == 0 => self.latch_command(val),
            2 if self.bank.get() == 0 => self.param_push(val),
            2 if self.bank.get() == 1 => {
                self.intmsk.set(val & 0x1F);
            }
            3 if self.bank.get() == 0 => {
                self.hchpctl.set(val);
            }
            3 if self.bank.get() == 1 => {
                if val & 0x7 != 0 {
                    let new_intsts = self.intsts.get() & !(val & 0x07);
                    self.intsts.set(new_intsts);
                    self.irq_line.set(self.irq_pending());
                    if new_intsts == 0 {
                        if self.int2_pending.get() {
                            self.int2_pending.set(false);
                            self.second_request.set(true);
                            self.irq_line.set(false);
                        } else if self.int1_pending.get() {
                            self.int1_pending.set(false);
                            self.second_request.set(true);
                            self.irq_line.set(false);
                        } else if self.pending_cmd.get().is_some() {
                            self.deliver_first();
                        }
                    }
                }
                if val & 0x40 != 0 {
                    self.param_clear();
                }
            }
            _ => {}
        }
    }

    pub fn take_second_request(&self) -> bool {
        let v = self.second_request.get();
        self.second_request.set(false);
        v
    }

    pub fn take_second_dirty(&self) -> bool {
        let v = self.second_dirty.get();
        self.second_dirty.set(false);
        v
    }

    pub fn deliver_second_now(&self, disc_layout: Option<&DiscLayout>, disc_bin: Option<&[u8]>) {
        let pending = self.pending_second.get();
        if pending == 0 {
            return;
        }
        self.deliver_second(disc_layout, disc_bin);
        if pending == 6 && self.playing.get() && self.mode.get() & 0x04 != 0 {
            self.pending_second.set(6);
            self.int1_pending.set(true);
            self.second_cycles
                .set(Self::second_response_cycles_for(0x03));
            return;
        }
        if pending == 5 && self.read_mode.get() != 0 {
            self.pending_second.set(5);
            self.int1_pending.set(true);
            let read_cmd = if self.read_mode.get() == 2 {
                0x1B
            } else {
                0x06
            };
            self.second_cycles
                .set(Self::second_response_cycles_for(read_cmd));
        }
    }

    fn deliver_second(&self, disc_layout: Option<&DiscLayout>, disc_bin: Option<&[u8]>) {
        match self.pending_second.get() {
            1 => {
                self.busy.set(false);
                self.result_clear();
                self.result_push(self.stat_byte());
                self.intsts.set(2);
            }
            2 => {
                self.busy.set(false);
                self.result_clear();
                if self.disc_inserted.get() {
                    self.result_push(self.stat_byte());
                    self.result_push(0x00);
                    self.result_push(0x20);
                    self.result_push(0x00);
                    for letra in b"SCEA" {
                        self.result_push(*letra);
                    }
                    self.intsts.set(2);
                } else {
                    self.result_push(0x08);
                    self.result_push(0x40);
                    for _ in 0..6 {
                        self.result_push(0x00);
                    }
                    self.intsts.set(5);
                }
            }
            3 => {
                self.busy.set(false);
                self.seeking.set(false);
                self.result_clear();
                self.result_push(self.stat_byte());
                self.intsts.set(2);
            }
            4 => {
                self.busy.set(false);
                self.reading.set(false);
                self.result_clear();
                self.result_push(self.stat_byte());
                self.intsts.set(2);
            }
            5 => {
                self.busy.set(false);
                self.result_clear();
                self.result_push(self.stat_byte());
                self.intsts.set(1);
                let buf = if let (Some(layout), Some(bin)) = (disc_layout, disc_bin) {
                    read_sector_from_disc(
                        layout,
                        bin,
                        self.read_pos_mm.get(),
                        self.read_pos_ss.get(),
                        self.read_pos_ff.get(),
                    )
                } else {
                    None
                };
                if let Some(buf) = buf {
                    self.data_buffer.set(buf);
                } else {
                    let mut stub = [0u8; 2048];
                    for (i, b) in stub.iter_mut().enumerate() {
                        *b = (i as u8).wrapping_add(1);
                    }
                    self.data_buffer.set(stub);
                }
                self.data_pos.set(0);
                self.hchpctl.set(0);
                advance_read_pos(&self.read_pos_mm, &self.read_pos_ss, &self.read_pos_ff);
            }
            6 => {
                self.busy.set(false);
                self.result_clear();
                self.intsts.set(1);
                loop {
                    advance_read_pos(&self.read_pos_mm, &self.read_pos_ss, &self.read_pos_ff);
                    if bcd_to_int(self.read_pos_ff.get()) % 10 == 0 {
                        break;
                    }
                }
                let amm = self.read_pos_mm.get();
                let ass = self.read_pos_ss.get();
                let asect = self.read_pos_ff.get();
                let (track, index, inicio) = self.trilha_em(disc_layout, amm, ass, asect);
                if self.play_track.get() == 0 {
                    self.play_track.set(track);
                }
                if self.mode.get() & 0x02 != 0 && track != self.play_track.get() {
                    self.playing.set(false);
                    self.result_push(self.stat_byte());
                    self.intsts.set(4);
                    self.pending_second.set(0);
                    return;
                }
                let absoluto = (bcd_to_int(asect) / 10) % 2 == 0;
                self.result_push(self.stat_byte());
                self.result_push(track);
                self.result_push(index);
                if absoluto {
                    self.result_push(amm);
                    self.result_push(ass);
                    self.result_push(asect);
                } else {
                    let (mm, ss, ff) = subtrai_msf((amm, ass, asect), inicio);
                    self.result_push(mm);
                    self.result_push(ss | 0x80);
                    self.result_push(ff);
                }
                self.result_push(0x00);
                self.result_push(0x00);
            }
            _ => {}
        }
        self.pending_second.set(0);
    }

    // § Report (L1246-1256) de docs/reference/06-cdrom.md quer trilha, index e o inicio dela
    // para o tempo relativo. Sem TOC (disco stub dos testes) vale a unica coisa verdadeira de
    // qualquer disco: a trilha 1 comeca em 00:02:00, pela convencao MSF/LBA.
    fn trilha_em(&self, layout: Option<&DiscLayout>, mm: u8, ss: u8, ff: u8) -> (u8, u8, Msf) {
        let padrao = (1u8, 1u8, (0x00u8, 0x02u8, 0x00u8));
        let Some(layout) = layout else { return padrao };
        let agora_lba = msf_para_quadros((mm, ss, ff)).saturating_sub(150);
        let mut achada = padrao;
        for t in &layout.tracks {
            if t.start_lba <= agora_lba {
                achada = (t.number, 1, quadros_para_msf(t.start_lba + 150));
            }
        }
        achada
    }

    pub fn _hchpctl(&self) -> u8 {
        self.hchpctl.get()
    }

    pub fn drqsts_active(&self) -> bool {
        self.data_pos.get() < 2048 && self.read_mode.get() != 0 && (self.hchpctl.get() & 0x80) != 0
    }

    pub fn irq_pending(&self) -> bool {
        (self.intsts.get() & self.intmsk.get() & 0x7) != 0
    }

    pub fn intsts(&self) -> u8 {
        self.intsts.get()
    }

    pub fn pending_cmd(&self) -> Option<u8> {
        self.pending_cmd.get()
    }

    pub fn take_irq2_edge(&self) -> bool {
        let nivel = self.irq_pending();
        let borda = nivel && !self.irq_line.get();
        self.irq_line.set(nivel);
        borda
    }
}

fn bcd_to_int(b: u8) -> u32 {
    ((b >> 4) as u32) * 10 + (b as u32 & 0xF)
}

fn int_to_bcd(n: u32) -> u8 {
    (((n / 10) << 4) | (n % 10)) as u8
}

type Msf = (u8, u8, u8);

fn msf_para_quadros((mm, ss, ff): Msf) -> u32 {
    (bcd_to_int(mm) * 60 + bcd_to_int(ss)) * 75 + bcd_to_int(ff)
}

fn quadros_para_msf(q: u32) -> Msf {
    (
        int_to_bcd(q / (60 * 75)),
        int_to_bcd((q / 75) % 60),
        int_to_bcd(q % 75),
    )
}

fn subtrai_msf(pos: Msf, inicio: Msf) -> Msf {
    let d = msf_para_quadros(pos).saturating_sub(msf_para_quadros(inicio));
    (
        int_to_bcd(d / (60 * 75)),
        int_to_bcd((d / 75) % 60),
        int_to_bcd(d % 75),
    )
}

fn advance_read_pos(mm: &Cell<u8>, ss: &Cell<u8>, ff: &Cell<u8>) {
    let f = bcd_to_int(ff.get()) + 1;
    if f >= 75 {
        ff.set(0);
        let s = bcd_to_int(ss.get()) + 1;
        if s >= 60 {
            ss.set(0);
            mm.set(int_to_bcd(bcd_to_int(mm.get()) + 1));
        } else {
            ss.set(int_to_bcd(s));
        }
    } else {
        ff.set(int_to_bcd(f));
    }
}

fn read_sector_from_disc(
    _layout: &DiscLayout,
    bin: &[u8],
    min_bcd: u8,
    sec_bcd: u8,
    sect_bcd: u8,
) -> Option<[u8; 2048]> {
    let abs_sector =
        bcd_to_int(min_bcd) * 60 * 75 + bcd_to_int(sec_bcd) * 75 + bcd_to_int(sect_bcd);
    let file_sector = abs_sector.checked_sub(150)?;
    let offset = file_sector as usize * 2352;
    if offset + 0x10 > bin.len() {
        return None;
    }
    let cabecalho = if bin[offset + 0x0F] == 0x02 {
        0x18
    } else {
        0x10
    };
    let data_start = offset + cabecalho;
    let data_end = data_start + 2048;
    if data_end > bin.len() {
        return None;
    }
    let mut buf = [0u8; 2048];
    buf.copy_from_slice(&bin[data_start..data_end]);
    Some(buf)
}

impl Default for Cdrom {
    fn default() -> Self {
        Self::new()
    }
}
