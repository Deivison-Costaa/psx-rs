use std::cell::Cell;

const TIMER_COUNT: usize = 3;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct Timer {
    counter: Cell<u16>,
    mode: Cell<u32>,
    target: u16,
    cycle_acc: Cell<u32>,
    prev_sync_signal: Cell<bool>,
    mode3_triggered: Cell<bool>,
    irq_fired_oneshot: Cell<bool>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Timers {
    timers: [Timer; TIMER_COUNT],
    gpu_cycles_per_pix: Cell<u16>,
    gpu_video_cycles_per_scanline: Cell<u16>,
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
            irq_fired_oneshot: Cell::new(false),
        }
    }
}

impl Timers {
    pub fn new() -> Self {
        Timers {
            timers: [Timer::new(), Timer::new(), Timer::new()],
            gpu_cycles_per_pix: Cell::new(10),
            gpu_video_cycles_per_scanline: Cell::new(3413),
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

    pub fn peek32(&self, offset: u32) -> u32 {
        let base = offset & !0xF;
        let reg = offset & 0xF;
        let idx = Self::timer_index(base);
        let t = &self.timers[idx];
        match reg {
            0x0 => t.counter.get() as u32,
            0x4 => t.mode.get(),
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
                t.irq_fired_oneshot.set(false);
            }
            0x8 => {
                self.timers[idx].target = (val & 0xFFFF) as u16;
            }
            _ => {}
        }
    }

    pub fn update_gpu_timing(&mut self, cycles_per_pix: u16, video_cycles_per_scanline: u16) {
        self.gpu_cycles_per_pix.set(cycles_per_pix);
        self.gpu_video_cycles_per_scanline
            .set(video_cycles_per_scanline);
    }

    pub fn tick(
        &mut self,
        base_addr: u32,
        cycles: u32,
        hblank_active: bool,
        vblank_active: bool,
    ) -> Option<u32> {
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
                    0 => !sync_signal,
                    1 => true,
                    2 => sync_signal,
                    3 => t.mode3_triggered.get(),
                    _ => true,
                },
                2 => !matches!(sync_mode, 0 | 3),
                _ => return None,
            }
        } else {
            true
        };

        if !increment {
            return None;
        }

        let (numer, denom): (u64, u64) = match idx {
            0 if clock_src == 1 || clock_src == 3 => {
                let cpp = self.gpu_cycles_per_pix.get() as u64;
                (11, 7 * cpp)
            }
            1 if clock_src == 1 || clock_src == 3 => {
                let vcs = self.gpu_video_cycles_per_scanline.get() as u64;
                (11, 7 * vcs)
            }
            2 if clock_src == 2 || clock_src == 3 => (1, 8),
            _ => (1, 1),
        };

        let prev_acc = t.cycle_acc.get() as u64;
        let total = prev_acc + (cycles as u64) * numer;
        let effective = (total / denom) as u32;
        t.cycle_acc.set((total % denom) as u32);

        let irq_enable_target = (mode >> 4) & 1 != 0;
        let irq_enable_ffff = (mode >> 5) & 1 != 0;
        let irq_once = (mode >> 6) & 1 == 0;
        let irq_toggle = (mode >> 7) & 1 != 0;
        let irq_bit = 4 + idx as u32;

        let mut irq_raised = false;

        for _ in 0..effective {
            let prev = t.counter.get();
            t.counter.set(prev.wrapping_add(1));

            let mut mode_now = t.mode.get();
            let mut mode_changed = false;

            if t.counter.get() == t.target {
                mode_now |= 1 << 11;
                mode_changed = true;
                if irq_enable_target && (!irq_once || !t.irq_fired_oneshot.get()) {
                    let old_bit10 = (mode_now >> 10) & 1;
                    if irq_toggle {
                        mode_now ^= 1 << 10;
                    } else {
                        mode_now &= !(1 << 10);
                    }
                    mode_changed = true;
                    let new_bit10 = (mode_now >> 10) & 1;
                    if old_bit10 == 1 && new_bit10 == 0 {
                        irq_raised = true;
                    }
                    if irq_once {
                        t.irq_fired_oneshot.set(true);
                    }
                }
                if reset_on_target {
                    t.counter.set(0);
                }
            }

            if prev == 0xFFFF && t.counter.get() == 0 {
                mode_now |= 1 << 12;
                mode_changed = true;
                if irq_enable_ffff && (!irq_once || !t.irq_fired_oneshot.get()) {
                    let old_bit10 = (mode_now >> 10) & 1;
                    if irq_toggle {
                        mode_now ^= 1 << 10;
                    } else {
                        mode_now &= !(1 << 10);
                    }
                    mode_changed = true;
                    let new_bit10 = (mode_now >> 10) & 1;
                    if old_bit10 == 1 && new_bit10 == 0 {
                        irq_raised = true;
                    }
                    if irq_once {
                        t.irq_fired_oneshot.set(true);
                    }
                }
            }

            if mode_changed {
                t.mode.set(mode_now);
            }
        }

        if !irq_toggle {
            let m = t.mode.get();
            t.mode.set(m | (1 << 10));
        }

        if irq_raised { Some(irq_bit) } else { None }
    }
}

impl Default for Timers {
    fn default() -> Self {
        Self::new()
    }
}
