mod support;

use psx_core::bus::{Bus, BusRead, BusWrite};
use psx_core::cdrom_bin_cue::{DiscLayout, TrackInfo, TrackType};
use support::asm;

const CD_BASE: u32 = 0x1F80_1800;
const ESPERA_PRIMEIRA_RESPOSTA: u32 = 0x1_4000;
// 06-cdrom.md L333-337: 2a resposta so apos o ack, com atraso fisico (divida 10.53).
const ESPERA_SEGUNDA_RESPOSTA: u32 = 0x6000;
const SUBHEADER: u8 = 0xAA;

fn bus() -> Bus {
    asm::bus_with_bios_empty()
}

fn cd_read(bus: &Bus, offset: u32) -> u8 {
    bus.read8::<BusRead>(CD_BASE + offset)
}

fn cd_write(bus: &mut Bus, offset: u32, val: u8) {
    bus.write8::<BusWrite>(CD_BASE + offset, val);
}

fn set_bank(bus: &mut Bus, b: u8) {
    cd_write(bus, 0, b);
}

fn send_command(bus: &mut Bus, cmd: u8) {
    set_bank(bus, 0);
    cd_write(bus, 1, cmd);
    bus.tick_timers(ESPERA_PRIMEIRA_RESPOSTA);
}

fn param_write(bus: &mut Bus, val: u8) {
    set_bank(bus, 0);
    cd_write(bus, 2, val);
}

fn hclrctl_write(bus: &mut Bus, val: u8) {
    set_bank(bus, 1);
    cd_write(bus, 3, val);
    set_bank(bus, 0);
}

fn result_read(bus: &mut Bus) -> u8 {
    set_bank(bus, 0);
    cd_read(bus, 1)
}

fn rddata_read(bus: &mut Bus) -> u8 {
    set_bank(bus, 0);
    cd_read(bus, 2)
}

fn layout() -> DiscLayout {
    DiscLayout {
        bin_path: "test.bin".to_string(),
        tracks: vec![TrackInfo {
            number: 1,
            file: "test.bin".to_string(),
            start_lba: 150,
            track_type: TrackType::Mode1_2048,
            index01_mm: 0,
            index01_ss: 2,
            index01_ff: 0,
            index00_mm: None,
            index00_ss: None,
            index00_ff: None,
            pregap_mm: None,
            pregap_ss: None,
            pregap_ff: None,
        }],
    }
}

fn grava_setor(bin: &mut [u8], frame: usize, modo: u8, marca: [u8; 4]) {
    let base = frame * 2352;
    bin[base] = 0x00;
    for b in bin.iter_mut().skip(base + 1).take(10) {
        *b = 0xFF;
    }
    bin[base + 0x0F] = modo;
    let dados = if modo == 0x02 {
        for b in bin.iter_mut().skip(base + 0x10).take(8) {
            *b = SUBHEADER;
        }
        base + 0x18
    } else {
        base + 0x10
    };
    bin[dados..dados + 4].copy_from_slice(&marca);
}

fn bin_com(frames: &[(usize, u8, [u8; 4])]) -> Vec<u8> {
    let ultimo = frames.iter().map(|f| f.0).max().unwrap_or(0);
    let mut bin = vec![0u8; (ultimo + 1) * 2352];
    for (frame, modo, marca) in frames {
        grava_setor(&mut bin, *frame, *modo, *marca);
    }
    bin
}

fn le_quatro_bytes_do_frame(bin: Vec<u8>, ss: u8, ff: u8) -> [u8; 4] {
    let mut bus = bus();
    bus.inject_disc(layout(), bin);
    bus.cdrom_mut().insert_disc();

    param_write(&mut bus, 0x00);
    param_write(&mut bus, ss);
    param_write(&mut bus, ff);
    send_command(&mut bus, 0x02);
    let _ = result_read(&mut bus);
    hclrctl_write(&mut bus, 0x07);

    send_command(&mut bus, 0x06);
    let _ = result_read(&mut bus);
    hclrctl_write(&mut bus, 0x07);
    bus.tick_timers(ESPERA_SEGUNDA_RESPOSTA);
    let _ = result_read(&mut bus);

    [
        rddata_read(&mut bus),
        rddata_read(&mut bus),
        rddata_read(&mut bus),
        rddata_read(&mut bus),
    ]
}

#[test]
fn setor_mode2_form1_le_os_dados_a_partir_de_0x18() {
    let bin = bin_com(&[(0, 0x02, [0xDE, 0xAD, 0xBE, 0xEF])]);

    assert_eq!(
        le_quatro_bytes_do_frame(bin, 0x02, 0x00),
        [0xDE, 0xAD, 0xBE, 0xEF],
        "spec § Mode2/Form1 (CD-XA): '010h 4 Sub-Header', '014h 4 Copy of Sub-Header', \
         '018h 800h Data' — os dados de usuario comecam em 018h, nao em 010h"
    );
}

#[test]
fn subheader_do_mode2_nao_entra_nos_dados() {
    let bin = bin_com(&[(0, 0x02, [0x11, 0x22, 0x33, 0x44])]);
    let lidos = le_quatro_bytes_do_frame(bin, 0x02, 0x00);

    assert_ne!(
        lidos[0], SUBHEADER,
        "ler a partir de 010h devolve o sub-header como se fosse dado, e desloca o setor \
         inteiro em 8 bytes"
    );
}

#[test]
fn setor_mode1_continua_lendo_a_partir_de_0x10() {
    let bin = bin_com(&[(0, 0x01, [0xCA, 0xFE, 0xBA, 0xBE])]);

    assert_eq!(
        le_quatro_bytes_do_frame(bin, 0x02, 0x00),
        [0xCA, 0xFE, 0xBA, 0xBE],
        "spec § Mode1 (Original CDROM): '010h 800h Data' — o Mode1 nao tem sub-header"
    );
}

#[test]
fn setor_com_modo_zero_usa_o_offset_do_mode1() {
    let bin = bin_com(&[(0, 0x00, [0x01, 0x02, 0x03, 0x04])]);

    assert_eq!(
        le_quatro_bytes_do_frame(bin, 0x02, 0x00),
        [0x01, 0x02, 0x03, 0x04],
        "spec § Mode0 (Empty): tambem comeca em 010h; e o formato dos .bin sinteticos dos \
         testes antigos, que nao podem quebrar"
    );
}

#[test]
fn o_modo_e_lido_de_cada_setor_e_nao_do_disco() {
    let bin = bin_com(&[
        (0, 0x01, [0x1A, 0x1B, 0x1C, 0x1D]),
        (1, 0x02, [0x2A, 0x2B, 0x2C, 0x2D]),
    ]);

    assert_eq!(
        le_quatro_bytes_do_frame(bin.clone(), 0x02, 0x00),
        [0x1A, 0x1B, 0x1C, 0x1D],
        "frame 0 do arquivo e Mode1"
    );
    assert_eq!(
        le_quatro_bytes_do_frame(bin, 0x02, 0x01),
        [0x2A, 0x2B, 0x2C, 0x2D],
        "frame 1 do arquivo e Mode2/Form1 no MESMO disco — o offset sai do byte 00Fh de cada setor"
    );
}

#[test]
fn setor_com_cabecalho_truncado_no_fim_do_bin_nao_estoura() {
    let mut bin = bin_com(&[(0, 0x02, [0xDE, 0xAD, 0xBE, 0xEF])]);
    bin.truncate(5);

    let lidos = le_quatro_bytes_do_frame(bin, 0x02, 0x00);

    assert_eq!(
        lidos,
        [1, 2, 3, 4],
        "o inicio do setor cabe no .bin mas o byte de modo (00Fh) nao; ler o modo antes de \
         checar o tamanho indexa fora do vetor. Sem setor valido, vale o stub (i+1)"
    );
}

#[test]
fn setor_alem_do_fim_do_bin_nao_estoura() {
    let bin = bin_com(&[(0, 0x02, [0xDE, 0xAD, 0xBE, 0xEF])]);
    let mut bus = bus();
    bus.inject_disc(layout(), bin);
    bus.cdrom_mut().insert_disc();

    param_write(&mut bus, 0x00);
    param_write(&mut bus, 0x30);
    param_write(&mut bus, 0x00);
    send_command(&mut bus, 0x02);
    let _ = result_read(&mut bus);

    send_command(&mut bus, 0x06);
    let _ = result_read(&mut bus);
    hclrctl_write(&mut bus, 0x07);
    bus.tick_timers(ESPERA_SEGUNDA_RESPOSTA);
    let _ = result_read(&mut bus);

    let _ = rddata_read(&mut bus);
}
