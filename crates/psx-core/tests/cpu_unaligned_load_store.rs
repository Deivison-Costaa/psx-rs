use psx_core::bus::BusRead;
use psx_core::cpu::Cpu;

mod support;
use support::asm::*;

const LWL: u32 = 0x22;
const LWR: u32 = 0x26;
const SWL: u32 = 0x2A;
const SWR: u32 = 0x2E;

fn encode_ual(primary: u32, rt: u32, rs: u32, imm: u16) -> u32 {
    encode_i_type(primary, rt, rs, imm)
}

const DATA_BASE: u32 = 0x100;

#[test]
fn par_lwl_lwr_reconstroi_palavra_desalinhada() {
    let mut bus = bus_with_bios_empty();
    bus.write32::<BusRead>(DATA_BASE, 0xDDCC_BBAA);
    bus.write32::<BusRead>(DATA_BASE + 4, 0x1122_3344);
    let mut cpu = Cpu::new();
    cpu.regs[8] = DATA_BASE + 1;
    cpu.pc = 0;
    bus.write32::<BusRead>(0, encode_ual(LWL, 2, 8, 3));
    bus.write32::<BusRead>(4, encode_ual(LWR, 2, 8, 0));
    bus.write32::<BusRead>(8, nop());
    cpu.step(&mut bus);
    cpu.step(&mut bus);
    cpu.step(&mut bus);
    assert_eq!(cpu.regs[2], 0x44DD_CCBB);
}

#[test]
fn lwl_offset_0_carrega_8bits_superiores() {
    let mut bus = bus_with_bios_empty();
    bus.write32::<BusRead>(DATA_BASE, 0x1234_56AB);
    let mut cpu = Cpu::new();
    cpu.regs[8] = DATA_BASE;
    cpu.regs[2] = 0xFFFF_FFFF;
    cpu.pc = 0;
    bus.write32::<BusRead>(0, encode_ual(LWL, 2, 8, 0));
    bus.write32::<BusRead>(4, nop());
    cpu.step(&mut bus);
    cpu.step(&mut bus);
    assert_eq!(cpu.regs[2], 0xABFF_FFFF);
}

#[test]
fn lwl_offset_1_carrega_16bits_superiores() {
    let mut bus = bus_with_bios_empty();
    bus.write32::<BusRead>(DATA_BASE, 0xABCD_1234);
    let mut cpu = Cpu::new();
    cpu.regs[8] = DATA_BASE;
    cpu.regs[2] = 0xFFFF_FFFF;
    cpu.pc = 0;
    bus.write32::<BusRead>(0, encode_ual(LWL, 2, 8, 1));
    bus.write32::<BusRead>(4, nop());
    cpu.step(&mut bus);
    cpu.step(&mut bus);
    assert_eq!(cpu.regs[2], 0x1234_FFFF);
}

#[test]
fn lwl_offset_2_carrega_24bits_superiores() {
    let mut bus = bus_with_bios_empty();
    bus.write32::<BusRead>(DATA_BASE, 0xAABB_CCDD);
    let mut cpu = Cpu::new();
    cpu.regs[8] = DATA_BASE;
    cpu.regs[2] = 0xFFFF_FFFF;
    cpu.pc = 0;
    bus.write32::<BusRead>(0, encode_ual(LWL, 2, 8, 2));
    bus.write32::<BusRead>(4, nop());
    cpu.step(&mut bus);
    cpu.step(&mut bus);
    assert_eq!(cpu.regs[2], 0xBBCC_DDFF);
}

#[test]
fn lwl_offset_3_carrega_todos_32bits() {
    let mut bus = bus_with_bios_empty();
    bus.write32::<BusRead>(DATA_BASE, 0x1234_5678);
    let mut cpu = Cpu::new();
    cpu.regs[8] = DATA_BASE;
    cpu.regs[2] = 0xFFFF_FFFF;
    cpu.pc = 0;
    bus.write32::<BusRead>(0, encode_ual(LWL, 2, 8, 3));
    bus.write32::<BusRead>(4, nop());
    cpu.step(&mut bus);
    cpu.step(&mut bus);
    assert_eq!(cpu.regs[2], 0x1234_5678);
}

#[test]
fn lwr_offset_0_carrega_todos_32bits() {
    let mut bus = bus_with_bios_empty();
    bus.write32::<BusRead>(DATA_BASE, 0x1234_5678);
    let mut cpu = Cpu::new();
    cpu.regs[8] = DATA_BASE;
    cpu.regs[2] = 0xFFFF_FFFF;
    cpu.pc = 0;
    bus.write32::<BusRead>(0, encode_ual(LWR, 2, 8, 0));
    bus.write32::<BusRead>(4, nop());
    cpu.step(&mut bus);
    cpu.step(&mut bus);
    assert_eq!(cpu.regs[2], 0x1234_5678);
}

#[test]
fn lwr_offset_1_carrega_24bits_inferiores() {
    let mut bus = bus_with_bios_empty();
    bus.write32::<BusRead>(DATA_BASE, 0xDDCC_BBAA);
    let mut cpu = Cpu::new();
    cpu.regs[8] = DATA_BASE;
    cpu.regs[2] = 0xAA00_0000;
    cpu.pc = 0;
    bus.write32::<BusRead>(0, encode_ual(LWR, 2, 8, 1));
    bus.write32::<BusRead>(4, nop());
    cpu.step(&mut bus);
    cpu.step(&mut bus);
    assert_eq!(cpu.regs[2], 0xAADD_CCBB);
}

#[test]
fn lwr_offset_2_carrega_16bits_inferiores() {
    let mut bus = bus_with_bios_empty();
    bus.write32::<BusRead>(DATA_BASE, 0xABCD_1234);
    let mut cpu = Cpu::new();
    cpu.regs[8] = DATA_BASE;
    cpu.regs[2] = 0xCCCC_0000;
    cpu.pc = 0;
    bus.write32::<BusRead>(0, encode_ual(LWR, 2, 8, 2));
    bus.write32::<BusRead>(4, nop());
    cpu.step(&mut bus);
    cpu.step(&mut bus);
    assert_eq!(cpu.regs[2], 0xCCCC_ABCD);
}

#[test]
fn lwr_offset_3_carrega_8bits_inferiores() {
    let mut bus = bus_with_bios_empty();
    bus.write32::<BusRead>(DATA_BASE, 0x1234_5678);
    let mut cpu = Cpu::new();
    cpu.regs[8] = DATA_BASE;
    cpu.regs[2] = 0xAAAA_AA00;
    cpu.pc = 0;
    bus.write32::<BusRead>(0, encode_ual(LWR, 2, 8, 3));
    bus.write32::<BusRead>(4, nop());
    cpu.step(&mut bus);
    cpu.step(&mut bus);
    assert_eq!(cpu.regs[2], 0xAAAA_AA12);
}

#[test]
fn swl_offset_0_armazena_8bits_superiores() {
    let mut bus = bus_with_bios_empty();
    bus.write32::<BusRead>(DATA_BASE, 0x1122_3344);
    let mut cpu = Cpu::new();
    cpu.regs[2] = 0xAABB_CCDD;
    cpu.regs[8] = DATA_BASE;
    cpu.pc = 0;
    bus.write32::<BusRead>(0, encode_ual(SWL, 2, 8, 0));
    cpu.step(&mut bus);
    let result = bus.read32::<BusRead>(DATA_BASE);
    assert_eq!(result, 0x1122_33AA);
}

#[test]
fn swl_offset_1_armazena_16bits_superiores() {
    let mut bus = bus_with_bios_empty();
    bus.write32::<BusRead>(DATA_BASE, 0x1111_2233);
    let mut cpu = Cpu::new();
    cpu.regs[2] = 0xAABB_CCDD;
    cpu.regs[8] = DATA_BASE;
    cpu.pc = 0;
    bus.write32::<BusRead>(0, encode_ual(SWL, 2, 8, 1));
    cpu.step(&mut bus);
    let result = bus.read32::<BusRead>(DATA_BASE);
    assert_eq!(result, 0x1111_AABB);
}

#[test]
fn swl_offset_2_armazena_24bits_superiores() {
    let mut bus = bus_with_bios_empty();
    bus.write32::<BusRead>(DATA_BASE, 0xFFEE_DDCC);
    let mut cpu = Cpu::new();
    cpu.regs[2] = 0xAABB_CCDD;
    cpu.regs[8] = DATA_BASE;
    cpu.pc = 0;
    bus.write32::<BusRead>(0, encode_ual(SWL, 2, 8, 2));
    cpu.step(&mut bus);
    let result = bus.read32::<BusRead>(DATA_BASE);
    assert_eq!(result, 0xFFAA_BBCC);
}

#[test]
fn swl_offset_3_armazena_todos_32bits() {
    let mut bus = bus_with_bios_empty();
    bus.write32::<BusRead>(DATA_BASE, 0xDEAD_BEEF);
    let mut cpu = Cpu::new();
    cpu.regs[2] = 0x1234_5678;
    cpu.regs[8] = DATA_BASE;
    cpu.pc = 0;
    bus.write32::<BusRead>(0, encode_ual(SWL, 2, 8, 3));
    cpu.step(&mut bus);
    let result = bus.read32::<BusRead>(DATA_BASE);
    assert_eq!(result, 0x1234_5678);
}

#[test]
fn swr_offset_0_armazena_todos_32bits() {
    let mut bus = bus_with_bios_empty();
    bus.write32::<BusRead>(DATA_BASE, 0xDEAD_BEEF);
    let mut cpu = Cpu::new();
    cpu.regs[2] = 0x1234_5678;
    cpu.regs[8] = DATA_BASE;
    cpu.pc = 0;
    bus.write32::<BusRead>(0, encode_ual(SWR, 2, 8, 0));
    cpu.step(&mut bus);
    let result = bus.read32::<BusRead>(DATA_BASE);
    assert_eq!(result, 0x1234_5678);
}

#[test]
fn swr_offset_1_armazena_24bits_inferiores() {
    let mut bus = bus_with_bios_empty();
    bus.write32::<BusRead>(DATA_BASE, 0xCCBB_AADD);
    let mut cpu = Cpu::new();
    cpu.regs[2] = 0x11AA_BBCC;
    cpu.regs[8] = DATA_BASE;
    cpu.pc = 0;
    bus.write32::<BusRead>(0, encode_ual(SWR, 2, 8, 1));
    cpu.step(&mut bus);
    let result = bus.read32::<BusRead>(DATA_BASE);
    assert_eq!(result, 0xAABB_CCDD);
}

#[test]
fn swr_offset_2_armazena_16bits_inferiores() {
    let mut bus = bus_with_bios_empty();
    bus.write32::<BusRead>(DATA_BASE, 0xAAAA_BBCC);
    let mut cpu = Cpu::new();
    cpu.regs[2] = 0x1234_DDEE;
    cpu.regs[8] = DATA_BASE;
    cpu.pc = 0;
    bus.write32::<BusRead>(0, encode_ual(SWR, 2, 8, 2));
    cpu.step(&mut bus);
    let result = bus.read32::<BusRead>(DATA_BASE);
    assert_eq!(result, 0xDDEE_BBCC);
}

#[test]
fn swr_offset_3_armazena_8bits_inferiores() {
    let mut bus = bus_with_bios_empty();
    bus.write32::<BusRead>(DATA_BASE, 0xAAAA_BBCC);
    let mut cpu = Cpu::new();
    cpu.regs[2] = 0x1234_56DD;
    cpu.regs[8] = DATA_BASE;
    cpu.pc = 0;
    bus.write32::<BusRead>(0, encode_ual(SWR, 2, 8, 3));
    cpu.step(&mut bus);
    let result = bus.read32::<BusRead>(DATA_BASE);
    assert_eq!(result, 0xDDAA_BBCC);
}

#[test]
fn lwl_seguido_de_lwr_no_mesmo_registrador_sem_nop_entre_eles() {
    let mut bus = bus_with_bios_empty();
    bus.write32::<BusRead>(DATA_BASE, 0xDDCC_BBAA);
    bus.write32::<BusRead>(DATA_BASE + 4, 0x1122_3344);
    let mut cpu = Cpu::new();
    cpu.regs[8] = DATA_BASE + 1;
    cpu.pc = 0;
    bus.write32::<BusRead>(0, encode_ual(LWL, 2, 8, 3));
    bus.write32::<BusRead>(4, encode_ual(LWR, 2, 8, 0));
    bus.write32::<BusRead>(8, nop());
    cpu.step(&mut bus);
    cpu.step(&mut bus);
    cpu.step(&mut bus);
    assert_eq!(cpu.regs[2], 0x44DD_CCBB);
}

#[test]
fn lwl_lwr_registradores_diferentes_independentes() {
    let mut bus = bus_with_bios_empty();
    bus.write32::<BusRead>(DATA_BASE, 0xDDCC_BBAA);
    bus.write32::<BusRead>(DATA_BASE + 4, 0x1122_3344);
    let mut cpu = Cpu::new();
    cpu.regs[8] = DATA_BASE + 1;
    cpu.regs[2] = 0xDEAD_BEEF;
    cpu.regs[3] = 0xDEAD_BEEF;
    cpu.pc = 0;
    bus.write32::<BusRead>(0, encode_ual(LWL, 2, 8, 3));
    bus.write32::<BusRead>(4, encode_ual(LWR, 3, 8, 0));
    bus.write32::<BusRead>(8, nop());
    cpu.step(&mut bus);
    cpu.step(&mut bus);
    cpu.step(&mut bus);
    assert_eq!(cpu.regs[2], 0x44AD_BEEF);
    assert_eq!(cpu.regs[3], 0xDEDD_CCBB);
}

#[test]
fn lwl_mantem_bits_nao_transferidos() {
    let mut bus = bus_with_bios_empty();
    bus.write32::<BusRead>(DATA_BASE, 0x1234_5678);
    let mut cpu = Cpu::new();
    cpu.regs[8] = DATA_BASE;
    cpu.regs[2] = 0x1234_5678;
    cpu.pc = 0;
    bus.write32::<BusRead>(0, encode_ual(LWL, 2, 8, 0));
    bus.write32::<BusRead>(4, nop());
    cpu.step(&mut bus);
    cpu.step(&mut bus);
    assert_eq!(cpu.regs[2], 0x7834_5678);
}

#[test]
fn lwr_mantem_bits_nao_transferidos() {
    let mut bus = bus_with_bios_empty();
    bus.write32::<BusRead>(DATA_BASE, 0xDDCC_BBAA);
    let mut cpu = Cpu::new();
    cpu.regs[8] = DATA_BASE;
    cpu.regs[2] = 0x8765_4321;
    cpu.pc = 0;
    bus.write32::<BusRead>(0, encode_ual(LWR, 2, 8, 3));
    bus.write32::<BusRead>(4, nop());
    cpu.step(&mut bus);
    cpu.step(&mut bus);
    assert_eq!(cpu.regs[2], 0x8765_43DD);
}

#[test]
fn lwl_com_imediato_negativo() {
    let mut bus = bus_with_bios_empty();
    bus.write32::<BusRead>(DATA_BASE, 0xAABB_CCDD);
    let mut cpu = Cpu::new();
    cpu.regs[8] = DATA_BASE + 8;
    cpu.regs[2] = 0xFFFF_FFFF;
    cpu.pc = 0;
    bus.write32::<BusRead>(0, encode_ual(LWL, 2, 8, (-8i16) as u16));
    bus.write32::<BusRead>(4, nop());
    cpu.step(&mut bus);
    cpu.step(&mut bus);
    assert_eq!(cpu.regs[2], 0xDDFF_FFFF);
}

#[test]
fn lwr_com_imediato_negativo() {
    let mut bus = bus_with_bios_empty();
    bus.write32::<BusRead>(DATA_BASE, 0xDDCC_BBAA);
    let mut cpu = Cpu::new();
    cpu.regs[8] = DATA_BASE + 8;
    cpu.regs[2] = 0xFF00_0000;
    cpu.pc = 0;
    bus.write32::<BusRead>(0, encode_ual(LWR, 2, 8, (-5i16) as u16));
    bus.write32::<BusRead>(4, nop());
    cpu.step(&mut bus);
    cpu.step(&mut bus);
    assert_eq!(cpu.regs[2], 0xFF00_00DD);
}

#[test]
fn swl_endereco_forca_alinhado_armazena_no_offset_3() {
    let mut bus = bus_with_bios_empty();
    bus.write32::<BusRead>(DATA_BASE, 0xFFFF_FFFF);
    let mut cpu = Cpu::new();
    cpu.regs[2] = 0x1234_5678;
    cpu.regs[8] = DATA_BASE + 2;
    cpu.pc = 0;
    bus.write32::<BusRead>(0, encode_ual(SWL, 2, 8, 1));
    cpu.step(&mut bus);
    let result = bus.read32::<BusRead>(DATA_BASE);
    assert_eq!(result, 0x1234_5678);
}

#[test]
fn swr_endereco_forca_alinhado_armazena_no_offset_0() {
    let mut bus = bus_with_bios_empty();
    bus.write32::<BusRead>(DATA_BASE + 4, 0xFFFF_FFFF);
    let mut cpu = Cpu::new();
    cpu.regs[2] = 0x1234_5678;
    cpu.regs[8] = DATA_BASE + 2;
    cpu.pc = 0;
    bus.write32::<BusRead>(0, encode_ual(SWR, 2, 8, 2));
    cpu.step(&mut bus);
    let result = bus.read32::<BusRead>(DATA_BASE + 4);
    assert_eq!(result, 0x1234_5678);
}

#[test]
fn round_trip_swl_swr_seguido_de_lwl_lwr() {
    let mut bus = bus_with_bios_empty();
    bus.write32::<BusRead>(DATA_BASE, 0xAABB_CCDD);
    bus.write32::<BusRead>(DATA_BASE + 4, 0x1122_3344);
    let mut cpu = Cpu::new();
    cpu.regs[8] = DATA_BASE + 1;
    cpu.regs[2] = 0x1122_3344;
    cpu.pc = 0;
    bus.write32::<BusRead>(0, encode_ual(SWL, 2, 8, 3));
    bus.write32::<BusRead>(4, encode_ual(SWR, 2, 8, 0));
    bus.write32::<BusRead>(8, encode_ual(LWL, 2, 8, 3));
    bus.write32::<BusRead>(12, encode_ual(LWR, 2, 8, 0));
    bus.write32::<BusRead>(16, nop());
    cpu.step(&mut bus);
    cpu.step(&mut bus);
    assert_eq!(bus.read32::<BusRead>(DATA_BASE), 0x2233_44DD);
    assert_eq!(bus.read32::<BusRead>(DATA_BASE + 4), 0x1122_3311);
    cpu.step(&mut bus);
    cpu.step(&mut bus);
    cpu.step(&mut bus);
    assert_eq!(cpu.regs[2], 0x1122_3344);
}

#[test]
fn lwl_enxerga_load_delay_de_lw_no_mesmo_registrador() {
    let mut bus = bus_with_bios_empty();
    bus.write32::<BusRead>(DATA_BASE, 0xAAAA_BBBB);
    bus.write32::<BusRead>(DATA_BASE + 8, 0x0000_00DD);
    let mut cpu = Cpu::new();
    cpu.regs[8] = DATA_BASE;
    cpu.regs[9] = DATA_BASE + 8;
    cpu.regs[2] = 0xDEAD_BEEF;
    cpu.pc = 0;
    bus.write32::<BusRead>(0, encode_i_type(0x23, 2, 8, 0));
    bus.write32::<BusRead>(4, encode_ual(LWL, 2, 9, 0));
    bus.write32::<BusRead>(8, nop());
    cpu.step(&mut bus);
    cpu.step(&mut bus);
    cpu.step(&mut bus);
    assert_eq!(cpu.regs[2], 0xDDAA_BBBB);
}
