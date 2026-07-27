use psx_core::bus::{Bus, Ram, BusRead, Bios};
use psx_core::cpu::Cpu;

const LUI_T0_1234: u32 = 0b001111_00000_01000_0001001000110100;
const ORI_T0_T0_5678: u32 = 0b001101_01000_01000_0101011001111000;

fn bus_with_bios_empty() -> Bus {
    let ram = Ram::new();
    let bios_bytes = vec![0u8; 0x80000];
    let bios = Bios::from_bytes(bios_bytes).unwrap();
    Bus::new(ram, bios)
}

#[test]
fn lui_sets_upper_and_clears_lower() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    cpu.pc = 0;
    cpu.regs[8] = 0;
    bus.write32::<BusRead>(0, LUI_T0_1234);
    cpu.step(&mut bus);
    assert_eq!(cpu.regs[8], 0x1234_0000, "LUI deve carregar imm nos 16 bits altos e zerar baixos");
    assert_eq!(cpu.pc, 4, "PC deve avançar 4 bytes");
}

#[test]
fn ori_zero_extends() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    cpu.pc = 0;
    cpu.regs[8] = 0x1234_0000;
    bus.write32::<BusRead>(0, ORI_T0_T0_5678);
    cpu.step(&mut bus);
    assert_eq!(
        cpu.regs[8],
        0x1234_5678,
        "ORI deve OR com zero-extended immediate"
    );
    assert_eq!(cpu.pc, 4, "PC deve avançar 4 bytes");
}

#[test]
fn sw_writes_to_ram_via_bus() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    cpu.pc = 0;
    cpu.regs[8] = 0;
    let sw_t0_r0_0100: u32 = 0b101011_00000_01000_0000000100000000;
    bus.write32::<BusRead>(0, sw_t0_r0_0100);
    cpu.regs[8] = 0xAABB_CCDD;
    cpu.step(&mut bus);
    let written = bus.read32::<BusRead>(0x100);
    assert_eq!(written, 0xAABB_CCDD, "SW deve escrever rt em [rs+imm]");
}

#[test]
fn r0_is_always_zero() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    cpu.pc = 0;
    let lui_r0: u32 = 0b001111_00000_00000_1111111111111111;
    bus.write32::<BusRead>(0, lui_r0);
    cpu.step(&mut bus);
    assert_eq!(cpu.regs[0], 0, "R0 deve permanecer 0 após LUI em R0");
}

#[test]
fn unknown_opcode_panics() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    cpu.pc = 0;
    let illegal: u32 = 0b111111_11111_11111_1111111111111111;
    bus.write32::<BusRead>(0, illegal);
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        cpu.step(&mut bus);
    }));
    assert!(result.is_err(), "Opcode desconhecido deve panic");
}

#[test]
fn step_advances_pc_and_fetches_next() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    cpu.pc = 0;
    cpu.regs[8] = 0;
    bus.write32::<BusRead>(0, LUI_T0_1234);
    bus.write32::<BusRead>(4, ORI_T0_T0_5678);
    cpu.step(&mut bus);
    assert_eq!(cpu.regs[8], 0x1234_0000);
    assert_eq!(cpu.pc, 4);
    cpu.step(&mut bus);
    assert_eq!(cpu.regs[8], 0x1234_5678);
    assert_eq!(cpu.pc, 8);
}
