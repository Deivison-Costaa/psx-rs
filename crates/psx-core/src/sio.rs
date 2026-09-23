use std::cell::{Cell, RefCell};

use crate::dualshock::{self, DualShock, Rumble, Sticks};
use crate::memcard::{self, MemoryCard, MemoryCardError};
use crate::sio1::Sio1;

const PAD_READ: u8 = 0x42;
const CTRL_PORT_2: u16 = 1 << 13;
const JOY_MODE_MASK: u16 = 0x013F;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Sio {
    tx_data: Cell<u8>,
    rx_fifo: RefCell<Vec<u8>>,
    stat: Cell<u32>,
    mode: Cell<u16>,
    ctrl: Cell<u16>,
    baud: Cell<u16>,
    byte_count: Cell<u8>,
    address: Cell<u8>,
    pad_connected: Cell<bool>,
    dualshock_connected: Cell<bool>,
    pad: RefCell<DualShock>,
    irq7_pending: Cell<bool>,
    ack_scheduled: Cell<bool>,
    ack_requested: Cell<bool>,
    memcard: RefCell<MemoryCard>,
    memcard_connected: Cell<bool>,
    sio1: RefCell<Sio1>,
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
            address: Cell::new(0),
            pad_connected: Cell::new(false),
            dualshock_connected: Cell::new(false),
            pad: RefCell::new(DualShock::new()),
            irq7_pending: Cell::new(false),
            ack_scheduled: Cell::new(false),
            ack_requested: Cell::new(false),
            memcard: RefCell::new(MemoryCard::new()),
            memcard_connected: Cell::new(false),
            sio1: RefCell::new(Sio1::default()),
        }
    }

    pub fn connect_digital_pad(&self, connected: bool) {
        self.pad_connected.set(connected);
        self.dualshock_connected.set(false);
    }

    pub fn connect_dualshock(&self, connected: bool) {
        self.pad_connected.set(connected);
        self.dualshock_connected.set(connected);
    }

    pub fn dualshock_connected(&self) -> bool {
        self.dualshock_connected.get()
    }

    pub fn connect_memory_card(&self, connected: bool) {
        self.memcard_connected.set(connected);
    }

    pub fn load_memory_card(&self, bytes: &[u8]) -> Result<(), MemoryCardError> {
        let cartao = MemoryCard::from_bytes(bytes)?;
        *self.memcard.borrow_mut() = cartao;
        self.memcard_connected.set(true);
        Ok(())
    }

    pub fn memory_card_image(&self) -> Vec<u8> {
        self.memcard.borrow().data().to_vec()
    }

    pub fn memory_card_dirty(&self) -> bool {
        self.memcard.borrow_mut().take_dirty()
    }

    pub fn set_buttons(&self, buttons: u16) {
        self.pad.borrow_mut().set_buttons(buttons);
    }

    pub fn buttons_state(&self) -> u16 {
        self.pad.borrow().buttons()
    }

    pub fn set_sticks(&self, sticks: Sticks) {
        self.pad.borrow_mut().set_sticks(sticks);
    }

    pub fn sticks(&self) -> Sticks {
        self.pad.borrow().sticks()
    }

    pub fn set_analog_mode(&self, analog: bool) {
        self.pad.borrow_mut().set_analog(analog);
    }

    pub fn analog_mode(&self) -> bool {
        self.pad.borrow().analog()
    }

    pub fn analog_locked(&self) -> bool {
        self.pad.borrow().locked()
    }

    pub fn press_analog_button(&self) -> bool {
        self.pad.borrow_mut().press_analog_button()
    }

    pub fn rumble(&self) -> Rumble {
        self.pad.borrow().rumble()
    }

    fn cs_asserted(&self) -> bool {
        (self.ctrl.get() & (1 << 1)) != 0
    }

    /// Ciclos que os 8 bits do byte levam para sair, pela taxa configurada em JOY_BAUD e pelo
    /// fator dos bits 0-1 de JOY_MODE. Com os valores do kernel (reload 88h, fator MUL1) dao os
    /// ~250 kHz que § Address byte (01h) being sent (L379-386) de
    /// docs/reference/10-controllers-memcards.md descreve — 136 ciclos por bit.
    pub fn transfer_cycles(&self) -> u64 {
        let fator: u64 = match self.mode.get() & 0x3 {
            2 => 16,
            3 => 64,
            _ => 1,
        };
        let reload = self.baud.get().max(1) as u64;
        8 * reload * fator
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
        byte
    }

    fn addressed_device_present(&self) -> bool {
        if (self.ctrl.get() & CTRL_PORT_2) != 0 {
            return false;
        }
        match self.address.get() {
            dualshock::ADDRESS => self.pad_connected.get(),
            memcard::ADDRESS => self.memcard_connected.get(),
            _ => false,
        }
    }

    fn send_byte(&self, val: u8) {
        self.tx_data.set(val);

        let count = self.byte_count.get();
        if count == 0 {
            self.address.set(val);
            match val {
                memcard::ADDRESS => self.memcard.borrow_mut().begin(),
                dualshock::ADDRESS => self.pad.borrow_mut().begin(),
                _ => {}
            }
        }

        let present = self.addressed_device_present();
        let (response, ack) = if self.address.get() == memcard::ADDRESS && present {
            self.memcard.borrow_mut().exchange(val)
        } else if self.dualshock_connected.get() && present {
            self.pad.borrow_mut().exchange(val)
        } else if count == 0 || !present {
            (0xFF, present)
        } else {
            self.digital_pad_exchange(count, val)
        };

        self.rx_fifo.borrow_mut().push(response);
        self.byte_count.set(count.saturating_add(1));
        if ack {
            self.ack_requested.set(true);
            self.ack_scheduled.set(true);
        }
    }

    fn digital_pad_exchange(&self, count: u8, val: u8) -> (u8, bool) {
        let buttons = self.pad.borrow().buttons();
        match count {
            1 if val == PAD_READ => (0x41, true),
            2 => (0x5A, true),
            3 => (buttons as u8, true),
            4 => ((buttons >> 8) as u8, false),
            _ => {
                self.address.set(0);
                (0xFF, false)
            }
        }
    }

    pub fn take_ack_request(&self) -> bool {
        let pedido = self.ack_requested.get();
        self.ack_requested.set(false);
        pedido
    }

    pub fn deliver_ack(&self) {
        if !self.ack_scheduled.get() {
            return;
        }
        self.ack_scheduled.set(false);

        let mut s = self.stat.get();
        s |= 0x80;
        self.stat.set(s);

        if self.dsr_irq_enabled() {
            let mut s = self.stat.get();
            s |= 1 << 9;
            self.stat.set(s);
            self.irq7_pending.set(true);
        }
    }

    pub fn end_ack_pulse(&self) {
        self.stat.set(self.stat.get() & !0x80);
    }

    fn ack_line_low(&self) -> bool {
        (self.stat.get() & 0x80) != 0
    }

    fn update_ctrl(&self, val: u16) {
        let prev_cs = self.cs_asserted();
        self.ctrl.set(val);

        if !self.cs_asserted() && prev_cs {
            self.byte_count.set(0);
            self.address.set(0);
            self.ack_scheduled.set(false);
            self.ack_requested.set(false);
            self.memcard.borrow_mut().begin();
            self.pad.borrow_mut().begin();
            self.rx_fifo.borrow_mut().clear();
            let mut s = self.stat.get();
            s &= !0x02;
            s &= !0x80;
            self.stat.set(s);
        }

        if (self.ctrl.get() & (1 << 4)) != 0 {
            if !self.ack_line_low() {
                self.stat.set(self.stat.get() & !(1 << 9));
            }
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
            self.address.set(0);
            self.ack_scheduled.set(false);
            self.ack_requested.set(false);
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
            0x1F80_1050..=0x1F80_105F => {
                (self.sio1.borrow().read16(phys) >> ((phys & 1) * 8)) as u8
            }
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
                self.mode.set(((m & 0xFF00) | (val as u16)) & JOY_MODE_MASK);
            }
            0x1F80_1049 => {
                let m = self.mode.get();
                self.mode
                    .set(((m & 0x00FF) | ((val as u16) << 8)) & JOY_MODE_MASK);
            }
            0x1F80_1050..=0x1F80_105F => self.sio1.borrow_mut().write8(phys, val),
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

    pub fn write_half(&self, phys: u32, val: u16) {
        match phys & !1 {
            0x1F80_1048 => self.mode.set(val & JOY_MODE_MASK),
            0x1F80_104A => self.update_ctrl(val),
            0x1F80_104E => self.baud.set(val),
            0x1F80_1050..=0x1F80_105F => self.sio1.borrow_mut().write16(phys, val),
            _ => {
                self.write_byte(phys, val as u8);
                self.write_byte(phys + 1, (val >> 8) as u8);
            }
        }
    }

    pub fn read_data(&self) -> u32 {
        self.pop_rx() as u32
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
