use std::cell::Cell;

use crate::cdrom_bin_cue::DiscLayout;

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
        }
    }

    pub fn insert_disc(&self) {
        self.disc_inserted.set(true);
        self.motor_on.set(true);
    }

    fn stat_byte(&self) -> u8 {
        let mut s = 0u8;
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
            0x06 => {
                if !self.disc_inserted.get() {
                    self.result_push(self.stat_byte() | 0x01);
                    self.result_push(0x80);
                    self.intsts.set(5);
                    self.busy.set(false);
                } else {
                    self.reading.set(true);
                    self.read_mode.set(1);
                    self.result_push(self.stat_byte());
                    self.intsts.set(3);
                    self.pending_second.set(5);
                }
            }
            0x09 => {
                let stat = self.stat_byte();
                self.result_push(stat);
                self.intsts.set(3);
                self.pending_second.set(4);
            }
            0x0A => {
                if self.pending_second.get() == 1 {
                    self.busy.set(false);
                    return;
                }
                if self.disc_inserted.get() {
                    self.motor_on.set(true);
                }
                self.result_push(self.stat_byte());
                self.intsts.set(3);
                self.pending_second.set(1);
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
                    self.pending_second.set(3);
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
                self.pending_second.set(2);
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
                    self.result_push(self.stat_byte());
                    self.intsts.set(3);
                    self.pending_second.set(5);
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
        disc_layout: Option<&DiscLayout>,
        disc_bin: Option<&[u8]>,
    ) {
        match offset & 0x3 {
            0 => self.set_bank(val),
            1 if self.bank.get() == 0 => self.send_command(val),
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
                }
                let pending = self.pending_second.get();
                if pending != 0 {
                    self.deliver_second(disc_layout, disc_bin);
                    if pending == 5 && self.read_mode.get() == 1 {
                        self.pending_second.set(5);
                    }
                }
                if val & 0x40 != 0 {
                    self.param_clear();
                }
            }
            _ => {}
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
                self.result_push(0x08);
                self.result_push(0x40);
                for _ in 0..6 {
                    self.result_push(0x00);
                }
                self.intsts.set(5);
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
                        self.seek_min.get(),
                        self.seek_sec.get(),
                        self.seek_sect.get(),
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
            }
            _ => {}
        }
        self.pending_second.set(0);
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

fn read_sector_from_disc(
    _layout: &DiscLayout,
    bin: &[u8],
    min_bcd: u8,
    sec_bcd: u8,
    sect_bcd: u8,
) -> Option<[u8; 2048]> {
    let abs_sector =
        bcd_to_int(min_bcd) * 60 * 75 + bcd_to_int(sec_bcd) * 75 + bcd_to_int(sect_bcd);
    let offset = abs_sector as usize * 2352;
    let data_start = offset + 0x10;
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
