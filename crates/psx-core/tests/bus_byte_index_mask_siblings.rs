use psx_core::bus::{Bios, Bus, BusRead, Ram};

fn bus_with_empty_bios() -> Bus {
    let bios = Bios::from_bytes(vec![0u8; 0x80000]).expect("BIOS de teste");
    Bus::new(Ram::new(), bios)
}

#[test]
fn mem_ctrl_read16_wraps_byte_index_at_word_end() {
    let mut bus = bus_with_empty_bios();
    bus.write32::<BusRead>(0x1F80_1000, 0xA1B2_C3D4);

    assert_eq!(
        bus.read16::<BusRead>(0x1F80_1003),
        0xD4A1,
        "MEM_CTRL: read16 no byte 3 deve ler byte 3 e voltar ao byte 0"
    );
}

#[test]
fn mem_ctrl_mirror_read16_wraps_byte_index_at_word_end() {
    let mut bus = bus_with_empty_bios();
    bus.write32::<BusRead>(0x1F80_1060, 0xA1B2_C3D4);

    assert_eq!(
        bus.read16::<BusRead>(0x1F80_1063),
        0xD4A1,
        "espelho do MEM_CTRL: read16 no byte 3 deve voltar ao byte 0"
    );
}

#[test]
fn bcc_read16_wraps_byte_index_at_word_end() {
    let mut bus = bus_with_empty_bios();
    bus.write32::<BusRead>(0xFFFE_0130, 0xA1B2_C3D4);

    assert_eq!(
        bus.read16::<BusRead>(0xFFFE_0133),
        0xD4A1,
        "BCC: read16 no byte 3 deve ler byte 3 e voltar ao byte 0"
    );
}

#[test]
fn dma_read16_wraps_byte_index_at_word_end() {
    let mut bus = bus_with_empty_bios();
    bus.write32::<BusRead>(0x1F80_1080, 0x00A1_B2C3);

    assert_eq!(
        bus.read16::<BusRead>(0x1F80_1083),
        0xC300,
        "DMA: read16 no byte 3 deve ler byte 3 e voltar ao byte 0"
    );
}
