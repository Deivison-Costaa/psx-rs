mod support;

use psx_core::bus::{Bus, BusRead};
use psx_core::cdrom_bin_cue::parse_cue;
use support::asm;

const CD_BASE: u32 = 0x1F80_1800;
const ESPERA_PRIMEIRA_RESPOSTA: u32 = 0x1_4000;

fn bus() -> Bus {
    asm::bus_with_bios_empty()
}

fn cd_read(bus: &Bus, offset: u32) -> u8 {
    bus.read8::<BusRead>(CD_BASE + offset)
}

fn cd_write(bus: &mut Bus, offset: u32, val: u8) {
    bus.write8::<BusRead>(CD_BASE + offset, val);
}

fn set_bank(bus: &mut Bus, b: u8) {
    cd_write(bus, 0, b);
}

fn hintsts_read_bank1(bus: &mut Bus) -> u8 {
    set_bank(bus, 1);
    let val = cd_read(bus, 3);
    set_bank(bus, 0);
    val
}

fn hclrctl_write(bus: &mut Bus, val: u8) {
    set_bank(bus, 1);
    cd_write(bus, 3, val);
    set_bank(bus, 0);
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

fn result_read(bus: &mut Bus) -> u8 {
    set_bank(bus, 0);
    cd_read(bus, 1)
}

fn rddata_read(bus: &mut Bus) -> u8 {
    set_bank(bus, 0);
    cd_read(bus, 2)
}

fn synthetic_disc(sectors: usize) -> Vec<u8> {
    let mut bin = vec![0u8; sectors * 2352];
    for (i, sector) in bin.chunks_mut(2352).enumerate() {
        sector[0x0F] = 0x02;
        for b in sector[0x18..0x18 + 2048].iter_mut() {
            *b = 0x40 + i as u8;
        }
    }
    bin
}

fn bus_with_synthetic_disc(sectors: usize) -> Bus {
    let mut bus = bus();
    let layout = parse_cue("FILE \"x.bin\" BINARY\n  TRACK 01 MODE2/2352\n    INDEX 01 00:00:00\n");
    bus.inject_disc(layout, synthetic_disc(sectors));
    bus.cdrom_mut().insert_disc();
    bus
}

fn setloc_readn_primeiro_byte(bus: &mut Bus, mm: u8, ss: u8, ff: u8) -> u8 {
    param_write(bus, mm);
    param_write(bus, ss);
    param_write(bus, ff);
    send_command(bus, 0x02);
    let _ = result_read(bus);

    send_command(bus, 0x06);
    let hintsts = hintsts_read_bank1(bus);
    assert_eq!(hintsts & 0x7, 3, "INT3 apos ReadN (06h)");
    let _ = result_read(bus);
    hclrctl_write(bus, 0x07);
    bus.tick_timers(0x6000);

    let hintsts2 = hintsts_read_bank1(bus);
    assert_eq!(
        hintsts2 & 0x7,
        1,
        "INT1 com dados apos o atraso contado do ack do INT3"
    );
    let _ = result_read(bus);

    rddata_read(bus)
}

#[test]
fn setloc_msf_absoluto_mapeia_para_setor_do_arquivo_menos_150() {
    let mut bus = bus_with_synthetic_disc(8);

    let byte0 = setloc_readn_primeiro_byte(&mut bus, 0x00, 0x02, 0x05);

    assert_eq!(
        byte0, 0x45,
        "MSF absoluto 00:02:05 (setor absoluto 155) deve entregar o setor 5 DO ARQUIVO \
         (155 - 150 do pregap): user data 0x45, nao 0x{:02X}",
        byte0
    );
}

#[test]
fn setloc_no_inicio_da_trilha_entrega_o_primeiro_setor_do_arquivo() {
    let mut bus = bus_with_synthetic_disc(8);

    let byte0 = setloc_readn_primeiro_byte(&mut bus, 0x00, 0x02, 0x00);

    assert_eq!(
        byte0, 0x40,
        "MSF 00:02:00 e o comeco da trilha 1: setor 0 do arquivo (user data 0x40)"
    );
}

#[test]
fn setloc_dentro_do_pregap_nao_estoura_nem_entrega_dado() {
    let mut bus = bus_with_synthetic_disc(8);

    param_write(&mut bus, 0x00);
    param_write(&mut bus, 0x01);
    param_write(&mut bus, 0x00);
    send_command(&mut bus, 0x02);
    let _ = result_read(&mut bus);

    send_command(&mut bus, 0x06);
    let _ = result_read(&mut bus);
    hclrctl_write(&mut bus, 0x07);
    bus.tick_timers(ESPERA_PRIMEIRA_RESPOSTA);

    let byte0 = rddata_read(&mut bus);
    assert!(
        !(0x40..0x48).contains(&byte0),
        "MSF 00:01:00 (setor absoluto 75, dentro do pregap) nao pode entregar dado do \
         arquivo (veio 0x{:02X})",
        byte0
    );
}
