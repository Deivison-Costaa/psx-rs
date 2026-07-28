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
