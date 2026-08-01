mod support;

use psx_core::bus::{Bus, BusRead};
use support::asm;

fn bus_com_dma() -> Bus {
    asm::bus_with_bios_empty()
}

const CD_BASE: u32 = 0x1F80_1800;
const ESPERA_PRIMEIRA_RESPOSTA: u32 = 0x1_4000;
// 06-cdrom.md L333-337: 2a resposta so apos o ack, com atraso fisico (divida 10.53).
const ESPERA_SEGUNDA_RESPOSTA: u32 = 0x6000;
const D2_MADR: u32 = 0x1F80_10A0;
const D2_BCR: u32 = 0x1F80_10A4;
const D2_CHCR: u32 = 0x1F80_10A8;
const D3_MADR: u32 = 0x1F80_10B0;
const D3_BCR: u32 = 0x1F80_10B4;
const D3_CHCR: u32 = 0x1F80_10B8;
const D6_MADR: u32 = 0x1F80_10E0;
const D6_BCR: u32 = 0x1F80_10E4;
const D6_CHCR: u32 = 0x1F80_10E8;
const DPCR: u32 = 0x1F80_10F0;

fn cd_write(bus: &mut Bus, offset: u32, val: u8) {
    bus.write8::<BusRead>(CD_BASE + offset, val);
}

fn cd_read(bus: &Bus, offset: u32) -> u8 {
    bus.read8::<BusRead>(CD_BASE + offset)
}

fn set_bank(bus: &mut Bus, b: u8) {
    cd_write(bus, 0, b);
}

fn write_ram32(bus: &mut Bus, addr: u32, val: u32) {
    bus.write32::<BusRead>(addr, val);
}

fn enable_dma_channel(bus: &mut Bus, chan: u32) {
    let dpcr = bus.read32::<BusRead>(DPCR);
    let bit = match chan {
        2 => 11,
        3 => 15,
        6 => 27,
        _ => return,
    };
    bus.write32::<BusRead>(DPCR, dpcr | (1 << bit));
}

fn preparar_cdrom_para_dma3(bus: &mut Bus) {
    bus.cdrom_mut().insert_disc();

    set_bank(bus, 0);
    cd_write(bus, 2, 0x02);
    cd_write(bus, 2, 0x10);
    cd_write(bus, 2, 0x00);
    cd_write(bus, 1, 0x02);
    bus.tick_timers(ESPERA_PRIMEIRA_RESPOSTA);
    let _ = cd_read(bus, 1);

    cd_write(bus, 1, 0x06);
    bus.tick_timers(ESPERA_PRIMEIRA_RESPOSTA);
    let _ = cd_read(bus, 1);

    set_bank(bus, 1);
    cd_write(bus, 3, 0x07);
    bus.tick_timers(ESPERA_SEGUNDA_RESPOSTA);
    let hintsts = cd_read(bus, 3) & 0x7;
    set_bank(bus, 0);
    assert_eq!(
        hintsts, 1,
        "pre-condicao: o setor tem de ter chegado (INT1), senao o teste negativo abaixo \
         fica verde por o drive estar morto"
    );
    let _ = cd_read(bus, 1);

    set_bank(bus, 0);
    cd_write(bus, 3, 0x80);
    set_bank(bus, 0);
}

// ── DPCR reset ──

#[test]
fn dpcr_reset_e_07654321h() {
    let bus = bus_com_dma();
    let val = bus.read32::<BusRead>(DPCR);
    assert_eq!(val, 0x0765_4321, "DPCR reset = 07654321h (spec L140)");
}

// ── OTC (canal 6) ──

#[test]
fn otc_nao_transfere_com_canal_desabilitado_no_dpcr() {
    let mut bus = bus_com_dma();
    let base: u32 = 0x0000_1000;
    bus.write32::<BusRead>(base, 0xCAFE_BABE);
    bus.write32::<BusRead>(D6_MADR, base);
    bus.write32::<BusRead>(D6_BCR, 1);
    bus.write32::<BusRead>(D6_CHCR, 0x1100_0002);
    let val = bus.read32::<BusRead>(base);
    assert_eq!(
        val, 0xCAFE_BABE,
        "RAM na base nao foi alterada — DPCR bit 27 = 0 (reset)"
    );
}

#[test]
fn otc_transfere_com_canal_habilitado_no_dpcr() {
    let mut bus = bus_com_dma();
    enable_dma_channel(&mut bus, 6);
    let base: u32 = 0x0000_1000;
    bus.write32::<BusRead>(D6_MADR, base);
    bus.write32::<BusRead>(D6_BCR, 1);
    bus.write32::<BusRead>(D6_CHCR, 0x1100_0002);
    let val = bus.read32::<BusRead>(base);
    assert_eq!(val, 0x00FF_FFFF, "OTC transfere com DPCR bit 27 = 1");
}

// ── DMA3 (CDROM) ──

#[test]
fn dma3_nao_transfere_com_canal_desabilitado_no_dpcr() {
    let mut bus = bus_com_dma();
    preparar_cdrom_para_dma3(&mut bus);

    let dest: u32 = 0x0001_0000;
    bus.write32::<BusRead>(dest, 0xDEAD_BEEF);
    bus.write32::<BusRead>(D3_MADR, dest);
    bus.write32::<BusRead>(D3_BCR, 1);
    bus.write32::<BusRead>(D3_CHCR, 0x1100_0000);
    let val = bus.read32::<BusRead>(dest);
    assert_eq!(
        val, 0xDEAD_BEEF,
        "RAM no destino nao foi alterada — DPCR bit 15 = 0 (reset)"
    );
}

#[test]
fn dma3_transfere_com_canal_habilitado_no_dpcr() {
    let mut bus = bus_com_dma();
    preparar_cdrom_para_dma3(&mut bus);
    enable_dma_channel(&mut bus, 3);

    let dest: u32 = 0x0001_0000;
    bus.write32::<BusRead>(dest, 0xDEAD_BEEF);
    bus.write32::<BusRead>(D3_MADR, dest);
    bus.write32::<BusRead>(D3_BCR, 1);
    bus.write32::<BusRead>(D3_CHCR, 0x1100_0000);
    let val = bus.read32::<BusRead>(dest);
    assert_ne!(val, 0xDEAD_BEEF, "DMA3 transfere com DPCR bit 15 = 1");
}

// ── DMA2 (GPU) ──

#[test]
fn dma2_nao_transfere_com_canal_desabilitado_no_dpcr() {
    let mut bus = bus_com_dma();
    let list_addr: u32 = 0x0000_0100;
    let header = 0x00FF_FFFF | (1 << 24);
    write_ram32(&mut bus, list_addr, header);
    write_ram32(&mut bus, list_addr + 4, 0xE100_001F);

    bus.write32::<BusRead>(D2_MADR, list_addr);
    bus.write32::<BusRead>(D2_BCR, 0);
    bus.write32::<BusRead>(D2_CHCR, 0x1100_0401);

    let gpustat = bus.read32::<BusRead>(0x1F80_1814);
    assert_eq!(
        gpustat & 0x7FF,
        0,
        "GPUSTAT nao foi alterado — DPCR bit 11 = 0 (reset)"
    );
}

#[test]
fn dma2_transfere_com_canal_habilitado_no_dpcr() {
    let mut bus = bus_com_dma();
    enable_dma_channel(&mut bus, 2);
    let list_addr: u32 = 0x0000_0100;
    let header = 0x00FF_FFFF | (1 << 24);
    write_ram32(&mut bus, list_addr, header);
    write_ram32(&mut bus, list_addr + 4, 0xE100_001F);

    bus.write32::<BusRead>(D2_MADR, list_addr);
    bus.write32::<BusRead>(D2_BCR, 0);
    bus.write32::<BusRead>(D2_CHCR, 0x1100_0401);

    let gpustat = bus.read32::<BusRead>(0x1F80_1814);
    assert_eq!(
        gpustat & 0x7FF,
        0x1F,
        "GPUSTAT ajustado pelo comando E1 via DMA2 com DPCR bit 11 = 1"
    );
}
