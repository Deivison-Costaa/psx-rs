use psx_core::bus::BusRead;
use psx_core::cpu::Cpu;

mod support;
use support::asm::{bus_with_bios_empty, encode_i_type, encode_special, nop};

const CODIGO: u32 = 0x0000_0100;
const RAM: u32 = 0x8000_2000;
const BIOS: u32 = 0xBFC0_0000;
const VETOR_EXCECAO: u32 = 0x8000_0080;

fn lw(rt: u32, rs: u32, imm: u16) -> u32 {
    encode_i_type(0x23, rt, rs, imm)
}

// Roda a instrucao em CODIGO (que pode ou nao lancar excecao) e devolve o custo, em ciclos,
// da PROXIMA instrucao executada depois dela — seja ela o handler (0x8000_0080) ou a
// instrucao seguinte em CODIGO.
fn ciclos_da_instrucao_seguinte(primeira: u32, base_rs: u32, pc_da_seguinte: u32) -> u64 {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    cpu.pc = CODIGO;
    cpu.regs[8] = base_rs;
    bus.write32::<BusRead>(CODIGO, primeira);
    bus.write32::<BusRead>(pc_da_seguinte, nop());

    cpu.step(&mut bus); // primeira instrucao: pode desviar pro handler de excecao
    let antes = bus.total_cycles();
    cpu.step(&mut bus); // a "seguinte": handler ou proxima em CODIGO
    bus.total_cycles() - antes
}

#[test]
fn excecao_de_load_desalinhado_na_ram_nao_cobra_o_handler() {
    // lw com rs=8 (RAM alinhada) e imm=1 -> endereco RAM+1, desalinhado: AdEL (04h).
    assert_eq!(
        ciclos_da_instrucao_seguinte(lw(9, 8, 1), RAM, VETOR_EXCECAO),
        1,
        "`docs/reference/02-cpu.md` L262-269 so tabela o custo de um acesso que chega ao \
         barramento; um `lw` que nunca chega la (AdEL por desalinhamento) nao pode deixar \
         `load_extra_cycles` setado pro handler herdar. `enter_exception` zera \
         `branch_target`/`delay_slot_pending`/`load_delay` mas ate agora nao zerava esse \
         acumulador — o NOP do handler custava 1+6=7 em vez de 1."
    );
}

#[test]
fn excecao_de_load_desalinhado_na_bios_nao_cobra_o_handler() {
    // Mesmo caso, mas com a regiao de custo mais caro (ROM, 27 ciclos) pra deixar o
    // vazamento grande e obvio se reaparecer.
    assert_eq!(
        ciclos_da_instrucao_seguinte(lw(9, 8, 1), BIOS, VETOR_EXCECAO),
        1,
        "mesmo caso do teste da RAM, mas com o custo de ROM (27 ciclos) — se o acumulador \
         vazasse, o NOP do handler custaria 1+26=27 em vez de 1"
    );
}

// `docs/reference/02-cpu.md` L262-263 diz que o custo de uma instrucao inclui "the 1-cycle
// issue": nenhum caminho de `step()` pode sair sem cobrar esse ciclo minimo. Os quatro casos
// abaixo sao justamente os caminhos que saiam de `step()` por `return` ANTES do
// `bus.tick_timers`, cobrando zero.
fn ciclos_de_um_step(prepara: impl FnOnce(&mut Cpu, &mut psx_core::bus::Bus)) -> u64 {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    cpu.pc = CODIGO;
    prepara(&mut cpu, &mut bus);
    let antes = bus.total_cycles();
    cpu.step(&mut bus);
    bus.total_cycles() - antes
}

#[test]
fn syscall_custa_o_ciclo_de_emissao() {
    let ciclos = ciclos_de_um_step(|_cpu, bus| {
        bus.write32::<BusRead>(CODIGO, encode_special(0x0C, 0, 0, 0));
    });
    assert_eq!(
        ciclos, 1,
        "SYSCALL e uma instrucao emitida; `docs/reference/02-cpu.md` L262-263 conta o ciclo \
         de emissao no custo de qualquer instrucao. O caminho de `pending_exception` em \
         `step()` retornava antes de `tick_timers`, entao todo o kernel do BIOS (que e feito \
         de syscalls) rodava com relogio emulado parado"
    );
}

#[test]
fn store_desalinhado_custa_o_ciclo_de_emissao() {
    // sw rt=0, imm=1 sobre rs=RAM (alinhada) -> endereco RAM+1: AdES (05h).
    let ciclos = ciclos_de_um_step(|cpu, bus| {
        cpu.regs[8] = RAM;
        bus.write32::<BusRead>(CODIGO, encode_i_type(0x2B, 0, 8, 1));
    });
    assert_eq!(
        ciclos, 1,
        "AdES tambem sai por `pending_exception`; o `sw` foi emitido e tem de custar o ciclo \
         de emissao de L262-263 mesmo sem chegar ao barramento"
    );
}

#[test]
fn fetch_desalinhado_custa_o_ciclo_de_emissao() {
    let ciclos = ciclos_de_um_step(|cpu, _bus| {
        cpu.pc = CODIGO | 2;
    });
    assert_eq!(
        ciclos, 1,
        "AdEL de fetch tem `return` proprio em `step()`, antes de qualquer contabilizacao; \
         um PC desalinhado fazia o emulador girar sem consumir tempo nenhum"
    );
}

#[test]
fn entrada_de_irq_custa_o_ciclo_de_emissao() {
    let ciclos = ciclos_de_um_step(|cpu, bus| {
        bus.irq_mut().raise(0);
        bus.write32::<BusRead>(0x1F80_1074, 0x001);
        cpu.cop0[12] = 0x0000_0401;
        bus.write32::<BusRead>(CODIGO, nop());
    });
    assert_eq!(
        ciclos, 1,
        "o desvio pro vetor de IRQ aborta a instrucao em PC e tem `return` proprio; sem \
         contabilizacao, cada entrada de IRQ era gratis no relogio emulado"
    );
}

#[test]
fn load_alinhado_continua_cobrando_a_regiao_normalmente() {
    // Controle: sem excecao, o load alinhado tem de continuar cobrando o custo cheio da
    // regiao, e a instrucao seguinte (sem load) continua custando 1 — a correcao do
    // vazamento nao pode zerar o acumulador ANTES do load ser cobrado.
    assert_eq!(
        ciclos_da_instrucao_seguinte(lw(9, 8, 0), BIOS, CODIGO + 4),
        1,
        "controle: sem excecao a instrucao seguinte ao load nao deveria ser afetada pelo \
         acumulador — ela nunca fez load nenhum"
    );
}
