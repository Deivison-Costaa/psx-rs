mod support;

use psx_core::bus::{Bus, BusRead};
use support::asm;

const CD_BASE: u32 = 0x1F80_1800;
const ESPERA_PRIMEIRA_RESPOSTA: u32 = 0x1_4000;
// 06-cdrom.md L333-337 + L2077-2078: 2a resposta so apos o ack, e o atraso do SeekL e' o
// proprio tempo de busca — depende da distancia e varia. Sai do estado, nao de constante.

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

fn insert_stub_disc(bus: &mut Bus) {
    bus.cdrom_mut().insert_disc();
}

fn bcd_minute(m: u8) -> u8 {
    m
}

fn bcd_second(s: u8) -> u8 {
    s
}

fn bcd_sector(f: u8) -> u8 {
    f
}

#[test]
fn setloc_com_bcd_valido_retorna_int3_e_armazena_posicao() {
    let mut bus = bus();
    insert_stub_disc(&mut bus);
    param_write(&mut bus, bcd_minute(0x02));
    param_write(&mut bus, bcd_second(0x10));
    param_write(&mut bus, bcd_sector(0x00));
    send_command(&mut bus, 0x02);
    let hintsts = hintsts_read_bank1(&mut bus);
    assert_eq!(hintsts & 0x7, 3, "INT3 apos Setloc (02h)");
    let stat = result_read(&mut bus);
    assert_eq!(stat & 0x01, 0, "stat bit0=0 — sem erro no Setloc");
}

#[test]
fn setloc_rejeita_segundo_bcd_invalido() {
    let mut bus = bus();
    insert_stub_disc(&mut bus);
    param_write(&mut bus, bcd_minute(0x02));
    param_write(&mut bus, 0x60);
    param_write(&mut bus, bcd_sector(0x00));
    send_command(&mut bus, 0x02);
    let hintsts = hintsts_read_bank1(&mut bus);
    assert_eq!(hintsts & 0x7, 5, "INT5 apos Setloc com ss >= 60h");
    let stat = result_read(&mut bus);
    assert_eq!(stat & 0x01, 0x01, "stat bit0=1 — erro");
    let err = result_read(&mut bus);
    assert_eq!(err, 0x10, "error byte = 10h (parametro invalido)");
}

#[test]
fn setloc_rejeita_setor_bcd_invalido() {
    let mut bus = bus();
    insert_stub_disc(&mut bus);
    param_write(&mut bus, bcd_minute(0x02));
    param_write(&mut bus, bcd_second(0x10));
    param_write(&mut bus, 0x75);
    send_command(&mut bus, 0x02);
    let hintsts = hintsts_read_bank1(&mut bus);
    assert_eq!(hintsts & 0x7, 5, "INT5 apos Setloc com asect >= 75h");
    let stat = result_read(&mut bus);
    assert_eq!(stat & 0x01, 0x01, "stat bit0=1 — erro");
    let err = result_read(&mut bus);
    assert_eq!(err, 0x10, "error byte = 10h (parametro invalido)");
}

#[test]
fn setloc_aceita_setor_0x74_bcd_valido() {
    let mut bus = bus();
    insert_stub_disc(&mut bus);
    param_write(&mut bus, bcd_minute(0x02));
    param_write(&mut bus, bcd_second(0x10));
    param_write(&mut bus, 0x74);
    send_command(&mut bus, 0x02);
    let hintsts = hintsts_read_bank1(&mut bus);
    assert_eq!(hintsts & 0x7, 3, "INT3 apos Setloc com setor 74h (valido)");
    let stat = result_read(&mut bus);
    assert_eq!(stat & 0x01, 0, "stat bit0=0 — sem erro");
}

#[test]
fn setloc_sem_disco_retorna_int3_mas_stat_com_bit0() {
    let mut bus = bus();
    param_write(&mut bus, bcd_minute(0x02));
    param_write(&mut bus, bcd_second(0x10));
    param_write(&mut bus, bcd_sector(0x00));
    send_command(&mut bus, 0x02);
    let hintsts = hintsts_read_bank1(&mut bus);
    assert_eq!(
        hintsts & 0x7,
        5,
        "INT5 quando parametros validos mas sem disco"
    );
    let stat = result_read(&mut bus);
    assert_eq!(stat & 0x01, 0x01, "stat bit0=1 — erro sem disco");
    let err = result_read(&mut bus);
    assert_eq!(err, 0x80, "error byte = 80h (sem disco)");
}

#[test]
fn seek_l_com_disco_retorna_int3_depois_int2() {
    let mut bus = bus();
    insert_stub_disc(&mut bus);
    param_write(&mut bus, bcd_minute(0x00));
    param_write(&mut bus, bcd_second(0x02));
    param_write(&mut bus, bcd_sector(0x00));
    send_command(&mut bus, 0x02);
    let _ = result_read(&mut bus);
    hclrctl_write(&mut bus, 0x07);
    send_command(&mut bus, 0x15);
    let hintsts = hintsts_read_bank1(&mut bus);
    assert_eq!(hintsts & 0x7, 3, "INT3 apos SeekL (15h)");
    let stat = result_read(&mut bus);
    assert_eq!(stat & 0x01, 0, "stat bit0=0 — sem erro");
    assert_ne!(stat & (1 << 6), 0, "stat bit6=1 — seeking");
    hclrctl_write(&mut bus, 0x07);
    let ciclos_da_segunda = bus.cdrom().second_response_cycles() as u32;
    bus.tick_timers(ciclos_da_segunda);
    let hintsts2 = hintsts_read_bank1(&mut bus);
    assert_eq!(hintsts2 & 0x7, 2, "INT2 apos acknowledge do SeekL");
    let stat2 = result_read(&mut bus);
    assert_eq!(stat2 & (1 << 6), 0, "stat bit6=0 — seek concluido");
    assert_ne!(stat2 & (1 << 1), 0, "stat bit1=1 — motor ligado");
}

#[test]
fn seek_l_sem_disco_retorna_int5() {
    let mut bus = bus();
    param_write(&mut bus, bcd_minute(0x00));
    param_write(&mut bus, bcd_second(0x02));
    param_write(&mut bus, bcd_sector(0x00));
    send_command(&mut bus, 0x02);
    let _ = result_read(&mut bus);
    hclrctl_write(&mut bus, 0x07);
    send_command(&mut bus, 0x15);
    let hintsts = hintsts_read_bank1(&mut bus);
    assert_eq!(hintsts & 0x7, 5, "INT5 apos SeekL sem disco");
    let stat = result_read(&mut bus);
    assert_ne!(stat & 0x01, 0, "stat bit0=1 — erro");
    let err = result_read(&mut bus);
    assert_eq!(err, 0x80, "error byte = 80h (sem disco)");
}

#[test]
fn pause_retorna_int3_depois_int2() {
    let mut bus = bus();
    insert_stub_disc(&mut bus);
    send_command(&mut bus, 0x09);
    let hintsts = hintsts_read_bank1(&mut bus);
    assert_eq!(hintsts & 0x7, 3, "INT3 apos Pause (09h)");
    let stat = result_read(&mut bus);
    assert_eq!(stat & 0x01, 0, "stat bit0=0 — sem erro");
    hclrctl_write(&mut bus, 0x07);
    let ciclos_da_segunda = bus.cdrom().second_response_cycles() as u32;
    bus.tick_timers(ciclos_da_segunda);
    let hintsts2 = hintsts_read_bank1(&mut bus);
    assert_eq!(hintsts2 & 0x7, 2, "INT2 apos acknowledge do Pause");
    let stat2 = result_read(&mut bus);
    assert_eq!(stat2 & (1 << 5), 0, "stat bit5=0 — nao esta lendo");
    assert_ne!(stat2 & (1 << 1), 0, "stat bit1=1 — motor ligado");
}

#[test]
fn setloc_consome_tres_parametros_fifo_alinhado() {
    let mut bus = bus();
    insert_stub_disc(&mut bus);
    param_write(&mut bus, bcd_minute(0x02));
    param_write(&mut bus, bcd_second(0x10));
    param_write(&mut bus, bcd_sector(0x00));
    send_command(&mut bus, 0x02);
    let _ = result_read(&mut bus);
    hclrctl_write(&mut bus, 0x07);
    param_write(&mut bus, 0x20);
    send_command(&mut bus, 0x19);
    let yy = result_read(&mut bus);
    assert_eq!(
        yy, 0x97,
        "GetStat(20h) funciona apos Setloc — FIFO alinhado"
    );
}

#[test]
fn stat_byte_reflete_motor_ligado_com_disco() {
    let mut bus = bus();
    insert_stub_disc(&mut bus);
    send_command(&mut bus, 0x0A);
    let stat = result_read(&mut bus);
    assert_ne!(
        stat & (1 << 1),
        0,
        "stat bit1=1 — motor ligado apos Init com disco"
    );
}

#[test]
fn stat_byte_sem_disco_motor_desligado() {
    let mut bus = bus();
    send_command(&mut bus, 0x0A);
    let stat = result_read(&mut bus);
    assert_eq!(
        stat & (1 << 1),
        0,
        "stat bit1=0 — motor desligado sem disco"
    );
}
