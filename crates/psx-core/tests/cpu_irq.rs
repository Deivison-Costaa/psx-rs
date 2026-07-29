mod support;

use psx_core::bus::{Bus, BusRead};
use psx_core::cpu::Cpu;
use support::asm;

fn bus_com_irq() -> Bus {
    asm::bus_with_bios_empty()
}

#[test]
fn i_stat_endereco_inicial_e_zero() {
    let bus = bus_com_irq();
    let val = bus.read32::<BusRead>(0x1F80_1070);
    assert_eq!(val & 0x7FF, 0);
}

#[test]
fn i_mask_gravavel_e_legivel_pelo_bus() {
    let mut bus = bus_com_irq();
    bus.write32::<BusRead>(0x1F80_1074, 0x005);
    let val = bus.read32::<BusRead>(0x1F80_1074);
    assert_eq!(val & 0x7FF, 0x005);
}

#[test]
fn i_stat_acknowledge_com_zero_limpa_bit() {
    let mut bus = bus_com_irq();
    bus.irq_mut().raise(0);
    bus.irq_mut().raise(3);
    let antes = bus.read32::<BusRead>(0x1F80_1070);
    assert_eq!(antes & 0x7FF, 0x009);
    bus.write32::<BusRead>(0x1F80_1070, 0xFFE);
    let depois = bus.read32::<BusRead>(0x1F80_1070);
    assert_eq!(depois & 0x7FF, 0x008);
}

#[test]
fn i_stat_acknowledge_com_um_nao_altera_bit() {
    let mut bus = bus_com_irq();
    bus.irq_mut().raise(0);
    bus.irq_mut().raise(2);
    let antes = bus.read32::<BusRead>(0x1F80_1070);
    assert_eq!(antes & 0x7FF, 0x005);
    bus.write32::<BusRead>(0x1F80_1070, 0xFFFF_FFFF);
    let depois = bus.read32::<BusRead>(0x1F80_1070);
    assert_eq!(depois & 0x7FF, 0x005);
}

#[test]
fn cause_bit10_reflete_irq_pendente() {
    let mut bus = bus_com_irq();
    bus.irq_mut().raise(0);
    bus.write32::<BusRead>(0x1F80_1074, 0x001);
    let mut cpu = Cpu::new();
    cpu.step(&mut bus);
    let cause = cpu.cop0[13];
    assert_eq!((cause >> 10) & 1, 1);
}

#[test]
fn cause_bit10_zero_quando_sem_irq() {
    let mut bus = bus_com_irq();
    bus.write32::<BusRead>(0x1F80_1074, 0x001);
    let mut cpu = Cpu::new();
    cpu.step(&mut bus);
    let cause = cpu.cop0[13];
    assert_eq!((cause >> 10) & 1, 0);
}

#[test]
fn interrupcao_dispara_excecao_int() {
    let mut bus = bus_com_irq();
    bus.irq_mut().raise(0);
    bus.write32::<BusRead>(0x1F80_1074, 0x001);
    let mut cpu = Cpu::new();
    cpu.cop0[12] = 1 | (1 << 10);
    let pc_antes = cpu.pc;
    cpu.step(&mut bus);
    let cause = cpu.cop0[13];
    assert_eq!(cause & 0x7C, 0x00);
    assert_ne!(cpu.pc, pc_antes);
    assert_eq!(cpu.pc, 0x8000_0080);
    assert_eq!(cpu.cop0[14], pc_antes);
}

#[test]
fn interrupcao_nao_dispara_sem_sr_iec() {
    let mut bus = bus_com_irq();
    bus.irq_mut().raise(0);
    bus.write32::<BusRead>(0x1F80_1074, 0x001);
    let mut cpu = Cpu::new();
    cpu.cop0[12] = 1 << 10;
    let pc_antes = cpu.pc;
    cpu.step(&mut bus);
    assert_eq!(cpu.pc, pc_antes.wrapping_add(4));
    assert_eq!(cpu.cop0[14], 0);
}

#[test]
fn interrupcao_nao_dispara_sem_sr_im_bit10() {
    let mut bus = bus_com_irq();
    bus.irq_mut().raise(0);
    bus.write32::<BusRead>(0x1F80_1074, 0x001);
    let mut cpu = Cpu::new();
    cpu.cop0[12] = 1;
    let pc_antes = cpu.pc;
    cpu.step(&mut bus);
    assert_eq!(cpu.pc, pc_antes.wrapping_add(4));
    assert_eq!(cpu.cop0[14], 0);
}

#[test]
fn i_stat_bits_11_a_15_sao_zero() {
    let mut bus = bus_com_irq();
    bus.irq_mut().raise(0);
    bus.irq_mut().raise(10);
    let val = bus.read32::<BusRead>(0x1F80_1070);
    assert_eq!((val >> 11) & 0x1F, 0);
}
