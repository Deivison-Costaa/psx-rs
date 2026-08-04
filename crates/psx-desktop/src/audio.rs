use std::sync::{Arc, Mutex};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use psx_core::audio::Ring;

/// Meio segundo de folga a 44,1 kHz. Acima disso o SPU esta produzindo mais rapido do
/// que o dispositivo consome e o anel descarta o mais antigo.
const RING_FRAMES: usize = 22050;

pub struct AudioOut {
    ring: Arc<Mutex<Ring>>,
    _stream: Option<cpal::Stream>,
    device_hz: u32,
}

impl AudioOut {
    /// Abre o dispositivo padrao. Falta de placa de som nao derruba o emulador: o anel
    /// continua existindo e o video roda igual.
    pub fn new() -> Self {
        let ring = Arc::new(Mutex::new(Ring::new(RING_FRAMES)));
        match Self::abrir(&ring) {
            Ok((stream, hz)) => AudioOut {
                ring,
                _stream: Some(stream),
                device_hz: hz,
            },
            Err(e) => {
                eprintln!("audio desligado: {e}");
                AudioOut {
                    ring,
                    _stream: None,
                    device_hz: 0,
                }
            }
        }
    }

    fn abrir(ring: &Arc<Mutex<Ring>>) -> Result<(cpal::Stream, u32), String> {
        let host = cpal::default_host();
        let device = host
            .default_output_device()
            .ok_or_else(|| "sem dispositivo de saida".to_string())?;
        let config = device
            .default_output_config()
            .map_err(|e| format!("configuracao padrao: {e}"))?;
        let hz = config.sample_rate().0;
        let canais = config.channels() as usize;
        if let Ok(mut anel) = ring.lock() {
            anel.set_output_rate(hz);
        }
        let compartilhado = Arc::clone(ring);
        let stream = device
            .build_output_stream(
                &config.config(),
                move |saida: &mut [f32], _: &cpal::OutputCallbackInfo| {
                    let mut anel = match compartilhado.lock() {
                        Ok(a) => a,
                        Err(envenenado) => envenenado.into_inner(),
                    };
                    if canais == 2 {
                        anel.fill_interleaved(saida);
                    } else {
                        Self::preenche_nao_estereo(&mut anel, saida, canais);
                    }
                },
                |e| eprintln!("erro no stream de audio: {e}"),
                None,
            )
            .map_err(|e| format!("abrindo stream: {e}"))?;
        stream.play().map_err(|e| format!("iniciando: {e}"))?;
        Ok((stream, hz))
    }

    fn preenche_nao_estereo(anel: &mut Ring, saida: &mut [f32], canais: usize) {
        let mut par = [0.0f32; 2];
        for quadro in saida.chunks_mut(canais.max(1)) {
            anel.fill_interleaved(&mut par);
            let media = (par[0] + par[1]) / 2.0;
            for amostra in quadro.iter_mut() {
                *amostra = media;
            }
        }
    }

    /// O ganho e aplicado aqui, antes do anel: mexer no volume nao pode exigir refazer
    /// o stream do cpal, e o SPU nao tem nocao de volume de aplicativo.
    pub fn push(&self, quadros: &[(i16, i16)], ganho: f32) {
        if quadros.is_empty() {
            return;
        }
        let mut anel = match self.ring.lock() {
            Ok(a) => a,
            Err(envenenado) => envenenado.into_inner(),
        };
        if (ganho - 1.0).abs() < f32::EPSILON {
            anel.push_frames(quadros);
            return;
        }
        let escalados: Vec<(i16, i16)> = quadros
            .iter()
            .map(|(e, d)| {
                (
                    (f32::from(*e) * ganho) as i16,
                    (f32::from(*d) * ganho) as i16,
                )
            })
            .collect();
        anel.push_frames(&escalados);
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
