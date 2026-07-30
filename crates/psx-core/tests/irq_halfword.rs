mod support;

use psx_core::bus::{Bus, BusRead};
use psx_core::cpu::Cpu;
use support::asm;

fn bus_irq() -> Bus {
    asm::bus_with_bios_empty()
}

fn sh(rt: u32, rs: u32, imm: u16) -> u32 {
    asm::encode_i_type(0x29, rt, rs, imm)
}

fn sb(rt: u32, rs: u32, imm: u16) -> u32 {
    asm::encode_i_type(0x28, rt, rs, imm)
}

fn lhu(rt: u32, rs: u32, imm: u16) -> u32 {
    asm::encode_i_type(0x25, rt, rs, imm)
}

fn lui(rt: u32, imm: u16) -> u32 {
    asm::encode_i_type(0x0F, rt, 0, imm)
}

fn ori(rt: u32, rs: u32, imm: u16) -> u32 {
    asm::encode_i_type(0x0D, rt, rs, imm)
}

#[test]
fn sh_i_mask_escreve_16_bits_e_le_volta() {
    let mut bus = bus_irq();
    let mut cpu = Cpu::new();
    cpu.pc = 0;

    cpu.regs[8] = 0x0000_0005;
    bus.write32::<BusRead>(0x0000, lui(1, 0x1F80));
    bus.write32::<BusRead>(0x0004, ori(1, 1, 0x1074));
    bus.write32::<BusRead>(0x0008, sh(8, 1, 0));
    bus.write32::<BusRead>(0x000C, asm::nop());

    cpu.step(&mut bus);
    cpu.step(&mut bus);
    cpu.step(&mut bus);
    cpu.step(&mut bus);

    let mask = bus.read32::<BusRead>(0x1F80_1074) & 0x7FF;
    assert_eq!(
        mask, 0x005,
        "SH em I_MASK deve gravar bits 0-10 corretamente"
    );
}

#[test]
fn sh_i_stat_acknowledge_limpa_bit_com_zero() {
    let mut bus = bus_irq();
    let mut cpu = Cpu::new();
    cpu.pc = 0;

    bus.irq_mut().raise(0);
    bus.irq_mut().raise(2);

    cpu.regs[8] = 0x0000_FFFE;
    bus.write32::<BusRead>(0x0000, lui(1, 0x1F80));
    bus.write32::<BusRead>(0x0004, ori(1, 1, 0x1070));
    bus.write32::<BusRead>(0x0008, sh(8, 1, 0));
    bus.write32::<BusRead>(0x000C, asm::nop());

    cpu.step(&mut bus);
    cpu.step(&mut bus);
    cpu.step(&mut bus);
    cpu.step(&mut bus);

    let stat = bus.read32::<BusRead>(0x1F80_1070) & 0x7FF;
    assert_eq!(
        stat, 0x004,
        "SH em I_STAT com bit 0=0 deve limpar bit 0; bit 2 permanece"
    );
}

#[test]
fn sh_i_stat_com_um_nao_altera_bit() {
    let mut bus = bus_irq();
    let mut cpu = Cpu::new();
    cpu.pc = 0;

    bus.irq_mut().raise(0);
    bus.irq_mut().raise(3);

    cpu.regs[8] = 0x0000_FFFF;
    bus.write32::<BusRead>(0x0000, lui(1, 0x1F80));
    bus.write32::<BusRead>(0x0004, ori(1, 1, 0x1070));
    bus.write32::<BusRead>(0x0008, sh(8, 1, 0));
    bus.write32::<BusRead>(0x000C, asm::nop());

    cpu.step(&mut bus);
    cpu.step(&mut bus);
    cpu.step(&mut bus);
    cpu.step(&mut bus);

    let stat = bus.read32::<BusRead>(0x1F80_1070) & 0x7FF;
    assert_eq!(
        stat, 0x009,
        "SH em I_STAT com bits=1 nao deve alterar bits setados"
    );
}

#[test]
fn sb_i_mask_escreve_byte_e_le_volta() {
    let mut bus = bus_irq();
    let mut cpu = Cpu::new();
    cpu.pc = 0;

    cpu.regs[8] = 0x0000_0002;
    bus.write32::<BusRead>(0x0000, lui(1, 0x1F80));
    bus.write32::<BusRead>(0x0004, ori(1, 1, 0x1074));
    bus.write32::<BusRead>(0x0008, sb(8, 1, 0));
    bus.write32::<BusRead>(0x000C, asm::nop());

    cpu.step(&mut bus);
    cpu.step(&mut bus);
    cpu.step(&mut bus);
    cpu.step(&mut bus);

    let mask = bus.read32::<BusRead>(0x1F80_1074) & 0x7FF;
    assert_eq!(mask, 0x002, "SB em I_MASK deve gravar byte corretamente");
}

#[test]
fn sb_i_stat_acknowledge_limpa_bit_com_zero() {
    let mut bus = bus_irq();
    let mut cpu = Cpu::new();
    cpu.pc = 0;

    bus.irq_mut().raise(0);
    bus.irq_mut().raise(1);

    cpu.regs[8] = 0x0000_00FE;
    bus.write32::<BusRead>(0x0000, lui(1, 0x1F80));
    bus.write32::<BusRead>(0x0004, ori(1, 1, 0x1070));
    bus.write32::<BusRead>(0x0008, sb(8, 1, 0));
    bus.write32::<BusRead>(0x000C, asm::nop());

    cpu.step(&mut bus);
    cpu.step(&mut bus);
    cpu.step(&mut bus);
    cpu.step(&mut bus);

    let stat = bus.read32::<BusRead>(0x1F80_1070) & 0x7FF;
    assert_eq!(
        stat, 0x002,
        "SB em I_STAT com bit 0=0 deve limpar bit 0; bit 1 permanece"
    );
}

#[test]
fn lh_i_mask_le_16_bits() {
    let mut bus = bus_irq();
    bus.write32::<BusRead>(0x1F80_1074, 0x003);

    let mut cpu = Cpu::new();
    cpu.pc = 0;

    bus.write32::<BusRead>(0x0000, lui(1, 0x1F80));
    bus.write32::<BusRead>(0x0004, ori(1, 1, 0x1074));
    bus.write32::<BusRead>(0x0008, lhu(10, 1, 0));
    bus.write32::<BusRead>(0x000C, asm::nop());

    cpu.step(&mut bus);
    cpu.step(&mut bus);
    cpu.step(&mut bus);
    cpu.step(&mut bus);

    assert_eq!(
        cpu.regs[10] & 0x7FF,
        0x003,
        "LH em I_MASK deve ler 16 bits corretos"
    );
}

#[test]
fn lh_i_stat_le_16_bits() {
    let mut bus = bus_irq();
    bus.irq_mut().raise(0);
    bus.irq_mut().raise(4);

    let mut cpu = Cpu::new();
    cpu.pc = 0;

    bus.write32::<BusRead>(0x0000, lui(1, 0x1F80));
    bus.write32::<BusRead>(0x0004, ori(1, 1, 0x1070));
    bus.write32::<BusRead>(0x0008, lhu(10, 1, 0));
    bus.write32::<BusRead>(0x000C, asm::nop());

    cpu.step(&mut bus);
    cpu.step(&mut bus);
    cpu.step(&mut bus);
    cpu.step(&mut bus);

    assert_eq!(
        cpu.regs[10] & 0x7FF,
        0x011,
        "LH em I_STAT deve ler 16 bits corretos"
    );
}

#[test]
fn sh_i_mask_segundo_halfword_preserva_bits_baixos() {
    let mut bus = bus_irq();
    let mut cpu = Cpu::new();
    cpu.pc = 0;

    bus.write32::<BusRead>(0x1F80_1074, 0x005);

    cpu.regs[8] = 0x0000_0000;
    bus.write32::<BusRead>(0x0000, lui(1, 0x1F80));
    bus.write32::<BusRead>(0x0004, ori(1, 1, 0x1076));
    bus.write32::<BusRead>(0x0008, sh(8, 1, 0));
    bus.write32::<BusRead>(0x000C, asm::nop());

    cpu.step(&mut bus);
    cpu.step(&mut bus);
    cpu.step(&mut bus);
    cpu.step(&mut bus);

    let mask = bus.read32::<BusRead>(0x1F80_1074) & 0x7FF;
    assert_eq!(
        mask, 0x005,
        "SH no segundo halfword de I_MASK nao deve zerar bits 0-10"
    );
}

#[test]
fn sh_i_stat_segundo_halfword_preserva_bits_baixos() {
    let mut bus = bus_irq();
    let mut cpu = Cpu::new();
    cpu.pc = 0;

    bus.irq_mut().raise(0);
    bus.irq_mut().raise(2);

    cpu.regs[8] = 0x0000_FFFF;
    bus.write32::<BusRead>(0x0000, lui(1, 0x1F80));
    bus.write32::<BusRead>(0x0004, ori(1, 1, 0x1072));
    bus.write32::<BusRead>(0x0008, sh(8, 1, 0));
    bus.write32::<BusRead>(0x000C, asm::nop());

    cpu.step(&mut bus);
    cpu.step(&mut bus);
    cpu.step(&mut bus);
    cpu.step(&mut bus);

    let stat = bus.read32::<BusRead>(0x1F80_1070) & 0x7FF;
    assert_eq!(
        stat, 0x005,
        "SH no segundo halfword de I_STAT deve preservar bits 0-7"
    );
}
