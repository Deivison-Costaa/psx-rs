mod support;

use psx_core::bus::{Bus, BusRead};
use psx_core::cdrom_bin_cue::{DiscLayout, TrackInfo, TrackType};
use support::asm;

const CD_BASE: u32 = 0x1F80_1800;
const ESPERA_PRIMEIRA_RESPOSTA: u32 = 0x1_4000;
const INICIO_DO_ARQUIVO_DA_TRILHA_10: u32 = 300;

fn cd_write(bus: &mut Bus, offset: u32, val: u8) {
    bus.write8::<BusRead>(CD_BASE + offset, val);
}

fn set_bank(bus: &mut Bus, b: u8) {
    cd_write(bus, 0, b);
}

fn ack(bus: &mut Bus) {
    set_bank(bus, 1);
    cd_write(bus, 3, 0x07);
    set_bank(bus, 0);
}

fn result_read(bus: &mut Bus) -> u8 {
    set_bank(bus, 0);
    bus.read8::<BusRead>(CD_BASE + 1)
}

fn send(bus: &mut Bus, cmd: u8, params: &[u8]) {
    set_bank(bus, 0);
    for p in params {
        cd_write(bus, 2, *p);
    }
    cd_write(bus, 1, cmd);
    bus.tick_timers(ESPERA_PRIMEIRA_RESPOSTA);
}

fn trilha(number: u8, file: &str, offset: u32, index00: Option<u8>, index01_ss: u8) -> TrackInfo {
    TrackInfo {
        number,
        file: file.to_string(),
        start_lba: offset + index01_ss as u32 * 75,
        track_type: if number == 1 {
            TrackType::Mode2_2352
        } else {
            TrackType::Audio
        },
        index01_mm: 0,
        index01_ss,
        index01_ff: 0,
        index00_mm: index00.map(|_| 0),
        index00_ss: index00,
        index00_ff: index00.map(|_| 0),
        pregap_mm: None,
        pregap_ss: None,
        pregap_ff: None,
    }
}

// Rip por trilha: a trilha 10 ocupa um arquivo proprio que comeca no LBA 300, com
// INDEX 00 00:00:00 e INDEX 01 00:02:00 (pregap de 150 setores dentro do arquivo).
fn bus_com_pregap() -> Bus {
    let mut bus = asm::bus_with_bios_empty();
    let layout = DiscLayout {
        bin_path: "t1.bin".to_string(),
        tracks: vec![
            trilha(1, "t1.bin", 0, None, 0),
            trilha(10, "t10.bin", INICIO_DO_ARQUIVO_DA_TRILHA_10, Some(0), 2),
        ],
    };
    bus.inject_disc(layout, vec![0u8; 2352 * 600]);
    bus.cdrom_mut().insert_disc();
    bus
}

fn msf_bcd(lba: u32) -> [u8; 3] {
    let q = lba + 150;
    let bcd = |n: u32| ((n / 10) << 4 | (n % 10)) as u8;
    [bcd(q / 4500), bcd((q / 75) % 60), bcd(q % 75)]
}

fn getlocp_apos_seekp(bus: &mut Bus, lba: u32) -> [u8; 8] {
    send(bus, 0x02, &msf_bcd(lba));
    ack(bus);
    send(bus, 0x16, &[]);
    ack(bus);
    let ciclos = bus.cdrom().second_response_cycles() as u32;
    bus.tick_timers(ciclos);
    ack(bus);
    send(bus, 0x11, &[]);
    let mut r = [0u8; 8];
    for b in r.iter_mut() {
        *b = result_read(bus);
    }
    ack(bus);
    r
}

// § GetlocP (06-cdrom.md L1073-1086): trilha e index vem do Subchannel Q, em BCD. § GetTD
// (L1098-1104): a regiao Index=0 fica antes do Index=1 da mesma trilha. No pregap o Q
// traz a trilha seguinte com index 00 e o tempo relativo contando para tras ate o Index=1.
#[test]
fn getlocp_no_pregap_devolve_a_trilha_seguinte_com_index_zero() {
    let mut bus = bus_com_pregap();
    let lba = INICIO_DO_ARQUIVO_DA_TRILHA_10 + 50;
    let r = getlocp_apos_seekp(&mut bus, lba);
    assert_eq!(r[0], 0x10, "trilha 10 em BCD (10h), nao 0Ah");
    assert_eq!(r[1], 0x00, "index 00 dentro do pregap");
    assert_eq!(
        &r[2..5],
        &[0x00, 0x01, 0x25],
        "faltam 100 setores ate o Index=1"
    );
    assert_eq!(&r[5..8], &msf_bcd(lba), "posicao absoluta do SeekP");
}

#[test]
fn getlocp_depois_do_index_um_conta_para_frente() {
    let mut bus = bus_com_pregap();
    let lba = INICIO_DO_ARQUIVO_DA_TRILHA_10 + 150 + 10;
    let r = getlocp_apos_seekp(&mut bus, lba);
    assert_eq!(r[0], 0x10, "trilha 10 em BCD");
    assert_eq!(r[1], 0x01, "index 01 depois do pregap");
    assert_eq!(
        &r[2..5],
        &[0x00, 0x00, 0x10],
        "10 setores depois do Index=1"
    );
}
