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

fn encode_load_store(primary: u32, rt: u32, rs: u32, imm: u16) -> u32 {
    (primary << 26) | (rs << 21) | (rt << 16) | (imm as u32)
}

fn nop() -> u32 {
    encode_special(0x00, 0, 0, 0)
}

fn setup_patterns(bus: &mut Bus) {
    // aligned words 0x1000..0x100F
    bus.write32::<BusRead>(0x1000, 0xAABB_CCDD);
    bus.write32::<BusRead>(0x1004, 0x1122_3344);
}

// LWL/LWR — load word left/right

#[test]
fn lwl_addr_aligned_offset_0_upper_8bits() {
    let mut bus = bus_with_bios_empty();
    setup_patterns(&mut bus);
    let mut cpu = Cpu::new();
    cpu.pc = 0;
    cpu.regs[8] = 0x1000;         // aligned addr, offset 0
    cpu.regs[10] = 0xFFFF_FFFF;   // rt preloaded
    // LWL r10, 0(r8) — offset 0: upper 8 bits, rest intact
    bus.write32::<BusRead>(0, encode_load_store(0x22, 10, 8, 0x0000));
    bus.write32::<BusRead>(4, nop());
    cpu.step(&mut bus);
    cpu.step(&mut bus);
    // memory[0x1000] = 0xAABB_CCDD, offset 0: upper 8 bits = 0xAA,
    // rt merge: upper 8 bits of word mem, lower 24 bits of rt
    // merged = 0xAA | (0xFFFF_FFFF & 0x00FF_FFFF) = 0xAAFF_FFFF
    assert_eq!(cpu.regs[10], 0xAAFF_FFFF);
}

#[test]
fn lwl_addr_offset_1_upper_16bits() {
    let mut bus = bus_with_bios_empty();
    setup_patterns(&mut bus);
    let mut cpu = Cpu::new();
    cpu.pc = 0;
    cpu.regs[8] = 0x1001;         // offset 1
    cpu.regs[10] = 0xFFFF_FFFF;
    bus.write32::<BusRead>(0, encode_load_store(0x22, 10, 8, 0x0000));
    bus.write32::<BusRead>(4, nop());
    cpu.step(&mut bus);
    cpu.step(&mut bus);
    // aligned word at 0x1000, offset 1: upper 16 bits of word = 0xAABB,
    // merge with rt[31:16] replaced, rt[15:0] intact
    assert_eq!(cpu.regs[10], 0xAABB_FFFF);
}

#[test]
fn lwl_addr_offset_2_upper_24bits() {
    let mut bus = bus_with_bios_empty();
    setup_patterns(&mut bus);
    let mut cpu = Cpu::new();
    cpu.pc = 0;
    cpu.regs[8] = 0x1002;
    cpu.regs[10] = 0xFFFF_FFFF;
    bus.write32::<BusRead>(0, encode_load_store(0x22, 10, 8, 0x0000));
    bus.write32::<BusRead>(4, nop());
    cpu.step(&mut bus);
    cpu.step(&mut bus);
    assert_eq!(cpu.regs[10], 0xAABB_CCFF);
}

#[test]
fn lwl_addr_offset_3_whole_32bits() {
    let mut bus = bus_with_bios_empty();
    setup_patterns(&mut bus);
    let mut cpu = Cpu::new();
    cpu.pc = 0;
    cpu.regs[8] = 0x1003;
    cpu.regs[10] = 0xFFFF_FFFF;
    bus.write32::<BusRead>(0, encode_load_store(0x22, 10, 8, 0x0000));
    bus.write32::<BusRead>(4, nop());
    cpu.step(&mut bus);
    cpu.step(&mut bus);
    // offset 3: whole 32 bits transferred
    assert_eq!(cpu.regs[10], 0xAABB_CCDD);
}

#[test]
fn lwr_addr_offset_0_whole_32bits() {
    let mut bus = bus_with_bios_empty();
    setup_patterns(&mut bus);
    let mut cpu = Cpu::new();
    cpu.pc = 0;
    cpu.regs[8] = 0x1000;
    cpu.regs[10] = 0xFFFF_FFFF;
    bus.write32::<BusRead>(0, encode_load_store(0x26, 10, 8, 0x0000));
    bus.write32::<BusRead>(4, nop());
    cpu.step(&mut bus);
    cpu.step(&mut bus);
    assert_eq!(cpu.regs[10], 0xAABB_CCDD);
}

#[test]
fn lwr_addr_offset_1_lower_24bits() {
    let mut bus = bus_with_bios_empty();
    setup_patterns(&mut bus);
    let mut cpu = Cpu::new();
    cpu.pc = 0;
    cpu.regs[8] = 0x1001;
    cpu.regs[10] = 0xFFFF_FFFF;
    bus.write32::<BusRead>(0, encode_load_store(0x26, 10, 8, 0x0000));
    bus.write32::<BusRead>(4, nop());
    cpu.step(&mut bus);
    cpu.step(&mut bus);
    // offset 1: lower 24 bits from mem word, upper 8 from rt
    assert_eq!(cpu.regs[10], 0xFF_BB_CCDD);
}

#[test]
fn lwr_addr_offset_2_lower_16bits() {
    let mut bus = bus_with_bios_empty();
    setup_patterns(&mut bus);
    let mut cpu = Cpu::new();
    cpu.pc = 0;
    cpu.regs[8] = 0x1002;
    cpu.regs[10] = 0xFFFF_FFFF;
    bus.write32::<BusRead>(0, encode_load_store(0x26, 10, 8, 0x0000));
    bus.write32::<BusRead>(4, nop());
    cpu.step(&mut bus);
    cpu.step(&mut bus);
    // offset 2: lower 16 bits from mem word, upper 16 from rt
    assert_eq!(cpu.regs[10], 0xFFFF_CCDD);
}

#[test]
fn lwr_addr_offset_3_lower_8bits() {
    let mut bus = bus_with_bios_empty();
    setup_patterns(&mut bus);
    let mut cpu = Cpu::new();
    cpu.pc = 0;
    cpu.regs[8] = 0x1003;
    cpu.regs[10] = 0xFFFF_FFFF;
    bus.write32::<BusRead>(0, encode_load_store(0x26, 10, 8, 0x0000));
    bus.write32::<BusRead>(4, nop());
    cpu.step(&mut bus);
    cpu.step(&mut bus);
    // offset 3: lower 8 bits from mem word, upper 24 from rt
    assert_eq!(cpu.regs[10], 0xFFFF_FFDD);
}

#[test]
fn lwl_lwr_pair_load_unaligned_word() {
    // Classic pair: LWL then LWR on same rt to load a misaligned word
    let mut bus = bus_with_bios_empty();
    setup_patterns(&mut bus);
    let mut cpu = Cpu::new();
    cpu.pc = 0;
    cpu.regs[8] = 0x1002;         // word straddles 0x1002..0x1005
    cpu.regs[10] = 0xFFFF_FFFF;
    // LWL r10, 3(r8) — offset (0x1002+3)&3 = 1 → upper 16 bits from 0x1004
    // Actually: standard idiom is LWL then LWR with offset+3 and offset
    // LWL r10, 3(r8): addr=0x1005, aligned=0x1004, offset=1 → upper 16 bits of 0x11223344 = 0x1122
    bus.write32::<BusRead>(0, encode_load_store(0x22, 10, 8, 0x0003));
    // LWR r10, 0(r8): addr=0x1002, aligned=0x1000, offset=2 → lower 16 bits of 0xAABBCCDD = 0xCCDD
    bus.write32::<BusRead>(4, encode_load_store(0x26, 10, 8, 0x0000));
    // no load delay between lwl/lwr (per spec), but need nop before reading rt
    bus.write32::<BusRead>(8, nop());
    cpu.step(&mut bus);
    cpu.step(&mut bus);
    cpu.step(&mut bus);
    // merged: upper 16 from lwl (0x1122), lower 16 from lwr (0xCCDD)
    // But lwl wrote rt before lwr: lwl is step 1 (offset 1 → 0x1122_FFFF),
    // then lwr is step 2 (offset 2 → rt[15:0] from mem[15:0] = 0xCCDD, rt[31:16] intact = 0x1122)
    assert_eq!(cpu.regs[10], 0x1122_CCDD);
}

// SWL/SWR — store word left/right

#[test]
fn swl_addr_offset_0_upper_8bits() {
    let mut bus = bus_with_bios_empty();
    setup_patterns(&mut bus);
    let mut cpu = Cpu::new();
    cpu.pc = 0;
    cpu.regs[8] = 0x1000;
    cpu.regs[10] = 0xAABB_CCDD;
    bus.write32::<BusRead>(0, encode_load_store(0x2A, 10, 8, 0x0000));
    bus.write32::<BusRead>(4, nop());
    cpu.step(&mut bus);
    cpu.step(&mut bus);
    // SWL offset 0: upper 8 bits of rt → mem byte at [0x1000]
    // mem[0x1000..3] was 0xAABBCCDD → becomes 0xAABBCC**AA** — wait no.
    // SWL offset 0: transfers upper 8 bits of rt to [aligned+0]
    // rt[31:24] = 0xAA → mem[0x1000] = 0xAA
    // mem[0x1001..3] intact = [0xBB, 0xCC, 0xDD]
    let word = bus.read32::<BusRead>(0x1000);
    assert_eq!(word, 0xAABB_CCAA);
}

#[test]
fn swl_addr_offset_1_upper_16bits() {
    let mut bus = bus_with_bios_empty();
    setup_patterns(&mut bus);
    let mut cpu = Cpu::new();
    cpu.pc = 0;
    cpu.regs[8] = 0x1001;
    cpu.regs[10] = 0xAABB_CCDD;
    bus.write32::<BusRead>(0, encode_load_store(0x2A, 10, 8, 0x0000));
    bus.write32::<BusRead>(4, nop());
    cpu.step(&mut bus);
    cpu.step(&mut bus);
    // SWL offset 1: upper 16 bits of rt to [aligned+0..1]
    // rt[31:16] = 0xAABB → mem[0x1000] = 0xBB, mem[0x1001] = 0xAA
    let word = bus.read32::<BusRead>(0x1000);
    assert_eq!(word, 0xAA_CCDD_BB);  // wait, little-endian
    // memory at 0x1000 after: [0x1000]=BB, [0x1001]=AA, [0x1002]=CC, [0x1003]=DD
    assert_eq!(bus.read8::<BusRead>(0x1000), 0xBB);
    assert_eq!(bus.read8::<BusRead>(0x1001), 0xAA);
    assert_eq!(bus.read8::<BusRead>(0x1002), 0xCC);
    assert_eq!(bus.read8::<BusRead>(0x1003), 0xDD);
}

#[test]
fn swl_addr_offset_2_upper_24bits() {
    let mut bus = bus_with_bios_empty();
    setup_patterns(&mut bus);
    let mut cpu = Cpu::new();
    cpu.pc = 0;
    cpu.regs[8] = 0x1002;
    cpu.regs[10] = 0xAABB_CCDD;
    bus.write32::<BusRead>(0, encode_load_store(0x2A, 10, 8, 0x0000));
    bus.write32::<BusRead>(4, nop());
    cpu.step(&mut bus);
    cpu.step(&mut bus);
    // SWL offset 2: upper 24 bits of rt to [aligned+0..2]
    let d0 = bus.read8::<BusRead>(0x1000);
    let d1 = bus.read8::<BusRead>(0x1001);
    let d2 = bus.read8::<BusRead>(0x1002);
    let d3 = bus.read8::<BusRead>(0x1003);
    assert_eq!(d0, 0xDD);
    assert_eq!(d1, 0xCC);
    assert_eq!(d2, 0xBB);
    assert_eq!(d3, 0xDD, "byte at offset 3 intact");
}

#[test]
fn swl_addr_offset_3_whole_32bits() {
    let mut bus = bus_with_bios_empty();
    setup_patterns(&mut bus);
    let mut cpu = Cpu::new();
    cpu.pc = 0;
    cpu.regs[8] = 0x1003;
    cpu.regs[10] = 0xAABB_CCDD;
    bus.write32::<BusRead>(0, encode_load_store(0x2A, 10, 8, 0x0000));
    bus.write32::<BusRead>(4, nop());
    cpu.step(&mut bus);
    cpu.step(&mut bus);
    let word = bus.read32::<BusRead>(0x1000);
    assert_eq!(word, 0xAABB_CCDD);
}

#[test]
fn swr_addr_offset_0_whole_32bits() {
    let mut bus = bus_with_bios_empty();
    setup_patterns(&mut bus);
    let mut cpu = Cpu::new();
    cpu.pc = 0;
    cpu.regs[8] = 0x1000;
    cpu.regs[10] = 0xAABB_CCDD;
    bus.write32::<BusRead>(0, encode_load_store(0x2E, 10, 8, 0x0000));
    bus.write32::<BusRead>(4, nop());
    cpu.step(&mut bus);
    cpu.step(&mut bus);
    let word = bus.read32::<BusRead>(0x1000);
    assert_eq!(word, 0xAABB_CCDD);
}

#[test]
fn swr_addr_offset_1_lower_24bits() {
    let mut bus = bus_with_bios_empty();
    setup_patterns(&mut bus);
    let mut cpu = Cpu::new();
    cpu.pc = 0;
    cpu.regs[8] = 0x1001;
    cpu.regs[10] = 0xAABB_CCDD;
    bus.write32::<BusRead>(0, encode_load_store(0x2E, 10, 8, 0x0000));
    bus.write32::<BusRead>(4, nop());
    cpu.step(&mut bus);
    cpu.step(&mut bus);
    // SWR offset 1: lower 24 bits of rt to [aligned+1..3]
    // rt lower 24 bits = 0xBB_CCDD (little-endian: DD CC BB)
    let d0 = bus.read8::<BusRead>(0x1000);
    let d1 = bus.read8::<BusRead>(0x1001);
    let d2 = bus.read8::<BusRead>(0x1002);
    let d3 = bus.read8::<BusRead>(0x1003);
    assert_eq!(d0, 0xAA, "byte at offset 0 intact (was 0xAA from setup)");
    assert_eq!(d1, 0xDD);
    assert_eq!(d2, 0xCC);
    assert_eq!(d3, 0xBB);
}

#[test]
fn swr_addr_offset_2_lower_16bits() {
    let mut bus = bus_with_bios_empty();
    setup_patterns(&mut bus);
    let mut cpu = Cpu::new();
    cpu.pc = 0;
    cpu.regs[8] = 0x1002;
    cpu.regs[10] = 0xAABB_CCDD;
    bus.write32::<BusRead>(0, encode_load_store(0x2E, 10, 8, 0x0000));
    bus.write32::<BusRead>(4, nop());
    cpu.step(&mut bus);
    cpu.step(&mut bus);
    let d0 = bus.read8::<BusRead>(0x1000);
    let d1 = bus.read8::<BusRead>(0x1001);
    let d2 = bus.read8::<BusRead>(0x1002);
    let d3 = bus.read8::<BusRead>(0x1003);
    assert_eq!(d0, 0xAA);
    assert_eq!(d1, 0xBB);
    assert_eq!(d2, 0xDD);
    assert_eq!(d3, 0xCC);
}

#[test]
fn swr_addr_offset_3_lower_8bits() {
    let mut bus = bus_with_bios_empty();
    setup_patterns(&mut bus);
    let mut cpu = Cpu::new();
    cpu.pc = 0;
    cpu.regs[8] = 0x1003;
    cpu.regs[10] = 0xAABB_CCDD;
    bus.write32::<BusRead>(0, encode_load_store(0x2E, 10, 8, 0x0000));
    bus.write32::<BusRead>(4, nop());
    cpu.step(&mut bus);
    cpu.step(&mut bus);
    let d0 = bus.read8::<BusRead>(0x1000);
    let d1 = bus.read8::<BusRead>(0x1001);
    let d2 = bus.read8::<BusRead>(0x1002);
    let d3 = bus.read8::<BusRead>(0x1003);
    assert_eq!(d0, 0xAA);
    assert_eq!(d1, 0xBB);
    assert_eq!(d2, 0xCC);
    assert_eq!(d3, 0xDD);
}

#[test]
fn lwl_preserves_rt_when_offset_0() {
    // When offset=0, only upper 8 bits of rt are replaced
    let mut bus = bus_with_bios_empty();
    setup_patterns(&mut bus);
    let mut cpu = Cpu::new();
    cpu.pc = 0;
    cpu.regs[8] = 0x1000;
    cpu.regs[10] = 0x1234_5678;
    bus.write32::<BusRead>(0, encode_load_store(0x22, 10, 8, 0x0000));
    bus.write32::<BusRead>(4, nop());
    cpu.step(&mut bus);
    cpu.step(&mut bus);
    // upper 8 replaced with 0xAA, lower 24 = 0x34_5678 intact
    assert_eq!(cpu.regs[10], 0xAA34_5678);
}
