use std::cell::Cell;

const TIMER_COUNT: usize = 3;
const IRQ_N: u32 = 1 << 10;
const REACHED_TARGET: u32 = 1 << 11;
const REACHED_FFFF: u32 = 1 << 12;
const STICKY_BITS: u32 = 0x7C00;
const WRITABLE_BITS: u32 = 0x3FF;
const RESET_HOLD_TICKS: u8 = 1;
const TARGET_RESET_TICKS: u8 = 2;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Source {
    System,
    SystemDiv8,
    Dot,
    Hblank,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct Timer {
    counter: Cell<u16>,
    mode: Cell<u32>,
    target: u16,
    cycle_acc: Cell<u32>,
    prev_sync_signal: Cell<bool>,
    mode3_triggered: Cell<bool>,
    irq_fired_oneshot: Cell<bool>,
    at_target: Cell<bool>,
    hold: Cell<u8>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Timers {
    timers: [Timer; TIMER_COUNT],
    gpu_cycles_per_pix: Cell<u16>,
    gpu_video_cycles_per_scanline: Cell<u16>,
    #[serde(skip)]
    pending: Cell<u32>,
    #[serde(skip)]
    budget: Cell<u32>,
    #[serde(skip)]
    pending_sync: Cell<(bool, bool)>,
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
            at_target: Cell::new(false),
            hold: Cell::new(0),
        }
    }

    fn source(&self, idx: usize) -> Source {
        match (idx, (self.mode.get() >> 8) & 0x3) {
            (0, 1 | 3) => Source::Dot,
            (1, 1 | 3) => Source::Hblank,
            (2, 2 | 3) => Source::SystemDiv8,
            _ => Source::System,
        }
    }

    fn hold_after_write(&self, idx: usize) -> u8 {
        if self.source(idx) == Source::System {
            RESET_HOLD_TICKS
        } else {
            0
        }
    }

    fn restart_after_target(&self, source: Source) {
        self.counter.set(0);
        self.at_target.set(false);
        if source == Source::System {
            self.hold.set(TARGET_RESET_TICKS);
        }
    }

    fn signal_irq(&self, mode: &mut u32, enabled: bool) -> bool {
        let once = *mode & (1 << 6) == 0;
        if !enabled || (once && self.irq_fired_oneshot.get()) {
            return false;
        }
        let was_high = *mode & IRQ_N != 0;
        if *mode & (1 << 7) != 0 {
            *mode ^= IRQ_N;
        } else {
            *mode &= !IRQ_N;
        }
        if once {
            self.irq_fired_oneshot.set(true);
        }
        was_high && *mode & IRQ_N == 0
    }

    fn counting(&self, idx: usize, sync_signal: bool) -> bool {
        let mode = self.mode.get();
        if mode & 1 == 0 {
            return true;
        }
        match (idx, (mode >> 1) & 0x3) {
            (0 | 1, 0) => !sync_signal,
            (0 | 1, 1) => true,
            (0 | 1, 2) => sync_signal,
            (0 | 1, _) => self.mode3_triggered.get(),
            (_, sync_mode) => sync_mode == 1 || sync_mode == 2,
        }
    }

    fn apply_sync_edge(&self, idx: usize, sync_signal: bool) {
        let rising = sync_signal && !self.prev_sync_signal.get();
        self.prev_sync_signal.set(sync_signal);
        let mode = self.mode.get();
        if idx > 1 || !rising || mode & 1 == 0 {
            return;
        }
        match (mode >> 1) & 0x3 {
            1 | 2 => self.restart_from_sync(),
            3 if !self.mode3_triggered.get() => {
                self.restart_from_sync();
                self.mode3_triggered.set(true);
            }
            _ => {}
        }
    }

    fn restart_from_sync(&self) {
        self.counter.set(0);
        self.cycle_acc.set(0);
        self.at_target.set(false);
    }

    fn consume_hold(&self, ticks: u32) -> u32 {
        let held = u32::from(self.hold.get()).min(ticks);
        self.hold.set(self.hold.get() - held as u8);
        ticks - held
    }

    fn advance(&self, mut ticks: u32, source: Source) -> bool {
        let mut mode = self.mode.get();
        let reset_on_target = mode & (1 << 3) != 0;
        let irq_on_target = mode & (1 << 4) != 0;
        let irq_on_ffff = mode & (1 << 5) != 0;
        let target = u32::from(self.target);
        let mut irq = false;

        while ticks > 0 {
            let current = u32::from(self.counter.get());
            let to_wrap = 0x1_0000 - current;
            let to_target = if target > current {
                target - current
            } else {
                u32::MAX
            };
            let step = ticks.min(to_wrap).min(to_target);
            ticks -= step;
            let next = (current + step) & 0xFFFF;
            self.counter.set(next as u16);

            let wrapped = step == to_wrap;
            if wrapped {
                mode |= REACHED_FFFF;
                irq |= self.signal_irq(&mut mode, irq_on_ffff);
            }
            if next == target && (wrapped || step == to_target) {
                mode |= REACHED_TARGET;
                irq |= self.signal_irq(&mut mode, irq_on_target);
                if reset_on_target {
                    if ticks == 0 {
                        self.at_target.set(true);
                    } else {
                        self.restart_after_target(source);
                        ticks = self.consume_hold(ticks);
                    }
                }
            }
        }
        self.mode.set(mode);
        irq
    }
}

impl Timers {
    pub fn new() -> Self {
        Timers {
            timers: [Timer::new(), Timer::new(), Timer::new()],
            gpu_cycles_per_pix: Cell::new(10),
            gpu_video_cycles_per_scanline: Cell::new(3413),
            pending: Cell::new(0),
            budget: Cell::new(0),
            pending_sync: Cell::new((false, false)),
        }
    }

    fn timer_index(base_addr: u32) -> usize {
        ((base_addr.wrapping_sub(0x1F80_1100) / 0x10) & 0x3) as usize
    }

    pub fn read32(&self, offset: u32) -> u32 {
        self.flush();
        let t = &self.timers[Self::timer_index(offset & !0xF)];
        match offset & 0xF {
            0x0 => t.counter.get() as u32,
            0x4 => {
                let val = t.mode.get();
                t.mode.set(val & !(REACHED_TARGET | REACHED_FFFF));
                val
            }
            0x8 => t.target as u32,
            _ => 0,
        }
    }

    pub fn peek32(&self, offset: u32) -> u32 {
        self.flush();
        let t = &self.timers[Self::timer_index(offset & !0xF)];
        match offset & 0xF {
            0x0 => t.counter.get() as u32,
            0x4 => t.mode.get(),
            0x8 => t.target as u32,
            _ => 0,
        }
    }

    pub fn write32(&mut self, offset: u32, val: u32) {
        self.flush();
        self.budget.set(0);
        let idx = Self::timer_index(offset & !0xF);
        let t = &mut self.timers[idx];
        match offset & 0xF {
            0x0 => {
                t.counter.set((val & 0xFFFF) as u16);
                t.at_target.set(false);
                t.hold.set(t.hold_after_write(idx));
            }
            0x4 => {
                let prev = t.mode.get();
                t.mode
                    .set((prev & STICKY_BITS) | (val & WRITABLE_BITS) | IRQ_N);
                t.counter.set(0);
                t.cycle_acc.set(0);
                t.mode3_triggered.set(false);
                t.irq_fired_oneshot.set(false);
                t.at_target.set(false);
                t.hold.set(t.hold_after_write(idx));
            }
            0x8 => t.target = (val & 0xFFFF) as u16,
            _ => {}
        }
    }

    #[inline]
    pub fn update_gpu_timing(&mut self, cycles_per_pix: u16, video_cycles_per_scanline: u16) {
        if self.gpu_cycles_per_pix.get() != cycles_per_pix {
            self.flush();
            self.budget.set(0);
        }
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
        self.tick_with_hblanks(base_addr, cycles, hblank_active, vblank_active, 0)
    }

    pub fn tick_with_hblanks(
        &mut self,
        base_addr: u32,
        cycles: u32,
        hblank_active: bool,
        vblank_active: bool,
        hblank_edges: u32,
    ) -> Option<u32> {
        let idx = Self::timer_index(base_addr);
        self.tick_one(idx, cycles, (hblank_active, vblank_active), hblank_edges)
    }

    #[inline]
    pub fn defer(&self, cycles: u32, hblank_active: bool, vblank_active: bool) -> bool {
        let total = self.pending.get().saturating_add(cycles);
        if total > self.budget.get() || self.pending_sync.get() != (hblank_active, vblank_active) {
            return false;
        }
        self.pending.set(total);
        true
    }

    pub fn flush(&self) {
        let cycles = self.pending.replace(0);
        if cycles == 0 {
            return;
        }
        self.budget.set(self.budget.get().saturating_sub(cycles));
        for idx in 0..TIMER_COUNT {
            let irq = self.tick_one(idx, cycles, self.pending_sync.get(), 0);
            debug_assert!(irq.is_none());
        }
    }

    pub fn plan(&self, hblank_active: bool, vblank_active: bool) {
        let sync = (hblank_active, vblank_active);
        self.pending_sync.set(sync);
        let budget = (0..TIMER_COUNT)
            .map(|idx| self.safe_cycles(idx, sync))
            .min()
            .unwrap_or(0);
        self.budget.set(u32::try_from(budget).unwrap_or(u32::MAX));
    }

    fn safe_cycles(&self, idx: usize, (hb, vb): (bool, bool)) -> u64 {
        let t = &self.timers[idx];
        let sync_signal = match idx {
            0 => hb,
            1 => vb,
            _ => false,
        };
        if t.at_target.get() || sync_signal != t.prev_sync_signal.get() {
            return 0;
        }
        if !t.counting(idx, sync_signal) {
            return u64::MAX;
        }
        let current = u64::from(t.counter.get());
        let target = u64::from(t.target);
        let to_target = if target > current {
            target - current
        } else {
            u64::MAX
        };
        let ticks = (0x1_0000 - current).min(to_target);
        let acc = u64::from(t.cycle_acc.get());
        match t.source(idx) {
            Source::System => u64::from(t.hold.get()) + ticks - 1,
            Source::SystemDiv8 => (8 * ticks - 1).saturating_sub(acc),
            Source::Dot => {
                let denom = 7 * u64::from(self.gpu_cycles_per_pix.get());
                (denom * ticks - 1).saturating_sub(acc) / 11
            }
            Source::Hblank => u64::MAX,
        }
    }

    fn tick_one(&self, idx: usize, cycles: u32, sync: (bool, bool), edges: u32) -> Option<u32> {
        let cycles_per_pix = u64::from(self.gpu_cycles_per_pix.get());
        let t = &self.timers[idx];
        let source = t.source(idx);
        let sync_signal = match idx {
            0 => sync.0,
            1 => sync.1,
            _ => false,
        };
        t.apply_sync_edge(idx, sync_signal);

        if cycles > 0 && t.at_target.get() {
            t.restart_after_target(source);
        }
        let free_cycles = t.consume_hold(cycles);
        if !t.counting(idx, sync_signal) {
            return None;
        }

        let ticks = match source {
            Source::System => free_cycles,
            Source::SystemDiv8 => Self::scaled_ticks(t, cycles, 1, 8),
            Source::Dot => Self::scaled_ticks(t, cycles, 11, 7 * cycles_per_pix),
            Source::Hblank => edges,
        };
        let irq = t.advance(ticks, source);
        if t.mode.get() & (1 << 7) == 0 {
            t.mode.set(t.mode.get() | IRQ_N);
        }
        irq.then_some(4 + idx as u32)
    }

    fn scaled_ticks(t: &Timer, cycles: u32, numer: u64, denom: u64) -> u32 {
        let total = u64::from(t.cycle_acc.get()) + u64::from(cycles) * numer;
        t.cycle_acc.set((total % denom) as u32);
        (total / denom) as u32
    }
}

impl Default for Timers {
    fn default() -> Self {
        Self::new()
    }
}
