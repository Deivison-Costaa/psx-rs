use super::adpcm::{self, BLOCK_BYTES, BLOCK_SAMPLES, Flags};
use super::envelope::{Envelope, Rate};
use super::gauss;

const BLOCO_EM_CONTADOR: u32 = (BLOCK_SAMPLES as u32) << 12;
const PITCH_MAXIMO: u32 = 0x4000;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Phase {
    #[default]
    Off,
    Attack,
    Decay,
    Sustain,
    Release,
}

/// Volume de canal: valor fixo (bit15=0) ou envoltoria de sweep (bit15=1).
/// § 1F801C00h+N*10h - Voice 0..23 Volume Left (L468) de docs/reference/08-spu.md.
#[derive(Debug, Clone, Copy, Default)]
pub struct Volume {
    pub raw: u16,
    pub env: Envelope,
}

impl Volume {
    pub fn write(&mut self, val: u16) {
        self.raw = val;
        if val & 0x8000 == 0 {
            self.env.level = i32::from((val << 1) as i16);
        }
    }

    pub fn tick(&mut self) {
        if self.raw & 0x8000 == 0 {
            return;
        }
        self.env.tick(Rate {
            step: (self.raw & 0x3) as u8,
            shift: ((self.raw >> 2) & 0x1F) as u8,
            exponential: self.raw & (1 << 14) != 0,
            decreasing: self.raw & (1 << 13) != 0,
            phase_negative: self.raw & (1 << 12) != 0,
        });
    }

    pub fn level(&self) -> i32 {
        self.env.level
    }
}

#[derive(Debug, Clone, Default)]
pub struct Voice {
    pub volume_left: Volume,
    pub volume_right: Volume,
    pub pitch: u16,
    pub start_address: u16,
    pub repeat_address: u16,
    pub adsr: u32,
    pub adsr_env: Envelope,
    pub phase: Phase,
    pub current_address: u32,
    pub counter: u32,
    pub block: [i16; BLOCK_SAMPLES],
    pub flags: Flags,
    pub previous: [i16; 3],
    pub prev1: i32,
    pub prev2: i32,
    pub out: i16,
}

/// O que o passo de uma voz devolve ao mixer alem da amostra.
#[derive(Debug, Clone, Copy, Default)]
pub struct StepResult {
    pub out: i16,
    pub reached_end: bool,
    pub irq: bool,
}

impl Voice {
    pub fn sample_index(&self) -> usize {
        (self.counter >> 12) as usize
    }

    pub fn interpolation_index(&self) -> usize {
        ((self.counter >> 4) & 0xFF) as usize
    }

    pub fn key_on(&mut self, ram: &[u8], irq_address: u32) -> bool {
        self.current_address = (u32::from(self.start_address) * 8) & ram_mask(ram);
        self.repeat_address = self.start_address;
        self.counter = 0;
        self.prev1 = 0;
        self.prev2 = 0;
        self.previous = [0; 3];
        self.adsr_env = Envelope::default();
        self.phase = Phase::Attack;
        self.load_block(ram, irq_address)
    }

    pub fn key_off(&mut self) {
        if self.phase != Phase::Off {
            self.phase = Phase::Release;
            self.adsr_env.counter = 0;
        }
    }

    fn load_block(&mut self, ram: &[u8], irq_address: u32) -> bool {
        let base = (self.current_address & ram_mask(ram)) as usize;
        let fim = (base + BLOCK_BYTES).min(ram.len());
        let bruto = &ram[base..fim];
        self.flags = Flags::from_bits(bruto.get(1).copied().unwrap_or(0));
        if self.flags.loop_start {
            self.repeat_address = (self.current_address / 8) as u16;
        }
        let (bloco, p1, p2) = adpcm::decode_block(bruto, self.prev1, self.prev2);
        self.block = bloco;
        self.prev1 = p1;
        self.prev2 = p2;
        irq_address == self.current_address
    }

    fn sample_at(&self, index: i32) -> i16 {
        if index >= 0 {
            self.block[index as usize]
        } else {
            self.previous[(3 + index) as usize]
        }
    }

    fn adsr_rate(&self) -> Rate {
        let lo = self.adsr as u16;
        let hi = (self.adsr >> 16) as u16;
        match self.phase {
            Phase::Attack => Rate {
                step: ((lo >> 8) & 0x3) as u8,
                shift: ((lo >> 10) & 0x1F) as u8,
                exponential: lo & 0x8000 != 0,
                decreasing: false,
                phase_negative: false,
            },
            Phase::Decay => Rate {
                step: 0,
                shift: ((lo >> 4) & 0x0F) as u8,
                exponential: true,
                decreasing: true,
                phase_negative: false,
            },
            Phase::Sustain => Rate {
                step: ((hi >> 6) & 0x3) as u8,
                shift: ((hi >> 8) & 0x1F) as u8,
                exponential: hi & 0x8000 != 0,
                decreasing: hi & 0x4000 != 0,
                phase_negative: false,
            },
            Phase::Release => Rate {
                step: 0,
                shift: (hi & 0x1F) as u8,
                exponential: hi & (1 << 5) != 0,
                decreasing: true,
                phase_negative: false,
            },
            Phase::Off => Rate::default(),
        }
    }

    fn sustain_level(&self) -> i32 {
        ((self.adsr as i32 & 0x0F) + 1) * 0x800
    }

    fn tick_adsr(&mut self) {
        if self.phase == Phase::Off {
            return;
        }
        self.adsr_env.tick(self.adsr_rate());
        // O encadeamento e um laco porque uma fase pode terminar sem gastar um ciclo:
        // com nivel de sustain 0Fh (8000h) o decay acaba no instante em que comeca.
        loop {
            let antes = self.phase;
            match self.phase {
                Phase::Attack if self.adsr_env.level >= 0x7FFF => {
                    self.phase = Phase::Decay;
                    self.adsr_env.counter = 0;
                }
                Phase::Decay if self.adsr_env.level <= self.sustain_level() => {
                    self.phase = Phase::Sustain;
                    self.adsr_env.counter = 0;
                }
                Phase::Release if self.adsr_env.level <= 0 => self.phase = Phase::Off,
                _ => {}
            }
            if self.phase == antes {
                return;
            }
        }
    }

    fn pitch_step(&self, modulator: Option<i16>) -> u32 {
        let mut step = u32::from(self.pitch);
        if let Some(anterior) = modulator {
            let fator = i32::from(anterior) + 0x8000;
            let assinado = i32::from(self.pitch as i16);
            step = (((assinado * fator) >> 15) as u32) & 0xFFFF;
        }
        if step > 0x3FFF { PITCH_MAXIMO } else { step }
    }

    /// Um ciclo de 44,1 kHz desta voz: amostra interpolada, envoltoria, avanco do
    /// contador de pitch e troca de bloco quando as 28 amostras acabam.
    pub fn step(
        &mut self,
        ram: &[u8],
        ruido: Option<i16>,
        modulator: Option<i16>,
        irq_address: u32,
    ) -> StepResult {
        if self.phase == Phase::Off && self.adsr_env.level == 0 {
            self.out = 0;
            return StepResult::default();
        }

        let bruta = match ruido {
            Some(nivel) => i32::from(nivel),
            None => {
                let i = self.sample_index() as i32;
                gauss::interpolate(
                    self.interpolation_index(),
                    self.sample_at(i - 3),
                    self.sample_at(i - 2),
                    self.sample_at(i - 1),
                    self.sample_at(i),
                )
            }
        };
        let amostra = bruta.clamp(-0x8000, 0x7FFF);
        self.out = ((amostra * self.adsr_env.level) >> 15).clamp(-0x8000, 0x7FFF) as i16;
        let saida = self.out;

        self.tick_adsr();
        self.volume_left.tick();
        self.volume_right.tick();

        self.counter += self.pitch_step(modulator);
        let mut resultado = StepResult {
            out: saida,
            ..StepResult::default()
        };
        while self.counter >= BLOCO_EM_CONTADOR {
            self.counter -= BLOCO_EM_CONTADOR;
            let (irq, fim) = self.advance_block(ram, irq_address);
            resultado.irq |= irq;
            resultado.reached_end |= fim;
        }
        resultado
    }

    fn advance_block(&mut self, ram: &[u8], irq_address: u32) -> (bool, bool) {
        let terminou = self.flags;
        self.previous = [
            self.block[BLOCK_SAMPLES - 3],
            self.block[BLOCK_SAMPLES - 2],
            self.block[BLOCK_SAMPLES - 1],
        ];
        if terminou.loop_end {
            self.current_address = (u32::from(self.repeat_address) * 8) & ram_mask(ram);
            if !terminou.loop_repeat {
                self.phase = Phase::Release;
                self.adsr_env.level = 0;
                self.adsr_env.counter = 0;
            }
        } else {
            self.current_address =
                self.current_address.wrapping_add(BLOCK_BYTES as u32) & ram_mask(ram);
        }
        (self.load_block(ram, irq_address), terminou.loop_end)
    }
}

fn ram_mask(ram: &[u8]) -> u32 {
    (ram.len() as u32).wrapping_sub(1)
}
