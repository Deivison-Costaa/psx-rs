use psx_core::bus::BusRead;
use psx_core::cpu::Cpu;

mod support;
use support::asm::*;

// SPECIAL secondary opcodes for MULT/DIV family
const MULT: u32 = 0x18;
const MULTU: u32 = 0x19;
const DIV: u32 = 0x1A;
const DIVU: u32 = 0x1B;
const MFHI: u32 = 0x10;
const MTHI: u32 = 0x11;
const MFLO: u32 = 0x12;
const MTLO: u32 = 0x13;

// ===== MULT: signed multiply =====

#[test]
fn mult_positivo() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    cpu.pc = 0;
    cpu.regs[8] = 1000;
    cpu.regs[9] = 2000;
    bus.write32::<BusRead>(0, encode_special(MULT, 0, 9, 8));
    cpu.step(&mut bus);
    assert_eq!(cpu.hi, 0);
    assert_eq!(cpu.lo, 2_000_000);
}

#[test]
fn mult_negativo() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    cpu.pc = 0;
    cpu.regs[8] = (-1000i32) as u32;
    cpu.regs[9] = 2000u32;
    bus.write32::<BusRead>(0, encode_special(MULT, 0, 9, 8));
    cpu.step(&mut bus);
    assert_eq!(cpu.hi, 0xFFFF_FFFF);
    assert_eq!(cpu.lo, (-2_000_000i32) as u32);
}

#[test]
fn mult_64bits_hi_lo() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    cpu.pc = 0;
    cpu.regs[8] = 0x1000_0001;
    cpu.regs[9] = 0x0002_0000;
    bus.write32::<BusRead>(0, encode_special(MULT, 0, 9, 8));
    cpu.step(&mut bus);
    assert_eq!(cpu.hi, 0x0002_0000);
    assert_eq!(cpu.lo, 0x0002_0000);
}

// ===== MULTU: unsigned multiply =====

#[test]
fn multu_basico() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    cpu.pc = 0;
    cpu.regs[8] = 0x8000_0000;
    cpu.regs[9] = 2;
    bus.write32::<BusRead>(0, encode_special(MULTU, 0, 9, 8));
    cpu.step(&mut bus);
    assert_eq!(cpu.hi, 1);
    assert_eq!(cpu.lo, 0);
}

#[test]
fn multu_grande() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    cpu.pc = 0;
    cpu.regs[8] = 0xAAAA_AAAA;
    cpu.regs[9] = 3;
    bus.write32::<BusRead>(0, encode_special(MULTU, 0, 9, 8));
    cpu.step(&mut bus);
    assert_eq!(cpu.hi, 1);
    assert_eq!(cpu.lo, 0xFFFF_FFFE);
}

// ===== DIV: signed divide =====

#[test]
fn div_basico() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    cpu.pc = 0;
    cpu.regs[8] = 100;
    cpu.regs[9] = 7;
    bus.write32::<BusRead>(0, encode_special(DIV, 0, 9, 8));
    cpu.step(&mut bus);
    assert_eq!(cpu.lo, 14);
    assert_eq!(cpu.hi, 2);
}

#[test]
fn div_negativo() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    cpu.pc = 0;
    cpu.regs[8] = (-100i32) as u32;
    cpu.regs[9] = 7u32;
    bus.write32::<BusRead>(0, encode_special(DIV, 0, 9, 8));
    cpu.step(&mut bus);
    assert_eq!(cpu.lo, (-14i32) as u32);
    assert_eq!(cpu.hi, (-2i32) as u32);
}

#[test]
fn div_por_zero_rs_positivo() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    cpu.pc = 0;
    cpu.regs[8] = 100;
    cpu.regs[9] = 0;
    bus.write32::<BusRead>(0, encode_special(DIV, 0, 9, 8));
    cpu.step(&mut bus);
    assert_eq!(cpu.hi, 100);
    assert_eq!(cpu.lo, 0xFFFF_FFFF);
}

#[test]
fn div_por_zero_rs_negativo() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    cpu.pc = 0;
    cpu.regs[8] = (-50i32) as u32;
    cpu.regs[9] = 0u32;
    bus.write32::<BusRead>(0, encode_special(DIV, 0, 9, 8));
    cpu.step(&mut bus);
    assert_eq!(cpu.hi, (-50i32) as u32);
    assert_eq!(cpu.lo, 1);
}

#[test]
fn div_overflow_80000000_por_menos_1() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    cpu.pc = 0;
    cpu.regs[8] = 0x8000_0000;
    cpu.regs[9] = 0xFFFF_FFFF;
    bus.write32::<BusRead>(0, encode_special(DIV, 0, 9, 8));
    cpu.step(&mut bus);
    assert_eq!(cpu.hi, 0);
    assert_eq!(cpu.lo, 0x8000_0000);
}

// ===== DIVU: unsigned divide =====

#[test]
fn divu_basico() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    cpu.pc = 0;
    cpu.regs[8] = 100;
    cpu.regs[9] = 7;
    bus.write32::<BusRead>(0, encode_special(DIVU, 0, 9, 8));
    cpu.step(&mut bus);
    assert_eq!(cpu.lo, 14);
    assert_eq!(cpu.hi, 2);
}

#[test]
fn divu_por_zero() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    cpu.pc = 0;
    cpu.regs[8] = 100;
    cpu.regs[9] = 0;
    bus.write32::<BusRead>(0, encode_special(DIVU, 0, 9, 8));
    cpu.step(&mut bus);
    assert_eq!(cpu.hi, 100);
    assert_eq!(cpu.lo, 0xFFFF_FFFF);
}

// ===== MFHI / MFLO / MTHI / MTLO =====

#[test]
fn mfhi_le_hi() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    cpu.pc = 0;
    cpu.hi = 0xDEAD_BEEF;
    bus.write32::<BusRead>(0, encode_special(MFHI, 0, 0, 8));
    cpu.step(&mut bus);
    assert_eq!(cpu.regs[8], 0xDEAD_BEEF);
}

#[test]
fn mflo_le_lo() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    cpu.pc = 0;
    cpu.lo = 0xCAFE_BABE;
    bus.write32::<BusRead>(0, encode_special(MFLO, 0, 0, 8));
    cpu.step(&mut bus);
    assert_eq!(cpu.regs[8], 0xCAFE_BABE);
}

#[test]
fn mthi_escreve_hi() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    cpu.pc = 0;
    cpu.regs[8] = 0xAABB_CCDD;
    bus.write32::<BusRead>(0, encode_special(MTHI, 0, 0, 8));
    cpu.step(&mut bus);
    assert_eq!(cpu.hi, 0xAABB_CCDD);
}

#[test]
fn mtlo_escreve_lo() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    cpu.pc = 0;
    cpu.regs[8] = 0x1122_3344;
    bus.write32::<BusRead>(0, encode_special(MTLO, 0, 0, 8));
    cpu.step(&mut bus);
    assert_eq!(cpu.lo, 0x1122_3344);
}

#[test]
fn mfhi_r0_ignorado() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    cpu.pc = 0;
    cpu.hi = 0x1234_5678;
    bus.write32::<BusRead>(0, encode_special(MFHI, 0, 0, 0));
    cpu.step(&mut bus);
    assert_eq!(cpu.regs[0], 0);
}

#[test]
fn mult_e_depois_mflo_mfhi() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    cpu.pc = 0;
    cpu.regs[8] = 0x0001_0000;
    cpu.regs[9] = 0x0002_0001;
    bus.write32::<BusRead>(0, encode_special(MULT, 0, 9, 8));
    bus.write32::<BusRead>(4, encode_special(MFLO, 0, 0, 10));
    bus.write32::<BusRead>(8, encode_special(MFHI, 0, 0, 11));
    cpu.step(&mut bus);
    cpu.step(&mut bus);
    cpu.step(&mut bus);
    assert_eq!(cpu.regs[10], cpu.lo);
    assert_eq!(cpu.regs[11], cpu.hi);
}
