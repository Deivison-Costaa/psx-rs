use psx_core::bus::BusRead;
use psx_core::cpu::Cpu;

mod support;
use support::asm::{bus_with_bios_empty, encode_i_type, nop};

const CODIGO: u32 = 0x0000_0100;
const RAM: u32 = 0x8000_2000;
const SCRATCHPAD: u32 = 0x1F80_0010;
const IO_ON_DIE: u32 = 0x1F80_1070;
const BIOS: u32 = 0xBFC0_0000;
const VETOR_EXCECAO: u32 = 0x8000_0080;
const CU2: u32 = 1 << 30;

fn lwc2(rt: u32, rs: u32, imm: u16) -> u32 {
    encode_i_type(0x32, rt, rs, imm)
}

fn swc2(rt: u32, rs: u32, imm: u16) -> u32 {
    encode_i_type(0x3A, rt, rs, imm)
}

fn ciclos_de(instr: u32, base: u32, sr: u32) -> u64 {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    cpu.pc = CODIGO;
    cpu.regs[8] = base;
    cpu.set_sr(sr);
    bus.write32::<BusRead>(CODIGO, instr);
    let antes = bus.total_cycles();
    cpu.step(&mut bus);
    bus.total_cycles() - antes
}

// § Load Timing (L260-269) de docs/reference/02-cpu.md: LWC2 le memoria como qualquer
// outro load, entao paga o mesmo custo por regiao. A spec de GTE (07-gte.md) nao documenta
// um timing proprio pra LWC2 -- a tabela geral de load e quem vale.

#[test]
fn lwc2_da_ram_custa_7_ciclos() {
    assert_eq!(
        ciclos_de(lwc2(0, 8, 0), RAM, CU2),
        7,
        "`docs/reference/02-cpu.md` L268: LWC2 le a RAM como qualquer lw, 7 ciclos"
    );
}

#[test]
fn lwc2_do_scratchpad_custa_1_ciclo() {
    assert_eq!(
        ciclos_de(lwc2(0, 8, 0), SCRATCHPAD, CU2),
        1,
        "`docs/reference/02-cpu.md` L266: scratchpad e SRAM on-chip, 1 ciclo"
    );
}

#[test]
fn lwc2_de_io_on_die_custa_5_ciclos() {
    assert_eq!(
        ciclos_de(lwc2(0, 8, 0), IO_ON_DIE, CU2),
        5,
        "`docs/reference/02-cpu.md` L267: registrador de I/O on-die, 5 ciclos"
    );
}

#[test]
fn lwc2_da_bios_custa_27_ciclos() {
    assert_eq!(
        ciclos_de(lwc2(0, 8, 0), BIOS, CU2),
        27,
        "`docs/reference/02-cpu.md` L269: ROM da BIOS, 27 ciclos — antes deste fix a LWC2 \
         custava 1 ciclo mesmo vindo da ROM, porque so os opcodes 0x20..=0x26 disparavam \
         `load_extra_cycles`"
    );
}

#[test]
fn swc2_na_ram_custa_1_ciclo() {
    assert_eq!(
        ciclos_de(swc2(0, 8, 0), RAM, CU2),
        1,
        "`docs/reference/02-cpu.md` L305-306: SWC2 e store, vai pra write-queue como \
         qualquer outro store — nao ganha o custo de regiao do load"
    );
}

// Controle: prova que o Degrau 1 (0209) era pre-requisito. Com CU2 desligado, LWC2 lanca
// CpU (0Bh) antes de tocar o barramento -- `load_extra_cycles` e setado no topo de
// `execute()` (mesma linha que atende lw/lh/etc) mas a excecao entra logo depois. Sem o
// reset em `enter_exception`, o NOP do handler herdaria o custo cheio da regiao (27 pra
// BIOS) mesmo o load nunca tendo acontecido.
#[test]
fn lwc2_com_cop2_desabilitado_lanca_e_nao_cobra_a_regiao_do_handler() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    cpu.pc = CODIGO;
    cpu.regs[8] = BIOS;
    cpu.set_sr(0); // CU2 desligado
    bus.write32::<BusRead>(CODIGO, lwc2(0, 8, 0));
    bus.write32::<BusRead>(VETOR_EXCECAO, nop());

    cpu.step(&mut bus); // LWC2 lanca CpU, desvia pro handler
    assert_eq!(
        cpu.pc, VETOR_EXCECAO,
        "CU2 desligado tem que lançar CpU (0Bh)"
    );
    let antes = bus.total_cycles();
    cpu.step(&mut bus); // NOP do handler
    assert_eq!(
        bus.total_cycles() - antes,
        1,
        "o NOP do handler não pode herdar o custo de região (27 ciclos da BIOS) de um LWC2 \
         que nunca chegou ao barramento"
    );
}
