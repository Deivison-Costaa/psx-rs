mod support;

use psx_core::bus::{Bus, BusRead};
use support::asm;

const CD_BASE: u32 = 0x1F80_1800;

// § INT1 Rate (06-cdrom.md L2093-2101): "SystemClock*930h/4/44100Hz for Single Speed
// (and half as much for Double Speed)". SystemClock/44100 == CPU_CYCLES_PER_SAMPLE
// (spu.rs, 768) — 768*0x930/4 = 451584; a metade em dobro de velocidade e' 225792.
const CADENCIA_VELOCIDADE_NORMAL: u32 = 451_584;
const CADENCIA_DOBRO_VELOCIDADE: u32 = 225_792;

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
    bus.tick_timers(0x1_4000);
}

fn param_write(bus: &mut Bus, val: u8) {
    set_bank(bus, 0);
    cd_write(bus, 2, val);
}

fn result_read(bus: &mut Bus) -> u8 {
    set_bank(bus, 0);
    cd_read(bus, 1)
}

fn setmode(bus: &mut Bus, mode: u8) {
    param_write(bus, mode);
    send_command(bus, 0x0E);
    let _ = result_read(bus);
    hclrctl_write(bus, 0x07);
}

/// Prepara um ReadN em andamento (Setloc + ReadN + acknowledge do primeiro INT1) e
/// devolve o bus pronto pra medir a cadencia do SEGUNDO setor em diante.
fn read_n_em_andamento(bus: &mut Bus) {
    bus.cdrom_mut().insert_disc();
    param_write(bus, 0x02);
    param_write(bus, 0x10);
    param_write(bus, 0x00);
    send_command(bus, 0x02);
    let _ = result_read(bus);
    hclrctl_write(bus, 0x07);

    send_command(bus, 0x06);
    let _ = result_read(bus);
    hclrctl_write(bus, 0x07);
    bus.tick_timers(0x6000);
    let _ = result_read(bus);
    hclrctl_write(bus, 0x07);
}

#[test]
fn velocidade_normal_espera_451584_ciclos_entre_setores() {
    let mut bus = bus();
    read_n_em_andamento(&mut bus);

    bus.tick_timers(CADENCIA_VELOCIDADE_NORMAL - 1);
    assert_eq!(
        hintsts_read_bank1(&mut bus) & 0x7,
        0,
        "INT1 nao pode chegar antes da cadencia exata da spec"
    );

    bus.tick_timers(1);
    assert_eq!(
        hintsts_read_bank1(&mut bus) & 0x7,
        1,
        "INT1 chega exatamente em SystemClock*930h/4/44100Hz (06-cdrom.md L2100)"
    );
}

#[test]
fn dobro_de_velocidade_espera_metade_dos_ciclos_entre_setores() {
    let mut bus = bus();
    setmode(&mut bus, 0x80);
    read_n_em_andamento(&mut bus);

    bus.tick_timers(CADENCIA_DOBRO_VELOCIDADE - 1);
    assert_eq!(
        hintsts_read_bank1(&mut bus) & 0x7,
        0,
        "INT1 nao pode chegar antes da metade da cadencia normal"
    );

    bus.tick_timers(1);
    assert_eq!(
        hintsts_read_bank1(&mut bus) & 0x7,
        1,
        "dobro de velocidade cobra metade dos ciclos (06-cdrom.md L2100-2101)"
    );
}
