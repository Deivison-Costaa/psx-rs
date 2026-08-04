use std::cell::{Cell, RefCell};

use crate::memcard::{self, MemoryCard, MemoryCardError};

const ADDRESS_CONTROLLER: u8 = 0x01;

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
    button_state: Cell<u16>,
    irq7_pending: Cell<bool>,
    ack_scheduled: Cell<bool>,
    ack_requested: Cell<bool>,
    memcard: RefCell<MemoryCard>,
    memcard_connected: Cell<bool>,
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
            button_state: Cell::new(0xFFFF),
            irq7_pending: Cell::new(false),
            ack_scheduled: Cell::new(false),
            ack_requested: Cell::new(false),
            memcard: RefCell::new(MemoryCard::new()),
            memcard_connected: Cell::new(false),
        }
    }

    pub fn connect_digital_pad(&self, connected: bool) {
        self.pad_connected.set(connected);
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
        self.button_state.set(buttons);
    }

    pub fn buttons_state(&self) -> u16 {
        self.button_state.get()
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
        let mut s = self.stat.get();
        s &= !0x80;
        self.stat.set(s);
        byte
    }

    fn addressed_device_present(&self) -> bool {
        match self.address.get() {
            ADDRESS_CONTROLLER => self.pad_connected.get(),
            memcard::ADDRESS => self.memcard_connected.get(),
            _ => false,
        }
    }

    fn send_byte(&self, val: u8) {
        self.tx_data.set(val);

        let count = self.byte_count.get();
        if count == 0 {
            self.address.set(val);
            if val == memcard::ADDRESS {
                self.memcard.borrow_mut().begin();
            }
        }

        let present = self.addressed_device_present();
        let (response, ack) = if self.address.get() == memcard::ADDRESS && present {
            self.memcard.borrow_mut().exchange(val)
        } else if count == 0 || !present {
            (0xFF, present)
        } else {
            let r = match count {
                1 => 0x41,
                2 => 0x5A,
                3 => (self.button_state.get() & 0xFF) as u8,
                4 => (self.button_state.get() >> 8) as u8,
                _ => 0xFF,
            };
            (r, true)
        };

        self.rx_fifo.borrow_mut().push(response);
        self.byte_count.set(count + 1);
        if ack {
            self.ack_requested.set(true);
            self.ack_scheduled.set(true);
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

    fn update_ctrl(&self, val: u16) {
        let prev_cs = self.cs_asserted();
        self.ctrl.set(val);

        if !self.cs_asserted() && prev_cs {
            self.byte_count.set(0);
            self.address.set(0);
            self.ack_scheduled.set(false);
            self.ack_requested.set(false);
            self.memcard.borrow_mut().begin();
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
