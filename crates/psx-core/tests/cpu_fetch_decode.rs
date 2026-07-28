use psx_core::bus::{Bios, Bus, BusRead, Ram};
use psx_core::cpu::Cpu;

const LUI_T0_1234: u32 = 0x3C081234;
const ORI_T0_T0_5678: u32 = 0x35085678;

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
    assert_eq!(
        cpu.regs[8], 0x1234_0000,
        "LUI deve carregar imm nos 16 bits altos e zerar baixos"
    );
    assert_eq!(cpu.pc, 4, "PC deve avancar 4 bytes");
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
        cpu.regs[8], 0x1234_5678,
        "ORI deve OR com zero-extended immediate"
    );
    assert_eq!(cpu.pc, 4, "PC deve avancar 4 bytes");
}

#[test]
fn ori_sign_extend_mutation_catcher() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    cpu.pc = 0;
    cpu.regs[8] = 0x1234_0000;
    let ori_high_bit: u32 = 0x3508FFFF;
    bus.write32::<BusRead>(0, ori_high_bit);
    cpu.step(&mut bus);
    assert_eq!(
        cpu.regs[8], 0x1234_FFFF,
        "ORI com imm=0xFFFF deve produzir 0x1234_FFFF, nao sign-extend"
    );
}

#[test]
fn sw_writes_to_ram_via_bus() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    cpu.pc = 0;
    let sw_t0_r0_0100: u32 = 0xAC080100;
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
    let lui_r0: u32 = 0x3C00FFFF;
    bus.write32::<BusRead>(0, lui_r0);
    cpu.step(&mut bus);
    assert_eq!(cpu.regs[0], 0, "R0 deve permanecer 0 apos LUI em R0");
}

#[test]
fn unknown_opcode_gera_ri() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    cpu.pc = 0;
    let illegal: u32 = 0xFFFFFFFF;
    bus.write32::<BusRead>(0, illegal);
    cpu.step(&mut bus);
    let exc = (cpu.cop0[13] >> 2) & 0x1F;
    assert_eq!(
        exc, 0x0A,
        "Opcode desconhecido deve gerar RI (0Ah), veio 0x{:1X}",
        exc
    );
    assert_eq!(cpu.pc, 0x8000_0080);
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

#[test]
fn sw_offset_negativo_e_sign_extended() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    cpu.pc = 0;
    cpu.regs[9] = 0x0000_0200;
    cpu.regs[8] = 0xDEAD_BEEF;
    let sw_t0_menos4_t1: u32 = 0xAD28_FFFC;
    bus.write32::<BusRead>(0, sw_t0_menos4_t1);
    cpu.step(&mut bus);
    assert_eq!(
        bus.read32::<BusRead>(0x1FC),
        0xDEAD_BEEF,
        "offset de 16 bits e sinalizado: -4(rs) deve escrever em rs-4"
    );
}
