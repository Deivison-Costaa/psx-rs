use psx_core::bus::BusRead;
use psx_core::cpu::Cpu;

mod support;
use support::asm::{bus_with_bios_empty, encode_i_type, nop};

fn mfc0(rt: u32, cop0_reg: u32) -> u32 {
    (0x10 << 26) | (rt << 16) | (cop0_reg << 11)
}

fn mtc0(rt: u32, cop0_reg: u32) -> u32 {
    (0x10 << 26) | (0x04 << 21) | (rt << 16) | (cop0_reg << 11)
}

fn rfe() -> u32 {
    (0x10 << 26) | (1 << 25) | 0x10
}

// ============================================================================
// A1 — RFE: SR com 0x34 → RFE → SR deve valer 0x3D
//         SR com 0x0C → RFE → SR deve valer 0x03
// Spec: bit2-3 → bit0-1, bit4-5 → bit2-3, bits 4-5 inalterados
// SR=0x34=110100b →  bit0←bit2=1, bit1←bit3=0, bit2←bit4=1, bit3←bit5=1,
// bits4-5=11 → 111101b = 0x3D
// SR=0x0C=001100b →  bit0←bit2=1, bit1←bit3=1, bit2←bit4=0, bit3←bit5=0,
// bits4-5=00 → 000011b = 0x03
// O literal 0x34 nao distingue a implementacao correta de uma que limpa
// apenas os bits 0-1 antes do OR — mutar (sr & !0xF) para (sr & !0x3)
// escapa com 0x34 (OR da 0x3D por acaso), mas 0x0C pega (0x03 vs 0x0F).
// ============================================================================
#[test]
fn sr_rfe_move_campos_ie_ku_corretamente() {
    // Caso 1: SR=0x34 → 0x3D
    {
        let mut bus = bus_with_bios_empty();
        let mut cpu = Cpu::new();
        cpu.pc = 0;

        cpu.regs[8] = 0x0000_0034;
        bus.write32::<BusRead>(0x0000, mtc0(8, 12));
        bus.write32::<BusRead>(0x0004, rfe());
        bus.write32::<BusRead>(0x0008, mfc0(10, 12));
        bus.write32::<BusRead>(0x000C, nop());

        cpu.step(&mut bus);
        cpu.step(&mut bus);
        cpu.step(&mut bus);
        cpu.step(&mut bus);

        assert_eq!(
            cpu.regs[10], 0x0000_003D,
            "RFE: SR=0x34 → 0x3D (bit2→0, bit3→1, bit4→2, bit5→3, bits4-5 intactos)"
        );
    }

    // Caso 2: SR=0x0C → 0x03 (pega o mutante (sr & !0x3) no lugar de (sr & !0xF))
    {
        let mut bus = bus_with_bios_empty();
        let mut cpu = Cpu::new();
        cpu.pc = 0;

        // CU0 ligado: apos o RFE o SR fica com KUc=1 (usuario) e sem CU0 o `mfc0` seguinte
        // lancaria 0Bh — § cop0r16-r31 - Garbage (L892) de docs/reference/02-cpu.md.
        cpu.regs[8] = 0x1000_000C;
        bus.write32::<BusRead>(0x0000, mtc0(8, 12));
        bus.write32::<BusRead>(0x0004, rfe());
        bus.write32::<BusRead>(0x0008, mfc0(10, 12));
        bus.write32::<BusRead>(0x000C, nop());

        cpu.step(&mut bus);
        cpu.step(&mut bus);
        cpu.step(&mut bus);
        cpu.step(&mut bus);

        assert_eq!(
            cpu.regs[10], 0x1000_0003,
            "RFE: SR=0x0C → 0x03 (pega mutante que usa (sr & !0x3) em vez de (sr & !0xF))"
        );
    }
}

// ============================================================================
// A2 — CAUSE só aceita escrita nos bits 8-9
// Spec: CAUSE é "Read-only, except, Bit8-9 are R/W"
// Semeia CAUSE=0x20, executa MTC0 com 0xFFFF_FFFF → CAUSE deve valer 0x320
// ============================================================================
#[test]
fn cause_mascara_escrita_apenas_bits_8_e_9() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    cpu.pc = 0;

    cpu.cop0[13] = 0x0000_0020;
    cpu.regs[8] = 0xFFFF_FFFF;
    bus.write32::<BusRead>(0x0000, mtc0(8, 13));
    bus.write32::<BusRead>(0x0004, mfc0(10, 13));
    bus.write32::<BusRead>(0x0008, nop());

    cpu.step(&mut bus);
    cpu.step(&mut bus);
    cpu.step(&mut bus);

    assert_eq!(
        cpu.regs[10], 0x0000_0320,
        "CAUSE: seed 0x20, MTC0 c/ 0xFFFF_FFFF → 0x320 (bits 8-9 = Sw)"
    );
}

// ============================================================================
// A3 — MFC0 tem load delay de UM opcode
// Spec: "the next opcode cannot use the destination register as operand"
// ============================================================================
#[test]
fn mfc0_tem_load_delay_de_um_opcode() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    cpu.pc = 0;

    cpu.cop0[15] = 0x0000_0002;

    bus.write32::<BusRead>(0x0000, mfc0(2, 15));
    bus.write32::<BusRead>(0x0004, encode_i_type(0x0D, 3, 2, 0x0000));
    bus.write32::<BusRead>(0x0008, nop());

    cpu.step(&mut bus);
    cpu.step(&mut bus);
    cpu.step(&mut bus);

    assert_eq!(
        cpu.regs[3], 0x0000_0000,
        "MFC0 load delay: ORI ve r2 antigo (=0), nao o PRID (=0x2)"
    );
    assert_eq!(
        cpu.regs[2], 0x0000_0002,
        "MFC0 load delay: r2 atualizado apos o delay slot"
    );
}

// ============================================================================
// Armadilha 3 — MTC0 NÃO tem store delay
// Spec: "one can read from a cop0 register immediately after writing to it"
// ============================================================================
#[test]
fn mtc0_nao_tem_store_delay() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    cpu.pc = 0;

    cpu.regs[8] = 0xDEAD_BEEF;
    cpu.cop0[12] = 0x0000_0000;

    bus.write32::<BusRead>(0x0000, mtc0(8, 12));
    bus.write32::<BusRead>(0x0004, mfc0(10, 12));
    bus.write32::<BusRead>(0x0008, nop());

    cpu.step(&mut bus);
    cpu.step(&mut bus);
    cpu.step(&mut bus);

    assert_eq!(
        cpu.regs[10], 0xDEAD_BEEF,
        "MTC0 sem store delay: MFC0 imediatamente apos MTC0 le o valor escrito"
    );
}

// ============================================================================
// PRID retorna 0x00000002 (CXD8606CQ)
// ============================================================================
#[test]
fn prid_retorna_0x00000002() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    cpu.pc = 0;

    bus.write32::<BusRead>(0x0000, mfc0(10, 15));
    bus.write32::<BusRead>(0x0004, nop());

    cpu.step(&mut bus);
    cpu.step(&mut bus);

    assert_eq!(
        cpu.regs[10], 0x0000_0002,
        "PRID (cop0r15) deve retornar 0x00000002"
    );
}

// ============================================================================
// EPC — gravavel como comportamento ASSUMIDO (armadilha 4)
// ============================================================================
#[test]
fn epc_gravavel_comportamento_assumido() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    cpu.pc = 0;

    cpu.regs[8] = 0x8000_1234;
    bus.write32::<BusRead>(0x0000, mtc0(8, 14));
    bus.write32::<BusRead>(0x0004, mfc0(10, 14));
    bus.write32::<BusRead>(0x0008, nop());

    cpu.step(&mut bus);
    cpu.step(&mut bus);
    cpu.step(&mut bus);

    assert_eq!(
        cpu.regs[10], 0x8000_1234,
        "comportamento ASSUMIDO: EPC (cop0r14) aceita escrita via MTC0"
    );
}

// ============================================================================
// BadVaddr — gravavel como comportamento ASSUMIDO (armadilha 4)
// ============================================================================
#[test]
fn badvaddr_gravavel_comportamento_assumido() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    cpu.pc = 0;

    cpu.regs[8] = 0x1FC0_0000;
    bus.write32::<BusRead>(0x0000, mtc0(8, 8));
    bus.write32::<BusRead>(0x0004, mfc0(10, 8));
    bus.write32::<BusRead>(0x0008, nop());

    cpu.step(&mut bus);
    cpu.step(&mut bus);
    cpu.step(&mut bus);

    assert_eq!(
        cpu.regs[10], 0x1FC0_0000,
        "comportamento ASSUMIDO: BadVaddr (cop0r8) aceita escrita via MTC0"
    );
}

// ============================================================================
// Escrita em SR via MTC0 — registrador comum R/W
// ============================================================================
#[test]
fn sr_aceita_escrita_completa_via_mtc0() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    cpu.pc = 0;

    cpu.regs[8] = 0xABCD_0005;
    bus.write32::<BusRead>(0x0000, mtc0(8, 12));
    bus.write32::<BusRead>(0x0004, mfc0(10, 12));
    bus.write32::<BusRead>(0x0008, nop());

    cpu.step(&mut bus);
    cpu.step(&mut bus);
    cpu.step(&mut bus);

    assert_eq!(
        cpu.regs[10], 0xABCD_0005,
        "SR (cop0r12) deve refletir o valor escrito via MTC0"
    );
}

// ============================================================================
// Registradores garbage (r16-r31) — leitura nao causa excecao
// ============================================================================
#[test]
fn registrador_garbage_nao_dispara_excecao() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    cpu.pc = 0;

    bus.write32::<BusRead>(0x0000, mfc0(10, 16));
    bus.write32::<BusRead>(0x0004, nop());

    cpu.step(&mut bus);
    cpu.step(&mut bus);

    assert_eq!(
        cpu.regs[10], 0,
        "reg garbage (cop0r16): nao dispara excecao, valor e arbitrario"
    );
}

// ============================================================================
// MTC0 com rt=R0 escreve 0 no COP0 (R0 é hardwired a zero)
// ============================================================================
#[test]
fn mtc0_com_r0_escreve_zero() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    cpu.pc = 0;

    cpu.cop0[12] = 0xFFFF_FFFF;
    bus.write32::<BusRead>(0x0000, mtc0(0, 12));
    bus.write32::<BusRead>(0x0004, mfc0(10, 12));
    bus.write32::<BusRead>(0x0008, nop());

    cpu.step(&mut bus);
    cpu.step(&mut bus);
    cpu.step(&mut bus);

    assert_eq!(
        cpu.regs[10], 0x0000_0000,
        "MTC0 com rt=R0: escreve 0 (R0 hardwired zero) no COP0, SR fica zerado"
    );
}
