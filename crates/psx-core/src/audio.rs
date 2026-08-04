use std::collections::VecDeque;

pub const FULL_SCALE: f32 = 32768.0;
pub const SOURCE_HZ: u32 = 44100;

/// Anel entre o SPU (produtor, 44,1 kHz emulados) e a saida de audio do frontend
/// (consumidor, callback de tamanho variavel). Puro: quem fala com o dispositivo e o
/// psx-desktop.
#[derive(Debug)]
pub struct Ring {
    frames: VecDeque<(i16, i16)>,
    capacity: usize,
    underruns: u64,
    dropped: u64,
    output_hz: u32,
    fase: u32,
}

impl Ring {
    pub fn new(capacity_frames: usize) -> Self {
        Ring {
            frames: VecDeque::with_capacity(capacity_frames),
            capacity: capacity_frames.max(1),
            underruns: 0,
            dropped: 0,
            output_hz: SOURCE_HZ,
            fase: 0,
        }
    }

    /// Taxa do dispositivo. O SPU sempre produz a 44,1 kHz; a conversao e por
    /// acumulador de fase (vizinho mais proximo), que acerta a taxa no longo prazo.
    pub fn set_output_rate(&mut self, hz: u32) {
        if hz > 0 && hz != self.output_hz {
            self.output_hz = hz;
            self.fase = 0;
        }
    }

    pub fn output_rate(&self) -> u32 {
        self.output_hz
    }

    pub fn push_frames(&mut self, frames: &[(i16, i16)]) {
        for f in frames {
            self.fase += self.output_hz;
            while self.fase >= SOURCE_HZ {
                self.fase -= SOURCE_HZ;
                if self.frames.len() == self.capacity {
                    self.frames.pop_front();
                    self.dropped += 1;
                }
                self.frames.push_back(*f);
            }
        }
    }

    /// Preenche o buffer do dispositivo em L,R intercalados. O que faltar vira silencio.
    pub fn fill_interleaved(&mut self, out: &mut [f32]) {
        for par in out.chunks_mut(2) {
            match self.frames.pop_front() {
                Some((l, r)) => {
                    par[0] = f32::from(l) / FULL_SCALE;
                    if par.len() > 1 {
                        par[1] = f32::from(r) / FULL_SCALE;
                    }
                }
                None => {
                    self.underruns += 1;
                    for amostra in par.iter_mut() {
                        *amostra = 0.0;
                    }
                }
            }
        }
    }

    pub fn frames_available(&self) -> usize {
        self.frames.len()
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }

    pub fn underruns(&self) -> u64 {
        self.underruns
    }

    pub fn dropped(&self) -> u64 {
        self.dropped
    }
}
