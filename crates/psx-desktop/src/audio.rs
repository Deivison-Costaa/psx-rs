use std::sync::{Arc, Mutex, MutexGuard};
use std::time::{Duration, Instant};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use psx_core::audio::{Counters, Ring, SOURCE_HZ};

/// Latência-alvo do anel. Precisa cobrir um quadro de vídeo atrasado (o produtor empurra
/// em rajadas de ~17 ms, até 50 ms); o anel soma a isso o maior callback do dispositivo.
const TARGET_SECONDS: f64 = 0.07;
/// Pedido ao dispositivo; o PipeWire, deixado no padrão, entrega callbacks de 100 ms.
const DEVICE_BUFFER_FRAMES: u32 = 1024;
const LOG_INTERVAL: Duration = Duration::from_secs(5);
const LOG_ENV: &str = "PSX_AUDIO_LOG";

pub struct AudioOut {
    ring: Arc<Mutex<Ring>>,
    _stream: Option<cpal::Stream>,
    device_hz: u32,
    escalados: Vec<(i16, i16)>,
    proximo_log: Option<Instant>,
    produzidos: usize,
}

fn trava(ring: &Mutex<Ring>) -> MutexGuard<'_, Ring> {
    match ring.lock() {
        Ok(a) => a,
        Err(envenenado) => envenenado.into_inner(),
    }
}

impl AudioOut {
    /// Abre o dispositivo padrão. Falta de placa de som não derruba o emulador: o anel
    /// continua existindo e o vídeo roda igual.
    pub fn new() -> Self {
        let alvo = (TARGET_SECONDS * f64::from(SOURCE_HZ)) as usize;
        let ring = Arc::new(Mutex::new(Ring::new(alvo)));
        let proximo_log = std::env::var_os(LOG_ENV).map(|_| Instant::now());
        let (stream, device_hz) = match Self::abrir(&ring) {
            Ok((stream, hz)) => (Some(stream), hz),
            Err(e) => {
                eprintln!("áudio desligado: {e}");
                (None, 0)
            }
        };
        AudioOut {
            ring,
            _stream: stream,
            device_hz,
            escalados: Vec::new(),
            proximo_log,
            produzidos: 0,
        }
    }

    fn abrir(ring: &Arc<Mutex<Ring>>) -> Result<(cpal::Stream, u32), String> {
        let host = cpal::default_host();
        let device = host
            .default_output_device()
            .ok_or_else(|| "sem dispositivo de saída".to_string())?;
        let config = device
            .default_output_config()
            .map_err(|e| format!("configuração padrão: {e}"))?;
        let hz = config.sample_rate().0;
        let canais = usize::from(config.channels()).max(1);
        trava(ring).set_output_rate(hz);
        let padrao = config.config();
        let stream = match config.buffer_size() {
            cpal::SupportedBufferSize::Range { min, max } => {
                let fixo = cpal::StreamConfig {
                    buffer_size: cpal::BufferSize::Fixed(DEVICE_BUFFER_FRAMES.clamp(*min, *max)),
                    ..padrao.clone()
                };
                Self::cria_stream(&device, &fixo, ring, canais)
                    .or_else(|_| Self::cria_stream(&device, &padrao, ring, canais))
            }
            cpal::SupportedBufferSize::Unknown => Self::cria_stream(&device, &padrao, ring, canais),
        }?;
        stream.play().map_err(|e| format!("iniciando: {e}"))?;
        Ok((stream, hz))
    }

    fn cria_stream(
        device: &cpal::Device,
        config: &cpal::StreamConfig,
        ring: &Arc<Mutex<Ring>>,
        canais: usize,
    ) -> Result<cpal::Stream, String> {
        let compartilhado = Arc::clone(ring);
        let mut estereo: Vec<f32> = Vec::new();
        device
            .build_output_stream(
                config,
                move |saida: &mut [f32], _: &cpal::OutputCallbackInfo| {
                    let mut anel = trava(&compartilhado);
                    if canais == 2 {
                        anel.fill_interleaved(saida);
                    } else {
                        Self::preenche_outros_canais(&mut anel, &mut estereo, saida, canais);
                    }
                },
                |e| eprintln!("erro no stream de áudio: {e}"),
                None,
            )
            .map_err(|e| format!("abrindo stream: {e}"))
    }

    /// Mono recebe a média de L e R; com mais de dois canais, L e R vão nos dois primeiros
    /// e o resto fica em silêncio.
    fn preenche_outros_canais(
        anel: &mut Ring,
        estereo: &mut Vec<f32>,
        saida: &mut [f32],
        canais: usize,
    ) {
        let quadros = saida.len() / canais;
        estereo.resize(quadros * 2, 0.0);
        anel.fill_interleaved(estereo);
        for (destino, par) in saida.chunks_mut(canais).zip(estereo.chunks(2)) {
            if canais == 1 {
                destino[0] = (par[0] + par[1]) / 2.0;
                continue;
            }
            for (i, amostra) in destino.iter_mut().enumerate() {
                *amostra = par.get(i).copied().unwrap_or(0.0);
            }
        }
    }

    /// O ganho é aplicado aqui, antes do anel: mexer no volume não pode exigir refazer
    /// o stream do cpal, e o SPU não tem noção de volume de aplicativo.
    pub fn push(&mut self, quadros: &[(i16, i16)], ganho: f32) {
        self.produzidos += quadros.len();
        if !quadros.is_empty() {
            let mut anel = trava(&self.ring);
            if (ganho - 1.0).abs() < f32::EPSILON {
                anel.push_frames(quadros);
            } else {
                self.escalados.clear();
                self.escalados.extend(quadros.iter().map(|(e, d)| {
                    (
                        (f32::from(*e) * ganho) as i16,
                        (f32::from(*d) * ganho) as i16,
                    )
                }));
                anel.push_frames(&self.escalados);
            }
        }
        self.talvez_registra();
    }

    fn talvez_registra(&mut self) {
        let Some(quando) = self.proximo_log else {
            return;
        };
        let agora = Instant::now();
        if agora < quando {
            return;
        }
        self.proximo_log = Some(agora + LOG_INTERVAL);
        let anel = trava(&self.ring);
        let c: Counters = anel.counters();
        let nivel = anel.frames_available();
        let produzidos = std::mem::take(&mut self.produzidos);
        eprintln!(
            "áudio: SPU {:.0} q/s, saída {} Hz, anel {} quadros ({:.1} ms, alvo {:.1} ms), razão {:+.3}%, maior callback {} quadros, underruns {}, quadros sem dados {}, estouros {}, quadros descartados {}",
            produzidos as f64 / LOG_INTERVAL.as_secs_f64(),
            self.device_hz,
            nivel,
            nivel as f64 * 1000.0 / f64::from(SOURCE_HZ),
            anel.target() as f64 * 1000.0 / f64::from(SOURCE_HZ),
            (anel.rate_ratio() - 1.0) * 100.0,
            anel.largest_request(),
            c.underruns,
            c.starved_frames,
            c.overflows,
            c.dropped,
        );
    }

    pub fn device_hz(&self) -> u32 {
        self.device_hz
    }

    pub fn ativo(&self) -> bool {
        self._stream.is_some()
    }
}

impl Default for AudioOut {
    fn default() -> Self {
        Self::new()
    }
}
