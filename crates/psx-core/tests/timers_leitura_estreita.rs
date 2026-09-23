mod support;

use psx_core::bus::{Bus, BusRead, BusWrite};
use support::asm;

const T0_CNT: u32 = 0x1F80_1100;
const T0_MODE: u32 = 0x1F80_1104;
const T0_TARGET: u32 = 0x1F80_1108;
const T2_MODE: u32 = 0x1F80_1124;

const REACHED_TARGET: u32 = 1 << 11;
const REACHED_FFFF: u32 = 1 << 12;

fn bus_com_alvo_alcancado() -> Bus {
    let mut bus = asm::bus_with_bios_empty();
    bus.write32::<BusWrite>(T0_TARGET, 2);
    bus.write32::<BusWrite>(T0_MODE, 0x0008);
    bus.timers_mut().tick(T0_CNT, 8, false, false);
    bus
}

#[test]
fn lhu_no_modo_ve_e_limpa_o_bit_de_alvo() {
    let bus = bus_com_alvo_alcancado();

    let lido = u32::from(bus.read16::<BusRead>(T0_MODE));

    assert_eq!(
        lido & REACHED_TARGET,
        REACHED_TARGET,
        "lhu tem de ver o bit 11"
    );
    assert_eq!(
        bus.read32::<BusRead>(T0_MODE) & REACHED_TARGET,
        0,
        "o bit 11 e 'Reset after Reading': o lhu anterior ja o zerou"
    );
}

#[test]
fn lbu_no_byte_alto_do_modo_ve_e_limpa_o_bit_de_alvo() {
    let bus = bus_com_alvo_alcancado();

    let byte_alto = u32::from(bus.read8::<BusRead>(T0_MODE + 1));

    assert_eq!(byte_alto & (REACHED_TARGET >> 8), REACHED_TARGET >> 8);
    assert_eq!(bus.read32::<BusRead>(T0_MODE) & REACHED_TARGET, 0);
}

#[test]
fn lhu_devolve_os_dois_bytes_do_mesmo_instante() {
    let bus = bus_com_alvo_alcancado();

    assert_eq!(
        bus.read16::<BusRead>(T0_MODE) & 0x1FFF,
        0x0C08,
        "bit3 escrito, bit10 setado pela escrita, bit11 do alvo: os dois bytes antes da limpeza"
    );
}

#[test]
fn lhu_no_modo_limpa_o_bit_de_ffff() {
    let mut bus = asm::bus_with_bios_empty();
    bus.write32::<BusWrite>(T2_MODE, 0);
    bus.timers_mut().tick(0x1F80_1120, 0x1_0010, false, false);

    let lido = u32::from(bus.read16::<BusRead>(T2_MODE));

    assert_eq!(lido & REACHED_FFFF, REACHED_FFFF, "o contador deu a volta");
    assert_eq!(
        u32::from(bus.read16::<BusRead>(T2_MODE)) & REACHED_FFFF,
        0,
        "sem a limpeza, quem mede tempo somando 0xFFFF por volta conta uma volta fantasma"
    );
}
