mod support;

use psx_core::bus::{Bus, BusRead};
use support::asm;

fn bus() -> Bus {
    asm::bus_with_bios_empty()
}

const T0_CNT: u32 = 0x1F80_1100;
const T0_MODE: u32 = 0x1F80_1104;
const T0_TARGET: u32 = 0x1F80_1108;

// Achado legado 10.52 (0118): bus.read16/read8 no registrador MODE do timer nao
// disparavam o "clear on read" dos bits 11/12 (target/FFFFh alcancado) — so
// bus.read32 (Timers::read32) tinha o efeito colateral; region_read_byte usava
// Timers::peek32 (sem efeito) tanto pro caminho de byte quanto, via duas
// chamadas independentes, pro de halfword.

#[test]
fn flag_target_alcancado_e_limpo_por_read16() {
    let mut bus = bus();
    bus.write32::<BusRead>(T0_TARGET, 0x0002);
    bus.write32::<BusRead>(T0_MODE, 0x0008);
    bus.timers_mut().tick(T0_CNT, 2, false, false);

    assert_eq!(
        bus.read16::<BusRead>(T0_MODE) & (1 << 11),
        1 << 11,
        "lhu no MODE deve ver o bit11 (target alcancado) setado"
    );
    assert_eq!(
        bus.read32::<BusRead>(T0_MODE) & (1 << 11),
        0,
        "o lhu anterior ja devia ter limpo o bit11 — read32 seguinte ve zero"
    );
}

#[test]
fn flag_target_alcancado_e_limpo_por_read8() {
    let mut bus = bus();
    bus.write32::<BusRead>(T0_TARGET, 0x0002);
    bus.write32::<BusRead>(T0_MODE, 0x0008);
    bus.timers_mut().tick(T0_CNT, 2, false, false);

    // bit 11 do MODE fica no segundo byte do registrador (bits 8-15), como bit3
    // local desse byte.
    let byte_alto = bus.read8::<BusRead>(T0_MODE + 1);
    assert_eq!(
        byte_alto & (1 << 3),
        1 << 3,
        "lbu no byte alto do MODE deve ver o bit11 (bit3 local) setado"
    );
    assert_eq!(
        bus.read32::<BusRead>(T0_MODE) & (1 << 11),
        0,
        "o lbu anterior ja devia ter limpo o bit11 — read32 seguinte ve zero"
    );
}

#[test]
fn read16_no_meio_da_transferencia_nao_corrompe_o_segundo_byte() {
    // Garante que o fix nao troca "sempre zero" por "so o primeiro dos dois
    // bytes compostos ve o valor certo": os DOIS bytes do halfword lido por
    // read16 tem que refletir o MESMO instantaneo pre-clear do MODE, nao um
    // par onde o segundo ja saiu limpo pelo primeiro.
    let mut bus = bus();
    bus.write32::<BusRead>(T0_TARGET, 0x0002);
    bus.write32::<BusRead>(T0_MODE, 0x0008);
    bus.timers_mut().tick(T0_CNT, 2, false, false);

    let half = bus.read16::<BusRead>(T0_MODE);
    assert_eq!(
        half, 0x0C08,
        "MODE=0x0008 (bit3, escrito) | 0x0400 (bit10, setado pela propria escrita \
         em MODE) | 0x0800 (bit11, target alcancado) = 0x0C08 — os dois bytes do \
         read16 tem que vir do MESMO valor pre-clear, nao um par inconsistente \
         onde o segundo byte ja saiu zerado"
    );
}
