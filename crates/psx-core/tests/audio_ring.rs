use psx_core::audio::{FULL_SCALE, Ring, SOURCE_HZ};
use psx_core::bus::{Bios, Bus, Ram};
use psx_core::spu::CPU_CYCLES_PER_SAMPLE;

const ALVO: usize = 3087;

fn seno(freq: f64, amplitude: f64, inicio: usize, n: usize) -> Vec<(i16, i16)> {
    (inicio..inicio + n)
        .map(|i| {
            let v = amplitude
                * f64::from(FULL_SCALE)
                * (2.0 * std::f64::consts::PI * freq * i as f64 / f64::from(SOURCE_HZ)).sin();
            (v.round() as i16, v.round() as i16)
        })
        .collect()
}

fn constante(valor: f32, n: usize) -> Vec<(i16, i16)> {
    let v = (valor * FULL_SCALE) as i16;
    vec![(v, v); n]
}

/// Consome em callbacks de ate 512 quadros, como um dispositivo de verdade.
fn puxa(anel: &mut Ring, quadros: usize) -> Vec<f32> {
    let mut saida = vec![0.0f32; quadros * 2];
    for callback in saida.chunks_mut(1024) {
        anel.fill_interleaved(callback);
    }
    saida.chunks(2).map(|p| p[0]).collect()
}

fn maior_salto(sinal: &[f32]) -> f32 {
    sinal
        .windows(2)
        .map(|w| (w[1] - w[0]).abs())
        .fold(0.0, f32::max)
}

/// Projeta o sinal numa senoide de `freq`: devolve amplitude e o RMS do que sobra.
fn ajusta_senoide(sinal: &[f32], freq: f64, hz: f64) -> (f64, f64) {
    let w = 2.0 * std::f64::consts::PI * freq / hz;
    let n = sinal.len() as f64;
    let (mut i, mut q) = (0.0, 0.0);
    for (k, s) in sinal.iter().enumerate() {
        i += f64::from(*s) * (w * k as f64).cos();
        q += f64::from(*s) * (w * k as f64).sin();
    }
    let (a, b) = (2.0 * i / n, 2.0 * q / n);
    let residuo: f64 = sinal
        .iter()
        .enumerate()
        .map(|(k, s)| {
            let modelo = a * (w * k as f64).cos() + b * (w * k as f64).sin();
            (f64::from(*s) - modelo).powi(2)
        })
        .sum::<f64>()
        / n;
    ((a * a + b * b).sqrt(), residuo.sqrt())
}

#[test]
fn seno_de_1khz_reamostrado_para_48khz_mantem_frequencia_e_amplitude() {
    let mut anel = Ring::new(ALVO);
    anel.set_output_rate(48000);
    anel.set_dynamic_rate(false);
    anel.push_frames(&seno(1000.0, 0.5, 0, ALVO));
    let mut produzidos = ALVO;
    let mut saida = Vec::new();
    for _ in 0..100 {
        anel.push_frames(&seno(1000.0, 0.5, produzidos, 441));
        produzidos += 441;
        saida.extend(puxa(&mut anel, 480));
    }
    let estavel = &saida[4800..];
    let (amplitude, residuo) = ajusta_senoide(estavel, 1000.0, 48000.0);
    assert!(
        (amplitude - 0.5).abs() < 0.0025,
        "amplitude {amplitude}, esperado 0,5"
    );
    assert!(
        residuo < 0.001,
        "RMS fora da senoide de 1 kHz = {residuo}: vizinho mais proximo daria ~0,015"
    );
    assert_eq!(anel.counters().underruns, 0);
    assert_eq!(anel.counters().overflows, 0);
}

#[test]
fn taxa_igual_a_do_spu_entrega_os_quadros_sem_alterar_depois_do_fade_in() {
    let mut anel = Ring::new(ALVO);
    anel.set_dynamic_rate(false);
    let entrada: Vec<(i16, i16)> = (0..ALVO + 600)
        .map(|i| ((i % 1000) as i16, -((i % 1000) as i16)))
        .collect();
    anel.push_frames(&entrada);
    let mut saida = vec![0.0f32; 600 * 2];
    anel.fill_interleaved(&mut saida);
    for k in 300..600 {
        let esperado = (k % 1000) as f32 / FULL_SCALE;
        assert!((saida[2 * k] - esperado).abs() < 1e-6, "quadro {k}");
        assert!(
            (saida[2 * k + 1] + esperado).abs() < 1e-6,
            "quadro {k} direito"
        );
    }
    assert_eq!(anel.output_rate(), 44100);
}

#[test]
fn quadros_de_saida_seguem_a_razao_entre_as_taxas() {
    let mut anel = Ring::new(ALVO);
    anel.set_output_rate(48000);
    anel.set_dynamic_rate(false);
    anel.push_frames(&constante(0.1, ALVO + 4410));
    puxa(&mut anel, 4800);
    assert!(
        anel.frames_available().abs_diff(ALVO) <= 1,
        "4800 quadros a 48 kHz consomem 4410 do SPU; sobraram {}",
        anel.frames_available()
    );
}

struct Simulacao {
    produtor_hz: f64,
    consumidor_hz: f64,
    velocidade: usize,
    callback: usize,
    segundos: f64,
    taxa_dinamica: bool,
}

struct Resultado {
    anel: Ring,
    underruns_depois: u64,
    estouros_depois: u64,
    nivel_min_depois: usize,
    maior_salto: f32,
}

/// Produtor em rajadas de um quadro de video (60 Hz no relogio dele) e consumidor em
/// callbacks de `callback` quadros no relogio do dispositivo, que diz ser de 44,1 kHz.
fn simula(s: Simulacao, convergencia: f64) -> Resultado {
    let mut anel = Ring::new(ALVO);
    anel.set_dynamic_rate(s.taxa_dinamica);
    let rajada = 1.0 / 60.0;
    let callback = s.callback as f64 / s.consumidor_hz;
    let (mut t_prod, mut t_cons, mut acumulado, mut gerados) = (0.0, 0.0, 0.0, 0usize);
    let mut marca = None;
    let mut nivel_min = usize::MAX;
    let mut maior = 0.0f32;
    let mut ultimo = 0.0f32;
    while t_cons < s.segundos {
        if t_prod <= t_cons {
            acumulado += s.produtor_hz * rajada * s.velocidade as f64;
            let n = acumulado as usize;
            acumulado -= n as f64;
            anel.push_frames(&seno(200.0, 0.8, gerados, n));
            gerados += n;
            t_prod += rajada;
            continue;
        }
        let mut bruto = vec![0.0f32; s.callback * 2];
        anel.fill_interleaved(&mut bruto);
        let saida: Vec<f32> = bruto.chunks(2).map(|p| p[0]).collect();
        t_cons += callback;
        if t_cons >= convergencia {
            if marca.is_none() {
                let c = anel.counters();
                marca = Some((c.underruns, c.overflows));
            }
            nivel_min = nivel_min.min(anel.frames_available());
            maior = maior
                .max(maior_salto(&saida))
                .max((saida[0] - ultimo).abs());
        }
        ultimo = saida[s.callback - 1];
    }
    let (u0, o0) = marca.unwrap_or_default();
    let c = anel.counters();
    Resultado {
        underruns_depois: c.underruns - u0,
        estouros_depois: c.overflows - o0,
        nivel_min_depois: nivel_min,
        maior_salto: maior,
        anel,
    }
}

#[test]
fn taxa_dinamica_segura_consumidor_mais_rapido_por_60_segundos() {
    let r = simula(
        Simulacao {
            produtor_hz: 44100.0,
            consumidor_hz: 44150.0,
            velocidade: 1,
            callback: 512,
            segundos: 60.0,
            taxa_dinamica: true,
        },
        20.0,
    );
    assert_eq!(r.anel.counters().underruns, 0, "{:?}", r.anel.counters());
    assert_eq!(r.estouros_depois, 0);
    assert!(
        r.anel.rate_ratio() < 1.0,
        "consumidor rapido pede razao < 1"
    );
    let nivel = r.anel.frames_available() as f64;
    assert!(
        (nivel - ALVO as f64).abs() < ALVO as f64 * 0.5,
        "ocupacao {nivel} longe do alvo {ALVO}"
    );
    assert!(
        r.nivel_min_depois > 600,
        "margem minima {}",
        r.nivel_min_depois
    );
}

#[test]
fn taxa_dinamica_segura_consumidor_mais_lento_sem_descartar() {
    let r = simula(
        Simulacao {
            produtor_hz: 44150.0,
            consumidor_hz: 44100.0,
            velocidade: 1,
            callback: 512,
            segundos: 60.0,
            taxa_dinamica: true,
        },
        20.0,
    );
    assert_eq!(r.anel.counters().overflows, 0, "{:?}", r.anel.counters());
    assert_eq!(r.anel.counters().underruns, 0);
    assert!(r.anel.rate_ratio() > 1.0);
}

#[test]
fn callback_de_100_ms_do_pipewire_eleva_o_alvo_e_nao_falta_quadro() {
    let r = simula(
        Simulacao {
            produtor_hz: 44100.0,
            consumidor_hz: 44150.0,
            velocidade: 1,
            callback: 4410,
            segundos: 60.0,
            taxa_dinamica: true,
        },
        5.0,
    );
    assert_eq!(r.underruns_depois, 0, "{:?}", r.anel.counters());
    assert_eq!(r.estouros_depois, 0);
    assert!(r.anel.target() >= ALVO + 4410, "alvo {}", r.anel.target());
    assert!(r.anel.high_water() > r.anel.target() + ALVO);
}

#[test]
fn sem_taxa_dinamica_o_mesmo_desvio_esvazia_o_anel() {
    let r = simula(
        Simulacao {
            produtor_hz: 44100.0,
            consumidor_hz: 44150.0,
            velocidade: 1,
            callback: 512,
            segundos: 60.0,
            taxa_dinamica: false,
        },
        0.0,
    );
    assert!(
        r.underruns_depois > 0,
        "o teste de cima so mede algo se sem controle houver underrun"
    );
}

#[test]
fn falta_de_quadros_decai_ate_zero_sem_degrau_e_volta_com_fade_in() {
    let mut anel = Ring::new(ALVO);
    anel.set_output_rate(48000);
    anel.push_frames(&constante(0.9, ALVO + 1000));
    let mut saida = puxa(&mut anel, 6000);
    assert_eq!(anel.counters().underruns, 1, "um evento, nao um por quadro");
    assert!(anel.counters().starved_frames > 0);
    assert_eq!(*saida.last().unwrap_or(&1.0), 0.0, "termina em silencio");
    let ate_zero = saida.iter().rposition(|v| *v != 0.0).unwrap_or(0);
    let cheio = saida.iter().rposition(|v| *v > 0.89).unwrap_or(0);
    assert!(
        ate_zero - cheio <= 260,
        "fade-out de ~5 ms, levou {} quadros",
        ate_zero - cheio
    );
    anel.push_frames(&constante(0.9, ALVO + 1000));
    saida.extend(puxa(&mut anel, 1000));
    assert!(
        maior_salto(&saida) < 0.01,
        "degrau de {} na falta ou na volta",
        maior_salto(&saida)
    );
    assert!(
        *saida.last().unwrap_or(&0.0) > 0.89,
        "voltou ao volume cheio"
    );
}

#[test]
fn jogo_parado_silencia_e_nao_repete_lixo() {
    let mut anel = Ring::new(ALVO);
    anel.push_frames(&seno(440.0, 0.7, 0, ALVO + 2000));
    puxa(&mut anel, ALVO + 2000);
    let parado = puxa(&mut anel, 44100);
    assert!(
        parado[300..].iter().all(|v| *v == 0.0),
        "um segundo sem quadro novo e silencio puro"
    );
    assert_eq!(anel.counters().underruns, 1);
}

#[test]
fn anel_acima_do_teto_descarta_em_bloco_com_crossfade() {
    let mut anel = Ring::new(ALVO);
    anel.push_frames(&seno(200.0, 0.8, 0, ALVO));
    let mut saida = puxa(&mut anel, 1000);
    anel.push_frames(&seno(200.0, 0.8, 7777, anel.high_water()));
    assert!(anel.frames_available() <= anel.high_water());
    assert_eq!(anel.counters().overflows, 1);
    assert!(anel.counters().dropped > 0);
    saida.extend(puxa(&mut anel, 4000));
    let natural = 0.8 * 2.0 * std::f32::consts::PI * 200.0 / 44100.0;
    assert!(
        maior_salto(&saida) < natural + 0.01,
        "emenda com degrau de {}",
        maior_salto(&saida)
    );
}

#[test]
fn fast_forward_4x_mantem_audio_continuo_e_latencia_limitada() {
    let r = simula(
        Simulacao {
            produtor_hz: 44100.0,
            consumidor_hz: 44100.0,
            velocidade: 4,
            callback: 512,
            segundos: 10.0,
            taxa_dinamica: true,
        },
        1.0,
    );
    assert_eq!(r.underruns_depois, 0);
    assert!(r.estouros_depois > 0, "o excesso sai em blocos contados");
    assert!(r.anel.frames_available() <= r.anel.high_water());
    let natural = 0.8 * 2.0 * std::f32::consts::PI * 200.0 / 44100.0;
    assert!(
        r.maior_salto < natural + 0.01,
        "degrau de {} no fast-forward",
        r.maior_salto
    );
}

#[test]
fn barramento_produz_um_quadro_a_cada_768_ciclos() {
    assert_eq!(
        CPU_CYCLES_PER_SAMPLE, 768,
        "33.868.800 Hz / 44.100 Hz = 768; 300h ciclos por amostra"
    );
    let mut bus = Bus::new(Ram::new(), Bios::from_bytes(vec![0u8; 512 * 1024]).unwrap());
    // Numero literal de proposito: usar CPU_CYCLES_PER_SAMPLE aqui faria o teste andar
    // junto com o periodo e nao mediria nada.
    for _ in 0..100 {
        bus.tick_timers(768);
    }
    let quadros = bus.drain_audio();
    assert_eq!(
        quadros.len(),
        100,
        "o SPU anda por evento do scheduler, um quadro por 768 ciclos"
    );
    assert!(
        bus.drain_audio().is_empty(),
        "drenar duas vezes nao repete quadro"
    );
}
