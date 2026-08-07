mod support;

use psx_core::bus::{Bus, BusRead};
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

fn setloc(bus: &mut Bus, mm: u8, ss: u8, ff: u8) {
    param_write(bus, mm);
    param_write(bus, ss);
    param_write(bus, ff);
    send_command(bus, 0x02);
    let _ = result_read(bus);
    hclrctl_write(bus, 0x07);
}

#[test]
fn seek_p_com_disco_responde_int3_e_depois_int2() {
    let mut bus = bus();
    bus.cdrom_mut().insert_disc();
    setloc(&mut bus, 0x00, 0x10, 0x00);

    send_command(&mut bus, 0x16);
    let hintsts = hintsts_read_bank1(&mut bus);
    assert_eq!(hintsts & 0x7, 3, "INT3 na primeira resposta do SeekP (16h)");
    let stat = result_read(&mut bus);
    assert_eq!(stat & 0x01, 0, "stat bit0=0 — sem erro");
    assert_ne!(stat & (1 << 6), 0, "stat bit6=1 — buscando");

    hclrctl_write(&mut bus, 0x07);
    let ciclos_da_segunda = bus.cdrom().second_response_cycles() as u32;
    assert_ne!(
        ciclos_da_segunda, 0,
        "SeekP tem de armar uma segunda resposta"
    );
    bus.tick_timers(ciclos_da_segunda);
    let hintsts2 = hintsts_read_bank1(&mut bus);
    assert_eq!(hintsts2 & 0x7, 2, "INT2 apos o acknowledge do SeekP");
    let stat2 = result_read(&mut bus);
    assert_eq!(stat2 & (1 << 6), 0, "stat bit6=0 — busca concluida");
    assert_ne!(stat2 & (1 << 1), 0, "stat bit1=1 — motor ligado");
}

#[test]
fn seek_p_deixa_a_cabeca_no_alvo_do_setloc() {
    let mut bus = bus();
    bus.cdrom_mut().insert_disc();
    setloc(&mut bus, 0x00, 0x10, 0x25);

    send_command(&mut bus, 0x16);
    let _ = result_read(&mut bus);
    hclrctl_write(&mut bus, 0x07);
    let ciclos_da_segunda = bus.cdrom().second_response_cycles() as u32;
    bus.tick_timers(ciclos_da_segunda);
    let _ = result_read(&mut bus);
    hclrctl_write(&mut bus, 0x07);

    send_command(&mut bus, 0x11);
    let _track = result_read(&mut bus);
    let _index = result_read(&mut bus);
    let _mm = result_read(&mut bus);
    let _ss = result_read(&mut bus);
    let _ff = result_read(&mut bus);
    let amm = result_read(&mut bus);
    let ass = result_read(&mut bus);
    let asect = result_read(&mut bus);
    assert_eq!(
        (amm, ass, asect),
        (0x00, 0x10, 0x25),
        "GetlocP devolve o MM:SS:FF inteiro do Setloc apos o SeekP"
    );
}

#[test]
fn seek_p_desliga_o_bit7_de_playback() {
    let mut bus = bus();
    bus.cdrom_mut().insert_disc();
    setloc(&mut bus, 0x00, 0x02, 0x00);
    send_command(&mut bus, 0x03);
    let stat_play = result_read(&mut bus);
    assert_ne!(stat_play & (1 << 7), 0, "stat bit7=1 — tocando apos Play");
    hclrctl_write(&mut bus, 0x07);

    setloc(&mut bus, 0x00, 0x20, 0x00);
    send_command(&mut bus, 0x16);
    let _ = result_read(&mut bus);
    hclrctl_write(&mut bus, 0x07);
    let ciclos_da_segunda = bus.cdrom().second_response_cycles() as u32;
    bus.tick_timers(ciclos_da_segunda);
    let hintsts2 = hintsts_read_bank1(&mut bus);
    assert_eq!(hintsts2 & 0x7, 2, "INT2 apos o acknowledge do SeekP");
    let stat2 = result_read(&mut bus);
    assert_eq!(
        stat2 & (1 << 7),
        0,
        "stat bit7=0 — playback desligado apos o SeekP"
    );
}

#[test]
fn seek_p_sem_disco_responde_int5_com_erro_80h() {
    let mut bus = bus();
    send_command(&mut bus, 0x16);
    let hintsts = hintsts_read_bank1(&mut bus);
    assert_eq!(hintsts & 0x7, 5, "INT5 no SeekP sem disco");
    let stat = result_read(&mut bus);
    assert_ne!(stat & 0x01, 0, "stat bit0=1 — erro");
    let err = result_read(&mut bus);
    assert_eq!(err, 0x80, "error byte = 80h (sem disco)");
}
