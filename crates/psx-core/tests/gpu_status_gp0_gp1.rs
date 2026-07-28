use psx_core::bus::{Bios, Bus, BusRead, Ram};

#[test]
fn reset_gpu_produz_golden_value() {
    let ram = Ram::new();
    let bios = Bios::from_bytes(vec![0u8; 0x80000]).expect("BIOS de teste");
    let mut bus = Bus::new(ram, bios);

    let gp1_addr: u32 = 0xBF80_1814;
    bus.write32::<BusRead>(gp1_addr, 0x00 << 24);

    let actual = bus.read32::<BusRead>(gp1_addr);
    assert_eq!(
        actual, 0x1480_2000,
        "A1: GP1(00h) reset → GPUSTAT deve ser 0x14802000h (L763), obtido 0x{:08X}",
        actual
    );
}

#[test]
fn gp1_08h_display_mode_mapeia_para_gpustat() {
    let ram = Ram::new();
    let bios = Bios::from_bytes(vec![0u8; 0x80000]).expect("BIOS de teste");
    let mut bus = Bus::new(ram, bios);

    let gp1_addr: u32 = 0xBF80_1814;

    bus.write32::<BusRead>(gp1_addr, 0x08 << 24);
    let mut stat = bus.read32::<BusRead>(gp1_addr);
    assert_eq!((stat >> 16) & 1, 0, "A2 modo=00h: HR2(bit16)=0");
    assert_eq!((stat >> 17) & 3, 0, "A2 modo=00h: HR1(bits17-18)=0");
    assert_eq!((stat >> 19) & 1, 0, "A2 modo=00h: VR(bit19)=0");
    assert_eq!((stat >> 20) & 1, 0, "A2 modo=00h: VM(bit20)=0");
    assert_eq!((stat >> 21) & 1, 0, "A2 modo=00h: CD(bit21)=0");
    assert_eq!((stat >> 22) & 1, 0, "A2 modo=00h: VI(bit22)=0");
    assert_eq!((stat >> 14) & 1, 0, "A2 modo=00h: Flip(bit14)=0");

    bus.write32::<BusRead>(gp1_addr, (0x08 << 24) | 0xFF);
    stat = bus.read32::<BusRead>(gp1_addr);
    assert_eq!((stat >> 16) & 1, 1, "A2 modo=FFh: HR2(bit16)=1 (368)");
    assert_eq!((stat >> 17) & 3, 3, "A2 modo=FFh: HR1(bits17-18)=3 (640)");
    assert_eq!((stat >> 19) & 1, 1, "A2 modo=FFh: VR(bit19)=1 (480)");
    assert_eq!((stat >> 20) & 1, 1, "A2 modo=FFh: VM(bit20)=1 (PAL)");
    assert_eq!((stat >> 21) & 1, 1, "A2 modo=FFh: CD(bit21)=1 (24bit)");
    assert_eq!((stat >> 22) & 1, 1, "A2 modo=FFh: VI(bit22)=1 (interlace)");
    assert_eq!((stat >> 14) & 1, 1, "A2 modo=FFh: Flip(bit14)=1");
}

#[test]
fn gp0_e1h_draw_mode_escreve_gpustat_0_10_e_15() {
    let ram = Ram::new();
    let bios = Bios::from_bytes(vec![0u8; 0x80000]).expect("BIOS de teste");
    let mut bus = Bus::new(ram, bios);

    let gp0_addr: u32 = 0xBF80_1810;
    let gp1_addr: u32 = 0xBF80_1814;

    bus.write32::<BusRead>(gp0_addr, (0xE1 << 24) | 0xFFF);
    let stat = bus.read32::<BusRead>(gp1_addr);

    assert_eq!(stat & 0xF, 0xF, "A3: GPUSTAT.0-3 (TexPage X)=0xF");
    assert_eq!((stat >> 4) & 1, 1, "A3: GPUSTAT.4 (TexPage Y1)=1");
    assert_eq!((stat >> 5) & 3, 3, "A3: GPUSTAT.5-6 (SemiTransp)=3");
    assert_eq!((stat >> 7) & 3, 3, "A3: GPUSTAT.7-8 (TexColors)=3");
    assert_eq!((stat >> 9) & 1, 1, "A3: GPUSTAT.9 (Dither)=1");
    assert_eq!((stat >> 10) & 1, 1, "A3: GPUSTAT.10 (DrawToDisplay)=1");
    assert_eq!((stat >> 15) & 1, 1, "A3: GPUSTAT.15 (TexPage Y2)=1");
}

#[test]
fn gp0_e6h_mask_bit_escreve_gpustat_11_12() {
    let ram = Ram::new();
    let bios = Bios::from_bytes(vec![0u8; 0x80000]).expect("BIOS de teste");
    let mut bus = Bus::new(ram, bios);

    let gp0_addr: u32 = 0xBF80_1810;
    let gp1_addr: u32 = 0xBF80_1814;

    bus.write32::<BusRead>(gp0_addr, (0xE6 << 24) | 0x3);
    let stat = bus.read32::<BusRead>(gp1_addr);

    assert_eq!((stat >> 11) & 1, 1, "A3: GPUSTAT.11 (MaskSet)=1");
    assert_eq!((stat >> 12) & 1, 1, "A3: GPUSTAT.12 (MaskCheck)=1");
}

#[test]
fn gp1_03h_alterna_bit_23_display() {
    let ram = Ram::new();
    let bios = Bios::from_bytes(vec![0u8; 0x80000]).expect("BIOS de teste");
    let mut bus = Bus::new(ram, bios);

    let gp1_addr: u32 = 0xBF80_1814;

    bus.write32::<BusRead>(gp1_addr, 0x03 << 24);
    let stat = bus.read32::<BusRead>(gp1_addr);
    assert_eq!(
        (stat >> 23) & 1,
        0,
        "A4: GP1(03h).0=0 → Display On → GPUSTAT.23=0 (L781)"
    );

    bus.write32::<BusRead>(gp1_addr, (0x03 << 24) | 0x01);
    let stat = bus.read32::<BusRead>(gp1_addr);
    assert_eq!(
        (stat >> 23) & 1,
        1,
        "A4: GP1(03h).0=1 → Display Off → GPUSTAT.23=1 (L781)"
    );
}

#[test]
fn gp1_04h_dma_direction_escreve_bits_29_30() {
    let ram = Ram::new();
    let bios = Bios::from_bytes(vec![0u8; 0x80000]).expect("BIOS de teste");
    let mut bus = Bus::new(ram, bios);

    let gp1_addr: u32 = 0xBF80_1814;

    bus.write32::<BusRead>(gp1_addr, 0x04 << 24);
    let stat = bus.read32::<BusRead>(gp1_addr);
    assert_eq!((stat >> 29) & 3, 0, "A4: DMA direction 0 → bits 29-30=0");

    bus.write32::<BusRead>(gp1_addr, (0x04 << 24) | 0x01);
    let stat = bus.read32::<BusRead>(gp1_addr);
    assert_eq!((stat >> 29) & 3, 1, "A4: DMA direction 1 → bits 29-30=1");

    bus.write32::<BusRead>(gp1_addr, (0x04 << 24) | 0x02);
    let stat = bus.read32::<BusRead>(gp1_addr);
    assert_eq!((stat >> 29) & 3, 2, "A4: DMA direction 2 → bits 29-30=2");

    bus.write32::<BusRead>(gp1_addr, (0x04 << 24) | 0x03);
    let stat = bus.read32::<BusRead>(gp1_addr);
    assert_eq!((stat >> 29) & 3, 3, "A4: DMA direction 3 → bits 29-30=3");
}

#[test]
fn gpustat_bit_25_espelha_dma_direction() {
    let ram = Ram::new();
    let bios = Bios::from_bytes(vec![0u8; 0x80000]).expect("BIOS de teste");
    let mut bus = Bus::new(ram, bios);

    let gp1_addr: u32 = 0xBF80_1814;

    bus.write32::<BusRead>(gp1_addr, 0x04 << 24);
    let stat = bus.read32::<BusRead>(gp1_addr);
    assert_eq!(
        (stat >> 25) & 1,
        0,
        "A4: DMA direction 0 → bit 25 sempre zero (L1024)"
    );

    bus.write32::<BusRead>(gp1_addr, (0x04 << 24) | 0x01);
    let stat = bus.read32::<BusRead>(gp1_addr);
    assert_eq!(
        (stat >> 25) & 1,
        1,
        "A4: DMA direction 1 → bit 25 = FIFO not full (L1025)"
    );

    bus.write32::<BusRead>(gp1_addr, (0x04 << 24) | 0x02);
    let stat = bus.read32::<BusRead>(gp1_addr);
    let bit28 = (stat >> 28) & 1;
    assert_eq!(
        (stat >> 25) & 1,
        bit28,
        "A4: DMA direction 2 → bit 25 = bit 28 ({bit28}) (L1026)"
    );

    bus.write32::<BusRead>(gp1_addr, (0x04 << 24) | 0x03);
    let stat = bus.read32::<BusRead>(gp1_addr);
    let bit27 = (stat >> 27) & 1;
    assert_eq!(
        (stat >> 25) & 1,
        bit27,
        "A4: DMA direction 3 → bit 25 = bit 27 ({bit27}) (L1027)"
    );
}

#[test]
fn gpuread_stub_retorna_zero() {
    let ram = Ram::new();
    let bios = Bios::from_bytes(vec![0u8; 0x80000]).expect("BIOS de teste");
    let bus = Bus::new(ram, bios);

    let gpuread_addr: u32 = 0xBF80_1810;
    let val = bus.read32::<BusRead>(gpuread_addr);
    assert_eq!(val, 0, "GPUREAD stub deve retornar 0");
}

#[test]
fn gp0_00h_nop_nao_altera_gpustat() {
    let ram = Ram::new();
    let bios = Bios::from_bytes(vec![0u8; 0x80000]).expect("BIOS de teste");
    let mut bus = Bus::new(ram, bios);

    let gp0_addr: u32 = 0xBF80_1810;
    let gp1_addr: u32 = 0xBF80_1814;

    let antes = bus.read32::<BusRead>(gp1_addr);
    bus.write32::<BusRead>(gp0_addr, 0x00 << 24);
    let depois = bus.read32::<BusRead>(gp1_addr);
    assert_eq!(depois, antes, "GP0(00h) NOP nao deve alterar GPUSTAT");
}

#[test]
fn gp0_mirrors_de_nop_nao_alteram_gpustat() {
    let ram = Ram::new();
    let bios = Bios::from_bytes(vec![0u8; 0x80000]).expect("BIOS de teste");
    let mut bus = Bus::new(ram, bios);

    let gp0_addr: u32 = 0xBF80_1810;
    let gp1_addr: u32 = 0xBF80_1814;

    let mirrors = [0x04u32, 0x1E, 0xE0, 0xE7, 0xEF];
    for cmd in mirrors {
        let antes = bus.read32::<BusRead>(gp1_addr);
        bus.write32::<BusRead>(gp0_addr, cmd << 24);
        let depois = bus.read32::<BusRead>(gp1_addr);
        assert_eq!(
            depois, antes,
            "GP0({:02X}h) mirror de NOP nao deve alterar GPUSTAT",
            cmd
        );
    }
}

#[test]
fn gp1_01h_reset_command_buffer_nao_altera_gpustat() {
    let ram = Ram::new();
    let bios = Bios::from_bytes(vec![0u8; 0x80000]).expect("BIOS de teste");
    let mut bus = Bus::new(ram, bios);

    let gp1_addr: u32 = 0xBF80_1814;

    bus.write32::<BusRead>(gp1_addr, (0x08 << 24) | 0xFF);
    let antes = bus.read32::<BusRead>(gp1_addr);

    bus.write32::<BusRead>(gp1_addr, 0x01 << 24);
    let depois = bus.read32::<BusRead>(gp1_addr);
    assert_eq!(
        depois, antes,
        "GP1(01h) Reset Command Buffer nao deve alterar GPUSTAT (L767)"
    );
}

#[test]
fn gp1_02h_ack_irq1_limpa_bit_24() {
    let ram = Ram::new();
    let bios = Bios::from_bytes(vec![0u8; 0x80000]).expect("BIOS de teste");
    let mut bus = Bus::new(ram, bios);

    let gp1_addr: u32 = 0xBF80_1814;

    bus.write32::<BusRead>(gp1_addr, 0x02 << 24);
    let stat = bus.read32::<BusRead>(gp1_addr);
    assert_eq!(
        (stat >> 24) & 1,
        0,
        "A4: GP1(02h) deve limpar GPUSTAT.24 (L777)"
    );
}

#[test]
fn reset_gpustat_bits_de_pronto() {
    let ram = Ram::new();
    let bios = Bios::from_bytes(vec![0u8; 0x80000]).expect("BIOS de teste");
    let mut bus = Bus::new(ram, bios);

    let gp1_addr: u32 = 0xBF80_1814;

    bus.write32::<BusRead>(gp1_addr, 0x00 << 24);
    let stat = bus.read32::<BusRead>(gp1_addr);

    assert_eq!((stat >> 26) & 1, 1, "Reset: bit 26 (Ready Cmd) = 1");
    assert_eq!((stat >> 28) & 1, 1, "Reset: bit 28 (Ready DMA) = 1");
    assert_eq!((stat >> 27) & 1, 0, "Reset: bit 27 (Ready VRAM→CPU) = 0");
}

fn bus_zerado() -> Bus {
    let bios = Bios::from_bytes(vec![0u8; 0x80000]).expect("BIOS de teste");
    Bus::new(Ram::new(), bios)
}

#[test]
fn gp1_01h_reseta_command_buffer_e_libera_o_gp0() {
    let mut bus = bus_zerado();
    let gp0: u32 = 0xBF80_1810;
    let gp1: u32 = 0xBF80_1814;

    bus.write32::<BusRead>(gp0, 0xA0u32 << 24);
    bus.write32::<BusRead>(gp0, 0x0000_0000);
    assert_eq!(
        (bus.read32::<BusRead>(gp1) >> 26) & 1,
        0,
        "cabecalho A0h incompleto: bit26=0"
    );

    bus.write32::<BusRead>(gp1, 0x01u32 << 24);
    assert_eq!(
        (bus.read32::<BusRead>(gp1) >> 26) & 1,
        1,
        "GP1(01h) limpa o buffer de comando e religa bit26 (L767-771)"
    );

    bus.write32::<BusRead>(gp0, (0x02u32 << 24) | 0x00FF_FFFF);
    bus.write32::<BusRead>(gp0, 0x0000_0000);
    bus.write32::<BusRead>(gp0, 0x0001_0010);
    assert_eq!(
        bus.gpu().vram_pixel(0, 0),
        0x7FFF,
        "fill apos GP1(01h) executa como comando"
    );
}

#[test]
fn bit26_cai_durante_os_parametros_do_gp0_80h() {
    let mut bus = bus_zerado();
    let gp0: u32 = 0xBF80_1810;
    let gp1: u32 = 0xBF80_1814;

    bus.write32::<BusRead>(gp0, 0x80u32 << 24);
    assert_eq!(
        (bus.read32::<BusRead>(gp1) >> 26) & 1,
        0,
        "GP0(80h) recebido, faltam 3 params: bit26=0 (L1051-1053)"
    );
    bus.write32::<BusRead>(gp0, 0x0000_0000);
    assert_eq!(
        (bus.read32::<BusRead>(gp1) >> 26) & 1,
        0,
        "1o param do 80h: bit26=0"
    );
    bus.write32::<BusRead>(gp0, 0x0000_0000);
    bus.write32::<BusRead>(gp0, 0x0001_0001);
    assert_eq!(
        (bus.read32::<BusRead>(gp1) >> 26) & 1,
        1,
        "params completos: bit26=1"
    );
}

#[test]
fn leitura_de_16_bits_alcanca_o_halfword_alto_do_gpustat() {
    let bus = bus_zerado();

    let lo = bus.read16::<BusRead>(0xBF80_1814);
    let hi = bus.read16::<BusRead>(0xBF80_1816);
    assert_eq!(lo, 0x2000, "GPUSTAT=14802000h: halfword baixo 0x2000");
    assert_eq!(
        hi, 0x1480,
        "GPUSTAT=14802000h: halfword alto 0x1480, obtido 0x{:04X} — \
         region_read_byte tem que somar o parametro offset",
        hi
    );
}
