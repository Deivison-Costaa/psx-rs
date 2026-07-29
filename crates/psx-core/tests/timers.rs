mod support;

use psx_core::bus::{Bus, BusRead};
use support::asm;

fn bus() -> Bus {
    asm::bus_with_bios_empty()
}

const T0_CNT: u32 = 0x1F80_1100;
const T0_MODE: u32 = 0x1F80_1104;
const T0_TARGET: u32 = 0x1F80_1108;
const T1_CNT: u32 = 0x1F80_1110;
const T2_CNT: u32 = 0x1F80_1120;
const T2_MODE: u32 = 0x1F80_1124;

#[test]
fn cnt_gravavel_e_legivel() {
    let mut bus = bus();
    bus.write32::<BusRead>(T0_CNT, 0x1234);
    let val = bus.read32::<BusRead>(T0_CNT);
    assert_eq!(val & 0xFFFF, 0x1234);
    assert_eq!(val >> 16, 0, "bits 16-31 sao garbage (leem zero)");
}

#[test]
fn mode_gravavel_e_legivel() {
    let mut bus = bus();
    bus.write32::<BusRead>(T0_MODE, 0xA5);
    let val = bus.read32::<BusRead>(T0_MODE);
    assert_eq!(val & 0x1FF, 0xA5, "bits 0-8 gravaveis");
    assert_eq!(
        val & 0x7C00,
        0,
        "bits 10-14 sao read-only flags do hardware"
    );
}

#[test]
fn target_gravavel_e_legivel() {
    let mut bus = bus();
    bus.write32::<BusRead>(T0_TARGET, 0xBEEF);
    let val = bus.read32::<BusRead>(T0_TARGET);
    assert_eq!(val & 0xFFFF, 0xBEEF);
}

#[test]
fn escrever_mode_reseta_cnt() {
    let mut bus = bus();
    bus.write32::<BusRead>(T0_CNT, 0x0042);
    bus.write32::<BusRead>(T0_MODE, 0x0100);
    let val = bus.read32::<BusRead>(T0_CNT);
    assert_eq!(val & 0xFFFF, 0, "CNT deve resetar ao escrever MODE");
}

#[test]
fn cnt_em_zero_no_reset() {
    let bus = bus();
    let val = bus.read32::<BusRead>(T0_CNT);
    assert_eq!(val & 0xFFFF, 0, "CNT comeca em zero");
}

#[test]
fn tres_timers_independentes() {
    let mut bus = bus();
    bus.write32::<BusRead>(T0_CNT, 0x0001);
    bus.write32::<BusRead>(T1_CNT, 0x0002);
    bus.write32::<BusRead>(T2_CNT, 0x0003);
    assert_eq!(bus.read32::<BusRead>(T0_CNT) & 0xFFFF, 0x0001);
    assert_eq!(bus.read32::<BusRead>(T1_CNT) & 0xFFFF, 0x0002);
    assert_eq!(bus.read32::<BusRead>(T2_CNT) & 0xFFFF, 0x0003);
}

#[test]
fn tick_incrementa_cnt_modo_system_clock() {
    let mut bus = bus();
    bus.write32::<BusRead>(T0_MODE, 0x0000);
    bus.timers_mut().tick(T0_CNT, 1);
    let val = bus.read32::<BusRead>(T0_CNT);
    assert_eq!(val & 0xFFFF, 1, "CNT incrementou de 0 para 1 apos 1 tick");
}

#[test]
fn timer2_modo_0_sync_ativo_para_contador() {
    let mut bus = bus();
    bus.write32::<BusRead>(T2_MODE, 0x0001);
    bus.write32::<BusRead>(T2_CNT, 0x000A);
    bus.timers_mut().tick(T2_CNT, 5);
    let val = bus.read32::<BusRead>(T2_CNT);
    assert_eq!(
        val & 0xFFFF,
        0x000A,
        "T2 sync mode 0 para o contador permanentemente"
    );
}

#[test]
fn cnt_wrap_em_ffff_sem_target() {
    let mut bus = bus();
    bus.write32::<BusRead>(T0_MODE, 0x0000);
    bus.write32::<BusRead>(T0_CNT, 0xFFFF);
    bus.timers_mut().tick(T0_CNT, 1);
    let val = bus.read32::<BusRead>(T0_CNT);
    assert_eq!(
        val & 0xFFFF,
        0,
        "CNT deve dar wrap para 0 ao passar de FFFFh"
    );
}

#[test]
fn cnt_reseta_no_target_com_bit3_setado() {
    let mut bus = bus();
    bus.write32::<BusRead>(T0_TARGET, 0x0003);
    bus.write32::<BusRead>(T0_MODE, 0x0008);
    bus.timers_mut().tick(T0_CNT, 3);
    let val = bus.read32::<BusRead>(T0_CNT);
    assert_eq!(val & 0xFFFF, 0, "CNT voltou a 0 apos atingir target=3");
}

#[test]
fn flag_target_alcancado_setado_e_limpo_na_leitura() {
    let mut bus = bus();
    bus.write32::<BusRead>(T0_TARGET, 0x0002);
    bus.write32::<BusRead>(T0_MODE, 0x0008);
    assert_eq!(
        bus.read32::<BusRead>(T0_MODE) & (1 << 11),
        0,
        "bit11 limpo antes do tick"
    );
    bus.timers_mut().tick(T0_CNT, 2);
    assert_eq!(
        bus.read32::<BusRead>(T0_MODE) & (1 << 11),
        1 << 11,
        "bit11 setado apos atingir target"
    );
    assert_eq!(
        bus.read32::<BusRead>(T0_MODE) & (1 << 11),
        0,
        "bit11 limpo apos leitura"
    );
}

#[test]
fn flag_ffff_alcancado_setado_e_limpo_na_leitura() {
    let mut bus = bus();
    bus.write32::<BusRead>(T0_MODE, 0x0000);
    bus.write32::<BusRead>(T0_CNT, 0xFFFE);
    bus.timers_mut().tick(T0_CNT, 2);
    let mode = bus.read32::<BusRead>(T0_MODE);
    assert_eq!(mode & (1 << 12), 1 << 12, "bit12 setado apos passar de FFFFh");
    assert_eq!(mode & (1 << 11), 0, "bit11 NAO setado — FFFFh nao e target");
    assert_eq!(
        bus.read32::<BusRead>(T0_MODE) & (1 << 12),
        0,
        "bit12 limpo apos leitura"
    );
}

#[test]
fn tick_respeita_divisor_de_clock_do_timer2() {
    let mut bus = bus();
    bus.write32::<BusRead>(T2_MODE, 0x0200);
    bus.timers_mut().tick(T2_CNT, 1);
    let val = bus.read32::<BusRead>(T2_CNT);
    assert_eq!(val & 0xFFFF, 0, "T2 clock/8: 1 tick nao incrementa");
    bus.timers_mut().tick(T2_CNT, 7);
    let val = bus.read32::<BusRead>(T2_CNT);
    assert_eq!(val & 0xFFFF, 1, "T2 clock/8: 8 ticks incrementam 1");
}
