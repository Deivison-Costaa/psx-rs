mod support;

use psx_core::bus::{Bus, BusRead};
use support::asm;

const CD_BASE: u32 = 0x1F80_1800;
const ESPERA_PRIMEIRA_RESPOSTA: u32 = 0x1_4000;
const ATRASO_SEGUNDA: u32 = 0x6000;

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

fn result_read(bus: &mut Bus) -> u8 {
    set_bank(bus, 0);
    cd_read(bus, 1)
}

fn init_ate_int3(bus: &mut Bus) {
    bus.cdrom_mut().insert_disc();
    send_command(bus, 0x0A);
    let hintsts = hintsts_read_bank1(bus);
    assert_eq!(hintsts & 0x7, 3, "INT3 apos Init (0Ah)");
    let _ = result_read(bus);
}

#[test]
fn ack_nao_entrega_a_segunda_no_mesmo_instante() {
    let mut bus = bus();
    init_ate_int3(&mut bus);

    hclrctl_write(&mut bus, 0x07);

    let hintsts = hintsts_read_bank1(&mut bus);
    assert_eq!(
        hintsts & 0x7,
        0,
        "no instante do ack NAO pode haver INT novo: a 2a resposta tem atraso fisico \
         (§ Second Response: GetID medio 4A00h cycles) — entrega inline dentro da ISR \
         faz a BIOS engolir o INT2 ao limpar I_STAT antes de retornar"
    );
}

#[test]
fn segunda_resposta_chega_apos_o_atraso_do_ack() {
    let mut bus = bus();
    init_ate_int3(&mut bus);

    hclrctl_write(&mut bus, 0x07);
    bus.tick_timers(ATRASO_SEGUNDA);

    let hintsts = hintsts_read_bank1(&mut bus);
    assert_eq!(
        hintsts & 0x7,
        2,
        "INT2 do Init aparece depois do atraso contado a partir do ack do INT3"
    );
    let stat = result_read(&mut bus);
    assert_ne!(stat & (1 << 1), 0, "stat do INT2 do Init com motor ligado (bit1)");
}

#[test]
fn segunda_nao_atropela_a_primeira_sem_ack() {
    let mut bus = bus();
    init_ate_int3(&mut bus);

    bus.tick_timers(1_000_000);

    let hintsts = hintsts_read_bank1(&mut bus);
    assert_eq!(
        hintsts & 0x7,
        3,
        "sem ack do INT3, o INT2 fica enfileirado para sempre (fila da spec, L333-337)"
    );
}
