use psx_core::bus::BusRead;
use psx_core::cpu::Cpu;

mod support;
use support::asm::*;

const SENTINELA: u32 = 0xDEAD_BEEF;
const NEGATIVO: u32 = 0xFFFF_FFFF;
const POSITIVO: u32 = 1;
const ALVO: u32 = 0x14;
const FALLTHROUGH: u32 = 0x8;

fn passo_bcondz(rt: u32, rs_val: u32, rs_reg: u32) -> (u32, u32) {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    cpu.pc = 0;
    cpu.regs[rs_reg as usize] = rs_val;
    if rs_reg != 31 {
        cpu.regs[31] = SENTINELA;
    }
    bus.write32::<BusRead>(0, encode_i_type(0x01, rt, rs_reg, 0x0004));
    bus.write32::<BusRead>(4, nop());
    cpu.step(&mut bus);
    let ra_apos_primeiro_passo = cpu.regs[31];
    cpu.step(&mut bus);
    (cpu.pc, ra_apos_primeiro_passo)
}

// Achado do relatorio Amidog (psxtest_cpu, familia b_0xNN): dos 32 valores de `rt` do
// opcode 000001b, a condicao de desvio depende so do bit0 (par = bltz, impar = bgez) e o
// link ($ra = pc+8) só ocorre para rt=10h/11h exatamente — nao para qualquer rt com bit4=1.
#[test]
fn varredura_dos_32_valores_de_rt() {
    for rt in 0u32..32 {
        let linka = rt == 0x10 || rt == 0x11;
        let desvia_com_negativo = rt & 1 == 0;
        let desvia_com_positivo = rt & 1 == 1;

        let (pc, ra) = passo_bcondz(rt, NEGATIVO, 5);
        assert_eq!(
            pc,
            if desvia_com_negativo {
                ALVO
            } else {
                FALLTHROUGH
            },
            "rt={rt:#04x} rs<0: pc"
        );
        assert_eq!(
            ra,
            if linka { 8 } else { SENTINELA },
            "rt={rt:#04x} rs<0: $ra"
        );

        let (pc, ra) = passo_bcondz(rt, POSITIVO, 5);
        assert_eq!(
            pc,
            if desvia_com_positivo {
                ALVO
            } else {
                FALLTHROUGH
            },
            "rt={rt:#04x} rs>=0: pc"
        );
        assert_eq!(
            ra,
            if linka { 8 } else { SENTINELA },
            "rt={rt:#04x} rs>=0: $ra"
        );
    }
}

#[test]
fn desvio_nao_tomado_ainda_assim_escreve_ra() {
    let (pc, ra) = passo_bcondz(0x10, POSITIVO, 5);
    assert_eq!(pc, FALLTHROUGH, "bltzal com rs>=0 nao desvia");
    assert_eq!(ra, 8, "bltzal linka mesmo sem desviar");
}

#[test]
fn rs_igual_ra_compara_valor_antigo_antes_do_link() {
    for (rt, espera_desvio) in [(0x10u32, true), (0x11u32, false)] {
        let (pc, ra) = passo_bcondz(rt, NEGATIVO, 31);
        assert_eq!(ra, 8, "rt={rt:#04x} rs=$ra: link sempre escreve pc+8");
        assert_eq!(
            pc,
            if espera_desvio { ALVO } else { FALLTHROUGH },
            "rt={rt:#04x} rs=$ra: a comparacao usa o valor de $ra ANTES do link, nao o pc+8"
        );
    }
}
