mod support;

use psx_core::bus::BusRead;
use support::asm;

const D6_MADR: u32 = 0x1F80_10E0;
const D6_BCR: u32 = 0x1F80_10E4;
const D6_CHCR: u32 = 0x1F80_10E8;
const DPCR: u32 = 0x1F80_10F0;

#[test]
fn otc_armado_antes_do_dpcr_e_reexecutado_ao_habilitar_canal() {
    let mut bus = asm::bus_with_bios_empty();
    let base = 0x0000_1000;
    let sentinel = 0xCAFE_BABE;

    bus.write32::<BusRead>(base, sentinel);
    bus.write32::<BusRead>(D6_MADR, base);
    bus.write32::<BusRead>(D6_BCR, 1);
    bus.write32::<BusRead>(D6_CHCR, 0x1100_0002);

    assert_eq!(bus.read32::<BusRead>(base), sentinel);

    let dpcr = bus.read32::<BusRead>(DPCR);
    bus.write32::<BusRead>(DPCR, dpcr | (1 << 27));

    assert_eq!(
        bus.read32::<BusRead>(base),
        0x00FF_FFFF,
        "OTC pendente deve executar quando o DPCR habilita o canal 6"
    );
}
