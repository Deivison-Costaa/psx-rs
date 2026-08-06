mod support;

use psx_core::bus::{Bus, BusRead};
use support::asm;

const D2_MADR: u32 = 0x1F80_10A0;
const D2_BCR: u32 = 0x1F80_10A4;
const D2_CHCR: u32 = 0x1F80_10A8;
const D6_MADR: u32 = 0x1F80_10E0;
const D6_BCR: u32 = 0x1F80_10E4;
const D6_CHCR: u32 = 0x1F80_10E8;
const DPCR: u32 = 0x1F80_10F0;
const DICR: u32 = 0x1F80_10F4;
const DICR_BUS_ERROR_AND_MASTER: u32 = 0x8000_8000;

fn bus_com_dma() -> Bus {
    asm::bus_with_bios_empty()
}

#[test]
fn dma6_otc_fora_da_ram_levanta_bus_error_no_dicr() {
    let mut bus = bus_com_dma();
    bus.write32::<BusRead>(D6_MADR, 0);
    bus.write32::<BusRead>(D6_BCR, 2);
    bus.write32::<BusRead>(DPCR, 0x0765_4321 | (1 << 27));
    bus.write32::<BusRead>(D6_CHCR, 0x1100_0002);

    let dicr = bus.read32::<BusRead>(DICR);
    assert_eq!(
        dicr & DICR_BUS_ERROR_AND_MASTER,
        DICR_BUS_ERROR_AND_MASTER,
        "OTC fora da RAM deve levantar DICR bit15 e derivar bit31"
    );
}

#[test]
fn dma2_burst_fora_da_ram_levanta_bus_error_no_dicr() {
    let mut bus = bus_com_dma();
    bus.write32::<BusRead>(D2_MADR, 0);
    bus.write32::<BusRead>(D2_BCR, 2);
    bus.write32::<BusRead>(DPCR, 0x0765_4321 | (1 << 11));
    bus.write32::<BusRead>(D2_CHCR, 0x1100_0003);

    let dicr = bus.read32::<BusRead>(DICR);
    assert_eq!(
        dicr & DICR_BUS_ERROR_AND_MASTER,
        DICR_BUS_ERROR_AND_MASTER,
        "DMA2 burst fora da RAM deve levantar DICR bit15 e derivar bit31"
    );
}
