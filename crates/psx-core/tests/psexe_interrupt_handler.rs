use psx_core::bus::{Bios, Bus, BusRead, Ram};
use psx_core::cpu::Cpu;

fn make_fake_psexe() -> Vec<u8> {
    let mut data = vec![0u8; 0x800 + 4];
    data[0..8].copy_from_slice(b"PS-X EXE");
    data[0x10..0x14].copy_from_slice(&0x8001_0000u32.to_le_bytes());
    data[0x18..0x1C].copy_from_slice(&0x8001_0000u32.to_le_bytes());
    data[0x1C..0x20].copy_from_slice(&4u32.to_le_bytes());
    data[0x30..0x34].copy_from_slice(&0x801F_FFF0u32.to_le_bytes());
    data[0x34..0x38].copy_from_slice(&0u32.to_le_bytes());
    data[0x800..0x804].copy_from_slice(&0x0000_0000u32.to_le_bytes());
    data
}

#[test]
fn load_psexe_configura_sr_com_interrupcoes_habilitadas() {
    let exe_data = make_fake_psexe();
    let bios = Bios::from_bytes(vec![0u8; 0x80000]).expect("BIOS vazia");
    let ram = Ram::new();
    let mut bus = Bus::new(ram, bios);
    let mut cpu = Cpu::new();

    psx_core::psexe::load_psexe(&exe_data, &mut bus, &mut cpu)
        .expect("PS-EXE deve carregar sem erro");

    let sr = cpu.sr();
    assert_eq!(sr & 0x1, 0x1, "IEc deve estar setado (bit 0 = 1)");
    assert_eq!(
        sr & (1 << 12),
        1 << 12,
        "IM[2] deve estar setado (bit 12 = 1)"
    );
}

#[test]
fn install_return_stubs_instala_handler_que_acknowledge_istat() {
    let bios = Bios::from_bytes(vec![0u8; 0x80000]).expect("BIOS vazia");
    let ram = Ram::new();
    let mut bus = Bus::new(ram, bios);

    psx_core::psexe::install_return_stubs(&mut bus);

    let instr0 = bus.read32::<BusRead>(0x8000_0080);
    let instr1 = bus.read32::<BusRead>(0x8000_0084);
    let instr2 = bus.read32::<BusRead>(0x8000_0088);
    let instr3 = bus.read32::<BusRead>(0x8000_008C);
    let instr4 = bus.read32::<BusRead>(0x8000_0090);

    assert_eq!(instr0, 0x3C081F80, "lui t0, 0x1F80");
    assert_eq!(instr1, 0x35091070, "ori t1, t0, 0x1070");
    assert_eq!(instr2, 0x8D280000, "lw t0, 0(t1)");
    assert_eq!(instr3, 0xAD280000, "sw t0, 0(t1)");
    assert_eq!(instr4, 0x42000010, "rfe");
}

#[test]
fn handler_no_vector_0x80_acknowledge_istat_e_rfe_restaura_iec() {
    let bios = Bios::from_bytes(vec![0u8; 0x80000]).expect("BIOS vazia");
    let ram = Ram::new();
    let mut bus = Bus::new(ram, bios);
    let mut cpu = Cpu::new();

    cpu.set_sr(0x0000_1001);

    psx_core::psexe::install_return_stubs(&mut bus);

    bus.irq_mut().write_mask(1);
    bus.irq_mut().raise(0);

    let sr_antes = cpu.sr();
    assert_eq!(
        sr_antes & 0x1,
        0x1,
        "IEc deve estar setado antes da interrupcao"
    );

    for _ in 0..10 {
        cpu.step(&mut bus);
    }

    let sr_depois = cpu.sr();
    assert_eq!(
        sr_depois & 0x1,
        0x1,
        "IEc deve estar setado apos handler RFE"
    );
}
