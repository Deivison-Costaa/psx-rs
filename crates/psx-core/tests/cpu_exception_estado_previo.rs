use psx_core::bus::BusRead;
use psx_core::cpu::Cpu;

mod support;
use support::asm::{bus_with_bios_empty, encode_i_type};

fn syscall() -> u32 {
    0x0000_000C
}

// ============================================================================
// M-B — Duas excecoes em sequencia: BD e BT devem ser limpos entre excecoes.
// A primeira excecao em delay slot (beq r0,r0,+1 com syscall no slot) seta
// BD=1, BT=1 (CAUSE = 0xC000_0020). A segunda excecao e um syscall isolado
// em 0x0100 e deve produzir CAUSE = 0x0000_0020 (BD e BT zerados).
// Mutante: mascara (self.cop0[13] & !0x0000_007C) sem limpar bits 31-30.
// ============================================================================
#[test]
fn excecao_sequencial_limpa_bd_e_bt() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    cpu.pc = 0;

    // beq r0, r0, +1 em 0x0000 (sempre tomado)
    bus.write32::<BusRead>(0x0000, encode_i_type(0x04, 0, 0, 1));
    // syscall no delay slot em 0x0004
    bus.write32::<BusRead>(0x0004, syscall());

    cpu.step(&mut bus); // beq: delay_slot_pending, branch_taken, PC=0x0004
    cpu.step(&mut bus); // syscall no delay slot → excecao

    assert_eq!(
        cpu.cop0[13], 0xC000_0020u32,
        "M-B primeira excecao: BD=1, BT=1, ExcCode(Sys=08h) → 0xC0000020"
    );

    // Segunda excecao: syscall isolado em 0x0100
    cpu.pc = 0x0100;
    bus.write32::<BusRead>(0x0100, syscall());

    cpu.step(&mut bus); // syscall isolado → segunda excecao

    assert_eq!(
        cpu.cop0[13], 0x0000_0020,
        "M-B segunda excecao: BD=0, BT=0, ExcCode(Sys=08h) → 0x00000020"
    );
}
