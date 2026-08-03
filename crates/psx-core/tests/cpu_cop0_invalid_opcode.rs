use psx_core::bus::BusRead;
use psx_core::cpu::Cpu;

mod support;
use support::asm::{bus_with_bios_empty, nop};

const VETOR_GERAL: u32 = 0x8000_0080;
const RESERVED_INSTRUCTION: u32 = 0x0A;

fn cop0_co(cop0cmd: u32) -> u32 {
    (0x10 << 26) | (1 << 25) | cop0cmd
}

fn lancou_reservado(opcode: u32) -> bool {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    cpu.pc = 0;
    bus.write32::<BusRead>(0x0000, opcode);
    bus.write32::<BusRead>(0x0004, nop());

    cpu.step(&mut bus);

    let cause = (cpu.cop0[13] >> 2) & 0x1F;
    cpu.pc == VETOR_GERAL && cause == RESERVED_INSTRUCTION
}

// ps1-tests/cpu/cop testCop0InvalidOpcode monta exatamente 0x43E00000: primary=10h (COP0),
// bit25=1 (formato CO), bits21-24=Fh (regiao "N/A" do encoding, nao zerada de proposito) e
// cop0cmd (bits0-5) = 00h — nem RFE (10h) nem um dos quatro TLBxx. O gabarito
// (tests/exes/ps1-tests/cpu/cop/psx.log, linha 4) marca "pass" para esse teste, ou seja
// wasExceptionThrown() == false: o hardware real NAO lanca Reserved Instruction aqui, ao
// contrario do que faziamos (raise_exception(0x0A) para qualquer cop0cmd fora de 10h).
#[test]
fn cop0cmd_invalido_nao_tlb_nao_lanca_excecao() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    cpu.pc = 0;
    bus.write32::<BusRead>(0x0000, 0x43E0_0000);
    bus.write32::<BusRead>(0x0004, nop());

    cpu.step(&mut bus);

    assert_eq!(
        cpu.pc, 0x0000_0004,
        "testCop0InvalidOpcode (hardware real): cop0cmd=00h nao-TLB nao desvia para o vetor \
         de excecao — wasExceptionThrown() tem de dar false"
    );
    assert_eq!(
        (cpu.cop0[13] >> 2) & 0x1F,
        0,
        "CAUSE.ExcCode tem de permanecer 0: nenhuma excecao foi lancada"
    );
}

// § cop0cmd=01h,02h,06h,08h - TLBR,TLBWI,TLBWR,TLBP (L876-878) de docs/reference/02-cpu.md:
// "The PSX supports only one cop0cmd (cop0cmd=10h aka RFE). Trying to execute the TLBxx
// opcodes causes a Reserved Instruction Exception (excode=0Ah)." Estes quatro continuam
// lancando — so o restante da faixa (10..=1Fh, exceto 10h e os quatro TLB) deixou de lancar.
#[test]
fn tlb_ops_continuam_lancando_reservado() {
    for cop0cmd in [0x01u32, 0x02, 0x06, 0x08] {
        assert!(
            lancou_reservado(cop0_co(cop0cmd)),
            "cop0cmd={cop0cmd:#04x} (TLBxx) tem de lancar Reserved Instruction (0Ah)"
        );
    }
}

// RFE (cop0cmd=10h) permanece um no-op de excecao: e a unica operacao definida na faixa.
#[test]
fn rfe_continua_sem_lancar_excecao() {
    assert!(
        !lancou_reservado(cop0_co(0x10)),
        "RFE (cop0cmd=10h) nao lanca excecao"
    );
}
