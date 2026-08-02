use psx_core::bus::{Bios, Bus, BusRead, BusWrite, Ram};
use psx_core::cpu::Cpu;

#[test]
fn vetoriza_e_le_i_stat_por_opcodes_reais() {
    const VECTOR: u32 = 0x8000_0080;
    const SENTINEL: u32 = 0x8000_1000;
    const I_STAT: u32 = 0x1F80_1070;
    const I_MASK: u32 = 0x1F80_1074;

    let bios = Bios::from_bytes(vec![0; 0x80000]).expect("BIOS de teste valida");
    let mut bus = Bus::new(Ram::new(), bios);
    let mut cpu = Cpu::new();

    bus.write32::<BusWrite>(SENTINEL, 0xA5A5_A5A5);
    bus.write32::<BusWrite>(VECTOR, 0x3C08_1F80);
    bus.write32::<BusWrite>(VECTOR + 4, 0x3508_1070);
    bus.write32::<BusWrite>(VECTOR + 8, 0x8D09_0000);
    bus.write32::<BusWrite>(VECTOR + 12, 0x0000_0000);
    bus.write32::<BusWrite>(VECTOR + 16, 0x3C0A_8000);
    bus.write32::<BusWrite>(VECTOR + 20, 0x354A_1000);
    bus.write32::<BusWrite>(VECTOR + 24, 0xAD49_0000);
    bus.write32::<BusWrite>(I_MASK, 0x0000_000C);
    bus.irq_mut().raise(2);
    bus.irq_mut().raise(3);
    cpu.set_sr(0x0000_0401);

    cpu.step(&mut bus);
    assert_eq!(
        cpu.pc, VECTOR,
        "a IRQ habilitada deve vetorizar; docs/reference/11-interrupts.md § Interrupt Request / Execution (L45-L50)"
    );
    for _ in 0..7 {
        cpu.step(&mut bus);
    }

    assert_eq!(
        bus.read32::<BusRead>(I_STAT),
        0x0000_000C,
        "I_STAT deve manter CDROM e DMA pendentes; docs/reference/11-interrupts.md § 1F801074h I_MASK - Interrupt mask register (R/W) (L27-L39)"
    );
    assert_eq!(
        bus.read32::<BusRead>(SENTINEL),
        0x0000_000C,
        "opcodes reais devem copiar I_STAT para o sentinela; docs/reference/11-interrupts.md § 1F801074h I_MASK - Interrupt mask register (R/W) (L27-L39)"
    );
}
