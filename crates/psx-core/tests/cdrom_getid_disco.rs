mod support;

use psx_core::bus::{Bus, BusRead, BusWrite};
use support::asm;

const CD_BASE: u32 = 0x1F80_1800;
const ESPERA_PRIMEIRA_RESPOSTA: u32 = 0x1_4000;
const GETID: u8 = 0x1A;

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

fn hintsts(bus: &mut Bus) -> u8 {
    set_bank(bus, 1);
    let val = cd_read(bus, 3) & 0x7;
    set_bank(bus, 0);
    val
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

fn segunda_resposta_do_getid(bus: &mut Bus) -> Vec<u8> {
    send_command(bus, GETID);
    let _ = result_read(bus);
    hclrctl_write(bus, 0x07);
    (0..8).map(|_| result_read(bus)).collect()
}

fn com_disco() -> Bus {
    let mut bus = bus();
    bus.cdrom_mut().insert_disc();
    bus
}

#[test]
fn getid_com_disco_responde_int2_e_nao_int5() {
    let mut bus = com_disco();
    send_command(&mut bus, GETID);
    let _ = result_read(&mut bus);
    hclrctl_write(&mut bus, 0x07);

    assert_eq!(
        hintsts(&mut bus),
        2,
        "spec § GetID: a linha Licensed:Mode2 responde INT2; INT5 e a resposta de disco ausente \
         ou nao licenciado, e o shell da BIOS desiste do boot com ela"
    );
}

#[test]
fn getid_com_disco_devolve_os_oito_bytes_da_spec() {
    let mut bus = com_disco();
    let resp = segunda_resposta_do_getid(&mut bus);

    assert_eq!(
        resp,
        vec![0x02, 0x00, 0x20, 0x00, 0x53, 0x43, 0x45, 0x41],
        "spec § GetID, linha Licensed:Mode2: INT2(02h,00h, 20h,00h, 53h,43h,45h,4xh); com a \
         SCPH1001 (NTSC-U) a quarta letra e 'A'"
    );
}

#[test]
fn primeiro_byte_da_segunda_resposta_e_o_stat() {
    let mut bus = com_disco();
    let resp = segunda_resposta_do_getid(&mut bus);

    assert_eq!(
        resp[0], 0x02,
        "spec § GetID: '1st byte: stat (as usually...)'; com disco e motor ligado o stat e 02h"
    );
}

#[test]
fn flags_dizem_licenciado_presente_e_nao_audio() {
    let mut bus = com_disco();
    let resp = segunda_resposta_do_getid(&mut bus);
    let flags = resp[1];

    assert_eq!(flags & (1 << 7), 0, "bit7=0: Licensed Data CD");
    assert_eq!(flags & (1 << 6), 0, "bit6=0: Disk Present");
    assert_eq!(flags & (1 << 4), 0, "bit4=0: Data CD, nao Audio CD");
}

#[test]
fn tipo_do_disco_e_mode2() {
    let mut bus = com_disco();
    let resp = segunda_resposta_do_getid(&mut bus);

    assert_eq!(
        resp[2], 0x20,
        "spec § GetID: '3rd byte: Disk type (from TOC Point=A0h) ... 20h=Mode2'"
    );
    assert_eq!(resp[3], 0x00, "4th byte: usually 00h");
}

#[test]
fn regiao_e_scea_para_a_bios_ntsc_u() {
    let mut bus = com_disco();
    let resp = segunda_resposta_do_getid(&mut bus);

    assert_eq!(
        &resp[4..8],
        b"SCEA",
        "spec § GetID: 'SCEA' (America/NTSC); a PSX recusa o boot se a regiao nao casar"
    );
}

#[test]
fn getid_sem_disco_continua_na_linha_no_disk() {
    let mut bus = bus();
    let resp = segunda_resposta_do_getid(&mut bus);

    assert_eq!(
        resp,
        vec![0x08, 0x40, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
        "spec § GetID, linha No Disk: INT5(08h,40h, 00h,00h, 00h,00h,00h,00h) — o caminho sem \
         disco nao muda"
    );
}

#[test]
fn getid_sem_disco_continua_respondendo_int5() {
    let mut bus = bus();
    send_command(&mut bus, GETID);
    let _ = result_read(&mut bus);
    hclrctl_write(&mut bus, 0x07);

    assert_eq!(
        hintsts(&mut bus),
        5,
        "sem disco a segunda resposta e INT5, como antes desta iteracao"
    );
}
