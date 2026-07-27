use psx_core::bus::{Bios, Bus, BusRead, Ram};
use psx_core::cpu::Cpu;

fn bus_with_bios_empty() -> Bus {
    let ram = Ram::new();
    let bios_bytes = vec![0u8; 0x80000];
    let bios = Bios::from_bytes(bios_bytes).unwrap();
    Bus::new(ram, bios)
}

fn encode_special(secondary: u32, rd: u32, rt: u32, rs: u32) -> u32 {
    (rs << 21) | (rt << 16) | (rd << 11) | secondary
}

fn encode_alu_imm(primary: u32, rt: u32, rs: u32, imm: u16) -> u32 {
    (primary << 26) | (rs << 21) | (rt << 16) | (imm as u32)
}

// SPECIAL: ADDU rd,rs,rt (secondary=0x21)
#[test]
fn addu_basico() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    cpu.pc = 0;
    cpu.regs[8] = 0x0000_0005;
    cpu.regs[9] = 0x0000_0003;
    let instr = encode_special(0x21, 10, 9, 8);
    bus.write32::<BusRead>(0, instr);
    cpu.step(&mut bus);
    assert_eq!(cpu.regs[10], 0x0000_0008, "ADDU: 5+3=8");
}

#[test]
fn addu_saturacao_32bits() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    cpu.pc = 0;
    cpu.regs[8] = 0xFFFF_FFFF;
    cpu.regs[9] = 0x0000_0001;
    let instr = encode_special(0x21, 10, 9, 8);
    bus.write32::<BusRead>(0, instr);
    cpu.step(&mut bus);
    assert_eq!(
        cpu.regs[10], 0x0000_0000,
        "ADDU: wraparound 0xFFFF_FFFF+1=0"
    );
}

#[test]
fn addu_r0_ignorado() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    cpu.pc = 0;
    cpu.regs[8] = 0x0000_0005;
    cpu.regs[9] = 0x0000_0003;
    let instr = encode_special(0x21, 0, 9, 8);
    bus.write32::<BusRead>(0, instr);
    cpu.step(&mut bus);
    assert_eq!(cpu.regs[0], 0, "ADDU em R0 deve manter R0=0");
}

// SPECIAL: SUBU rd,rs,rt (secondary=0x23)
#[test]
fn subu_basico() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    cpu.pc = 0;
    cpu.regs[8] = 0x0000_000A;
    cpu.regs[9] = 0x0000_0003;
    let instr = encode_special(0x23, 10, 9, 8);
    bus.write32::<BusRead>(0, instr);
    cpu.step(&mut bus);
    assert_eq!(cpu.regs[10], 0x0000_0007, "SUBU: 10-3=7");
}

#[test]
fn subu_wraparound() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    cpu.pc = 0;
    cpu.regs[8] = 0x0000_0000;
    cpu.regs[9] = 0x0000_0001;
    let instr = encode_special(0x23, 10, 9, 8);
    bus.write32::<BusRead>(0, instr);
    cpu.step(&mut bus);
    assert_eq!(cpu.regs[10], 0xFFFF_FFFF, "SUBU: 0-1=0xFFFF_FFFF");
}

// SPECIAL: AND rd,rs,rt (secondary=0x24)
#[test]
fn and_basico() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    cpu.pc = 0;
    cpu.regs[8] = 0x00FF_00FF;
    cpu.regs[9] = 0x0F0F_0F0F;
    let instr = encode_special(0x24, 10, 9, 8);
    bus.write32::<BusRead>(0, instr);
    cpu.step(&mut bus);
    assert_eq!(cpu.regs[10], 0x000F_000F, "AND: 0x00FF00FF AND 0x0F0F0F0F");
}

// SPECIAL: OR rd,rs,rt (secondary=0x25)
#[test]
fn or_basico() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    cpu.pc = 0;
    cpu.regs[8] = 0x00FF_00FF;
    cpu.regs[9] = 0x0F0F_0F0F;
    let instr = encode_special(0x25, 10, 9, 8);
    bus.write32::<BusRead>(0, instr);
    cpu.step(&mut bus);
    assert_eq!(cpu.regs[10], 0x0FFF_0FFF, "OR: 0x00FF00FF OR 0x0F0F0F0F");
}

// SPECIAL: XOR rd,rs,rt (secondary=0x26)
#[test]
fn xor_basico() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    cpu.pc = 0;
    cpu.regs[8] = 0x00FF_00FF;
    cpu.regs[9] = 0x0F0F_0F0F;
    let instr = encode_special(0x26, 10, 9, 8);
    bus.write32::<BusRead>(0, instr);
    cpu.step(&mut bus);
    assert_eq!(cpu.regs[10], 0x0FF0_0FF0, "XOR: 0x00FF00FF XOR 0x0F0F0F0F");
}

// SPECIAL: NOR rd,rs,rt (secondary=0x27)
#[test]
fn nor_basico() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    cpu.pc = 0;
    cpu.regs[8] = 0x0000_0000;
    cpu.regs[9] = 0x0000_0000;
    let instr = encode_special(0x27, 10, 9, 8);
    bus.write32::<BusRead>(0, instr);
    cpu.step(&mut bus);
    assert_eq!(cpu.regs[10], 0xFFFF_FFFF, "NOR: 0 NOR 0 = 0xFFFF_FFFF");
}

#[test]
fn nor_nao_e_or() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    cpu.pc = 0;
    cpu.regs[8] = 0x0F0F_0F0F;
    cpu.regs[9] = 0x00FF_00FF;
    let instr = encode_special(0x27, 10, 9, 8);
    bus.write32::<BusRead>(0, instr);
    cpu.step(&mut bus);
    assert_eq!(
        cpu.regs[10], 0xF000_F000,
        "NOR: ~(0x0F0F0F0F OR 0x00FF00FF) = 0xF000_F000"
    );
}

// SPECIAL: SLT rd,rs,rt (secondary=0x2A) - signed comparison
#[test]
fn slt_rs_menor_rt_signed() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    cpu.pc = 0;
    cpu.regs[8] = 0xFFFF_FFFF;
    cpu.regs[9] = 0x0000_0000;
    let instr = encode_special(0x2A, 10, 9, 8);
    bus.write32::<BusRead>(0, instr);
    cpu.step(&mut bus);
    assert_eq!(cpu.regs[10], 1, "SLT: -1 < 0 signed -> 1");
}

#[test]
fn slt_rs_nao_menor_rt_signed() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    cpu.pc = 0;
    cpu.regs[8] = 0x0000_0000;
    cpu.regs[9] = 0xFFFF_FFFF;
    let instr = encode_special(0x2A, 10, 9, 8);
    bus.write32::<BusRead>(0, instr);
    cpu.step(&mut bus);
    assert_eq!(cpu.regs[10], 0, "SLT: 0 < -1 signed -> 0");
}

// SPECIAL: SLTU rd,rs,rt (secondary=0x2B) - unsigned comparison
#[test]
fn sltu_rs_menor_rt_unsigned() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    cpu.pc = 0;
    cpu.regs[8] = 0x0000_0000;
    cpu.regs[9] = 0xFFFF_FFFF;
    let instr = encode_special(0x2B, 10, 9, 8);
    bus.write32::<BusRead>(0, instr);
    cpu.step(&mut bus);
    assert_eq!(cpu.regs[10], 1, "SLTU: 0 < 0xFFFF_FFFF unsigned -> 1");
}

#[test]
fn sltu_rs_maior_rt_unsigned() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    cpu.pc = 0;
    cpu.regs[8] = 0xFFFF_FFFF;
    cpu.regs[9] = 0x0000_0000;
    let instr = encode_special(0x2B, 10, 9, 8);
    bus.write32::<BusRead>(0, instr);
    cpu.step(&mut bus);
    assert_eq!(cpu.regs[10], 0, "SLTU: 0xFFFF_FFFF < 0 unsigned -> 0");
}

// ALU-IMM: ADDIU rt,rs,imm (primary=0x09) - sign-extended
#[test]
fn addiu_basico() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    cpu.pc = 0;
    cpu.regs[8] = 0x0000_0005;
    let instr = encode_alu_imm(0x09, 10, 8, 0x0003);
    bus.write32::<BusRead>(0, instr);
    cpu.step(&mut bus);
    assert_eq!(cpu.regs[10], 0x0000_0008, "ADDIU: 5+3=8");
}

#[test]
fn addiu_imm_negativo() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    cpu.pc = 0;
    cpu.regs[8] = 0x0000_0005;
    let instr = encode_alu_imm(0x09, 10, 8, 0xFFFC);
    bus.write32::<BusRead>(0, instr);
    cpu.step(&mut bus);
    assert_eq!(cpu.regs[10], 0x0000_0001, "ADDIU: 5 + (-4) = 1");
}

#[test]
fn addiu_sign_extends_imm() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    cpu.pc = 0;
    cpu.regs[8] = 0x0000_0000;
    let instr = encode_alu_imm(0x09, 10, 8, 0x8000);
    bus.write32::<BusRead>(0, instr);
    cpu.step(&mut bus);
    assert_eq!(
        cpu.regs[10], 0xFFFF_8000,
        "ADDIU com 0x8000: sign-extend -> 0xFFFF_8000 + 0 = 0xFFFF_8000"
    );
}

// ALU-IMM: ADDI rt,rs,imm (primary=0x08) - same as ADDIU for now (no trap)
#[test]
fn addi_basico() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    cpu.pc = 0;
    cpu.regs[8] = 0x0000_0005;
    let instr = encode_alu_imm(0x08, 10, 8, 0x0003);
    bus.write32::<BusRead>(0, instr);
    cpu.step(&mut bus);
    assert_eq!(cpu.regs[10], 0x0000_0008, "ADDI: 5+3=8");
}

#[test]
fn addi_sign_extends_imm() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    cpu.pc = 0;
    cpu.regs[8] = 0x0000_0000;
    let instr = encode_alu_imm(0x08, 10, 8, 0x8000);
    bus.write32::<BusRead>(0, instr);
    cpu.step(&mut bus);
    assert_eq!(
        cpu.regs[10], 0xFFFF_8000,
        "ADDI com 0x8000: sign-extend -> 0xFFFF_8000"
    );
}

// ALU-IMM: ANDI rt,rs,imm (primary=0x0C) - zero-extended
#[test]
fn andi_zero_extends() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    cpu.pc = 0;
    cpu.regs[8] = 0xFFFF_FFFF;
    let instr = encode_alu_imm(0x0C, 10, 8, 0xFFFF);
    bus.write32::<BusRead>(0, instr);
    cpu.step(&mut bus);
    assert_eq!(
        cpu.regs[10], 0x0000_FFFF,
        "ANDI: 0xFFFF_FFFF AND 0x0000_FFFF = 0x0000_FFFF"
    );
}

// ALU-IMM: XORI rt,rs,imm (primary=0x0E) - zero-extended
#[test]
fn xori_zero_extends() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    cpu.pc = 0;
    cpu.regs[8] = 0xFFFF_0000;
    let instr = encode_alu_imm(0x0E, 10, 8, 0xFFFF);
    bus.write32::<BusRead>(0, instr);
    cpu.step(&mut bus);
    assert_eq!(
        cpu.regs[10], 0xFFFF_FFFF,
        "XORI: 0xFFFF_0000 XOR 0x0000_FFFF = 0xFFFF_FFFF"
    );
}

// ALU-IMM: SLTI rt,rs,imm (primary=0x0A) - sign-extended, signed compare
#[test]
fn slti_rs_menor_imm_signed() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    cpu.pc = 0;
    cpu.regs[8] = 0xFFFF_FFFF;
    let instr = encode_alu_imm(0x0A, 10, 8, 0x0000);
    bus.write32::<BusRead>(0, instr);
    cpu.step(&mut bus);
    assert_eq!(cpu.regs[10], 1, "SLTI: -1 < 0 signed -> 1");
}

#[test]
fn slti_rs_maior_imm_signed() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    cpu.pc = 0;
    cpu.regs[8] = 0x0000_0000;
    let instr = encode_alu_imm(0x0A, 10, 8, 0xFFFF);
    bus.write32::<BusRead>(0, instr);
    cpu.step(&mut bus);
    assert_eq!(
        cpu.regs[10], 0,
        "SLTI: 0 < -1 signed -> 0 (imm 0xFFFF sign-extends to -1)"
    );
}

// ALU-IMM: SLTIU rt,rs,imm (primary=0x0B) - sign-extended, unsigned compare
#[test]
fn sltiu_rs_menor_imm_unsigned() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    cpu.pc = 0;
    cpu.regs[8] = 0x0000_0000;
    let instr = encode_alu_imm(0x0B, 10, 8, 0x8000);
    bus.write32::<BusRead>(0, instr);
    cpu.step(&mut bus);
    assert_eq!(
        cpu.regs[10], 1,
        "SLTIU: 0 < 0xFFFF_8000 unsigned -> 1 (imm sign-extends to 0xFFFF_8000)"
    );
}

#[test]
fn sltiu_imm_alto_comparacao_unsigned() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    cpu.pc = 0;
    cpu.regs[8] = 0x0000_0005;
    let instr = encode_alu_imm(0x0B, 10, 8, 0x0003);
    bus.write32::<BusRead>(0, instr);
    cpu.step(&mut bus);
    assert_eq!(cpu.regs[10], 0, "SLTIU: 5 < 3 unsigned -> 0");
}

#[test]
fn opcode_desconhecido_especial_panics() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    cpu.pc = 0;
    let instr = encode_special(0x00, 0, 0, 0);
    bus.write32::<BusRead>(0, instr);
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        cpu.step(&mut bus);
    }));
    assert!(result.is_err(), "SPECIAL unknown secondary must panic");
}
