mod support;

use psx_core::bus::{Bus, BusRead};
use support::asm;

fn bus_com_dma() -> Bus {
    asm::bus_with_bios_empty()
}

const D2_MADR: u32 = 0x1F80_10A0;
const D2_BCR: u32 = 0x1F80_10A4;
const D2_CHCR: u32 = 0x1F80_10A8;
const GPUSTAT: u32 = 0x1F80_1814;

fn write_ram32(bus: &mut Bus, addr: u32, val: u32) {
    bus.write32::<BusRead>(addr, val);
}

#[test]
fn dma2_madr_gravavel_e_legivel() {
    let mut bus = bus_com_dma();
    bus.write32::<BusRead>(D2_MADR, 0x0012_3456);
    let val = bus.read32::<BusRead>(D2_MADR);
    assert_eq!(val & 0x00FF_FFFF, 0x0012_3456);
}

#[test]
fn dma2_bcr_gravavel_e_legivel() {
    let mut bus = bus_com_dma();
    bus.write32::<BusRead>(D2_BCR, 0x0042);
    let val = bus.read32::<BusRead>(D2_BCR);
    assert_eq!(val, 0x0042);
}

#[test]
fn dma2_chcr_gravavel_e_legivel() {
    let mut bus = bus_com_dma();
    bus.write32::<BusRead>(D2_CHCR, 0x0100_0001);
    let val = bus.read32::<BusRead>(D2_CHCR);
    assert_eq!(val, 0x0100_0001, "canal 2 CHCR sem restricao de bits");
}

#[test]
fn dma2_linked_list_transfere_comandos_para_gp0() {
    let mut bus = bus_com_dma();

    let list_addr: u32 = 0x0000_0100;
    let header = 0x00FF_FFFF | (3 << 24);
    write_ram32(&mut bus, list_addr, header);
    write_ram32(&mut bus, list_addr + 4, 0xE100_001F);
    write_ram32(&mut bus, list_addr + 8, 0xE600_0000);
    write_ram32(&mut bus, list_addr + 12, 0xE200_0000);

    bus.write32::<BusRead>(D2_MADR, list_addr);
    bus.write32::<BusRead>(D2_BCR, 0);
    bus.write32::<BusRead>(D2_CHCR, 0x1100_0401);

    let gpustat = bus.read32::<BusRead>(GPUSTAT);
    assert_eq!(
        gpustat & 0x7FF,
        0x1F,
        "bits 0-10 do GPUSTAT ajustados pelo comando E1 via linked-list DMA"
    );
}

#[test]
fn dma2_linked_list_end_marker_para_transferencia() {
    let mut bus = bus_com_dma();

    let list_addr: u32 = 0x0000_0100;
    let header = 0x00FF_FFFF | (1 << 24);
    write_ram32(&mut bus, list_addr, header);
    write_ram32(&mut bus, list_addr + 4, 0xE100_001F);

    let ghost_addr: u32 = 0x0000_0200;
    let ghost_header = 0x00FF_FFFF | (1 << 24);
    write_ram32(&mut bus, ghost_addr, ghost_header);
    write_ram32(&mut bus, ghost_addr + 4, 0xE100_003F);

    bus.write32::<BusRead>(D2_MADR, list_addr);
    bus.write32::<BusRead>(D2_BCR, 0);
    bus.write32::<BusRead>(D2_CHCR, 0x1100_0401);

    let gpustat = bus.read32::<BusRead>(GPUSTAT);
    assert_eq!(
        gpustat & 0x7FF,
        0x1F,
        "end-marker 0x00FFFFFF parou a transferencia; segundo no nao executou"
    );
}

#[test]
fn dma2_linked_list_bit24_nao_dispara_sem_start() {
    let mut bus = bus_com_dma();

    let list_addr: u32 = 0x0000_0100;
    let header = 0x00FF_FFFF | (1 << 24);
    write_ram32(&mut bus, list_addr, header);
    write_ram32(&mut bus, list_addr + 4, 0xE100_001F);

    bus.write32::<BusRead>(D2_MADR, list_addr);
    bus.write32::<BusRead>(D2_BCR, 0);
    bus.write32::<BusRead>(D2_CHCR, 0x0000_0401);

    let gpustat = bus.read32::<BusRead>(GPUSTAT);
    assert_eq!(
        gpustat & 0x1F,
        0,
        "bit 24=0: DMA nao disparou, GPUSTAT bits 0-4 inalterados"
    );
}

#[test]
fn dma2_chcr_bit24_e_limpo_apos_linked_list() {
    let mut bus = bus_com_dma();

    let list_addr: u32 = 0x0000_0100;
    let header = 0x00FF_FFFF | (1 << 24);
    write_ram32(&mut bus, list_addr, header);
    write_ram32(&mut bus, list_addr + 4, 0xE100_001F);

    bus.write32::<BusRead>(D2_MADR, list_addr);
    bus.write32::<BusRead>(D2_BCR, 0);
    bus.write32::<BusRead>(D2_CHCR, 0x1100_0401);

    let chcr = bus.read32::<BusRead>(D2_CHCR);
    assert_eq!(
        chcr & (1 << 24),
        0,
        "bit 24 deve ser limpo apos transferencia linked-list"
    );
}

#[test]
fn dma2_block_transfere_dados_para_vram() {
    let mut bus = bus_com_dma();

    bus.write32::<BusRead>(0x1F80_1810, 0xA000_0000);
    bus.write32::<BusRead>(0x1F80_1810, 0x0000_0000);
    bus.write32::<BusRead>(0x1F80_1810, 0x0001_0001);

    let data_addr: u32 = 0x0000_0100;
    write_ram32(&mut bus, data_addr, 0xABAB_CDCD);

    bus.write32::<BusRead>(D2_MADR, data_addr);
    bus.write32::<BusRead>(D2_BCR, 0x0001_0001);
    bus.write32::<BusRead>(D2_CHCR, 0x1100_0201);

    let gpustat = bus.read32::<BusRead>(GPUSTAT);
    assert_eq!(
        gpustat & (1 << 26),
        1 << 26,
        "bit 26 (Ready) setado apos transferencia block de 1 palavra"
    );
    let pixel = bus.gpu().vram_pixel(0, 0);
    assert_eq!(
        pixel, 0xCDCD,
        "VRAM (0,0) contem metade inferior de ABABCDCD"
    );
}

#[test]
fn dma2_block_bit24_nao_dispara_sem_start() {
    let mut bus = bus_com_dma();

    bus.write32::<BusRead>(0x1F80_1810, 0xA000_0000);
    bus.write32::<BusRead>(0x1F80_1810, 0x0000_0000);
    bus.write32::<BusRead>(0x1F80_1810, 0x0001_0001);

    let data_addr: u32 = 0x0000_0100;
    write_ram32(&mut bus, data_addr, 0xABAB_CDCD);

    bus.write32::<BusRead>(D2_MADR, data_addr);
    bus.write32::<BusRead>(D2_BCR, 0x0001_0001);
    bus.write32::<BusRead>(D2_CHCR, 0x0000_0201);

    let gpustat = bus.read32::<BusRead>(GPUSTAT);
    assert_eq!(
        gpustat & (1 << 26),
        0,
        "bit 24=0: DMA block nao disparou, bit Ready nao setado"
    );
}

#[test]
fn dma2_chcr_bit24_e_limpo_apos_block() {
    let mut bus = bus_com_dma();

    bus.write32::<BusRead>(0x1F80_1810, 0xA000_0000);
    bus.write32::<BusRead>(0x1F80_1810, 0x0000_0000);
    bus.write32::<BusRead>(0x1F80_1810, 0x0001_0001);

    let data_addr: u32 = 0x0000_0100;
    write_ram32(&mut bus, data_addr, 0xABAB_CDCD);

    bus.write32::<BusRead>(D2_MADR, data_addr);
    bus.write32::<BusRead>(D2_BCR, 0x0001_0001);
    bus.write32::<BusRead>(D2_CHCR, 0x1100_0201);

    let chcr = bus.read32::<BusRead>(D2_CHCR);
    assert_eq!(
        chcr & (1 << 24),
        0,
        "bit 24 deve ser limpo apos transferencia block"
    );
}

#[test]
fn dma2_block_multiplas_palavras() {
    let mut bus = bus_com_dma();

    bus.write32::<BusRead>(0x1F80_1810, 0xA000_0000);
    bus.write32::<BusRead>(0x1F80_1810, 0x0000_0000);
    bus.write32::<BusRead>(0x1F80_1810, 0x0003_0002);

    let data_addr: u32 = 0x0000_0100;
    write_ram32(&mut bus, data_addr, 0x1111_1111);
    write_ram32(&mut bus, data_addr + 4, 0x2222_2222);
    write_ram32(&mut bus, data_addr + 8, 0x3333_3333);
    write_ram32(&mut bus, data_addr + 12, 0x4444_4444);
    write_ram32(&mut bus, data_addr + 16, 0x5555_5555);
    write_ram32(&mut bus, data_addr + 20, 0x6666_6666);

    bus.write32::<BusRead>(D2_MADR, data_addr);
    bus.write32::<BusRead>(D2_BCR, 0x0002_0003);
    bus.write32::<BusRead>(D2_CHCR, 0x1100_0201);

    let gpustat = bus.read32::<BusRead>(GPUSTAT);
    assert_ne!(
        gpustat & (1 << 26),
        0,
        "bit 26 (Ready) setado apos transferencia block de 6 palavras"
    );
}

#[test]
fn dma2_linked_list_multiplos_nos() {
    let mut bus = bus_com_dma();

    let node1: u32 = 0x0000_0100;
    let node2: u32 = 0x0000_0200;
    let header1 = (node2 & 0x00FF_FFFF) | (1 << 24);
    write_ram32(&mut bus, node1, header1);
    write_ram32(&mut bus, node1 + 4, 0xE100_0005);

    let header2 = 0x00FF_FFFF | (1 << 24);
    write_ram32(&mut bus, node2, header2);
    write_ram32(&mut bus, node2 + 4, 0xE100_000A);

    bus.write32::<BusRead>(D2_MADR, node1);
    bus.write32::<BusRead>(D2_BCR, 0);
    bus.write32::<BusRead>(D2_CHCR, 0x1100_0401);

    let gpustat = bus.read32::<BusRead>(GPUSTAT);
    assert_eq!(
        gpustat & 0x7FF,
        0x000A,
        "segundo no executou: GPUSTAT bits 0-10 = 0x000A"
    );
    assert_ne!(
        gpustat & 0x7FF,
        0x0005,
        "primeiro no foi sobrescrito pelo segundo"
    );
}

#[test]
fn dma2_linked_list_madr_contem_end_marker_apos_transferencia() {
    let mut bus = bus_com_dma();

    let list_addr: u32 = 0x0000_0100;
    let header = 0x00FF_FFFF | (2 << 24);
    write_ram32(&mut bus, list_addr, header);
    write_ram32(&mut bus, list_addr + 4, 0xE100_0005);
    write_ram32(&mut bus, list_addr + 8, 0xE100_000A);

    bus.write32::<BusRead>(D2_MADR, list_addr);
    bus.write32::<BusRead>(D2_BCR, 0);
    bus.write32::<BusRead>(D2_CHCR, 0x1100_0401);

    let madr = bus.read32::<BusRead>(D2_MADR);
    assert_eq!(
        madr, 0x00FF_FFFF,
        "MADR deve conter end-marker 0x00FFFFFF apos transferencia linked-list"
    );
}
