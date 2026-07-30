mod support;

use psx_core::bus::{Bus, BusRead};
use support::asm;

fn bus() -> Bus {
    asm::bus_with_bios_empty()
}

const T0_CNT: u32 = 0x1F80_1100;
const T0_MODE: u32 = 0x1F80_1104;
const T0_TARGET: u32 = 0x1F80_1108;

fn tick(bus: &mut Bus, base: u32, cycles: u32) -> Option<u32> {
    let hb = bus.gpu().hblank_active();
    let vb = bus.gpu().vblank_active();
    bus.timers_mut().tick(base, cycles, hb, vb)
}

#[test]
fn bit10_fica_em_zero_apos_irq_pulse_pelo_menos_um_tick() {
    let mut bus = bus();
    bus.write32::<BusRead>(T0_TARGET, 0x0001);
    bus.write32::<BusRead>(T0_MODE, 0x0010);

    let irq = tick(&mut bus, T0_CNT, 1);
    assert_eq!(irq, Some(4), "IRQ4 deve disparar no primeiro tick");

    let bit10 = bus.read32::<BusRead>(T0_MODE) & (1 << 10);
    assert_eq!(
        bit10, 0,
        "bit10 deve ficar em 0 apos IRQ pulse — BIOS precisa ler 0 para detectar o IRQ do timer"
    );
}

#[test]
fn bit10_volta_a_um_no_segundo_tick_apos_irq_pulse() {
    let mut bus = bus();
    bus.write32::<BusRead>(T0_TARGET, 0x0001);
    bus.write32::<BusRead>(T0_MODE, 0x0010);

    tick(&mut bus, T0_CNT, 1);
    tick(&mut bus, T0_CNT, 1);

    let bit10 = bus.read32::<BusRead>(T0_MODE) & (1 << 10);
    assert_eq!(
        bit10,
        1 << 10,
        "bit10 deve voltar a 1 no tick seguinte ao IRQ — pulso dura poucos ciclos"
    );
}

#[test]
fn bit10_restaurado_ao_escrever_no_mode() {
    let mut bus = bus();
    bus.write32::<BusRead>(T0_TARGET, 0x0001);
    bus.write32::<BusRead>(T0_MODE, 0x0010);

    tick(&mut bus, T0_CNT, 1);

    let bit10_antes = bus.read32::<BusRead>(T0_MODE) & (1 << 10);
    assert_eq!(bit10_antes, 0, "bit10 em 0 apos IRQ pulse");

    bus.write32::<BusRead>(T0_MODE, 0x0010);

    let bit10_depois = bus.read32::<BusRead>(T0_MODE) & (1 << 10);
    assert_eq!(
        bit10_depois,
        1 << 10,
        "bit10 restaurado apos escrever no mode register"
    );
}

#[test]
fn toggle_mode_bit10_nao_e_restaurado_automaticamente() {
    let mut bus = bus();
    bus.write32::<BusRead>(T0_TARGET, 0x0001);
    bus.write32::<BusRead>(T0_MODE, 0x00D0);

    let irq1 = tick(&mut bus, T0_CNT, 1);
    assert_eq!(irq1, Some(4), "toggle: 1→0 gera IRQ4");
    let bit10 = bus.read32::<BusRead>(T0_MODE) & (1 << 10);
    assert_eq!(bit10, 0, "toggle: bit10 fica em 0 apos primeiro IRQ");

    bus.write32::<BusRead>(T0_CNT, 0);
    tick(&mut bus, T0_CNT, 1);

    let bit10 = bus.read32::<BusRead>(T0_MODE) & (1 << 10);
    assert_eq!(bit10, 1 << 10, "toggle: bit10 volta a 1 no segundo IRQ");
}

#[test]
fn bit10_fica_em_zero_apos_irq_ffff_pulse() {
    let mut bus = bus();
    bus.write32::<BusRead>(T0_MODE, 0x0020);
    bus.write32::<BusRead>(T0_CNT, 0xFFFF);

    let irq = tick(&mut bus, T0_CNT, 1);
    assert_eq!(irq, Some(4), "IRQ4 deve disparar quando FFFF alcancado com bit5=1");

    let bit10 = bus.read32::<BusRead>(T0_MODE) & (1 << 10);
    assert_eq!(
        bit10, 0,
        "bit10 deve ficar em 0 apos IRQ por overflow de FFFF em pulse mode"
    );
}
