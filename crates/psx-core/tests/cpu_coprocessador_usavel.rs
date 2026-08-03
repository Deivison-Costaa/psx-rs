use psx_core::bus::BusRead;
use psx_core::cpu::Cpu;

mod support;
use support::asm::{bus_with_bios_empty, nop};

const VETOR_GERAL: u32 = 0x8000_0080;
const COP_UNUSABLE: u32 = 0x0B;

/// SR com o bit CU<n> ligado ou desligado. Bit 1 (KUc) em 0 = modo kernel.
/// § cop0r12 - SR (L744-747) de docs/reference/02-cpu.md: bit 28 CU0, 29 CU1, 30 CU2, 31 CU3.
fn sr_com(cu: Option<u32>) -> u32 {
    match cu {
        Some(n) => 1u32 << (28 + n),
        None => 0,
    }
}

fn roda(opcode: u32, sr: u32) -> (u32, u32) {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    cpu.pc = 0;
    cpu.set_sr(sr);
    bus.write32::<BusRead>(0, opcode);
    bus.write32::<BusRead>(4, nop());
    cpu.step(&mut bus);
    (cpu.pc, (cpu.cop0[13] >> 2) & 0x1F)
}

fn lancou(opcode: u32, sr: u32) -> bool {
    let (pc, code) = roda(opcode, sr);
    pc == VETOR_GERAL && code == COP_UNUSABLE
}

fn cop(n: u32) -> u32 {
    (0x10 | n) << 26
}
fn swc(n: u32) -> u32 {
    (0x38 | n) << 26
}
fn lwc(n: u32) -> u32 {
    (0x30 | n) << 26
}

// As expectativas abaixo vem do gabarito de hardware real do ps1-tests (`cpu/cop/psx.log`),
// nao da minha leitura da spec: a prosa do psx-spx diz que load/store de cop0 "causes a
// Coprocessor Unusable Exception" sem qualificar, e o hardware nao lanca com CU0 ligado.

#[test]
fn cop2_exige_cu2_ligado() {
    assert!(
        lancou(cop(2), sr_com(None)),
        "GTE com CU2 desligado tem de lancar 0Bh (testCop2Disabled)"
    );
    assert!(
        !lancou(cop(2), sr_com(Some(2))),
        "GTE com CU2 ligado nao lanca (testCop2Enabled)"
    );
}

#[test]
fn store_e_load_de_cop2_exigem_cu2_ligado() {
    assert!(
        lancou(swc(2), sr_com(None)),
        "swc2 com CU2 desligado tem de lancar (testSwc2Disabled)"
    );
    assert!(
        lancou(lwc(2), sr_com(None)),
        "lwc2 com CU2 desligado tem de lancar"
    );
}

#[test]
fn coprocessadores_inexistentes_nao_lancam_quando_habilitados() {
    for n in [1u32, 3] {
        assert!(
            !lancou(cop(n), sr_com(Some(n))),
            "cop{n} com CU{n} ligado nao lanca no hardware (testCop{n}Enabled)"
        );
        assert!(
            !lancou(swc(n), sr_com(Some(n))),
            "swc{n} com CU{n} ligado nao lanca no hardware (testSwc{n}Enabled)"
        );
    }
}

#[test]
fn coprocessadores_inexistentes_lancam_quando_desabilitados() {
    for n in [1u32, 3] {
        assert!(
            lancou(cop(n), sr_com(None)),
            "cop{n} com CU{n} desligado lanca (testCop{n}Disabled)"
        );
    }
}

#[test]
fn store_de_cop0_nao_lanca_com_cu0_ligado() {
    assert!(
        !lancou(swc(0), sr_com(Some(0))),
        "swc0 com CU0 ligado nao lanca no hardware (testSwc0Enabled)"
    );
}

// A isencao de modo kernel vale para `cop0`, nao para `lwc0`/`swc0`: e essa assimetria que
// concilia a prosa do psx-spx com o gabarito. Sem esta assercao, apagar o `is_cop_op` do
// predicado passa despercebido.
#[test]
fn load_e_store_de_cop0_lancam_sem_cu0_mesmo_em_modo_kernel() {
    for opcode in [swc(0), lwc(0)] {
        assert!(
            lancou(opcode, 0),
            "modo kernel nao isenta load/store de cop0: sem CU0 tem de lancar 0Bh"
        );
    }
}

// § cop0r16-r31 - Garbage (L892) de docs/reference/02-cpu.md: em modo usuario com cop0
// desabilitado (SR.bit1=1 e SR.bit28=0) o acesso a cop0 lanca 0Bh.
#[test]
fn cop0_em_modo_usuario_sem_cu0_lanca() {
    let sr_usuario = 1 << 1;
    assert!(
        lancou(cop(0), sr_usuario),
        "cop0 em modo usuario com CU0 desligado lanca 0Bh"
    );
    assert!(
        !lancou(cop(0), 0),
        "em modo kernel o cop0 e usavel mesmo com CU0 desligado"
    );
}
