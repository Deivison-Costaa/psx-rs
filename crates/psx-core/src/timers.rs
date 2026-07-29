use std::cell::Cell;

const TIMER_COUNT: usize = 3;

#[derive(Debug)]
struct Timer {
    counter: Cell<u16>,
    mode: Cell<u32>,
    target: u16,
    cycle_acc: Cell<u32>,
    prev_sync_signal: Cell<bool>,
    mode3_triggered: Cell<bool>,
}

#[derive(Debug)]
pub struct Timers {
    timers: [Timer; TIMER_COUNT],
}

impl Timer {
    fn new() -> Self {
        Timer {
            counter: Cell::new(0),
            mode: Cell::new(0),
            target: 0,
            cycle_acc: Cell::new(0),
            prev_sync_signal: Cell::new(false),
            mode3_triggered: Cell::new(false),
        }
    }
}

impl Timers {
    pub fn new() -> Self {
        Timers {
            timers: [Timer::new(), Timer::new(), Timer::new()],
        }
    }

    fn timer_index(base_addr: u32) -> usize {
        ((base_addr.wrapping_sub(0x1F80_1100) / 0x10) & 0x3) as usize
    }

    pub fn read32(&self, offset: u32) -> u32 {
        let base = offset & !0xF;
        let reg = offset & 0xF;
        let idx = Self::timer_index(base);
        let t = &self.timers[idx];
        match reg {
            0x0 => t.counter.get() as u32,
            0x4 => {
                let val = t.mode.get();
                t.mode.set(val & !((1 << 11) | (1 << 12)));
                val
            }
            0x8 => t.target as u32,
            _ => 0,
        }
    }

    pub fn write32(&mut self, offset: u32, val: u32) {
        let base = offset & !0xF;
        let reg = offset & 0xF;
        let idx = Self::timer_index(base);
        match reg {
            0x0 => {
                self.timers[idx].counter.set((val & 0xFFFF) as u16);
            }
            0x4 => {
                let t = &self.timers[idx];
                let prev = t.mode.get();
                let mut mode = (prev & 0x7C00) | (val & 0x3FF);
                mode |= 1 << 10;
                t.mode.set(mode);
                t.counter.set(0);
                t.cycle_acc.set(0);
                t.prev_sync_signal.set(false);
                t.mode3_triggered.set(false);
            }
            0x8 => {
                self.timers[idx].target = (val & 0xFFFF) as u16;
            }
            _ => {}
        }
    }

    pub fn tick(&mut self, base_addr: u32, cycles: u32, hblank_active: bool, vblank_active: bool) {
        let idx = Self::timer_index(base_addr);
        let t = &self.timers[idx];
        let mode = t.mode.get();
        let sync_enable = mode & 1 != 0;
        let sync_mode = (mode >> 1) & 0x3;
        let reset_on_target = (mode >> 3) & 1 != 0;
        let clock_src = (mode >> 8) & 0x3;

        let sync_signal = match idx {
            0 => hblank_active,
            1 => vblank_active,
            _ => false,
        };

        let prev_sync = t.prev_sync_signal.get();
        let rising_edge = sync_signal && !prev_sync;
        t.prev_sync_signal.set(sync_signal);

        if sync_enable && rising_edge {
            match sync_mode {
                1 | 2 => {
                    t.counter.set(0);
                    t.cycle_acc.set(0);
                }
                3 if !t.mode3_triggered.get() => {
                    t.counter.set(0);
                    t.cycle_acc.set(0);
                    t.mode3_triggered.set(true);
                }
                _ => {}
            }
        }

        let increment = if sync_enable {
            match idx {
                0 | 1 => match sync_mode {
                    0 => (clock_src == 0 || clock_src == 2) && !sync_signal,
                    1 => clock_src == 0 || clock_src == 2,
                    2 => (clock_src == 0 || clock_src == 2) && sync_signal,
                    3 => (clock_src == 0 || clock_src == 2) && t.mode3_triggered.get(),
                    _ => clock_src == 0 || clock_src == 2,
                },
                2 => !matches!(sync_mode, 0 | 3),
                _ => return,
            }
        } else {
            match idx {
                0 | 1 => clock_src == 0 || clock_src == 2,
                2 => true,
                _ => return,
            }
        };

        if !increment {
            return;
        }

        let divisor: u32 = match idx {
            2 if clock_src == 2 || clock_src == 3 => 8,
            _ => 1,
        };

        let prev_acc = t.cycle_acc.get();
        let total = prev_acc + cycles;
        let effective = total / divisor;
        t.cycle_acc.set(total % divisor);

        for _ in 0..effective {
            let prev = t.counter.get();
            t.counter.set(prev.wrapping_add(1));

            if reset_on_target && t.counter.get() == t.target {
                let m = t.mode.get();
                t.mode.set(m | (1 << 11));
                t.counter.set(0);
            }

            if prev == 0xFFFF && t.counter.get() == 0 {
                let m = t.mode.get();
                t.mode.set(m | (1 << 12));
            }
        }
    }
}

impl Default for Timers {
    fn default() -> Self {
        Self::new()
    }
}
