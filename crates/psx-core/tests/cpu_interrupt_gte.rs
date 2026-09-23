use psx_core::bus::{Bus, BusRead};
use psx_core::cpu::Cpu;

mod support;
use support::asm::{addiu, bus_with_bios_empty, nop};

// § Interrupts vs GTE Commands (docs/reference/02-cpu.md L767-777): a interrupcao que
// cai "sobre" um cop2cmd executa o comando, mas o EPC aponta para ele; o handler da BIOS
// soma 4 ao EPC e pula o comando no retorno. Se o comando nao rodar antes do desvio,
// ele nunca roda (Crash Bandicoot: RTPT pulado deixa SXY velho e o poligono pisca).

const VETOR: u32 = 0x8000_0080;
const CODIGO: u32 = 0x0000_0100;
const CU2: u32 = 1 << 30;
const NCLIP: u32 = 0x1400006;
const MAC0: usize = 24;

fn gte_cmd(imm25: u32) -> u32 {
    (0x12 << 26) | (1 << 25) | (imm25 & 0x01FF_FFFF)
}

fn mfc2(rt: u32, rd: u32) -> u32 {
    (0x12 << 26) | (rt << 16) | (rd << 11)
}

fn triangulo_no_gte(bus: &mut Bus) {
    bus.gte_mut().write_data(12, 0);
    bus.gte_mut().write_data(13, 10);
    bus.gte_mut().write_data(14, 10 << 16);
    bus.gte_mut().write_data(MAC0, 0);
}

fn cenario(instr: u32) -> (Bus, Cpu) {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    cpu.pc = CODIGO;
    bus.write32::<BusRead>(CODIGO, instr);
    bus.write32::<BusRead>(CODIGO + 4, nop());
    triangulo_no_gte(&mut bus);
    bus.irq_mut().write_mask(0x0001);
    bus.irq_mut().raise(0);
    cpu.cop0[12] = CU2 | 0x0000_0401;
    (bus, cpu)
}

#[test]
fn interrupcao_sobre_comando_gte_executa_o_comando() {
    let (mut bus, mut cpu) = cenario(gte_cmd(NCLIP));
    cpu.step(&mut bus);

    assert_eq!(cpu.pc, VETOR, "a interrupcao tem de ser atendida");
    assert_eq!(
        cpu.cop0[14], CODIGO,
        "EPC continua apontando para o comando GTE"
    );
    assert_eq!(
        bus.gte().read_data(MAC0),
        100,
        "o NCLIP tem de rodar junto com a interrupcao: a BIOS pula o comando no retorno \
         (EPC+4), entao sem isso ele nunca executa (02-cpu.md L767-777)"
    );
}

#[test]
fn interrupcao_sobre_mfc2_nao_executa_a_instrucao() {
    let (mut bus, mut cpu) = cenario(mfc2(8, MAC0 as u32));
    cpu.regs[8] = 0x1234;
    cpu.step(&mut bus);

    assert_eq!(cpu.pc, VETOR);
    assert_eq!(
        cpu.regs[8], 0x1234,
        "so cop2cmd (opcode AND FE000000h = 4A000000h) roda junto; mfc2 volta pelo EPC"
    );
}

#[test]
fn interrupcao_sobre_instrucao_comum_nao_executa_a_instrucao() {
    let (mut bus, mut cpu) = cenario(addiu(8, 8, 1));
    cpu.step(&mut bus);

    assert_eq!(cpu.pc, VETOR);
    assert_eq!(
        cpu.regs[8], 0,
        "instrucao comum nao roda antes da interrupcao"
    );
}

#[test]
fn comando_gte_sem_cu2_nao_roda_na_interrupcao() {
    let (mut bus, mut cpu) = cenario(gte_cmd(NCLIP));
    cpu.cop0[12] = 0x0000_0401;
    cpu.step(&mut bus);

    assert_eq!(cpu.pc, VETOR);
    assert_eq!(
        bus.gte().read_data(MAC0),
        0,
        "sem COP2 habilitado o comando seria excecao, nao execucao"
    );
}
