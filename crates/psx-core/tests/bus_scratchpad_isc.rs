use psx_core::bus::BusRead;
use psx_core::cpu::Cpu;

mod support;
use support::asm::bus_with_bios_empty;

fn lui(rt: u32, imm: u16) -> u32 {
    (0x0F << 26) | (rt << 16) | (imm as u32)
}

fn ori(rt: u32, rs: u32, imm: u16) -> u32 {
    (0x0D << 26) | (rs << 21) | (rt << 16) | (imm as u32)
}

fn sw(rt: u32, rs: u32, imm: u16) -> u32 {
    (0x2B << 26) | (rs << 21) | (rt << 16) | (imm as u32)
}

fn lw(rt: u32, rs: u32, imm: u16) -> u32 {
    (0x23 << 26) | (rs << 21) | (rt << 16) | (imm as u32)
}

fn mtc0(rt: u32, rd: u32) -> u32 {
    (0x10 << 26) | (0x04 << 21) | (rt << 16) | (rd << 11)
}

fn nop() -> u32 {
    0
}

fn escreve_instrucoes(bus: &mut psx_core::bus::Bus, base: u32, words: &[u32]) {
    for (i, &w) in words.iter().enumerate() {
        bus.write32::<BusRead>(base + (i as u32) * 4, w);
    }
}

#[test]
fn scratchpad_nao_alias_ram() {
    let mut bus = bus_with_bios_empty();
    bus.write32::<BusRead>(0x0000_0000, 0xAAAA_AAAA);
    bus.write32::<BusRead>(0x1F80_0000, 0x5555_5555);
    assert_eq!(
        bus.read32::<BusRead>(0x0000_0000),
        0xAAAA_AAAA,
        "D1: RAM em 0x0000_0000 nao foi sobrescrita pelo scratchpad"
    );
    assert_eq!(
        bus.read32::<BusRead>(0x1F80_0000),
        0x5555_5555,
        "D1: scratchpad em 0x1F80_0000 tem o valor escrito"
    );
}

#[test]
fn scratchpad_kseg0_mirror_kseg1_nao() {
    let mut bus = bus_with_bios_empty();
    bus.write32::<BusRead>(0x1F80_0010, 0xC0DE_C0DE);
    assert_eq!(
        bus.read32::<BusRead>(0x9F80_0010),
        0xC0DE_C0DE,
        "D2: KSEG0 espelha scratchpad"
    );
    assert_eq!(
        bus.read32::<BusRead>(0xBF80_0010),
        0,
        "D2: KSEG1 NAO espelha scratchpad (comportamento ASSUMIDO sem Bus Error)"
    );
}

#[test]
fn scratchpad_limite_superior() {
    let mut bus = bus_with_bios_empty();
    bus.write32::<BusRead>(0x0000_03FC, 0xAAAA_AAAA);
    bus.write32::<BusRead>(0x1F80_03FC, 0x1234_5678);
    assert_eq!(
        bus.read32::<BusRead>(0x0000_03FC),
        0xAAAA_AAAA,
        "D3: RAM em 0x03FC nao foi sobrescrita pelo scratchpad alias"
    );
    assert_eq!(
        bus.read32::<BusRead>(0x1F80_03FC),
        0x1234_5678,
        "D3: ultimo word do scratchpad (0x3FC) funciona"
    );
}

#[test]
fn isc_engole_store() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    cpu.pc = 0;

    // r8 = 0xDEAD_BEEF
    // r9 = 0x0001_0000 (Isc bit)
    escreve_instrucoes(
        &mut bus,
        0,
        &[
            lui(8, 0xDEAD),
            ori(8, 8, 0xBEEF),
            sw(8, 0, 0x200),
            lui(9, 0x0001),
            mtc0(9, 12),
            sw(0, 0, 0x200),
            mtc0(0, 12),
            lw(10, 0, 0x200),
            nop(),
        ],
    );

    for _ in 0..9 {
        cpu.step(&mut bus);
    }

    assert_eq!(
        cpu.regs[10], 0xDEAD_BEEF,
        "D4: Isc=1 engole o store, lw retorna o valor original"
    );
}

#[test]
fn memctrl_nao_corrompe_ram() {
    let mut bus = bus_with_bios_empty();
    bus.write32::<BusRead>(0x1F80_1000, 0x1F00_0000);
    bus.write32::<BusRead>(0x1F80_1060, 0x0000_0B88);
    assert_eq!(
        bus.read32::<BusRead>(0x0000_1000),
        0,
        "D5: RAM em 0x1000 nao foi tocada pela escrita em 0x1F80_1000"
    );
    assert_eq!(
        bus.read32::<BusRead>(0x0000_1060),
        0,
        "D5: RAM em 0x1060 nao foi tocada pela escrita em 0x1F80_1060"
    );
    assert_eq!(
        bus.read32::<BusRead>(0x1F80_1060),
        0x0000_0B88,
        "D5: RAM_SIZE em 0x1F80_1060 retorna o valor escrito"
    );
}

#[test]
fn bcc_em_kseg2() {
    let mut bus = bus_with_bios_empty();
    bus.write32::<BusRead>(0x001E_0130, 0x1111_1111);
    bus.write32::<BusRead>(0xFFFE_0130, 0x0001_E988);
    assert_eq!(
        bus.read32::<BusRead>(0xFFFE_0130),
        0x0001_E988,
        "D6: BCC em KSEG2 retorna o valor escrito"
    );
    assert_eq!(
        bus.read32::<BusRead>(0x001E_0130),
        0x1111_1111,
        "D6: RAM em 0x001E_0130 nao foi tocada pela escrita em FFFE0130"
    );
}

fn ram_offset_of(addr: u32) -> u32 {
    addr & 0x1F_FF_FF
}

#[test]
fn memctrl_bcc_read8_read16_nao_alias_ram() {
    let mut bus = bus_with_bios_empty();
    let ram_alias = ram_offset_of(0x1F80_1060);
    bus.write32::<BusRead>(ram_alias, 0xEEEE_EEEE);
    bus.write32::<BusRead>(0x1F80_1060, 0x0000_0B88);
    assert_eq!(
        bus.read8::<BusRead>(0x1F80_1060),
        0x88,
        "F1: read8 RAM_SIZE byte0 le o registrador, nao a RAM"
    );
    assert_eq!(
        bus.read8::<BusRead>(0x1F80_1061),
        0x0B,
        "F1: read8 RAM_SIZE byte1 le o registrador, nao a RAM"
    );
    assert_eq!(
        bus.read16::<BusRead>(0x1F80_1060),
        0x0B88,
        "F1: read16 RAM_SIZE le o registrador"
    );
    let bcc_ram_alias = ram_offset_of(0xFFFE_0130);
    bus.write32::<BusRead>(bcc_ram_alias, 0xEEEE_EEEE);
    bus.write32::<BusRead>(0xFFFE_0130, 0xABCD_EF01);
    assert_eq!(
        bus.read8::<BusRead>(0xFFFE_0130),
        0x01,
        "F1: read8 BCC byte0 le o registrador"
    );
}

#[test]
fn io_catch_all_nao_corrompe_ram() {
    let mut bus = bus_with_bios_empty();
    bus.write32::<BusRead>(0x0000_1074, 0xEEEE_EEEE);
    bus.write32::<BusRead>(0x0000_1080, 0xEEEE_EEEE);
    bus.write32::<BusRead>(0x0000_1078, 0xEEEE_EEEE);
    bus.write32::<BusRead>(0x1F80_1074, 0x0000_0FFF);
    assert_eq!(
        bus.read32::<BusRead>(0x0000_1074),
        0xEEEE_EEEE,
        "F2: RAM em 0x1074 nao foi tocada por escrita big em I_STAT"
    );
    assert_eq!(
        bus.read32::<BusRead>(0x1F80_1074),
        0x7FF,
        "F2: I_MASK reflete mascara de bits 0-10 no read32"
    );
    assert_eq!(
        bus.read8::<BusRead>(0x1F80_1074),
        0,
        "F2: I_STAT stub devolve 0 no read8"
    );
    assert_eq!(
        bus.read16::<BusRead>(0x1F80_1074),
        0,
        "F2: I_STAT stub devolve 0 no read16"
    );
    bus.write32::<BusRead>(0x1F80_1080, 0xFFFF_FFFF);
    assert_eq!(
        bus.read32::<BusRead>(0x0000_1080),
        0xEEEE_EEEE,
        "F2: RAM em 0x1080 nao foi tocada por escrita em DMA"
    );
    assert_eq!(
        bus.read32::<BusRead>(0x1F80_1080),
        0,
        "F2: DMA stub devolve 0"
    );
    bus.write16::<BusRead>(0x1F80_1074, 0x0FFF);
    bus.write8::<BusRead>(0x1F80_1074, 0xFF);
    assert_eq!(
        bus.read32::<BusRead>(0x0000_1074),
        0xEEEE_EEEE,
        "F2: RAM nao corrompida por write16/write8 no I/O"
    );
    assert_eq!(
        bus.read32::<BusRead>(0x1F80_1074),
        0x7FF,
        "F2: read32 apos write16/write8 manteve I_MASK (byte writes sao catchall)"
    );
}

#[test]
fn isc_nao_engole_address_error_sw() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    cpu.pc = 0;
    cpu.regs[4] = 0x0000_0201;
    cpu.regs[5] = 0xCAFE_BABE;
    cpu.cop0[12] = 0x0001_0000;
    escreve_instrucoes(&mut bus, 0, &[sw(5, 4, 0), nop()]);
    cpu.step(&mut bus);
    assert_eq!(
        cpu.cop0[13] & 0x7C,
        0x14,
        "F3: CAUSE tem AdES (0x14) mesmo com Isc=1"
    );
    assert_eq!(cpu.cop0[8], 0x0000_0201, "F3: BadVaddr = 0x0000_0201");
}
