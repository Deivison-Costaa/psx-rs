use psx_core::bus::{Bios, Bus, BusRead, Ram};

#[test]
fn t1_gp1_05h_escreve_display_vram_start() {
    let ram = Ram::new();
    let bios = Bios::from_bytes(vec![0u8; 0x80000]).expect("BIOS de teste");
    let mut bus = Bus::new(ram, bios);

    let gp1_addr: u32 = 0xBF80_1814;

    bus.write32::<BusRead>(gp1_addr, (0x05 << 24) | 0x200 | (0x100 << 10));

    assert_eq!(bus.gpu().display_vram_x(), 0x200, "T1: GP1(05h) X=0x200");
    assert_eq!(bus.gpu().display_vram_y(), 0x100, "T1: GP1(05h) Y=0x100");
}

#[test]
fn t2_gp1_05h_mascara_bits() {
    let ram = Ram::new();
    let bios = Bios::from_bytes(vec![0u8; 0x80000]).expect("BIOS de teste");
    let mut bus = Bus::new(ram, bios);

    let gp1_addr: u32 = 0xBF80_1814;

    bus.write32::<BusRead>(gp1_addr, (0x05 << 24) | 0x3FF | (0x1FF << 10));

    assert_eq!(
        bus.gpu().display_vram_x(),
        0x3FF,
        "T2: GP1(05h) X=0x3FF (10 bits, maximo)"
    );
    assert_eq!(
        bus.gpu().display_vram_y(),
        0x1FF,
        "T2: GP1(05h) Y=0x1FF (9 bits, maximo)"
    );
}

#[test]
fn t3_gp1_05h_bits_altos_truncados() {
    let ram = Ram::new();
    let bios = Bios::from_bytes(vec![0u8; 0x80000]).expect("BIOS de teste");
    let mut bus = Bus::new(ram, bios);

    let gp1_addr: u32 = 0xBF80_1814;

    bus.write32::<BusRead>(gp1_addr, (0x05 << 24) | 0x7FF | (0x3FF << 10));

    assert_eq!(
        bus.gpu().display_vram_x(),
        0x3FF,
        "T3: GP1(05h) X truncado para 10 bits (0x3FF), obtido 0x{:03X}",
        bus.gpu().display_vram_x()
    );
    assert_eq!(
        bus.gpu().display_vram_y(),
        0x1FF,
        "T3: GP1(05h) Y truncado para 9 bits (0x1FF), obtido 0x{:03X}",
        bus.gpu().display_vram_y()
    );
}

#[test]
fn t4_gp1_06h_escreve_horizontal_range() {
    let ram = Ram::new();
    let bios = Bios::from_bytes(vec![0u8; 0x80000]).expect("BIOS de teste");
    let mut bus = Bus::new(ram, bios);

    let gp1_addr: u32 = 0xBF80_1814;

    bus.write32::<BusRead>(gp1_addr, (0x06 << 24) | 0x260 | (0xC60 << 12));

    assert_eq!(bus.gpu().display_x1(), 0x260, "T4: GP1(06h) X1=0x260");
    assert_eq!(bus.gpu().display_x2(), 0xC60, "T4: GP1(06h) X2=0xC60");
}

#[test]
fn t5_gp1_06h_mascara_12_bits() {
    let ram = Ram::new();
    let bios = Bios::from_bytes(vec![0u8; 0x80000]).expect("BIOS de teste");
    let mut bus = Bus::new(ram, bios);

    let gp1_addr: u32 = 0xBF80_1814;

    bus.write32::<BusRead>(gp1_addr, (0x06 << 24) | 0xFFF | (0xFFF << 12));

    assert_eq!(
        bus.gpu().display_x1(),
        0xFFF,
        "T5: GP1(06h) X1=0xFFF (12 bits, maximo)"
    );
    assert_eq!(
        bus.gpu().display_x2(),
        0xFFF,
        "T5: GP1(06h) X2=0xFFF (12 bits, maximo)"
    );
}

#[test]
fn t6_gp1_06h_bits_altos_truncados() {
    let ram = Ram::new();
    let bios = Bios::from_bytes(vec![0u8; 0x80000]).expect("BIOS de teste");
    let mut bus = Bus::new(ram, bios);

    let gp1_addr: u32 = 0xBF80_1814;

    bus.write32::<BusRead>(
        gp1_addr,
        (0x06 << 24) | ((0x1FFF | (0x1FFF << 12)) & 0xFF_FFFF),
    );

    assert_eq!(
        bus.gpu().display_x1(),
        0xFFF,
        "T6: GP1(06h) X1 truncado para 12 bits (0xFFF), obtido 0x{:03X}",
        bus.gpu().display_x1()
    );
    assert_eq!(
        bus.gpu().display_x2(),
        0xFFF,
        "T6: GP1(06h) X2 truncado para 12 bits (0xFFF), obtido 0x{:03X}",
        bus.gpu().display_x2()
    );
}

#[test]
fn t7_gp1_07h_escreve_vertical_range() {
    let ram = Ram::new();
    let bios = Bios::from_bytes(vec![0u8; 0x80000]).expect("BIOS de teste");
    let mut bus = Bus::new(ram, bios);

    let gp1_addr: u32 = 0xBF80_1814;

    bus.write32::<BusRead>(gp1_addr, (0x07 << 24) | 0x10 | (0xF0 << 10));

    assert_eq!(bus.gpu().display_y1(), 0x10, "T7: GP1(07h) Y1=0x10");
    assert_eq!(bus.gpu().display_y2(), 0xF0, "T7: GP1(07h) Y2=0xF0");
}

#[test]
fn t8_gp1_07h_mascara_10_bits() {
    let ram = Ram::new();
    let bios = Bios::from_bytes(vec![0u8; 0x80000]).expect("BIOS de teste");
    let mut bus = Bus::new(ram, bios);

    let gp1_addr: u32 = 0xBF80_1814;

    bus.write32::<BusRead>(gp1_addr, (0x07 << 24) | 0x3FF | (0x3FF << 10));

    assert_eq!(
        bus.gpu().display_y1(),
        0x3FF,
        "T8: GP1(07h) Y1=0x3FF (10 bits, maximo)"
    );
    assert_eq!(
        bus.gpu().display_y2(),
        0x3FF,
        "T8: GP1(07h) Y2=0x3FF (10 bits, maximo)"
    );
}

#[test]
fn t9_gp1_00h_reset_limpa_display_range() {
    let ram = Ram::new();
    let bios = Bios::from_bytes(vec![0u8; 0x80000]).expect("BIOS de teste");
    let mut bus = Bus::new(ram, bios);

    let gp1_addr: u32 = 0xBF80_1814;

    bus.write32::<BusRead>(gp1_addr, (0x05 << 24) | 0x3FF | (0x1FF << 10));
    bus.write32::<BusRead>(gp1_addr, (0x06 << 24) | 0xFFF | (0xFFF << 12));
    bus.write32::<BusRead>(gp1_addr, (0x07 << 24) | 0x3FF | (0x3FF << 10));

    bus.write32::<BusRead>(gp1_addr, 0x00 << 24);

    assert_eq!(
        bus.gpu().display_vram_x(),
        0,
        "T9: reset → display_vram_x=0"
    );
    assert_eq!(
        bus.gpu().display_vram_y(),
        0,
        "T9: reset → display_vram_y=0"
    );
    assert_eq!(
        bus.gpu().display_x1(),
        0x260,
        "T9: reset → display_x1=0x260 (padrao NTSC)"
    );
    assert_eq!(
        bus.gpu().display_x2(),
        0xC60,
        "T9: reset → display_x2=0xC60 (260h+320*8)"
    );
    assert_eq!(
        bus.gpu().display_y1(),
        0x88 - 120,
        "T9: reset → display_y1=0x10 (NTSC: 88h-120)"
    );
    assert_eq!(
        bus.gpu().display_y2(),
        0x88 + 120,
        "T9: reset → display_y2=0x100 (NTSC: 88h+120)"
    );
}

#[test]
fn t10_gp1_07h_bits_altos_truncados() {
    let ram = Ram::new();
    let bios = Bios::from_bytes(vec![0u8; 0x80000]).expect("BIOS de teste");
    let mut bus = Bus::new(ram, bios);

    let gp1_addr: u32 = 0xBF80_1814;

    bus.write32::<BusRead>(gp1_addr, (0x07 << 24) | (0x7FF | (0x7FF << 10)) & 0xFF_FFFF);

    assert_eq!(
        bus.gpu().display_y1(),
        0x3FF,
        "T10: GP1(07h) Y1 truncado para 10 bits (0x3FF), obtido 0x{:03X}",
        bus.gpu().display_y1()
    );
    assert_eq!(
        bus.gpu().display_y2(),
        0x3FF,
        "T10: GP1(07h) Y2 truncado para 10 bits (0x3FF), obtido 0x{:03X}",
        bus.gpu().display_y2()
    );
}
