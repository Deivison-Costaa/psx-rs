use psx_core::bus::{Bios, Bus, BusRead, BusWrite, Ram};
use psx_core::cpu::Cpu;

#[test]
fn ack_de_vblank_ocorre_antes_da_consulta_de_evento() {
    const SETUP: u32 = 0x0000_4A10;
    const ACK: u32 = 0x0000_4A1C;
    const I_STAT: u32 = 0x1F80_1070;
    const I_STAT_PTR: u32 = 0x0000_725C;
    const EVENT_GATE: u32 = 0x0000_74BC;

    let bios = Bios::from_bytes(vec![0; 0x80000]).expect("BIOS de teste valida");
    let mut bus = Bus::new(Ram::new(), bios);
    let mut cpu = Cpu::new();

    bus.write32::<BusWrite>(I_STAT_PTR, I_STAT);
    bus.write32::<BusWrite>(EVENT_GATE, 0);
    bus.irq_mut().raise(0);
    bus.irq_mut().raise(1);
    bus.write32::<BusWrite>(SETUP, 0x3C08_0000);
    bus.write32::<BusWrite>(SETUP + 4, 0x8D08_725C);
    bus.write32::<BusWrite>(SETUP + 8, 0x2419_FFFE);
    bus.write32::<BusWrite>(ACK, 0xAD19_0000);

    cpu.pc = SETUP;
    cpu.step(&mut bus);
    cpu.step(&mut bus);
    cpu.step(&mut bus);
    assert_eq!(cpu.pc, ACK);
    assert_eq!(bus.read32::<BusRead>(I_STAT), 0x0000_0003);

    cpu.step(&mut bus);

    assert_eq!(cpu.pc, ACK + 4);
    assert_eq!(bus.read32::<BusRead>(I_STAT), 0x0000_0002);
}
