use psx_core::bus::BusRead;
use psx_core::cpu::Cpu;

mod support;
use support::asm::{bus_with_bios_empty, encode_i_type, nop};

const CODIGO: u32 = 0x0000_0100;
const RAM: u32 = 0x8000_2000;
const SCRATCHPAD: u32 = 0x1F80_0010;
const IO_ON_DIE: u32 = 0x1F80_1070;
const BIOS: u32 = 0xBFC0_0000;

fn lw(rt: u32, rs: u32, imm: u16) -> u32 {
    encode_i_type(0x23, rt, rs, imm)
}

fn sw(rt: u32, rs: u32, imm: u16) -> u32 {
    encode_i_type(0x2B, rt, rs, imm)
}

fn ciclos_de(instr: u32, base: u32) -> u64 {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    cpu.pc = CODIGO;
    cpu.regs[8] = base;
    bus.write32::<BusRead>(CODIGO, instr);
    let antes = bus.total_cycles();
    cpu.step(&mut bus);
    bus.total_cycles() - antes
}

#[test]
fn lw_da_ram_principal_custa_7_ciclos() {
    assert_eq!(
        ciclos_de(lw(9, 8, 0), RAM),
        7,
        "`docs/reference/02-cpu.md` L268: main RAM custa 7 ciclos por lw, incluindo o ciclo de \
         emissão. Sem isso um laço de espera da BIOS termina cedo demais e a espera de VSync \
         nunca é satisfeita (item 4.4g)."
    );
}

#[test]
fn lw_do_scratchpad_custa_1_ciclo() {
    assert_eq!(
        ciclos_de(lw(9, 8, 0), SCRATCHPAD),
        1,
        "`docs/reference/02-cpu.md` L266: o scratchpad é SRAM on-chip, sem acesso ao barramento — \
         lê tão rápido quanto um registrador"
    );
}

#[test]
fn lw_de_io_on_die_custa_5_ciclos() {
    assert_eq!(
        ciclos_de(lw(9, 8, 0), IO_ON_DIE),
        5,
        "`docs/reference/02-cpu.md` L267: todo registrador de I/O on-die custa os mesmos 5 ciclos, \
         porque compartilham um único decodificador"
    );
}

#[test]
fn lw_da_bios_custa_27_ciclos() {
    assert_eq!(
        ciclos_de(lw(9, 8, 0), BIOS),
        27,
        "`docs/reference/02-cpu.md` L269: a ROM da BIOS é de 8 bits e custa 27..33 ciclos. \
         Fixamos a ponta baixa: o intervalo existe porque o atraso de barramento é programável e \
         varia entre consoles, e um número escolhido é melhor que um número sorteado."
    );
}

#[test]
fn store_na_ram_custa_1_ciclo() {
    assert_eq!(
        ciclos_de(sw(9, 8, 0), RAM),
        1,
        "`docs/reference/02-cpu.md` L305-306: store vai para a write-queue e executa em um único \
         ciclo. Não é simetria com o load — é fila de escrita."
    );
}

#[test]
fn instrucao_sem_acesso_a_memoria_custa_1_ciclo() {
    assert_eq!(
        ciclos_de(nop(), RAM),
        1,
        "controle: instrução que não toca o barramento continua custando 1 ciclo"
    );
}

#[test]
fn laco_de_espera_da_bios_cobre_um_frame() {
    const ORCAMENTO: u64 = 32_768;
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();

    let base = 0x8000_2000u32;
    cpu.pc = CODIGO;
    cpu.regs[29] = base;
    cpu.regs[8] = base;

    let corpo = [
        lw(2, 29, 0x1C),
        lw(24, 29, 0x1C),
        nop(),
        encode_i_type(0x09, 25, 24, 0xFFFF),
        nop(),
        sw(25, 29, 0x1C),
        nop(),
        lw(8, 8, 0),
        nop(),
        nop(),
        nop(),
        nop(),
    ];
    for (i, instr) in corpo.iter().enumerate() {
        bus.write32::<BusRead>(CODIGO + (i as u32) * 4, *instr);
    }

    let antes = bus.total_cycles();
    for _ in 0..corpo.len() {
        cpu.step(&mut bus);
    }
    let por_iteracao = bus.total_cycles() - antes;

    let frame = 566_187u64;
    assert!(
        por_iteracao * ORCAMENTO >= frame,
        "a espera de VSync da BIOS gasta um orçamento fixo de {ORCAMENTO} iterações de um laço de \
         12 instruções com 3 loads da RAM; ele TEM de cobrir um frame inteiro ({frame} ciclos), \
         senão a BIOS desiste antes do vblank e imprime `VSync: timeout` para sempre. Medido: \
         {por_iteracao} ciclos por iteração, {} no total. Com 1 ciclo por instrução dá 12 — 69% de \
         um frame, que foi exatamente o que travou o boot no item 4.4g.",
        por_iteracao * ORCAMENTO
    );
}
