use psx_core::bus::{Bios, Bus, BusRead, Ram};
use psx_core::cpu::Cpu;

fn bus_with_bios_empty() -> Bus {
    let ram = Ram::new();
    let bios_bytes = vec![0u8; 0x80000];
    let bios = Bios::from_bytes(bios_bytes).unwrap();
    Bus::new(ram, bios)
}

fn encode_shift_imm(secondary: u32, rd: u32, rt: u32, sa: u32) -> u32 {
    (rt << 16) | (rd << 11) | (sa << 6) | secondary
}

fn encode_shift_reg(secondary: u32, rd: u32, rt: u32, rs: u32) -> u32 {
    (rs << 21) | (rt << 16) | (rd << 11) | secondary
}

// SLL rd,rt,sa (secondary=0x00)
#[test]
fn sll_basico() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    cpu.pc = 0;
    cpu.regs[8] = 0x0000_0001;
    let instr = encode_shift_imm(0x00, 10, 8, 4);
    bus.write32::<BusRead>(0, instr);
    cpu.step(&mut bus);
    assert_eq!(cpu.regs[10], 0x0000_0010, "SLL: 1 << 4 = 0x10");
}

#[test]
fn sll_com_sa_zero_e_identidade() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    cpu.pc = 0;
    cpu.regs[8] = 0x0000_0001;
    let instr = encode_shift_imm(0x00, 10, 8, 0);
    bus.write32::<BusRead>(0, instr);
    cpu.step(&mut bus);
    assert_eq!(cpu.regs[10], 0x0000_0001, "SLL: 1 << 0 = 1");
}

// SRL rd,rt,sa (secondary=0x02)
#[test]
fn srl_basico() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    cpu.pc = 0;
    cpu.regs[8] = 0x0000_00F0;
    let instr = encode_shift_imm(0x02, 10, 8, 4);
    bus.write32::<BusRead>(0, instr);
    cpu.step(&mut bus);
    assert_eq!(cpu.regs[10], 0x0000_000F, "SRL: 0xF0 >> 4 = 0x0F");
}

#[test]
fn srl_logico_nao_propaga_sinal() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    cpu.pc = 0;
    cpu.regs[8] = 0x8000_0000;
    let instr = encode_shift_imm(0x02, 10, 8, 1);
    bus.write32::<BusRead>(0, instr);
    cpu.step(&mut bus);
    assert_eq!(
        cpu.regs[10], 0x4000_0000,
        "SRL: 0x80000000 >> 1 = 0x40000000 (logico)"
    );
}

// SRA rd,rt,sa (secondary=0x03)
#[test]
fn sra_aritmetico_propaga_sinal() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    cpu.pc = 0;
    cpu.regs[8] = 0x8000_0000;
    let instr = encode_shift_imm(0x03, 10, 8, 1);
    bus.write32::<BusRead>(0, instr);
    cpu.step(&mut bus);
    assert_eq!(
        cpu.regs[10], 0xC000_0000,
        "SRA: 0x80000000 >> 1 = 0xC0000000 (aritmetico)"
    );
}

#[test]
fn sra_positivo_igual_srl() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    cpu.pc = 0;
    cpu.regs[8] = 0x0000_00F0;
    let instr = encode_shift_imm(0x03, 10, 8, 4);
    bus.write32::<BusRead>(0, instr);
    cpu.step(&mut bus);
    assert_eq!(cpu.regs[10], 0x0000_000F, "SRA: 0xF0 >> 4 = 0x0F");
}

// SLLV rd,rt,rs (secondary=0x04)
#[test]
fn sllv_basico() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    cpu.pc = 0;
    cpu.regs[8] = 0x0000_0001;
    cpu.regs[9] = 0x0000_0004;
    let instr = encode_shift_reg(0x04, 10, 8, 9);
    bus.write32::<BusRead>(0, instr);
    cpu.step(&mut bus);
    assert_eq!(cpu.regs[10], 0x0000_0010, "SLLV: 1 << 4 = 0x10");
}

#[test]
fn sllv_shift_amount_mascara_0x1f() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    cpu.pc = 0;
    cpu.regs[8] = 0x0000_0001;
    cpu.regs[9] = 0x8000_0004;
    let instr = encode_shift_reg(0x04, 10, 8, 9);
    bus.write32::<BusRead>(0, instr);
    cpu.step(&mut bus);
    assert_eq!(
        cpu.regs[10], 0x0000_0010,
        "SLLV: 1 << (0x80000004 & 0x1F) = 1 << 4 = 0x10"
    );
}

// SRLV rd,rt,rs (secondary=0x06)
#[test]
fn srlv_basico() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    cpu.pc = 0;
    cpu.regs[8] = 0x0000_00F0;
    cpu.regs[9] = 0x0000_0004;
    let instr = encode_shift_reg(0x06, 10, 8, 9);
    bus.write32::<BusRead>(0, instr);
    cpu.step(&mut bus);
    assert_eq!(cpu.regs[10], 0x0000_000F, "SRLV: 0xF0 >> 4 = 0x0F");
}

#[test]
fn srlv_logico_nao_propaga_sinal() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    cpu.pc = 0;
    cpu.regs[8] = 0x8000_0000;
    cpu.regs[9] = 0x0000_0001;
    let instr = encode_shift_reg(0x06, 10, 8, 9);
    bus.write32::<BusRead>(0, instr);
    cpu.step(&mut bus);
    assert_eq!(
        cpu.regs[10], 0x4000_0000,
        "SRLV: 0x80000000 >> 1 = 0x40000000 (logico)"
    );
}

// SRAV rd,rt,rs (secondary=0x07)
#[test]
fn srav_aritmetico_propaga_sinal() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    cpu.pc = 0;
    cpu.regs[8] = 0x8000_0000;
    cpu.regs[9] = 0x0000_0001;
    let instr = encode_shift_reg(0x07, 10, 8, 9);
    bus.write32::<BusRead>(0, instr);
    cpu.step(&mut bus);
    assert_eq!(
        cpu.regs[10], 0xC000_0000,
        "SRAV: 0x80000000 >> 1 = 0xC0000000 (aritmetico)"
    );
}

// SLL $0,$0,0 = NOP
#[test]
fn sll_zero_zero_zero_e_nop() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    cpu.pc = 0;
    cpu.regs[1] = 0xDEAD_BEEF;
    let instr = encode_shift_imm(0x00, 0, 0, 0);
    bus.write32::<BusRead>(0, instr);
    cpu.step(&mut bus);
    assert_eq!(cpu.regs[0], 0, "SLL R0,R0,0 deve manter R0=0");
    assert_eq!(
        cpu.regs[1], 0xDEAD_BEEF,
        "NOP nao afeta outros registradores"
    );
}

// R0 como destino nunca muda
#[test]
fn shift_em_r0_ignorado() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    cpu.pc = 0;
    cpu.regs[8] = 0x0000_0001;
    let instr = encode_shift_imm(0x00, 0, 8, 4);
    bus.write32::<BusRead>(0, instr);
    cpu.step(&mut bus);
    assert_eq!(cpu.regs[0], 0, "SLL em R0 deve manter R0=0");
}

#[test]
fn opcode_shift_desconhecido_panics() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    cpu.pc = 0;
    let instr = encode_shift_imm(0x01, 0, 0, 0);
    bus.write32::<BusRead>(0, instr);
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        cpu.step(&mut bus);
    }));
    assert!(result.is_err(), "SPECIAL unknown secondary must panic");
}
