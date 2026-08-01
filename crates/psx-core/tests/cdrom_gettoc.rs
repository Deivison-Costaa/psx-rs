mod support;

use psx_core::bus::{Bus, BusRead, BusWrite};
use support::asm;

const CD_BASE: u32 = 0x1F80_1800;
const ESPERA_PRIMEIRA_RESPOSTA: u32 = 0x1_4000;
const PRIMEIRA_RESPOSTA_LONGA: u32 = 0x1_3CCE;
const GETTOC: u8 = 0x1E;

fn bus() -> Bus {
    let mut bus = asm::bus_with_bios_empty();
    bus.cdrom_mut().insert_disc();
    bus
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

fn manda_comando(bus: &mut Bus, cmd: u8) {
    set_bank(bus, 0);
    cd_write(bus, 1, cmd);
}

fn send_command(bus: &mut Bus, cmd: u8) {
    manda_comando(bus, cmd);
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

#[test]
fn gettoc_responde_int3_com_o_stat() {
    let mut bus = bus();
    send_command(&mut bus, GETTOC);

    assert_eq!(
        hintsts(&mut bus),
        3,
        "spec § Second Responses: 1Eh ReadTOC — INT3(late-stat), INT2(stat)"
    );
    assert_eq!(
        result_read(&mut bus),
        0x02,
        "a primeira resposta e o stat; com disco e motor ligado, 02h"
    );
}

#[test]
fn gettoc_arma_a_segunda_resposta_int2() {
    let mut bus = bus();
    send_command(&mut bus, GETTOC);
    let _ = result_read(&mut bus);

    hclrctl_write(&mut bus, 0x07);
    bus.tick_timers(0x6000);

    assert_eq!(
        hintsts(&mut bus),
        2,
        "spec § Second Responses: sem o INT2 o driver do kernel espera para sempre — foi onde o \
         boot parou na medicao da 0122"
    );
}

#[test]
fn segunda_resposta_do_gettoc_devolve_o_stat() {
    let mut bus = bus();
    send_command(&mut bus, GETTOC);
    let _ = result_read(&mut bus);
    hclrctl_write(&mut bus, 0x07);
    bus.tick_timers(0x6000);

    assert_eq!(
        result_read(&mut bus),
        0x02,
        "spec § Second Responses: INT2(stat) — a segunda resposta do ReadTOC e so o stat"
    );
    assert_eq!(
        cd_read(&bus, 0) & (1 << 5),
        0,
        "RSLRRDY baixo: INT2(stat) tem UM byte. O GetID tambem responde INT2 com o mesmo 02h no \
         primeiro byte, mas com oito bytes — sem olhar o tamanho, os dois sao indistinguiveis"
    );
}

#[test]
fn gettoc_nao_dispara_terceira_resposta() {
    let mut bus = bus();
    send_command(&mut bus, GETTOC);
    let _ = result_read(&mut bus);
    hclrctl_write(&mut bus, 0x07);
    bus.tick_timers(0x6000);
    let _ = result_read(&mut bus);

    hclrctl_write(&mut bus, 0x07);
    bus.tick_timers(0x6000);

    assert_eq!(
        hintsts(&mut bus),
        0,
        "o ReadTOC tem duas respostas, nao tres: depois do ack do INT2 nao sobra nada pendente"
    );
}

#[test]
fn gettoc_usa_o_atraso_longo_da_primeira_resposta() {
    let mut bus = bus();
    manda_comando(&mut bus, GETTOC);

    bus.tick_timers(PRIMEIRA_RESPOSTA_LONGA - 1);
    assert_eq!(
        hintsts(&mut bus),
        0,
        "spec § First Response: 'The ReadTOC command is doing similar initialization, and should \
         have similar timing as Init command' — 0013cceh, nao os 000c4e1h do caso comum"
    );

    bus.tick_timers(1);
    assert_eq!(hintsts(&mut bus), 3, "no prazo de 0013cceh a INT3 sai");
}

#[test]
fn gettoc_deixa_o_drive_pronto_para_o_proximo_comando() {
    let mut bus = bus();
    send_command(&mut bus, GETTOC);
    let _ = result_read(&mut bus);
    hclrctl_write(&mut bus, 0x07);
    bus.tick_timers(0x6000);
    let _ = result_read(&mut bus);
    hclrctl_write(&mut bus, 0x07);

    assert_eq!(
        cd_read(&bus, 0) & (1 << 7),
        0,
        "BUSYSTS baixo: o drive aceita comando novo depois da segunda resposta"
    );

    send_command(&mut bus, 0x01);
    assert_eq!(
        hintsts(&mut bus),
        3,
        "um GetStat depois do ReadTOC responde normalmente"
    );
}
