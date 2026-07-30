use std::cell::{Cell, RefCell};

#[derive(Debug)]
pub struct Sio {
    tx_data: Cell<u8>,
    rx_fifo: RefCell<Vec<u8>>,
    stat: Cell<u32>,
    mode: Cell<u16>,
    ctrl: Cell<u16>,
    baud: Cell<u16>,
    byte_count: Cell<u8>,
    pad_connected: Cell<bool>,
    irq7_pending: Cell<bool>,
}

impl Sio {
    pub fn new() -> Self {
        Sio {
            tx_data: Cell::new(0),
            rx_fifo: RefCell::new(Vec::new()),
            stat: Cell::new(0x0000_0005),
            mode: Cell::new(0x0000),
            ctrl: Cell::new(0x0000),
            baud: Cell::new(0x0000),
            byte_count: Cell::new(0),
            pad_connected: Cell::new(false),
            irq7_pending: Cell::new(false),
        }
    }

    pub fn connect_digital_pad(&self, connected: bool) {
        self.pad_connected.set(connected);
    }

    fn cs_asserted(&self) -> bool {
        (self.ctrl.get() & (1 << 1)) != 0
    }

    fn dsr_irq_enabled(&self) -> bool {
        (self.ctrl.get() & (1 << 12)) != 0
    }

    pub fn read_stat(&self) -> u32 {
        let mut s = self.stat.get();
        if self.rx_fifo.borrow().is_empty() {
            s &= !0x02;
        } else {
            s |= 0x02;
        }
        s
    }

    fn pop_rx(&self) -> u8 {
        let mut fifo = self.rx_fifo.borrow_mut();
        let byte = if fifo.is_empty() {
            0xFF
        } else {
            fifo.remove(0)
        };
        if fifo.is_empty() {
            let mut s = self.stat.get();
            s &= !0x02;
            self.stat.set(s);
        }
        let mut s = self.stat.get();
        s &= !0x80;
        self.stat.set(s);
        byte
    }

    fn send_byte(&self, val: u8) {
        self.tx_data.set(val);

        let count = self.byte_count.get();
        let response = if count == 0 {
            0xFF
        } else if self.pad_connected.get() {
            match count {
                1 => 0x41,
                2 => 0x5A,
                3 => 0xFF,
                4 => 0xFF,
                _ => 0xFF,
            }
        } else {
            0xFF
        };

        self.rx_fifo.borrow_mut().push(response);
        let mut s = self.stat.get();
        s |= 0x80;
        self.stat.set(s);
        self.byte_count.set(count + 1);

        if self.dsr_irq_enabled() {
            let mut s = self.stat.get();
            s |= 1 << 9;
            self.stat.set(s);
            self.irq7_pending.set(true);
        }
    }

    fn update_ctrl(&self, val: u16) {
        let prev_cs = self.cs_asserted();
        self.ctrl.set(val);

        if !self.cs_asserted() && prev_cs {
            self.byte_count.set(0);
            self.rx_fifo.borrow_mut().clear();
            let mut s = self.stat.get();
            s &= !0x02;
            s &= !0x80;
            self.stat.set(s);
        }

        if (self.ctrl.get() & (1 << 4)) != 0 {
            let mut s = self.stat.get();
            s &= !(1 << 9);
            self.stat.set(s);
            self.irq7_pending.set(false);
            self.ctrl.set(self.ctrl.get() & !(1 << 4));
        }

        if (self.ctrl.get() & (1 << 6)) != 0 {
            self.mode.set(0);
            self.ctrl.set(0);
            self.baud.set(0);
            self.tx_data.set(0);
            self.rx_fifo.borrow_mut().clear();
            self.byte_count.set(0);
            self.stat.set(0x0000_0005);
            self.irq7_pending.set(false);
        }
    }

    pub fn read_byte(&self, phys: u32) -> u8 {
        match phys {
            0x1F80_1040 => self.pop_rx(),
            0x1F80_1044 => (self.read_stat() & 0xFF) as u8,
            0x1F80_1045 => ((self.read_stat() >> 8) & 0xFF) as u8,
            0x1F80_1046 => ((self.read_stat() >> 16) & 0xFF) as u8,
            0x1F80_1047 => ((self.read_stat() >> 24) & 0xFF) as u8,
            0x1F80_1048 => (self.mode.get() & 0xFF) as u8,
            0x1F80_1049 => ((self.mode.get() >> 8) & 0xFF) as u8,
            0x1F80_104A => (self.ctrl.get() & 0xFF) as u8,
            0x1F80_104B => ((self.ctrl.get() >> 8) & 0xFF) as u8,
            0x1F80_104C | 0x1F80_104D => 0,
            0x1F80_104E => (self.baud.get() & 0xFF) as u8,
            0x1F80_104F => ((self.baud.get() >> 8) & 0xFF) as u8,
            _ => 0,
        }
    }

    pub fn write_byte(&self, phys: u32, val: u8) {
        match phys {
            0x1F80_1040 => {
                if self.cs_asserted() {
                    self.send_byte(val);
                }
            }
            0x1F80_1048 => {
                let m = self.mode.get();
                self.mode.set((m & 0xFF00) | (val as u16));
            }
            0x1F80_1049 => {
                let m = self.mode.get();
                self.mode.set((m & 0x00FF) | ((val as u16) << 8));
            }
            0x1F80_104A => {
                let new_ctrl = (self.ctrl.get() & 0xFF00) | (val as u16);
                self.update_ctrl(new_ctrl);
            }
            0x1F80_104B => {
                let new_ctrl = (self.ctrl.get() & 0x00FF) | ((val as u16) << 8);
                self.update_ctrl(new_ctrl);
            }
            0x1F80_104E => {
                let b = self.baud.get();
                self.baud.set((b & 0xFF00) | (val as u16));
            }
            0x1F80_104F => {
                let b = self.baud.get();
                self.baud.set((b & 0x00FF) | ((val as u16) << 8));
            }
            _ => {}
        }
    }

    pub fn read_data(&self) -> u32 {
        let fifo = self.rx_fifo.borrow();
        let b0 = fifo.first().copied().unwrap_or(0) as u32;
        let b1 = fifo.get(1).copied().unwrap_or(b0 as u8) as u32;
        let b2 = fifo.get(2).copied().unwrap_or(b1 as u8) as u32;
        let b3 = fifo.get(3).copied().unwrap_or(b2 as u8) as u32;
        b0 | (b1 << 8) | (b2 << 16) | (b3 << 24)
    }

    pub fn write_data(&self, val: u32) {
        if self.cs_asserted() {
            self.send_byte((val & 0xFF) as u8);
        }
    }

    pub fn write_ctrl(&self, val: u16) {
        self.update_ctrl(val);
    }

    pub fn read_ctrl(&self) -> u16 {
        self.ctrl.get()
    }

    pub fn write_tx(&self, val: u8) {
        self.write_byte(0x1F80_1040, val);
    }

    pub fn read_rx(&self) -> u8 {
        self.read_byte(0x1F80_1040)
    }

    pub fn take_irq7(&self) -> bool {
        let pending = self.irq7_pending.get();
        self.irq7_pending.set(false);
        pending
    }
}

impl Default for Sio {
    fn default() -> Self {
        Self::new()
    }
}
