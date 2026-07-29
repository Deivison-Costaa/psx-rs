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

fn set_hb(bus: &mut Bus, active: bool) {
    bus.gpu_mut().set_hblank_active(active);
}

fn set_vb(bus: &mut Bus, active: bool) {
    if active {
        bus.gpu_mut().enter_vblank();
    } else {
        bus.gpu_mut().exit_vblank();
    }
}

#[test]
fn timer0_sync_mode0_pausa_durante_hblank() {
    let mut bus = bus();
    bus.write32::<BusRead>(T0_MODE, 0x0001);
    set_hb(&mut bus, true);
    tick_timer(&mut bus, T0_CNT, 5);
    assert_eq!(
        bus.read32::<BusRead>(T0_CNT) & 0xFFFF,
        0,
        "CNT nao incrementa durante Hblank no modo 0"
    );
    set_hb(&mut bus, false);
    tick_timer(&mut bus, T0_CNT, 5);
    assert_eq!(
        bus.read32::<BusRead>(T0_CNT) & 0xFFFF,
        5,
        "CNT incrementa fora do Hblank no modo 0"
    );
}

#[test]
fn timer0_sync_mode1_reseta_no_hblank() {
    let mut bus = bus();
    bus.write32::<BusRead>(T0_MODE, 0x0003);
    set_hb(&mut bus, false);
    tick_timer(&mut bus, T0_CNT, 10);
    assert_eq!(
        bus.read32::<BusRead>(T0_CNT) & 0xFFFF,
        10,
        "CNT incrementa livremente no modo 1"
    );
    set_hb(&mut bus, true);
    tick_timer(&mut bus, T0_CNT, 5);
    assert_eq!(
        bus.read32::<BusRead>(T0_CNT) & 0xFFFF,
        5,
        "CNT reseta na borda de subida do Hblank e incrementa durante"
    );
}

#[test]
fn timer0_sync_mode2_reseta_no_hblank_e_pausa_fora() {
    let mut bus = bus();
    bus.write32::<BusRead>(T0_MODE, 0x0005);
    set_hb(&mut bus, false);
    tick_timer(&mut bus, T0_CNT, 10);
    assert_eq!(
        bus.read32::<BusRead>(T0_CNT) & 0xFFFF,
        0,
        "CNT pausado fora do Hblank no modo 2"
    );
    set_hb(&mut bus, true);
    tick_timer(&mut bus, T0_CNT, 5);
    assert_eq!(
        bus.read32::<BusRead>(T0_CNT) & 0xFFFF,
        5,
        "CNT reseta na borda e incrementa durante Hblank no modo 2"
    );
}

#[test]
fn timer0_sync_mode3_espera_primeiro_hblank_depois_free_run() {
    let mut bus = bus();
    bus.write32::<BusRead>(T0_MODE, 0x0007);
    set_hb(&mut bus, false);
    tick_timer(&mut bus, T0_CNT, 10);
    assert_eq!(
        bus.read32::<BusRead>(T0_CNT) & 0xFFFF,
        0,
        "CNT pausado ate o primeiro Hblank no modo 3"
    );
    set_hb(&mut bus, true);
    tick_timer(&mut bus, T0_CNT, 3);
    assert_eq!(
        bus.read32::<BusRead>(T0_CNT) & 0xFFFF,
        3,
        "CNT incrementa durante o primeiro Hblank (reset na borda)"
    );
    set_hb(&mut bus, false);
    tick_timer(&mut bus, T0_CNT, 2);
    assert_eq!(
        bus.read32::<BusRead>(T0_CNT) & 0xFFFF,
        5,
        "CNT continua incrementando fora do Hblank apos modo 3 disparado"
    );
    set_hb(&mut bus, true);
    tick_timer(&mut bus, T0_CNT, 3);
    assert_eq!(
        bus.read32::<BusRead>(T0_CNT) & 0xFFFF,
        8,
        "CNT NAO reseta na segunda borda de Hblank apos modo 3 disparado"
    );
}

#[test]
fn timer1_sync_mode0_pausa_durante_vblank() {
    let mut bus = bus();
    bus.write32::<BusRead>(T1_MODE, 0x0001);
    set_vb(&mut bus, true);
    tick_timer(&mut bus, T1_CNT, 5);
    assert_eq!(
        bus.read32::<BusRead>(T1_CNT) & 0xFFFF,
        0,
        "CNT nao incrementa durante Vblank no modo 0"
    );
    set_vb(&mut bus, false);
    tick_timer(&mut bus, T1_CNT, 5);
    assert_eq!(
        bus.read32::<BusRead>(T1_CNT) & 0xFFFF,
        5,
        "CNT incrementa fora do Vblank no modo 0"
    );
}

#[test]
fn timer1_sync_mode1_reseta_no_vblank() {
    let mut bus = bus();
    bus.write32::<BusRead>(T1_MODE, 0x0003);
    set_vb(&mut bus, false);
    tick_timer(&mut bus, T1_CNT, 10);
    assert_eq!(
        bus.read32::<BusRead>(T1_CNT) & 0xFFFF,
        10,
        "CNT incrementa livremente no modo 1"
    );
    set_vb(&mut bus, true);
    tick_timer(&mut bus, T1_CNT, 5);
    assert_eq!(
        bus.read32::<BusRead>(T1_CNT) & 0xFFFF,
        5,
        "CNT reseta na borda de subida do Vblank e incrementa durante"
    );
}

#[test]
fn timer1_sync_mode2_reseta_no_vblank_e_pausa_fora() {
    let mut bus = bus();
    bus.write32::<BusRead>(T1_MODE, 0x0005);
    set_vb(&mut bus, false);
    tick_timer(&mut bus, T1_CNT, 10);
    assert_eq!(
        bus.read32::<BusRead>(T1_CNT) & 0xFFFF,
        0,
        "CNT pausado fora do Vblank no modo 2"
    );
    set_vb(&mut bus, true);
    tick_timer(&mut bus, T1_CNT, 5);
    assert_eq!(
        bus.read32::<BusRead>(T1_CNT) & 0xFFFF,
        5,
        "CNT reseta na borda e incrementa durante Vblank no modo 2"
    );
}

#[test]
fn timer1_sync_mode3_espera_primeiro_vblank_depois_free_run() {
    let mut bus = bus();
    bus.write32::<BusRead>(T1_MODE, 0x0007);
    set_vb(&mut bus, false);
    tick_timer(&mut bus, T1_CNT, 10);
    assert_eq!(
        bus.read32::<BusRead>(T1_CNT) & 0xFFFF,
        0,
        "CNT pausado ate o primeiro Vblank no modo 3"
    );
    set_vb(&mut bus, true);
    tick_timer(&mut bus, T1_CNT, 4);
    assert_eq!(
        bus.read32::<BusRead>(T1_CNT) & 0xFFFF,
        4,
        "CNT incrementa durante o primeiro Vblank (reset na borda)"
    );
    set_vb(&mut bus, false);
    tick_timer(&mut bus, T1_CNT, 3);
    assert_eq!(
        bus.read32::<BusRead>(T1_CNT) & 0xFFFF,
        7,
        "CNT continua incrementando fora do Vblank apos modo 3 disparado"
    );
    set_vb(&mut bus, true);
    tick_timer(&mut bus, T1_CNT, 2);
    assert_eq!(
        bus.read32::<BusRead>(T1_CNT) & 0xFFFF,
        9,
        "CNT NAO reseta na segunda borda de Vblank apos modo 3 disparado"
    );
}

#[test]
fn timer2_modo_1_e_2_sao_free_run() {
    let mut bus = bus();
    bus.write32::<BusRead>(T2_MODE, 0x0003);
    bus.write32::<BusRead>(T2_CNT, 0x0005);
    tick_timer(&mut bus, T2_CNT, 3);
    assert_eq!(
        bus.read32::<BusRead>(T2_CNT) & 0xFFFF,
        8,
        "T2 sync mode 1 incrementa livremente"
    );
    bus.write32::<BusRead>(T2_MODE, 0x0005);
    bus.write32::<BusRead>(T2_CNT, 0x0005);
    tick_timer(&mut bus, T2_CNT, 3);
    assert_eq!(
        bus.read32::<BusRead>(T2_CNT) & 0xFFFF,
        8,
        "T2 sync mode 2 incrementa livremente"
    );
}

#[test]
fn escrever_mode_reseta_estado_de_sync() {
    let mut bus = bus();
    bus.write32::<BusRead>(T0_MODE, 0x0003);
    set_hb(&mut bus, true);
    tick_timer(&mut bus, T0_CNT, 5);
    assert_eq!(
        bus.read32::<BusRead>(T0_CNT) & 0xFFFF,
        5,
        "CNT incrementou apos reset na primeira borda"
    );
    bus.write32::<BusRead>(T0_MODE, 0x0003);
    tick_timer(&mut bus, T0_CNT, 3);
    assert_eq!(
        bus.read32::<BusRead>(T0_CNT) & 0xFFFF,
        3,
        "CNT resetou na borda ao re-escrever MODE com Hblank mantido ativo"
    );
}

#[test]
fn sync_disabled_free_run_independente_do_modo() {
    let mut bus = bus();
    bus.write32::<BusRead>(T0_MODE, 0x0000);
    set_hb(&mut bus, true);
    tick_timer(&mut bus, T0_CNT, 3);
    assert_eq!(
        bus.read32::<BusRead>(T0_CNT) & 0xFFFF,
        3,
        "sync_enable=0 incrementa mesmo com Hblank ativo"
    );
    set_hb(&mut bus, false);
    tick_timer(&mut bus, T0_CNT, 2);
    assert_eq!(
        bus.read32::<BusRead>(T0_CNT) & 0xFFFF,
        5,
        "sync_enable=0 continua incrementando"
    );
}
