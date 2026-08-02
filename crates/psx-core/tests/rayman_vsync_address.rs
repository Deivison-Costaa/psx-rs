use psx_core::bus::{Bios, Bus, BusRead, BusWrite, Ram};
use psx_core::cpu::Cpu;

#[test]
fn lw_e_sw_do_contador_no_mesmo_endereco_com_imediato_assinado() {
    const CODE: u32 = 0x0000_1000;
    const BASE: u32 = 0x801D_0000;
    const COUNTER: u32 = 0x801C_F2CC;
    const OLD_COUNTER: u32 = 0x801D_F2CC;
    const VALUE: u32 = 0x1357_9BDF;

    let bios = Bios::from_bytes(vec![0; 0x80000]).expect("BIOS de teste valida");
    let mut bus = Bus::new(Ram::new(), bios);
    let mut cpu = Cpu::new();

    bus.write32::<BusWrite>(COUNTER, VALUE);
    bus.write32::<BusWrite>(OLD_COUNTER, 0xDEAD_BEEF);
    bus.write32::<BusWrite>(CODE, 0x3C02_801D);
    bus.write32::<BusWrite>(CODE + 4, 0x8C42_F2CC);
    bus.write32::<BusWrite>(CODE + 8, 0);
    cpu.pc = CODE;
    cpu.step(&mut bus);
    cpu.step(&mut bus);
    cpu.step(&mut bus);

    assert_eq!(cpu.regs[2], VALUE, "lw deve ler 0x801CF2CC, nao 0x801DF2CC");
    assert_eq!(bus.read32::<BusRead>(OLD_COUNTER), 0xDEAD_BEEF);

    bus.write32::<BusWrite>(CODE + 0x10, 0xAC22_F2CC);
    cpu.regs[1] = BASE;
    cpu.regs[2] = 1;
    cpu.pc = CODE + 0x10;
    cpu.step(&mut bus);

    assert_eq!(bus.read32::<BusRead>(COUNTER), 1);
    assert_eq!(bus.read32::<BusRead>(OLD_COUNTER), 0xDEAD_BEEF);
}
