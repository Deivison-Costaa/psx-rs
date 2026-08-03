mod support;

use psx_core::bus::{Bus, BusRead};
use support::asm;

// dma/chain-looping (ps1-tests, tests/exes/ps1-tests/dma/chain-looping/psx.log):
// cadeia auto-referente nunca alcanca um end-marker; hardware real reporta
// "finished = false, irq = false" apos a janela de trabalho da CPU, ou seja, o
// canal continua ocupado para sempre. Valores vem do gabarito, nao do emulador.

fn bus_com_dma() -> Bus {
    asm::bus_with_bios_empty()
}

const D2_MADR: u32 = 0x1F80_10A0;
const D2_BCR: u32 = 0x1F80_10A4;
const D2_CHCR: u32 = 0x1F80_10A8;
const DPCR: u32 = 0x1F80_10F0;
const DICR: u32 = 0x1F80_10F4;

fn write_ram32(bus: &mut Bus, addr: u32, val: u32) {
    bus.write32::<BusRead>(addr, val);
}

fn habilitar_canal2(bus: &mut Bus) {
    let dpcr = bus.read32::<BusRead>(DPCR);
    bus.write32::<BusRead>(DPCR, dpcr | (1 << 11));
}

#[test]
fn dma2_linked_list_auto_referente_nunca_completa() {
    let mut bus = bus_com_dma();
    habilitar_canal2(&mut bus);

    let list_addr: u32 = 0x0000_0100;
    // no aponta para si mesmo, 0 palavras extras: nunca alcanca um end-marker.
    let header = list_addr & 0x00FF_FFFF;
    write_ram32(&mut bus, list_addr, header);

    bus.write32::<BusRead>(D2_MADR, list_addr);
    bus.write32::<BusRead>(D2_BCR, 0);
    bus.write32::<BusRead>(D2_CHCR, 0x1100_0401);

    let chcr = bus.read32::<BusRead>(D2_CHCR);
    assert_eq!(
        chcr & (1 << 24),
        1 << 24,
        "cadeia auto-referente nunca alcanca end-marker: bit24 (start/busy) \
         deve permanecer setado (psx.log: finished = false)"
    );

    let dicr = bus.read32::<BusRead>(DICR);
    assert_eq!(
        dicr & (1 << 26),
        0,
        "sem end-marker a transferencia nunca completa: flag de IRQ do canal 2 \
         (bit 26) nao deve ser setada (psx.log: irq = false)"
    );
}

#[test]
fn dma2_linked_list_ciclo_de_dois_nos_nunca_completa() {
    let mut bus = bus_com_dma();
    habilitar_canal2(&mut bus);

    let no_a: u32 = 0x0000_0100;
    let no_b: u32 = 0x0000_0200;
    // A aponta para B, B aponta de volta para A: ciclo sem end-marker.
    write_ram32(&mut bus, no_a, no_b & 0x00FF_FFFF);
    write_ram32(&mut bus, no_b, no_a & 0x00FF_FFFF);

    bus.write32::<BusRead>(D2_MADR, no_a);
    bus.write32::<BusRead>(D2_BCR, 0);
    bus.write32::<BusRead>(D2_CHCR, 0x1100_0401);

    let chcr = bus.read32::<BusRead>(D2_CHCR);
    assert_eq!(
        chcr & (1 << 24),
        1 << 24,
        "ciclo de dois nos nunca alcanca end-marker: bit24 deve permanecer setado"
    );
}
