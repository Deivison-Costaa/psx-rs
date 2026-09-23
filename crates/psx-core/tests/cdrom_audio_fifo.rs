mod support;

use psx_core::bus::{Bus, BusRead};
use psx_core::cdrom_bin_cue::{DiscLayout, TrackInfo, TrackType};
use psx_core::cdrom_xa::{CDDA_FRAMES, RAW_SECTOR_BYTES};
use psx_core::spu::CPU_CYCLES_PER_SAMPLE;
use support::asm;

const CD_BASE: u32 = 0x1F80_1800;
const ESPERA_PRIMEIRA_RESPOSTA: u32 = 0x1_4000;
// § INT1 Rate (06-cdrom.md L2093-2101): SystemClock*930h/4/44100Hz em velocidade normal.
const SETOR_1X: u32 = (CPU_CYCLES_PER_SAMPLE as u32) * 0x930 / 4;
const SETOR_2X: u32 = SETOR_1X / 2;
// § 15-cdrom-format.md: 18 grupos x 8 unidades x 28 amostras = 4032 amostras mono por
// setor; a 18900 Hz cada uma entra duas vezes no anel de 37800 Hz, que sai 7/6 a 44100 Hz.
const QUADROS_XA_MONO_18K9: u64 = 4032 * 2 * 7 / 6;
const QUADROS_XA_MONO_37K8: u64 = 4032 * 7 / 6;

const MODE_XA: u8 = 0x40;
const MODE_DOBRO: u8 = 0x80;
const MODE_REPORT: u8 = 0x04;

fn cd_write(bus: &mut Bus, offset: u32, val: u8) {
    bus.write8::<BusRead>(CD_BASE + offset, val);
}

fn set_bank(bus: &mut Bus, b: u8) {
    cd_write(bus, 0, b);
}

fn hintsts(bus: &mut Bus) -> u8 {
    set_bank(bus, 1);
    let val = bus.read8::<BusRead>(CD_BASE + 3);
    set_bank(bus, 0);
    val & 0x7
}

fn so_ack(bus: &mut Bus) {
    set_bank(bus, 1);
    cd_write(bus, 3, 0x07);
    cd_write(bus, 3, 0x40);
    set_bank(bus, 0);
}

fn ack(bus: &mut Bus) {
    so_ack(bus);
    let ciclos = bus.cdrom().second_response_cycles() as u32;
    bus.tick_timers(ciclos);
}

fn send(bus: &mut Bus, cmd: u8, params: &[u8]) {
    set_bank(bus, 0);
    for p in params {
        cd_write(bus, 2, *p);
    }
    cd_write(bus, 1, cmd);
    bus.tick_timers(ESPERA_PRIMEIRA_RESPOSTA);
}

fn trilha(number: u8, lba: u32, tipo: TrackType) -> TrackInfo {
    TrackInfo {
        number,
        file: format!("t{number}.bin"),
        start_lba: lba,
        track_type: tipo,
        index01_mm: 0,
        index01_ss: 0,
        index01_ff: 0,
        index00_mm: None,
        index00_ss: None,
        index00_ff: None,
        pregap_mm: None,
        pregap_ss: None,
        pregap_ff: None,
    }
}

fn setor_xa(coding: u8) -> Vec<u8> {
    let mut s = vec![0u8; RAW_SECTOR_BYTES];
    s[0x0F] = 0x02;
    s[0x12] = 0x64;
    s[0x13] = coding;
    for grupo in 0..18 {
        let base = 24 + grupo * 128;
        for blk in 0..4 {
            s[base + 4 + blk * 2] = 0x08;
            s[base + 5 + blk * 2] = 0x08;
        }
        for j in 16..128 {
            s[base + j] = 0x11;
        }
    }
    s
}

fn bus_com(bin: Vec<u8>, tipo: TrackType, mode: u8) -> Bus {
    let mut bus = asm::bus_with_bios_empty();
    let layout = DiscLayout {
        bin_path: "t1.bin".to_string(),
        tracks: vec![trilha(1, 0, tipo)],
    };
    bus.inject_disc(layout, bin);
    bus.cdrom_mut().insert_disc();
    send(&mut bus, 0x0E, &[mode]);
    ack(&mut bus);
    send(&mut bus, 0x02, &[0x00, 0x02, 0x00]);
    ack(&mut bus);
    bus
}

fn disco_xa(setores: usize, xa: &[usize], coding: u8) -> Vec<u8> {
    let mut bin = vec![0u8; RAW_SECTOR_BYTES * setores];
    let s = setor_xa(coding);
    for &i in xa {
        bin[i * RAW_SECTOR_BYTES..(i + 1) * RAW_SECTOR_BYTES].copy_from_slice(&s);
    }
    bin
}

fn le_setores(bus: &mut Bus, n: usize) {
    send(bus, 0x06, &[]);
    ack(bus);
    for _ in 0..n {
        so_ack(bus);
        bus.tick_timers(SETOR_2X);
    }
}

// § 15-cdrom-format.md (decode_sector + zigzag 37800->44100): um setor mono de 18900 Hz
// vira 9408 quadros de 44100 Hz — o buffer tem de caber o setor inteiro.
#[test]
fn setor_xa_mono_18k9_chega_inteiro_ao_buffer() {
    let bin = disco_xa(8, &[0], 0x04);
    let mut bus = bus_com(bin, TrackType::Mode2_2352, MODE_XA | MODE_DOBRO);
    le_setores(&mut bus, 2);
    let st = bus.cdrom().audio_stats();
    assert_eq!(
        st.dropped, 0,
        "nenhum quadro decodificado pode ser descartado"
    );
    assert_eq!(st.enqueued, QUADROS_XA_MONO_18K9);
}

// Mesmo intercalado a 1/16 (mono 37800 Hz a 2x), cada setor entrega 4704 quadros e o SPU
// consome 16 x 294 = 4704 no intervalo: nada se perde e o buffer nao cresce.
#[test]
fn stream_xa_mono_37k8_intercalado_nao_perde_nem_acumula() {
    let xa: Vec<usize> = (0..6).map(|k| k * 16).collect();
    let bin = disco_xa(6 * 16 + 4, &xa, 0x00);
    let mut bus = bus_com(bin, TrackType::Mode2_2352, MODE_XA | MODE_DOBRO);
    le_setores(&mut bus, 6 * 16);
    let st = bus.cdrom().audio_stats();
    assert_eq!(st.dropped, 0);
    assert_eq!(st.enqueued, 6 * QUADROS_XA_MONO_37K8);
    assert!(
        (bus.cdrom().audio_pending() as u64) <= QUADROS_XA_MONO_37K8 + CDDA_FRAMES as u64,
        "o nivel do buffer fica no maximo um setor, achei {}",
        bus.cdrom().audio_pending()
    );
}

// Setores XA colados (mais rapido que o SPU consome) nao podem crescer o buffer sem limite.
#[test]
fn buffer_de_audio_e_limitado_quando_o_disco_entrega_mais_rapido() {
    let xa: Vec<usize> = (0..12).collect();
    let bin = disco_xa(16, &xa, 0x04);
    let mut bus = bus_com(bin, TrackType::Mode2_2352, MODE_XA | MODE_DOBRO);
    le_setores(&mut bus, 12);
    let st = bus.cdrom().audio_stats();
    assert!(st.dropped > 0, "o excedente tem de ser descartado");
    assert!(
        (bus.cdrom().audio_pending() as u64) <= 3 * QUADROS_XA_MONO_18K9,
        "buffer cresceu demais: {}",
        bus.cdrom().audio_pending()
    );
}

fn disco_cdda(setores: usize) -> Vec<u8> {
    vec![0x11u8; RAW_SECTOR_BYTES * setores]
}

// § Play - Command 03h (06-cdrom.md): o CD-DA toca a 75 setores/s, um setor (588 quadros)
// por intervalo, com ou sem Report.
#[test]
fn play_sem_report_toca_um_setor_de_cdda_por_intervalo() {
    let mut bus = bus_com(disco_cdda(200), TrackType::Audio, 0);
    send(&mut bus, 0x03, &[]);
    ack(&mut bus);
    for _ in 0..19 {
        bus.tick_timers(SETOR_1X);
    }
    let st = bus.cdrom().audio_stats();
    assert_eq!(st.dropped, 0);
    assert_eq!(st.enqueued, 20 * CDDA_FRAMES as u64);
}

// § Report (06-cdrom.md L1254-1256): o relatorio sai em asect=00h,10h,20h... — um a cada
// dez setores tocados, nao um por intervalo de setor.
#[test]
fn report_sai_a_cada_dez_setores_tocados() {
    let mut bus = bus_com(disco_cdda(300), TrackType::Audio, MODE_REPORT);
    send(&mut bus, 0x03, &[]);
    ack(&mut bus);
    let mut relatorios = Vec::new();
    for intervalo in 0..45 {
        if hintsts(&mut bus) == 1 {
            relatorios.push((intervalo, bus.cdrom().audio_stats().enqueued));
            so_ack(&mut bus);
        }
        bus.tick_timers(SETOR_1X);
    }
    assert!(relatorios.len() >= 4, "relatorios: {relatorios:?}");
    for par in relatorios.windows(2) {
        assert_eq!(par[1].0 - par[0].0, 10, "relatorios: {relatorios:?}");
        assert_eq!(
            par[1].1 - par[0].1,
            10 * CDDA_FRAMES as u64,
            "entre dois relatorios tocam dez setores inteiros"
        );
    }
}

// § Setmode bit7 (06-cdrom.md L1238-1245): Play em dobro e avanco rapido audivel — o drive
// anda dois setores por intervalo de 588 quadros, entao o buffer so recebe metade.
#[test]
fn play_em_dobro_nao_acumula_audio() {
    let mut bus = bus_com(disco_cdda(400), TrackType::Audio, MODE_DOBRO);
    send(&mut bus, 0x03, &[]);
    ack(&mut bus);
    for _ in 0..60 {
        bus.tick_timers(SETOR_2X);
    }
    assert_eq!(bus.cdrom().audio_stats().dropped, 0);
    assert!(
        bus.cdrom().audio_pending() <= 2 * CDDA_FRAMES,
        "em dobro o buffer nao pode crescer: {}",
        bus.cdrom().audio_pending()
    );
}
