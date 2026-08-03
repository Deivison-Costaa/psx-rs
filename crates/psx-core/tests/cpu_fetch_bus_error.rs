mod support;

use psx_core::bus::{Bus, BusWrite};
use psx_core::cpu::Cpu;
use support::asm;

const VETOR_GERAL: u32 = 0x8000_0080;
const EXCODE_IBE: u32 = 0x06;
const BASE: u32 = 0x0000_1000;
const SCRATCHPAD: u32 = 0x1F80_0000;
const I_STAT: u32 = 0x1F80_1070;
const MDEC: u32 = 0x1F80_1820;
const DMA0_MADR: u32 = 0x1F80_1080;
const DPCR: u32 = 0x1F80_10F0;
const SPU_BASE: u32 = 0x1F80_1C00;

fn jr(rs: u32) -> u32 {
    asm::encode_special(0x08, 0, 0, rs)
}

fn lui(rt: u32, imm: u16) -> u32 {
    asm::encode_i_type(0x0F, rt, 0, imm)
}

fn ori(rt: u32, rs: u32, imm: u16) -> u32 {
    asm::encode_i_type(0x0D, rt, rs, imm)
}

fn excode(cpu: &Cpu) -> u32 {
    (cpu.cop0[13] >> 2) & 0x1F
}

/// Monta `lui/ori` do alvo em $t0 e um `jr` para ele, e roda ate vetorizar. Mesmo padrao de
/// `salta_para` em cpu_fetch_desalinhado.rs.
fn salta_para(destino: u32) -> Cpu {
    let mut bus = asm::bus_with_bios_empty();
    let mut cpu = Cpu::new();

    let programa = [
        lui(8, (destino >> 16) as u16),
        ori(8, 8, (destino & 0xFFFF) as u16),
        jr(8),
        asm::nop(),
    ];
    for (i, palavra) in programa.iter().enumerate() {
        bus.write32::<BusWrite>(BASE + (i as u32) * 4, *palavra);
    }

    cpu.pc = BASE;
    for _ in 0..8 {
        cpu.step(&mut bus);
        if cpu.pc == VETOR_GERAL {
            break;
        }
    }
    cpu
}

// § Scratchpad (L114) de docs/reference/01-memory-map.md, L137-140: "the scratchpad is NOT
// executable. Attempts to jump to this region will cause a bus error on the first
// instruction fetch." Gabarito ps1-tests/cpu/code-in-io confirma: testCodeInScratchpad
// espera `getExceptionType() == busErrorInstruction` (06h).
#[test]
fn jr_para_scratchpad_levanta_bus_error_de_instrucao() {
    let cpu = salta_para(SCRATCHPAD);

    assert_eq!(cpu.pc, VETOR_GERAL, "a excecao tem de vetorizar");
    assert_eq!(
        excode(&cpu),
        EXCODE_IBE,
        "buscar opcode no scratchpad e IBE (06h), nao AdEL nem execucao silenciosa"
    );
}

// A spec local (§ Memory Exceptions, L156-160) so cita "Unused Memory Regions" de forma
// generica — nao diz QUAIS blocos de I/O faltam ao buscar instrucao. O gabarito
// (ps1-tests/cpu/code-in-io/psx.log) e o oraculo aqui, medido por instrumentacao na 0172:
// testCodeInInterrupts e testCodeInMDEC lancam IBE; testCodeInDMA0/DMAControl/SPU NAO
// lancam (ver testes de controle abaixo). So os dois blocos comprovados entram na regra.
#[test]
fn jr_para_interrupt_control_levanta_bus_error_de_instrucao() {
    let cpu = salta_para(I_STAT);

    assert_eq!(cpu.pc, VETOR_GERAL, "a excecao tem de vetorizar");
    assert_eq!(
        excode(&cpu),
        EXCODE_IBE,
        "buscar opcode em I_STAT/I_MASK e IBE (06h)"
    );
}

#[test]
fn jr_para_mdec_levanta_bus_error_de_instrucao() {
    let cpu = salta_para(MDEC);

    assert_eq!(cpu.pc, VETOR_GERAL, "a excecao tem de vetorizar");
    assert_eq!(
        excode(&cpu),
        EXCODE_IBE,
        "buscar opcode nos registradores do MDEC e IBE (06h)"
    );
}

// Controles negativos: testCodeInDMA0/DMAControl/SPU passam no gabarito esperando
// `wasExceptionThrown() == false` — DMA e SPU NAO fazem parte da regra, ao contrario do que
// uma leitura genérica de "todo o bloco de I/O Ports falta" sugeriria.
#[test]
fn dma0_dpcr_e_spu_nao_causam_bus_error_de_fetch() {
    assert!(
        !Bus::fetch_causa_bus_error(DMA0_MADR),
        "testCodeInDMA0 (gabarito): fetch em MADR do canal 0 nao lanca"
    );
    assert!(
        !Bus::fetch_causa_bus_error(DPCR),
        "testCodeInDMAControl (gabarito): fetch em DPCR nao lanca"
    );
    assert!(
        !Bus::fetch_causa_bus_error(SPU_BASE),
        "testCodeInSPU (gabarito): fetch no banco de registradores da SPU nao lanca"
    );
}

#[test]
fn badvaddr_e_epc_recebem_o_endereco_do_fetch_que_falhou() {
    let cpu = salta_para(SCRATCHPAD);

    assert_eq!(
        cpu.cop0[8], SCRATCHPAD,
        "BadVaddr recebe o endereco do fetch que falhou"
    );
    assert_eq!(
        cpu.cop0[14], SCRATCHPAD,
        "EPC aponta para o fetch que falhou, nao para o jr"
    );
}

// Controle: testCodeInRam passa no gabarito — RAM comum continua executavel.
#[test]
fn salto_para_ram_nao_levanta_excecao() {
    let mut bus = asm::bus_with_bios_empty();
    let mut cpu = Cpu::new();
    let destino = 0x0000_2000u32;
    bus.write32::<BusWrite>(destino, asm::nop());

    let programa = [
        lui(8, (destino >> 16) as u16),
        ori(8, 8, (destino & 0xFFFF) as u16),
        jr(8),
        asm::nop(),
    ];
    for (i, palavra) in programa.iter().enumerate() {
        bus.write32::<BusWrite>(BASE + (i as u32) * 4, *palavra);
    }

    cpu.pc = BASE;
    for _ in 0..5 {
        cpu.step(&mut bus);
    }

    assert_ne!(cpu.pc, VETOR_GERAL, "RAM comum continua executavel");
}
