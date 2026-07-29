mod support;

use psx_core::bus::{Bus, BusRead};
use support::asm;

fn bus() -> Bus {
    asm::bus_with_bios_empty()
}

const T0_CNT: u32 = 0x1F80_1100;
const T0_MODE: u32 = 0x1F80_1104;
const T1_CNT: u32 = 0x1F80_1110;
const T1_MODE: u32 = 0x1F80_1114;
const T2_CNT: u32 = 0x1F80_1120;
const T2_MODE: u32 = 0x1F80_1124;

fn tick_timer(bus: &mut Bus, base: u32, cycles: u32) {
    let hb = bus.gpu().hblank_active();
    let vb = bus.gpu().vblank_active();
    bus.timers_mut().tick(base, cycles, hb, vb);
}

#[test]
fn timer0_dotclock_256px_razao_11_por_70_cpu_cycles() {
    let mut bus = bus();
    bus.timers_mut().update_gpu_timing(10, 3413);
    bus.write32::<BusRead>(T0_MODE, 0x0100);
    tick_timer(&mut bus, T0_CNT, 200);
    assert_eq!(
        bus.read32::<BusRead>(T0_CNT) & 0xFFFF,
        31,
        "200 CPU cycles a 11/70 dot/pulse = 31 pulses (2200/70=31, resto=30)"
    );
    tick_timer(&mut bus, T0_CNT, 100);
    assert_eq!(
        bus.read32::<BusRead>(T0_CNT) & 0xFFFF,
        76,
        "acumulado: resto=30 + 100*11=1100 → 1100+30*70=3200/70=45 → CNT=31+45=76"
    );
}

#[test]
fn timer0_dotclock_320px_razao_11_por_56_cpu_cycles() {
    let mut bus = bus();
    bus.timers_mut().update_gpu_timing(8, 3413);
    bus.write32::<BusRead>(T0_MODE, 0x0100);
    tick_timer(&mut bus, T0_CNT, 200);
    assert_eq!(
        bus.read32::<BusRead>(T0_CNT) & 0xFFFF,
        39,
        "200 CPU cycles a 11/56 dot/pulse = 39 pulses (2200/56=39, resto=16)"
    );
}

#[test]
fn timer1_hblank_ntsc_razao_11_por_23891_cpu_cycles() {
    let mut bus = bus();
    bus.timers_mut().update_gpu_timing(10, 3413);
    bus.write32::<BusRead>(T1_MODE, 0x0100);
    tick_timer(&mut bus, T1_CNT, 5000);
    assert_eq!(
        bus.read32::<BusRead>(T1_CNT) & 0xFFFF,
        2,
        "5000 CPU cycles a 11/23891 hblank/pulse = 2 pulsos (55000/23891=2, resto=7218)"
    );
    tick_timer(&mut bus, T1_CNT, 10000);
    assert_eq!(
        bus.read32::<BusRead>(T1_CNT) & 0xFFFF,
        7224,
        "acumulado: 2 + floor((7218*23891+10000*11)/23891) = 2+7222=7224"
    );
}

#[test]
fn timer1_hblank_pal_razao_11_por_23842_cpu_cycles() {
    let mut bus = bus();
    bus.timers_mut().update_gpu_timing(10, 3406);
    bus.write32::<BusRead>(T1_MODE, 0x0100);
    tick_timer(&mut bus, T1_CNT, 5000);
    assert_eq!(
        bus.read32::<BusRead>(T1_CNT) & 0xFFFF,
        2,
        "5000 CPU cycles a 11/23842 hblank/pulse = 2 pulsos (55000/23842=2, resto=7316)"
    );
}

#[test]
fn timer0_clock_src_0_system_clock_continua_funcionando() {
    let mut bus = bus();
    bus.timers_mut().update_gpu_timing(10, 3413);
    bus.write32::<BusRead>(T0_MODE, 0x0000);
    tick_timer(&mut bus, T0_CNT, 7);
    assert_eq!(
        bus.read32::<BusRead>(T0_CNT) & 0xFFFF,
        7,
        "clock_src=0 (system clock): 1 CPU cycle = 1 incremento"
    );
}

#[test]
fn timer0_dotclock_com_sync_mode0_pausa_durante_hblank() {
    let mut bus = bus();
    bus.timers_mut().update_gpu_timing(10, 3413);
    bus.write32::<BusRead>(T0_MODE, 0x0101);
    bus.gpu_mut().set_hblank_active(true);
    tick_timer(&mut bus, T0_CNT, 70);
    assert_eq!(
        bus.read32::<BusRead>(T0_CNT) & 0xFFFF,
        0,
        "dotclock + sync mode 0: pausado durante Hblank"
    );
    bus.gpu_mut().set_hblank_active(false);
    tick_timer(&mut bus, T0_CNT, 70);
    assert_eq!(
        bus.read32::<BusRead>(T0_CNT) & 0xFFFF,
        11,
        "dotclock + sync mode 0: 70 ciclos fora do Hblank = 11 pulsos"
    );
}

#[test]
fn timer1_hblank_com_sync_mode0_pausa_durante_vblank() {
    let mut bus = bus();
    bus.timers_mut().update_gpu_timing(10, 3413);
    bus.write32::<BusRead>(T1_MODE, 0x0101);
    bus.gpu_mut().enter_vblank();
    tick_timer(&mut bus, T1_CNT, 30000);
    assert_eq!(
        bus.read32::<BusRead>(T1_CNT) & 0xFFFF,
        0,
        "hblank source + sync mode 0: pausado durante Vblank"
    );
    bus.gpu_mut().exit_vblank();
    tick_timer(&mut bus, T1_CNT, 30000);
    let cnt = bus.read32::<BusRead>(T1_CNT) & 0xFFFF;
    assert!(
        cnt > 0,
        "hblank source + sync mode 0: incrementa fora do Vblank, CNT={}",
        cnt
    );
}

#[test]
fn timer2_clock_div_8_continua_funcionando() {
    let mut bus = bus();
    bus.timers_mut().update_gpu_timing(10, 3413);
    bus.write32::<BusRead>(T2_MODE, 0x0200);
    tick_timer(&mut bus, T2_CNT, 1);
    assert_eq!(
        bus.read32::<BusRead>(T2_CNT) & 0xFFFF,
        0,
        "T2 clock/8: 1 tick nao incrementa"
    );
    tick_timer(&mut bus, T2_CNT, 7);
    assert_eq!(
        bus.read32::<BusRead>(T2_CNT) & 0xFFFF,
        1,
        "T2 clock/8: 8 ticks incrementam 1"
    );
}

#[test]
fn escrever_mode_reseta_acumulador_fractional_de_clock() {
    let mut bus = bus();
    bus.timers_mut().update_gpu_timing(10, 3413);
    bus.write32::<BusRead>(T0_MODE, 0x0100);
    tick_timer(&mut bus, T0_CNT, 30);
    assert_eq!(
        bus.read32::<BusRead>(T0_CNT) & 0xFFFF,
        4,
        "30*11=330/70=4, resto=50"
    );
    bus.write32::<BusRead>(T0_MODE, 0x0100);
    tick_timer(&mut bus, T0_CNT, 70);
    assert_eq!(
        bus.read32::<BusRead>(T0_CNT) & 0xFFFF,
        11,
        "apos re-escrever MODE, cycle_acc resetado: 70*11/70 = 11 pulsos"
    );
}
