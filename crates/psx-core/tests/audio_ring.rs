use psx_core::audio::Ring;
use psx_core::bus::{Bios, Bus, Ram};
use psx_core::spu::CPU_CYCLES_PER_SAMPLE;

#[test]
fn anel_entrega_os_quadros_na_ordem_em_que_entraram() {
    let mut anel = Ring::new(8);
    anel.push_frames(&[(1, -1), (2, -2), (3, -3)]);
    assert_eq!(anel.frames_available(), 3);
    let mut saida = [0.0f32; 4];
    anel.fill_interleaved(&mut saida);
    assert_eq!(saida[0], 1.0 / 32768.0);
    assert_eq!(saida[1], -1.0 / 32768.0);
    assert_eq!(saida[2], 2.0 / 32768.0);
    assert_eq!(saida[3], -2.0 / 32768.0);
    assert_eq!(anel.frames_available(), 1);
}

#[test]
fn anel_cheio_descarta_o_quadro_mais_antigo() {
    let mut anel = Ring::new(3);
    anel.push_frames(&[(1, 1), (2, 2), (3, 3), (4, 4), (5, 5)]);
    assert_eq!(
        anel.frames_available(),
        3,
        "o teto e em quadros, nao em amostras"
    );
    let mut saida = [0.0f32; 6];
    anel.fill_interleaved(&mut saida);
    assert_eq!(saida[0], 3.0 / 32768.0, "1 e 2 sairam pela frente");
    assert_eq!(saida[4], 5.0 / 32768.0);
    assert_eq!(anel.dropped(), 2);
}

#[test]
fn falta_de_quadros_vira_silencio_e_conta_como_underrun() {
    let mut anel = Ring::new(8);
    anel.push_frames(&[(100, 200)]);
    let mut saida = [1.0f32; 6];
    anel.fill_interleaved(&mut saida);
    assert_eq!(saida[0], 100.0 / 32768.0);
    assert_eq!(saida[1], 200.0 / 32768.0);
    assert_eq!(&saida[2..], &[0.0; 4], "o resto e silencio, nao lixo");
    assert_eq!(anel.underruns(), 2, "faltaram dois quadros");
}

#[test]
fn sem_underrun_o_contador_nao_anda() {
    let mut anel = Ring::new(8);
    anel.push_frames(&[(1, 1), (2, 2)]);
    let mut saida = [0.0f32; 4];
    anel.fill_interleaved(&mut saida);
    assert_eq!(anel.underruns(), 0);
    assert_eq!(anel.dropped(), 0);
}

#[test]
fn escala_usa_32768_para_o_minimo_bater_em_menos_um() {
    let mut anel = Ring::new(4);
    anel.push_frames(&[(i16::MIN, i16::MAX)]);
    let mut saida = [0.0f32; 2];
    anel.fill_interleaved(&mut saida);
    assert_eq!(saida[0], -1.0);
    assert!(saida[1] < 1.0 && saida[1] > 0.9999);
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

#[test]
fn anel_converte_a_taxa_do_spu_para_a_taxa_do_dispositivo() {
    let mut anel = Ring::new(4096);
    anel.set_output_rate(48000);
    let entrada: Vec<(i16, i16)> = (0..441).map(|i| (i as i16, i as i16)).collect();
    anel.push_frames(&entrada);
    assert_eq!(
        anel.frames_available(),
        480,
        "441 quadros a 44100 Hz duram 480 quadros a 48000 Hz"
    );
    assert_eq!(anel.dropped(), 0);
}

#[test]
fn taxa_igual_a_do_spu_nao_duplica_nem_perde_quadro() {
    let mut anel = Ring::new(4096);
    anel.set_output_rate(psx_core::audio::SOURCE_HZ);
    let entrada: Vec<(i16, i16)> = (0..1000).map(|i| (i as i16, 0)).collect();
    anel.push_frames(&entrada);
    assert_eq!(anel.frames_available(), 1000);
    assert_eq!(anel.output_rate(), 44100);
}
