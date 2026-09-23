use psx_core::bus::{Bus, BusWrite};
use psx_core::cpu::Cpu;

mod support;
use support::asm::{bus_with_bios_empty, encode_i_type, nop};

const CODIGO: u32 = 0x0000_0100;
const LB: u32 = 0x20;
const LBU: u32 = 0x24;
const LHU: u32 = 0x25;
const LW: u32 = 0x23;

fn com_atrasos_da_bios(bus: &mut Bus) {
    for (reg, valor) in [
        (0x1F80_1008u32, 0x0013_243Fu32),
        (0x1F80_100C, 0x0000_3022),
        (0x1F80_1010, 0x0013_243F),
        (0x1F80_1014, 0x2009_31E1),
        (0x1F80_1018, 0x0002_0843),
        (0x1F80_101C, 0x0007_0777),
        (0x1F80_1020, 0x0003_1125),
    ] {
        bus.write32::<BusWrite>(reg, valor);
    }
}

fn roda(programa: &[u32], base: u32) -> u64 {
    let mut bus = bus_with_bios_empty();
    com_atrasos_da_bios(&mut bus);
    let mut cpu = Cpu::new();
    cpu.pc = CODIGO;
    cpu.regs[8] = base;
    for (i, instr) in programa.iter().enumerate() {
        bus.write32::<BusWrite>(CODIGO + 4 * i as u32, *instr);
    }
    let antes = bus.total_cycles();
    for _ in programa {
        cpu.step(&mut bus);
    }
    bus.total_cycles() - antes
}

fn carga(op: u32) -> u32 {
    encode_i_type(op, 9, 8, 0)
}

fn isolada(op: u32, base: u32) -> u64 {
    roda(&[carga(op), nop(), nop(), nop(), nop()], base) - 4
}

fn encostadas(op: u32, base: u32) -> u64 {
    roda(&[carga(op), carga(op)], base) - isolada(op, base)
}

#[test]
fn ram_custa_5_com_sombra_e_7_encostada() {
    assert_eq!(isolada(LW, 0x8000_2000), 5, "access-time: RAM 5.14");
    assert_eq!(isolada(LBU, 0x8000_2000), 5, "access-time: RAM 5.21");
    assert_eq!(encostadas(LW, 0x8000_2000), 7, "02-cpu.md Load Timing: 7");
}

#[test]
fn io_interno_custa_3_com_sombra_e_5_encostado() {
    assert_eq!(isolada(LW, 0x1F80_1070), 3, "access-time: I_STAT 3.1");
    assert_eq!(isolada(LHU, 0x1F80_1100), 3, "access-time: TIMER0 3.1");
    assert_eq!(isolada(LW, 0x1F80_1814), 3, "access-time: GPUSTAT 3.8");
    assert_eq!(encostadas(LW, 0x1F80_1070), 5, "02-cpu.md Load Timing: 5");
}

#[test]
fn scratchpad_e_cachectrl_custam_1() {
    assert_eq!(isolada(LW, 0x1F80_0010), 1);
    assert_eq!(encostadas(LW, 0x1F80_0010), 1, "sem barramento, sem sombra");
    assert_eq!(isolada(LW, 0xFFFE_0130), 1, "access-time: CACHECTRL 1.9");
}

#[test]
fn bios_de_8_bits_paga_um_acesso_por_byte() {
    assert_eq!(isolada(LBU, 0xBFC0_0000), 7, "access-time: BIOS 7.6");
    assert_eq!(isolada(LHU, 0xBFC0_0000), 13, "access-time: BIOS 12.94");
    assert_eq!(isolada(LW, 0xBFC0_0000), 25, "access-time: BIOS 24.94");
    assert_eq!(
        encostadas(LW, 0xBFC0_0000),
        27,
        "02-cpu.md Load Timing: 27..33"
    );
}

#[test]
fn cdrom_spu_e_expansoes_seguem_os_registradores_de_atraso() {
    assert_eq!(isolada(LB, 0x1F80_1800), 8, "access-time: CDROM 8.0");
    assert_eq!(isolada(LHU, 0x1F80_1800), 14, "access-time: CDROM 14.0");
    assert_eq!(isolada(LW, 0x1F80_1800), 26, "access-time: CDROM 25.93");
    assert_eq!(isolada(LHU, 0x1F80_1DAA), 18, "access-time: SPUCNT 17.99");
    assert_eq!(isolada(LW, 0x1F80_1DA8), 39, "access-time: SPUCNT 38.94");
    assert_eq!(isolada(LBU, 0x1F00_0000), 7, "access-time: EXPANSION1 6.94");
    assert_eq!(isolada(LW, 0x1F00_0000), 25, "access-time: EXPANSION1 25.7");
    assert_eq!(
        isolada(LBU, 0x1F80_2000),
        11,
        "access-time: EXPANSION2 10.99"
    );
    assert_eq!(
        isolada(LHU, 0x1F80_2000),
        26,
        "access-time: EXPANSION2 25.99"
    );
    assert_eq!(
        isolada(LW, 0x1F80_2000),
        56,
        "access-time: EXPANSION2 55.98"
    );
    assert_eq!(isolada(LBU, 0x1FA0_0000), 6, "access-time: EXPANSION3 6.7");
    assert_eq!(isolada(LHU, 0x1FA0_0000), 6, "access-time: EXPANSION3 6.1");
    assert_eq!(isolada(LW, 0x1FA0_0000), 10, "access-time: EXPANSION3 9.95");
}

#[test]
fn atraso_de_leitura_da_bios_vem_do_registrador() {
    let mut bus = bus_with_bios_empty();
    com_atrasos_da_bios(&mut bus);
    bus.write32::<BusWrite>(0x1F80_1010, 0x0013_247F);
    let mut cpu = Cpu::new();
    cpu.pc = CODIGO;
    cpu.regs[8] = 0xBFC0_0000;
    bus.write32::<BusWrite>(CODIGO, carga(LBU));
    let antes = bus.total_cycles();
    cpu.step(&mut bus);
    assert_eq!(
        bus.total_cycles() - antes,
        11,
        "leitura com atraso 7 em vez de 3: primeiro acesso = atraso + 4"
    );
}

#[test]
fn lwl_e_lwr_so_pagam_as_metades_que_tocam() {
    let par = [
        encode_i_type(0x22, 9, 8, 3),
        nop(),
        encode_i_type(0x26, 9, 8, 0),
        nop(),
        nop(),
        nop(),
        nop(),
    ];
    let custo = roda(&par, 0x1F80_1DAA) - 5;
    assert!(
        (38..=39).contains(&custo),
        "access-time: lwl/lwr em SPUCNT (1F801DAAh) somam 38.94 ciclos, nao 2x39; medido {custo}"
    );
}
