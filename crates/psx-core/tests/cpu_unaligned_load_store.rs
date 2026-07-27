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
    // aligned words at 0x1000 and 0x1004
    bus.write32::<BusRead>(0x1000, 0xAABB_CCDD);
    bus.write32::<BusRead>(0x1004, 0x1122_3344);
}

// ---------------------------------------------------------------------------
// LWL (Load Word Left) — primary 0x22
// ---------------------------------------------------------------------------

#[test]
fn lwl_offset_0_upper_8bits() {
    let mut bus = bus_with_bios_empty();
    setup_patterns(&mut bus);
    let mut cpu = Cpu::new();
    cpu.pc = 0;
    cpu.regs[8] = 0x1000;
    cpu.regs[10] = 0xFFFF_FFFF;
    bus.write32::<BusRead>(0, encode_load_store(0x22, 10, 8, 0x0000));
    bus.write32::<BusRead>(4, nop());
    cpu.step(&mut bus);
    cpu.step(&mut bus);
    // LWL offset 0: upper 8 bits of mem word → rt[31:24], rt[23:0] intact
    // mem word at 0x1000 = 0xAABB_CCDD; upper 8 = 0xAA; rt lower 24 = 0xFFFF_FF
    assert_eq!(cpu.regs[10], 0xAAFF_FFFF);
}

#[test]
fn lwl_offset_1_upper_16bits() {
    let mut bus = bus_with_bios_empty();
    setup_patterns(&mut bus);
    let mut cpu = Cpu::new();
    cpu.pc = 0;
    cpu.regs[8] = 0x1001;
    cpu.regs[10] = 0xFFFF_FFFF;
    bus.write32::<BusRead>(0, encode_load_store(0x22, 10, 8, 0x0000));
    bus.write32::<BusRead>(4, nop());
    cpu.step(&mut bus);
    cpu.step(&mut bus);
    // LWL offset 1: upper 16 bits of mem word → rt[31:16], rt[15:0] intact
    assert_eq!(cpu.regs[10], 0xAABB_FFFF);
}

#[test]
fn lwl_offset_2_upper_24bits() {
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
fn lwl_offset_3_whole_32bits() {
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
    assert_eq!(cpu.regs[10], 0xAABB_CCDD);
}

#[test]
fn lwl_preserves_lower_bits_rt() {
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
    // offset 0: upper 8 replaced with 0xAA, lower 24 = 0x34_5678 intact
    assert_eq!(cpu.regs[10], 0xAA34_5678);
}

// ---------------------------------------------------------------------------
// LWR (Load Word Right) — primary 0x26
// ---------------------------------------------------------------------------

#[test]
fn lwr_offset_0_whole_32bits() {
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
fn lwr_offset_1_lower_24bits() {
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
    // offset 1: lower 24 bits from mem word → rt[23:0], rt[31:24] intact
    assert_eq!(cpu.regs[10], 0xFFBB_CCDD);
}

#[test]
fn lwr_offset_2_lower_16bits() {
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
    assert_eq!(cpu.regs[10], 0xFFFF_CCDD);
}

#[test]
fn lwr_offset_3_lower_8bits() {
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
    assert_eq!(cpu.regs[10], 0xFFFF_FFDD);
}

// ---------------------------------------------------------------------------
// LWL + LWR pair: carregar palavra desalinhada
// ---------------------------------------------------------------------------

#[test]
fn lwl_lwr_pair_different_regs() {
    let mut bus = bus_with_bios_empty();
    setup_patterns(&mut bus);
    let mut cpu = Cpu::new();
    cpu.pc = 0;
    cpu.regs[8] = 0x1002;
    cpu.regs[9] = 0x0000_0000;
    // LWL r9, 3(r8)  → addr=0x1005, aligned=0x1004, offset=1
    //   mem word at 0x1004 = 0x1122_3344
    //   LWL offset 1: upper 16 bits of mem → rt[31:16] = 0x1122
    bus.write32::<BusRead>(0, encode_load_store(0x22, 9, 8, 0x0003));
    // LWR r10, 0(r8)  → separate rt, so no conflict
    //   addr=0x1002, aligned=0x1000, offset=2 → lower 16 bits of mem word = 0xCCDD
    bus.write32::<BusRead>(4, encode_load_store(0x26, 10, 8, 0x0000));
    bus.write32::<BusRead>(8, nop());
    cpu.step(&mut bus);
    cpu.step(&mut bus);
    cpu.step(&mut bus);
    assert_eq!(cpu.regs[9], 0x1122_0000);
    assert_eq!(cpu.regs[10], 0x0000_CCDD);
}

// ---------------------------------------------------------------------------
// SWL (Store Word Left) — primary 0x2A
// ---------------------------------------------------------------------------

// SWL: read aligned word from memory, merge rt's upper bits in, write back.
// We write a distinct rt value to make the merge visible.

#[test]
fn swl_offset_0_upper_8bits() {
    let mut bus = bus_with_bios_empty();
    setup_patterns(&mut bus);
    let mut cpu = Cpu::new();
    cpu.pc = 0;
    cpu.regs[8] = 0x1000;
    cpu.regs[10] = 0xDEAD_BEEF;
    bus.write32::<BusRead>(0, encode_load_store(0x2A, 10, 8, 0x0000));
    bus.write32::<BusRead>(4, nop());
    cpu.step(&mut bus);
    cpu.step(&mut bus);
    // aligned=0x1000, offset=0
    // old mem = 0xAABB_CCDD, rt[31:24] = 0xDE
    // merged = (0x00BB_CCDD) | (0xDE00_0000) = 0xDEBB_CCDD
    assert_eq!(bus.read32::<BusRead>(0x1000), 0xDEBB_CCDD);
}

#[test]
fn swl_offset_1_upper_16bits() {
    let mut bus = bus_with_bios_empty();
    setup_patterns(&mut bus);
    let mut cpu = Cpu::new();
    cpu.pc = 0;
    cpu.regs[8] = 0x1001;
    cpu.regs[10] = 0xDEAD_BEEF;
    bus.write32::<BusRead>(0, encode_load_store(0x2A, 10, 8, 0x0000));
    bus.write32::<BusRead>(4, nop());
    cpu.step(&mut bus);
    cpu.step(&mut bus);
    // aligned=0x1000, offset=1
    // old mem = 0xAABB_CCDD, rt[31:16] = 0xDEAD
    // merged = (0x0000_CCDD) | (0xDEAD_0000) = 0xDEAD_CCDD
    assert_eq!(bus.read32::<BusRead>(0x1000), 0xDEAD_CCDD);
}

#[test]
fn swl_offset_2_upper_24bits() {
    let mut bus = bus_with_bios_empty();
    setup_patterns(&mut bus);
    let mut cpu = Cpu::new();
    cpu.pc = 0;
    cpu.regs[8] = 0x1002;
    cpu.regs[10] = 0xDEAD_BEEF;
    bus.write32::<BusRead>(0, encode_load_store(0x2A, 10, 8, 0x0000));
    bus.write32::<BusRead>(4, nop());
    cpu.step(&mut bus);
    cpu.step(&mut bus);
    // aligned=0x1000, offset=2
    // old mem = 0xAABB_CCDD, rt[31:8] = 0xDEAD_BE
    // merged = (0x0000_00DD) | (0xDEAD_BE00) = 0xDEAD_BEDD
    assert_eq!(bus.read32::<BusRead>(0x1000), 0xDEAD_BEDD);
}

#[test]
fn swl_offset_3_whole_32bits() {
    let mut bus = bus_with_bios_empty();
    setup_patterns(&mut bus);
    let mut cpu = Cpu::new();
    cpu.pc = 0;
    cpu.regs[8] = 0x1003;
    cpu.regs[10] = 0xDEAD_BEEF;
    bus.write32::<BusRead>(0, encode_load_store(0x2A, 10, 8, 0x0000));
    bus.write32::<BusRead>(4, nop());
    cpu.step(&mut bus);
    cpu.step(&mut bus);
    assert_eq!(bus.read32::<BusRead>(0x1000), 0xDEAD_BEEF);
}

// ---------------------------------------------------------------------------
// SWR (Store Word Right) — primary 0x2E
// ---------------------------------------------------------------------------

#[test]
fn swr_offset_0_whole_32bits() {
    let mut bus = bus_with_bios_empty();
    setup_patterns(&mut bus);
    let mut cpu = Cpu::new();
    cpu.pc = 0;
    cpu.regs[8] = 0x1000;
    cpu.regs[10] = 0xDEAD_BEEF;
    bus.write32::<BusRead>(0, encode_load_store(0x2E, 10, 8, 0x0000));
    bus.write32::<BusRead>(4, nop());
    cpu.step(&mut bus);
    cpu.step(&mut bus);
    assert_eq!(bus.read32::<BusRead>(0x1000), 0xDEAD_BEEF);
}

#[test]
fn swr_offset_1_lower_24bits() {
    let mut bus = bus_with_bios_empty();
    setup_patterns(&mut bus);
    let mut cpu = Cpu::new();
    cpu.pc = 0;
    cpu.regs[8] = 0x1001;
    cpu.regs[10] = 0xDEAD_BEEF;
    bus.write32::<BusRead>(0, encode_load_store(0x2E, 10, 8, 0x0000));
    bus.write32::<BusRead>(4, nop());
    cpu.step(&mut bus);
    cpu.step(&mut bus);
    // aligned=0x1000, offset=1
    // old mem = 0xAABB_CCDD, rt[23:0] = 0xAD_BEEF
    // merged = (0xFF00_0000) | (0x00AD_BEEF)
    // Wait: rt[23:0] → AE_BEEF becomes... no.
    // rt=0xDEAD_BEEF. rt[23:0] = 0xAD_BEEF.
    // old mem = 0xAABB_CCDD. old[31:24] = 0xAA.
    // merged = (0xAA00_0000) | 0x00AD_BEEF = 0xAAAD_BEEF
    assert_eq!(bus.read32::<BusRead>(0x1000), 0xAAAD_BEEF);
}

#[test]
fn swr_offset_2_lower_16bits() {
    let mut bus = bus_with_bios_empty();
    setup_patterns(&mut bus);
    let mut cpu = Cpu::new();
    cpu.pc = 0;
    cpu.regs[8] = 0x1002;
    cpu.regs[10] = 0xDEAD_BEEF;
    bus.write32::<BusRead>(0, encode_load_store(0x2E, 10, 8, 0x0000));
    bus.write32::<BusRead>(4, nop());
    cpu.step(&mut bus);
    cpu.step(&mut bus);
    // aligned=0x1000, offset=2
    // old mem = 0xAABB_CCDD, rt[15:0] = 0xBEEF
    // merged = (0xAABB_0000) | 0x0000_BEEF = 0xAABB_BEEF
    assert_eq!(bus.read32::<BusRead>(0x1000), 0xAABB_BEEF);
}

#[test]
fn swr_offset_3_lower_8bits() {
    let mut bus = bus_with_bios_empty();
    setup_patterns(&mut bus);
    let mut cpu = Cpu::new();
    cpu.pc = 0;
    cpu.regs[8] = 0x1003;
    cpu.regs[10] = 0xDEAD_BEEF;
    bus.write32::<BusRead>(0, encode_load_store(0x2E, 10, 8, 0x0000));
    bus.write32::<BusRead>(4, nop());
    cpu.step(&mut bus);
    cpu.step(&mut bus);
    // aligned=0x1000, offset=3
    // old mem = 0xAABB_CCDD, rt[7:0] = 0xEF
    // merged = (0xAABB_CC00) | 0x0000_00EF = 0xAABB_CCEF
    assert_eq!(bus.read32::<BusRead>(0x1000), 0xAABB_CCEF);
}
