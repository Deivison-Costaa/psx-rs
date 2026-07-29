mod support;

use psx_core::bus::{Bus, BusRead};
use support::asm;

fn bus_com_dma() -> Bus {
    asm::bus_with_bios_empty()
}

const D6_MADR: u32 = 0x1F80_10E0;
const D6_BCR: u32 = 0x1F80_10E4;
const D6_CHCR: u32 = 0x1F80_10E8;
const DPCR: u32 = 0x1F80_10F0;
const DICR: u32 = 0x1F80_10F4;

#[test]
fn dma6_madr_gravavel_e_legivel() {
    let mut bus = bus_com_dma();
    bus.write32::<BusRead>(D6_MADR, 0x0012_3456);
    let val = bus.read32::<BusRead>(D6_MADR);
    assert_eq!(val & 0x00FF_FFFF, 0x0012_3456);
}

#[test]
fn dma6_bcr_gravavel_e_legivel() {
    let mut bus = bus_com_dma();
    bus.write32::<BusRead>(D6_BCR, 0x0042);
    let val = bus.read32::<BusRead>(D6_BCR);
    assert_eq!(val & 0xFFFF, 0x0042);
}

#[test]
fn dma6_chcr_apenas_bits_24_28_30_e_1_sao_gravaveis() {
    let mut bus = bus_com_dma();
    bus.write32::<BusRead>(D6_MADR, 0x0000_0010);
    bus.write32::<BusRead>(D6_BCR, 1);
    bus.write32::<BusRead>(DPCR, 0x0765_4321 | (1 << 27));
    bus.write32::<BusRead>(D6_CHCR, 0xFFFF_FFFF);
    let val = bus.read32::<BusRead>(D6_CHCR);
    assert_eq!(
        val & 0x5100_0002,
        0x4000_0002,
        "bits 24 e 28 limpos pelo OTC, bit 30 e bit 1 permanecem"
    );
    assert_eq!(val & !0x5100_0002, 0, "nenhum outro bit e gravavel");
}

#[test]
fn dpcr_reset_inicial_e_07654321h() {
    let bus = bus_com_dma();
    let val = bus.read32::<BusRead>(DPCR);
    assert_eq!(val, 0x0765_4321);
}

#[test]
fn dpcr_gravavel_e_legivel() {
    let mut bus = bus_com_dma();
    bus.write32::<BusRead>(DPCR, 0x1234_5678);
    let val = bus.read32::<BusRead>(DPCR);
    assert_eq!(val, 0x1234_5678);
}

#[test]
fn dicr_gravavel_e_legivel() {
    let mut bus = bus_com_dma();
    bus.write32::<BusRead>(DICR, 0x0080_0000);
    let val = bus.read32::<BusRead>(DICR);
    assert_eq!(val & 0x0080_0000, 0x0080_0000);
}

#[test]
fn dma6_otc_preenche_ram_com_linked_list() {
    let mut bus = bus_com_dma();
    let base: u32 = 0x0000_1000;
    let count: u32 = 4;
    bus.write32::<BusRead>(D6_MADR, base);
    bus.write32::<BusRead>(D6_BCR, count);
    bus.write32::<BusRead>(DPCR, 0x0765_4321 | (1 << 27));
    bus.write32::<BusRead>(D6_CHCR, 0x1100_0002);
    let slot_mais_baixo = bus.read32::<BusRead>(base.wrapping_sub(12));
    assert_eq!(
        slot_mais_baixo, 0x00FF_FFFF,
        "slot mais baixo = terminador (end marker)"
    );
    let slot_2 = bus.read32::<BusRead>(base.wrapping_sub(8));
    assert_eq!(
        slot_2,
        (base.wrapping_sub(12)) & 0x00FF_FFFC,
        "slot N-2 aponta para o slot mais baixo (N-3)"
    );
    let slot_1 = bus.read32::<BusRead>(base.wrapping_sub(4));
    assert_eq!(
        slot_1,
        (base.wrapping_sub(8)) & 0x00FF_FFFC,
        "slot N-1 aponta para o slot N-2"
    );
    let slot_mais_alto = bus.read32::<BusRead>(base);
    assert_eq!(
        slot_mais_alto,
        (base.wrapping_sub(4)) & 0x00FF_FFFC,
        "slot mais alto (N) aponta para o slot N-1"
    );
}

#[test]
fn dma6_chcr_bit24_e_limpo_apos_otc() {
    let mut bus = bus_com_dma();
    bus.write32::<BusRead>(D6_MADR, 0x0000_0100);
    bus.write32::<BusRead>(D6_BCR, 1);
    bus.write32::<BusRead>(DPCR, 0x0765_4321 | (1 << 27));
    bus.write32::<BusRead>(D6_CHCR, 0x1100_0002);
    let chcr = bus.read32::<BusRead>(D6_CHCR);
    assert_eq!(
        chcr & (1 << 24),
        0,
        "bit 24 deve ser limpo apos transferencia"
    );
}

#[test]
fn dma6_chcr_bit28_e_limpo_apos_otc() {
    let mut bus = bus_com_dma();
    bus.write32::<BusRead>(D6_MADR, 0x0000_0200);
    bus.write32::<BusRead>(D6_BCR, 1);
    bus.write32::<BusRead>(DPCR, 0x0765_4321 | (1 << 27));
    bus.write32::<BusRead>(D6_CHCR, 0x1100_0002);
    let chcr = bus.read32::<BusRead>(D6_CHCR);
    assert_eq!(
        chcr & (1 << 28),
        0,
        "bit 28 deve ser limpo apos transferencia"
    );
}

#[test]
fn dma6_otc_nao_dispara_sem_bit24() {
    let mut bus = bus_com_dma();
    bus.write32::<BusRead>(D6_MADR, 0x0000_0500);
    bus.write32::<BusRead>(D6_BCR, 1);
    bus.write32::<BusRead>(0x0000_0500, 0xCAFE_BABE);
    bus.write32::<BusRead>(DPCR, 0x0765_4321 | (1 << 27));
    bus.write32::<BusRead>(D6_CHCR, 0x1000_0002);
    let val = bus.read32::<BusRead>(0x0000_0500);
    assert_eq!(val, 0xCAFE_BABE, "RAM nao foi alterada sem bit 24");
}

#[test]
fn dma6_otc_bcr_zero_equivale_a_10000h() {
    let mut bus = bus_com_dma();
    let base: u32 = 0x0010_0000;
    bus.write32::<BusRead>(D6_MADR, base);
    bus.write32::<BusRead>(D6_BCR, 0);
    bus.write32::<BusRead>(DPCR, 0x0765_4321 | (1 << 27));
    bus.write32::<BusRead>(D6_CHCR, 0x1100_0002);
    let count = 0x10000usize;
    let mais_baixo = base.wrapping_sub((count as u32 - 1) * 4);
    let terminador = bus.read32::<BusRead>(mais_baixo);
    assert_eq!(
        terminador, 0x00FF_FFFF,
        "com BCR=0, slot mais baixo = terminador"
    );
    let penultimo_baixo = bus.read32::<BusRead>(mais_baixo.wrapping_add(4));
    assert_eq!(
        penultimo_baixo,
        mais_baixo & 0x00FF_FFFC,
        "slot logo acima do terminador aponta para o terminador"
    );
    let slot_mais_alto = bus.read32::<BusRead>(base);
    assert_eq!(
        slot_mais_alto,
        (base.wrapping_sub(4)) & 0x00FF_FFFC,
        "slot mais alto aponta para o slot N-1"
    );
}

#[test]
fn dma6_otc_nao_dispara_sem_bit28() {
    let mut bus = bus_com_dma();
    bus.write32::<BusRead>(D6_MADR, 0x0000_0600);
    bus.write32::<BusRead>(D6_BCR, 1);
    bus.write32::<BusRead>(0x0000_0600, 0xCAFE_BABE);
    bus.write32::<BusRead>(DPCR, 0x0765_4321 | (1 << 27));
    bus.write32::<BusRead>(D6_CHCR, 0x0100_0002);
    let val = bus.read32::<BusRead>(0x0000_0600);
    assert_eq!(
        val, 0xCAFE_BABE,
        "RAM nao foi alterada com bit24=1 e bit28=0 (sem force-start)"
    );
}

#[test]
fn dma6_chcr_bit24_gravavel_sem_executar_otc() {
    let mut bus = bus_com_dma();
    bus.write32::<BusRead>(D6_MADR, 0x0000_0700);
    bus.write32::<BusRead>(D6_BCR, 1);
    bus.write32::<BusRead>(0x0000_0700, 0xDEAD_BEEF);
    bus.write32::<BusRead>(DPCR, 0x0765_4321 | (1 << 27));
    bus.write32::<BusRead>(D6_CHCR, 0x0100_0002);
    let chcr = bus.read32::<BusRead>(D6_CHCR);
    assert_eq!(
        chcr & (1 << 24),
        1 << 24,
        "bit 24 permanece setado (OTC nao executou)"
    );
    let ram = bus.read32::<BusRead>(0x0000_0700);
    assert_eq!(ram, 0xDEAD_BEEF, "RAM nao foi alterada sem bit 28");
}
