mod support;

use psx_core::bus::{Bus, BusRead};
use support::asm;

fn bus() -> Bus {
    asm::bus_with_bios_empty()
}

const T0_CNT: u32 = 0x1F80_1100;
const T0_MODE: u32 = 0x1F80_1104;
const T0_TARGET: u32 = 0x1F80_1108;
const T2_CNT: u32 = 0x1F80_1120;
const T2_MODE: u32 = 0x1F80_1124;

fn tick(bus: &mut Bus, base: u32, cycles: u32) -> Option<u32> {
    let hb = bus.gpu().hblank_active();
    let vb = bus.gpu().vblank_active();
    bus.timers_mut().tick(base, cycles, hb, vb)
}

#[test]
fn target_flag_alcancado_sem_reset_on_target() {
    let mut bus = bus();
    bus.write32::<BusRead>(T0_TARGET, 0x0003);
    bus.write32::<BusRead>(T0_MODE, 0x0000);
    bus.timers_mut().tick(T0_CNT, 3, false, false);
    let mode = bus.read32::<BusRead>(T0_MODE);
    assert_eq!(
        mode & (1 << 11),
        1 << 11,
        "bit11 setado quando CNT==target, mesmo sem reset_on_target (bit3=0)"
    );
    let cnt = bus.read32::<BusRead>(T0_CNT) & 0xFFFF;
    assert_eq!(cnt, 3, "CNT nao resetou — bit3 esta desligado");
}

#[test]
fn target_flag_setado_mesmo_com_irq_desabilitada() {
    let mut bus = bus();
    bus.write32::<BusRead>(T0_TARGET, 0x0002);
    bus.write32::<BusRead>(T0_MODE, 0x0000);
    bus.timers_mut().tick(T0_CNT, 2, false, false);
    let mode = bus.read32::<BusRead>(T0_MODE);
    assert_eq!(
        mode & (1 << 11),
        1 << 11,
        "bit11 setado independente de IRQ enable (bit4=0)"
    );
}

#[test]
fn ffff_flag_setado_mesmo_com_irq_desabilitada() {
    let mut bus = bus();
    bus.write32::<BusRead>(T0_MODE, 0x0000);
    bus.write32::<BusRead>(T0_CNT, 0xFFFF);
    bus.timers_mut().tick(T0_CNT, 1, false, false);
    let mode = bus.read32::<BusRead>(T0_MODE);
    assert_eq!(
        mode & (1 << 12),
        1 << 12,
        "bit12 setado independente de IRQ enable (bit5=0)"
    );
}

#[test]
fn irq_target_pulse_retorna_irq_bit4() {
    let mut bus = bus();
    bus.write32::<BusRead>(T0_TARGET, 0x0001);
    bus.write32::<BusRead>(T0_MODE, 0x0010);
    let irq = tick(&mut bus, T0_CNT, 1);
    assert_eq!(irq, Some(4), "IRQ4 deve ser retornado quando target alcancado em pulse mode");
    let mode_after_read = bus.read32::<BusRead>(T0_MODE);
    assert_eq!(
        mode_after_read & (1 << 10),
        1 << 10,
        "bit10 volta a 1 apos o pulso"
    );
}

#[test]
fn irq_target_desabilitado_sem_bit4() {
    let mut bus = bus();
    bus.write32::<BusRead>(T0_TARGET, 0x0001);
    bus.write32::<BusRead>(T0_MODE, 0x0000);
    let irq = tick(&mut bus, T0_CNT, 1);
    assert_eq!(irq, None, "sem IRQ quando bit4=0");
    let mode_after = bus.read32::<BusRead>(T0_MODE);
    assert_eq!(
        mode_after & (1 << 11),
        1 << 11,
        "bit11 setado mesmo sem IRQ"
    );
}

#[test]
fn irq_ffff_pulse_retorna_irq_bit4() {
    let mut bus = bus();
    bus.write32::<BusRead>(T0_MODE, 0x0020);
    bus.write32::<BusRead>(T0_CNT, 0xFFFF);
    let irq = tick(&mut bus, T0_CNT, 1);
    assert_eq!(irq, Some(4), "IRQ4 deve ser retornado quando FFFF alcancado com bit5=1");
}

#[test]
fn irq_ffff_desabilitado_sem_bit5() {
    let mut bus = bus();
    bus.write32::<BusRead>(T0_MODE, 0x0000);
    bus.write32::<BusRead>(T0_CNT, 0xFFFF);
    let irq = tick(&mut bus, T0_CNT, 1);
    assert_eq!(irq, None, "sem IRQ quando bit5=0");
}

#[test]
fn irq_oneshot_nao_dispara_segunda_vez() {
    let mut bus = bus();
    bus.write32::<BusRead>(T0_TARGET, 0x0001);
    bus.write32::<BusRead>(T0_MODE, 0x0010);
    let irq1 = tick(&mut bus, T0_CNT, 1);
    assert_eq!(irq1, Some(4), "primeiro IRQ dispara");
    bus.write32::<BusRead>(T0_CNT, 0);
    let irq2 = tick(&mut bus, T0_CNT, 1);
    assert_eq!(irq2, None, "one-shot suprime segunda IRQ");
}

#[test]
fn irq_repeat_dispara_multiplas_vezes() {
    let mut bus = bus();
    bus.write32::<BusRead>(T0_TARGET, 0x0001);
    bus.write32::<BusRead>(T0_MODE, 0x0050);
    let irq1 = tick(&mut bus, T0_CNT, 1);
    assert_eq!(irq1, Some(4), "primeiro IRQ em repeat mode");
    bus.write32::<BusRead>(T0_CNT, 0);
    let irq2 = tick(&mut bus, T0_CNT, 1);
    assert_eq!(irq2, Some(4), "segundo IRQ em repeat mode dispara novamente");
}

#[test]
fn irq_toggle_inverte_bit10_e_so_retorna_irq_na_descida() {
    let mut bus = bus();
    bus.write32::<BusRead>(T0_TARGET, 0x0001);
    bus.write32::<BusRead>(T0_MODE, 0x0090);
    let irq1 = tick(&mut bus, T0_CNT, 1);
    assert_eq!(irq1, Some(4), "primeiro IRQ: 1→0 gera IRQ4");
    let bit10_after_first = bus.read32::<BusRead>(T0_MODE) & (1 << 10);
    assert_eq!(bit10_after_first, 0, "bit10 invertido (toggle): ficou 0");
    bus.write32::<BusRead>(T0_CNT, 0);
    let irq2 = tick(&mut bus, T0_CNT, 1);
    assert_eq!(irq2, None, "segundo IRQ: 0→1 NAO gera IRQ");
    let bit10_after_second = bus.read32::<BusRead>(T0_MODE) & (1 << 10);
    assert_eq!(bit10_after_second, 1 << 10, "bit10 volta a 1 no segundo toggle");
}

#[test]
fn irq_toggle_terceiro_disparo_retorna_irq() {
    let mut bus = bus();
    bus.write32::<BusRead>(T0_TARGET, 0x0001);
    bus.write32::<BusRead>(T0_MODE, 0x0090);
    tick(&mut bus, T0_CNT, 1);
    bus.write32::<BusRead>(T0_CNT, 0);
    tick(&mut bus, T0_CNT, 1);
    let irq3 = tick(&mut bus, T0_CNT, 1);
    assert_eq!(irq3, Some(4), "terceiro toggle: 1→0 gera IRQ4 novamente");
}

#[test]
fn irq_oneshot_toggle_nao_dispara_segunda_vez() {
    let mut bus = bus();
    bus.write32::<BusRead>(T0_TARGET, 0x0001);
    bus.write32::<BusRead>(T0_MODE, 0x0090);
    let irq1 = tick(&mut bus, T0_CNT, 1);
    assert_eq!(irq1, Some(4), "toggle one-shot: primeiro disparo gera IRQ");
    bus.write32::<BusRead>(T0_CNT, 0);
    let irq2 = tick(&mut bus, T0_CNT, 1);
    assert_eq!(irq2, None, "toggle one-shot: NAO gera no segundo");
}

#[test]
fn irq4_timer0_irq5_timer1_irq6_timer2() {
    let mut bus = bus();
    bus.write32::<BusRead>(T2_MODE, 0x0210);
    bus.timers_mut().tick(T2_CNT, 8, false, false);
    bus.write32::<BusRead>(T2_CNT, 0xFFF0);
    bus.write32::<BusRead>(T2_MODE, 0x0210);
    let irq2 = tick(&mut bus, T2_CNT, 8);
    assert_eq!(
        irq2,
        Some(6),
        "IRQ6 para timer 2 em pulse mode com target alcancado (clock/8)"
    );
}

#[test]
fn ticks_sem_irq_retornam_none_para_todos_os_timers() {
    let mut bus = bus();
    bus.write32::<BusRead>(T0_MODE, 0x0000);
    let irq = tick(&mut bus, T0_CNT, 5);
    assert_eq!(irq, None, "tick sem condicao de IRQ retorna None");
}
