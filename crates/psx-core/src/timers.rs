const TIMER_COUNT: usize = 3;

#[derive(Debug)]
struct Timer {
    counter: u16,
    mode: u32,
    target: u16,
}

#[derive(Debug)]
pub struct Timers {
    timers: [Timer; TIMER_COUNT],
}

impl Timer {
    fn new() -> Self {
        Timer {
            counter: 0,
            mode: 0,
            target: 0,
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
        match reg {
            0x0 => self.timers[idx].counter as u32,
            0x4 => self.timers[idx].mode,
            0x8 => self.timers[idx].target as u32,
            _ => 0,
        }
    }

    pub fn write32(&mut self, offset: u32, val: u32) {
        let base = offset & !0xF;
        let reg = offset & 0xF;
        let idx = Self::timer_index(base);
        match reg {
            0x0 => {
                self.timers[idx].counter = (val & 0xFFFF) as u16;
            }
            0x4 => {
                self.timers[idx].mode = val & 0x3FF;
                self.timers[idx].counter = 0;
            }
            0x8 => {
                self.timers[idx].target = (val & 0xFFFF) as u16;
            }
            _ => {}
        }
    }

    pub fn tick(&mut self, base_addr: u32, cycles: u32) {
        let idx = Self::timer_index(base_addr);
        let t = &mut self.timers[idx];
        for _ in 0..cycles {
            t.counter = t.counter.wrapping_add(1);
        }
    }
}

impl Default for Timers {
    fn default() -> Self {
        Self::new()
    }
}
