use std::collections::VecDeque;

pub const FULL_SCALE: f32 = 32768.0;
pub const SOURCE_HZ: u32 = 44100;
pub const MAX_RATE_DEVIATION: f64 = 0.005;
pub const FADE_SECONDS: f64 = 0.005;
pub const CROSSFADE_FRAMES: usize = 256;
const LEVEL_SMOOTHING_SECONDS: f64 = 0.5;
const HIGH_WATER_FACTOR: usize = 3;
const INTERPOLATION_TAPS: usize = 3;

type Frame = [f32; 2];

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Counters {
    pub underruns: u64,
    pub starved_frames: u64,
    pub overflows: u64,
    pub dropped: u64,
}

/// Anel entre o SPU (produtor, 44,1 kHz emulados) e o dispositivo (consumidor, no relogio
/// dele). Guarda quadros na taxa do SPU; a conversao e Hermite com fase fracionaria no
/// consumo, com a razao corrigida em ate 0,5% para manter a ocupacao no alvo.
#[derive(Debug)]
pub struct Ring {
    frames: VecDeque<(i16, i16)>,
    base_target: usize,
    output_hz: u32,
    dynamic_rate: bool,
    phase: f64,
    history: Frame,
    level: f64,
    ratio: f64,
    gain: f32,
    playing: bool,
    started: bool,
    held: Frame,
    counters: Counters,
    largest_request: usize,
}

impl Ring {
    pub fn new(target_frames: usize) -> Self {
        let target = target_frames.max(INTERPOLATION_TAPS);
        Ring {
            frames: VecDeque::with_capacity(target * HIGH_WATER_FACTOR + 1),
            base_target: target,
            output_hz: SOURCE_HZ,
            dynamic_rate: true,
            phase: 0.0,
            history: [0.0; 2],
            level: target as f64,
            ratio: 1.0,
            gain: 0.0,
            playing: false,
            started: false,
            held: [0.0; 2],
            counters: Counters::default(),
            largest_request: 0,
        }
    }

    pub fn set_output_rate(&mut self, hz: u32) {
        if hz > 0 {
            self.output_hz = hz;
        }
    }

    pub fn output_rate(&self) -> u32 {
        self.output_hz
    }

    pub fn set_dynamic_rate(&mut self, ligado: bool) {
        self.dynamic_rate = ligado;
        if !ligado {
            self.ratio = 1.0;
        }
    }

    /// Razao aplicada no ultimo preenchimento: acima de 1 consome o anel mais depressa.
    pub fn rate_ratio(&self) -> f64 {
        self.ratio
    }

    pub fn largest_request(&self) -> usize {
        self.largest_request
    }

    /// Ocupacao alvo medida no inicio de cada callback: a folga pedida mais o maior pedido
    /// do dispositivo (o PipeWire chega a pedir 100 ms de uma vez).
    pub fn target(&self) -> usize {
        let pedido = self.largest_request as f64 * f64::from(SOURCE_HZ) / f64::from(self.output_hz);
        self.base_target + pedido.ceil() as usize
    }

    pub fn high_water(&self) -> usize {
        self.target() + self.base_target * (HIGH_WATER_FACTOR - 1)
    }

    pub fn frames_available(&self) -> usize {
        self.frames.len()
    }

    pub fn counters(&self) -> Counters {
        self.counters
    }

    pub fn underruns(&self) -> u64 {
        self.counters.underruns
    }

    pub fn dropped(&self) -> u64 {
        self.counters.dropped
    }

    pub fn push_frames(&mut self, frames: &[(i16, i16)]) {
        self.frames.extend(frames.iter().copied());
        if self.frames.len() > self.high_water() {
            let excesso = self.frames.len() - self.target();
            self.trim_with_crossfade(excesso);
        }
    }

    /// Descarta os `d` quadros mais antigos. Os primeiros quadros que ficam sao misturados
    /// com os que sairiam, para a emenda nao virar degrau. E o caminho do fast-forward: o
    /// som sai em trechos na altura original, sem acelerar o tom nem estalar.
    fn trim_with_crossfade(&mut self, d: usize) {
        let largura = CROSSFADE_FRAMES.min(self.frames.len() - d);
        for i in 0..largura {
            let t = (i + 1) as f32 / (largura + 1) as f32;
            let (al, ar) = self.frames[i];
            let (bl, br) = self.frames[d + i];
            self.frames[d + i] = (mix(al, bl, t), mix(ar, br, t));
        }
        self.frames.drain(..d);
        self.counters.overflows += 1;
        self.counters.dropped += d as u64;
    }

    /// Preenche o buffer do dispositivo em L,R intercalados.
    pub fn fill_interleaved(&mut self, out: &mut [f32]) {
        self.largest_request = self.largest_request.max(out.len().div_ceil(2));
        self.update_ratio(out.len().div_ceil(2));
        let passo = f64::from(SOURCE_HZ) / f64::from(self.output_hz) * self.ratio;
        let rampa = (1.0 / (FADE_SECONDS * f64::from(self.output_hz)).max(1.0)) as f32;
        for par in out.chunks_mut(2) {
            let quadro = self.next_frame(passo, rampa);
            for (amostra, valor) in par.iter_mut().zip(quadro) {
                *amostra = valor;
            }
        }
    }

    fn update_ratio(&mut self, quadros_de_saida: usize) {
        let alfa = (quadros_de_saida as f64
            / (LEVEL_SMOOTHING_SECONDS * f64::from(self.output_hz)))
        .min(1.0);
        self.level += alfa * (self.frames.len() as f64 - self.level);
        if !self.dynamic_rate {
            return;
        }
        let meia_faixa = self.base_target as f64 / 2.0;
        let erro = ((self.level - self.target() as f64) / meia_faixa).clamp(-1.0, 1.0);
        self.ratio = 1.0 + erro * MAX_RATE_DEVIATION;
    }

    fn next_frame(&mut self, passo: f64, rampa: f32) -> Frame {
        if !self.playing && self.gain <= 0.0 && self.frames.len() >= self.target() {
            self.resume();
        }
        if self.playing && self.frames.len() < INTERPOLATION_TAPS {
            self.playing = false;
            self.counters.underruns += 1;
        }
        if !self.playing {
            if self.started {
                self.counters.starved_frames += 1;
            }
            self.gain = (self.gain - rampa).max(0.0);
            return self.held.map(|v| v * self.gain);
        }
        let bruto = self.interpolate();
        self.advance(passo);
        self.gain = (self.gain + rampa).min(1.0);
        self.held = bruto;
        bruto.map(|v| v * self.gain)
    }

    fn resume(&mut self) {
        self.playing = true;
        self.started = true;
        self.phase = 0.0;
        self.history = self.frames.front().map_or([0.0; 2], to_frame);
    }

    fn interpolate(&self) -> Frame {
        let x0 = self.frames.front().map_or([0.0; 2], to_frame);
        let x1 = self.frames.get(1).map_or(x0, to_frame);
        let x2 = self.frames.get(2).map_or(x1, to_frame);
        let t = self.phase as f32;
        let mut y = [0.0; 2];
        for (c, saida) in y.iter_mut().enumerate() {
            *saida = hermite(self.history[c], x0[c], x1[c], x2[c], t);
        }
        y
    }

    fn advance(&mut self, passo: f64) {
        self.phase += passo;
        while self.phase >= 1.0 {
            let Some(saiu) = self.frames.pop_front() else {
                self.phase = 0.0;
                return;
            };
            self.history = to_frame(&saiu);
            self.phase -= 1.0;
        }
    }
}

fn to_frame(f: &(i16, i16)) -> Frame {
    [f32::from(f.0) / FULL_SCALE, f32::from(f.1) / FULL_SCALE]
}

fn mix(a: i16, b: i16, t: f32) -> i16 {
    (f32::from(a) * (1.0 - t) + f32::from(b) * t).round() as i16
}

/// Catmull-Rom entre `x0` e `x1`, com `xm1` e `x2` como vizinhos.
pub fn hermite(xm1: f32, x0: f32, x1: f32, x2: f32, t: f32) -> f32 {
    let c1 = 0.5 * (x1 - xm1);
    let c2 = xm1 - 2.5 * x0 + 2.0 * x1 - 0.5 * x2;
    let c3 = 0.5 * (x2 - xm1) + 1.5 * (x0 - x1);
    ((c3 * t + c2) * t + c1) * t + x0
}
