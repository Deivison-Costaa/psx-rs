use psx_core::bus::BusRead;
use psx_core::cpu::Cpu;

mod support;
use support::asm::{bus_with_bios_empty, encode_i_type, nop};

const MADR0: u32 = 0x1F80_1080;
const DPCR: u32 = 0x1F80_10F0;
const RAM_ALVO: u32 = 0x0000_0100;

fn sb(rt: u32, rs: u32, imm: u16) -> u32 {
    encode_i_type(0x28, rt, rs, imm)
}
fn sh(rt: u32, rs: u32, imm: u16) -> u32 {
    encode_i_type(0x29, rt, rs, imm)
}
fn lw(rt: u32, rs: u32, imm: u16) -> u32 {
    encode_i_type(0x23, rt, rs, imm)
}

/// sb/sh no endereco `base`, com o registrador $9 valendo 0x12345678, seguido de lw do
/// mesmo endereco em $10. Tres passos: o `store`, o `lw` (inicia load delay) e um `nop`
/// (onde o load delay se resolve) — mesmo padrao de `mfc0_tem_load_delay_de_um_opcode`
/// em cpu_cop0_regs.rs.
fn armazena_e_le(store_op: u32, base: u32) -> u32 {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    cpu.pc = 0;
    cpu.regs[8] = base;
    cpu.regs[9] = 0x1234_5678;

    bus.write32::<BusRead>(0x0000, store_op);
    bus.write32::<BusRead>(0x0004, lw(10, 8, 0));
    bus.write32::<BusRead>(0x0008, nop());

    cpu.step(&mut bus);
    cpu.step(&mut bus);
    cpu.step(&mut bus);

    cpu.regs[10]
}

// § Caution - 8/16-bit writes to certain IO registers (L309) de docs/reference/02-cpu.md:
// "During an 8-bit or 16-bit store, all 32 bits of the GPR are placed on the bus. As such,
// when writing to certain 32-bit IO registers with an 8 or 16-bit store, it will behave
// like a 32-bit store, using the register's full value." O gabarito
// (tests/exes/ps1-tests/cpu/io-access-bitwidth/psx.log, linha "DMA0_ADDR") confirma: sb e
// sh em MADR do canal 0 com $9=0x12345678 leem de volta 0x00345678 — o mesmo valor de um
// sw puro, mascarado pelos 24 bits ja existentes em `Dma::write_madr`.
#[test]
fn sb_em_madr_usa_valor_cheio_do_registrador() {
    assert_eq!(
        armazena_e_le(sb(9, 8, 0), MADR0),
        0x0034_5678,
        "sb em MADR ch0: gabarito DMA0_ADDR (8bit) le de volta 0x345678"
    );
}

#[test]
fn sh_em_madr_usa_valor_cheio_do_registrador() {
    assert_eq!(
        armazena_e_le(sh(9, 8, 0), MADR0),
        0x0034_5678,
        "sh em MADR ch0: gabarito DMA0_ADDR (16bit) le de volta 0x345678"
    );
}

// DPCR nao mascara nada (todos os 32 bits sao R/W) — gabarito DMAC_CTRL le de volta o
// valor cheio 0x12345678 tanto para sb quanto sh.
#[test]
fn sb_em_dpcr_usa_valor_cheio_do_registrador() {
    assert_eq!(
        armazena_e_le(sb(9, 8, 0), DPCR),
        0x1234_5678,
        "sb em DPCR: gabarito DMAC_CTRL (8bit) le de volta 0x12345678"
    );
}

#[test]
fn sh_em_dpcr_usa_valor_cheio_do_registrador() {
    assert_eq!(
        armazena_e_le(sh(9, 8, 0), DPCR),
        0x1234_5678,
        "sh em DPCR: gabarito DMAC_CTRL (16bit) le de volta 0x12345678"
    );
}

// Controle: a RAM comum NAO tem essa peculiaridade de barramento — sb continua escrevendo
// so o byte baixo (gabarito RAM: 8bit -> 0x78, nao 0x12345678).
#[test]
fn sb_em_ram_nao_usa_valor_cheio_do_registrador() {
    assert_eq!(
        armazena_e_le(sb(9, 8, 0), RAM_ALVO),
        0x0000_0078,
        "sb em RAM comum: so o byte baixo e escrito, sem o eco de 32 bits"
    );
}
