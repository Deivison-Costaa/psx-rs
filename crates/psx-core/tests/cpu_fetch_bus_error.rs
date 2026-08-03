mod support;

use psx_core::bus::BusWrite;
use psx_core::cpu::Cpu;
use support::asm;

const VETOR_GERAL: u32 = 0x8000_0080;
const EXCODE_IBE: u32 = 0x06;
const BASE: u32 = 0x0000_1000;
const SCRATCHPAD: u32 = 0x1F80_0000;
const IO_PORTS: u32 = 0x1F80_1070; // I_STAT, dentro do bloco de 4K de I/O Ports.

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

// § Memory Exceptions (L156) de docs/reference/01-memory-map.md, L160: "Bus Error ------>
// Unused Memory Regions (including Gaps in I/O Region)". testCodeInMDEC/Interrupts/SPU/
// DMA0/DMAControl do gabarito cobrem o mesmo bloco de 4K de I/O Ports (1F801000h-1F801FFFh)
// e esperam o mesmo IBE.
#[test]
fn jr_para_io_ports_levanta_bus_error_de_instrucao() {
    let cpu = salta_para(IO_PORTS);

    assert_eq!(cpu.pc, VETOR_GERAL, "a excecao tem de vetorizar");
    assert_eq!(excode(&cpu), EXCODE_IBE, "buscar opcode em I/O Ports e IBE (06h)");
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
