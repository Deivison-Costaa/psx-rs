use psx_core::bus::{Bus, BusRead, BusWrite};

mod support;
use support::asm::bus_with_bios_empty;

const VALOR: u32 = 0x1234_5678;

fn sb(bus: &mut Bus, addr: u32) {
    bus.write8_gpr_completo::<BusWrite>(addr, VALOR);
}

fn sh(bus: &mut Bus, addr: u32) {
    bus.write16_gpr_completo::<BusWrite>(addr, VALOR);
}

#[test]
fn sb_em_registrador_de_16_ou_32_bits_leva_o_gpr_inteiro() {
    let mut bus = bus_with_bios_empty();
    sb(&mut bus, 0x1F80_1074);
    assert_eq!(bus.read16::<BusRead>(0x1F80_1074), 0x678, "I_MASK");
    sb(&mut bus, 0x1F80_1108);
    assert_eq!(bus.read16::<BusRead>(0x1F80_1108), 0x5678, "T0_TARGET");
    sb(&mut bus, 0x1F80_1DAA);
    assert_eq!(bus.read16::<BusRead>(0x1F80_1DAA), 0x5678, "SPUCNT");
}

#[test]
fn dicr_so_guarda_os_bits_0_a_5_baixos() {
    for escreve in [sb, sh] {
        let mut bus = bus_with_bios_empty();
        escreve(&mut bus, 0x1F80_10F4);
        assert_eq!(bus.read32::<BusRead>(0x1F80_10F4), 0x0034_0038);
        assert_eq!(bus.read16::<BusRead>(0x1F80_10F4), 0x38);
        assert_eq!(bus.read8::<BusRead>(0x1F80_10F4), 0x38);
    }
}

#[test]
fn joy_mode_mascara_e_joy_ctrl_com_reset_zera() {
    let mut bus = bus_with_bios_empty();
    sh(&mut bus, 0x1F80_1048);
    assert_eq!(bus.read16::<BusRead>(0x1F80_1048), 0x38, "JOY_MODE");
    assert_eq!(bus.read32::<BusRead>(0x1F80_1048), 0x38, "JOY_MODE");
    sh(&mut bus, 0x1F80_104A);
    assert_eq!(bus.read16::<BusRead>(0x1F80_104A), 0, "JOY_CTRL.6 reseta");
    sb(&mut bus, 0x1F80_104A);
    assert_eq!(bus.read16::<BusRead>(0x1F80_104A), 0, "JOY_CTRL.6 reseta");
}

#[test]
fn sio1_mode_e_ctrl_existem() {
    let mut bus = bus_with_bios_empty();
    sh(&mut bus, 0x1F80_1058);
    assert_eq!(bus.read16::<BusRead>(0x1F80_1058), 0x78, "SIO_MODE");
    assert_eq!(bus.read8::<BusRead>(0x1F80_1058), 0x78, "SIO_MODE");
    assert_eq!(bus.read32::<BusRead>(0x1F80_1058), 0x78, "SIO_MODE");
    sh(&mut bus, 0x1F80_105A);
    assert_eq!(bus.read16::<BusRead>(0x1F80_105A), 0, "SIO_CTRL.6 reseta");
    assert_eq!(bus.read8::<BusRead>(0x1F80_105A), 0, "SIO_CTRL.6 reseta");
}

#[test]
fn cdrom_de_8_bits_sem_autoincremento_repete_o_mesmo_registrador() {
    let mut bus = bus_with_bios_empty();
    sb(&mut bus, 0x1F80_1800);
    assert_eq!(bus.read8::<BusRead>(0x1F80_1800), 0x18);
    assert_eq!(bus.read16::<BusRead>(0x1F80_1800), 0x1818);
    assert_eq!(bus.read32::<BusRead>(0x1F80_1800), 0x1818_1818);
    sh(&mut bus, 0x1F80_1800);
    assert_eq!(
        bus.read16::<BusRead>(0x1F80_1800),
        0x1A1A,
        "0x78 e depois 0x56"
    );
    bus.write32::<BusWrite>(0x1F80_1800, VALOR);
    assert_eq!(
        bus.read32::<BusRead>(0x1F80_1800),
        0x1A1A_1A1A,
        "4 bytes no indice"
    );
}

#[test]
fn mdec_depois_do_reset_le_80040000() {
    let mut bus = bus_with_bios_empty();
    bus.write32::<BusWrite>(0x1F80_1824, 0x8000_0000);
    assert_eq!(bus.read32::<BusRead>(0x1F80_1824), 0x8004_0000);
    sb(&mut bus, 0x1F80_1824);
    assert_eq!(bus.read32::<BusRead>(0x1F80_1824), 0x8004_0000);
}

#[test]
fn sw_em_porta_do_sio_so_escreve_o_registrador_enderecado() {
    let mut bus = bus_with_bios_empty();
    bus.write32::<BusWrite>(0x1F80_1048, VALOR);
    assert_eq!(bus.read32::<BusRead>(0x1F80_1048), 0x38, "JOY_CTRL intacto");
    bus.write32::<BusWrite>(0x1F80_1058, VALOR);
    assert_eq!(bus.read32::<BusRead>(0x1F80_1058), 0x78, "SIO_CTRL intacto");
}
