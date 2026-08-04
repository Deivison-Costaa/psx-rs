/// Taxa de envoltoria: passo (0..3), shift (0..1Fh), modo e direcao.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Rate {
    pub step: u8,
    pub shift: u8,
    pub exponential: bool,
    pub decreasing: bool,
    pub phase_negative: bool,
}

impl Rate {
    fn todos_os_bits(&self) -> bool {
        (u32::from(self.step) | (u32::from(self.shift) << 2)) == 0x7F
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Envelope {
    pub level: i32,
    pub counter: i32,
}

impl Envelope {
    /// § Envelope Operation depending on Shift/Step/Mode/Direction (L507) de
    /// docs/reference/08-spu.md, transcrito passo a passo.
    pub fn tick(&mut self, rate: Rate) {
        let mut step = 7 - i32::from(rate.step);
        if rate.decreasing != rate.phase_negative {
            step = !step;
        }
        step <<= 11u32.saturating_sub(u32::from(rate.shift));
        let mut incremento = 0x8000i32 >> u32::from(rate.shift).saturating_sub(11);

        if rate.exponential && !rate.decreasing && self.level > 0x6000 {
            if rate.shift < 10 {
                step >>= 2;
            } else if rate.shift >= 11 {
                incremento >>= 2;
            } else {
                step >>= 1;
                incremento >>= 1;
            }
        } else if rate.exponential && rate.decreasing {
            step = ((i64::from(step) * i64::from(self.level)) >> 15) as i32;
        }

        if !rate.todos_os_bits() {
            incremento = incremento.max(1);
        }

        self.counter += incremento;
        if self.counter & 0x8000 == 0 {
            return;
        }
        self.counter &= 0x7FFF;
        self.level += step;
        self.level = if !rate.decreasing {
            self.level.clamp(-0x8000, 0x7FFF)
        } else if rate.phase_negative {
            self.level.clamp(-0x8000, 0)
        } else {
            self.level.max(0)
        };
    }
}
