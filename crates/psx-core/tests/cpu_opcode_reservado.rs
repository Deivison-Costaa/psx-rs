use psx_core::bus::BusRead;
use psx_core::cpu::Cpu;

mod support;
use support::asm::{bus_with_bios_empty, encode_j_type};

#[test]
fn opcode_primario_inexistente_gera_ri() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    cpu.pc = 0;

    bus.write32::<BusRead>(0x0000, 0x3F << 26);

    cpu.step(&mut bus);

    let cause = cpu.cop0[13];
    assert_eq!(
        cause & 0x7C,
        0x0A << 2,
        "A1: CAUSE.ExcCode = 0Ah (RI). CAUSE=0x{:08X}",
        cause
    );
    assert_eq!(
        cpu.cop0[14], 0x0000_0000,
        "A1: EPC = endereco da instrucao reservada (0x00000000)"
    );
    assert_eq!(
        cpu.pc, 0x8000_0080,
        "A1: PC = vetor geral de excecao (0x80000080)"
    );
}

#[test]
fn secondary_inexistente_gera_ri() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    cpu.pc = 0;

    bus.write32::<BusRead>(0x0000, 0x3E);

    cpu.step(&mut bus);

    let cause = cpu.cop0[13];
    assert_eq!(
        cause & 0x7C,
        0x0A << 2,
        "A1b: secondary inexistente → ExcCode=0Ah. CAUSE=0x{:08X}",
        cause
    );
    assert_eq!(cpu.cop0[14], 0x0000_0000);
    assert_eq!(cpu.pc, 0x8000_0080);
}

#[test]
fn swc0_gera_cpu() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    cpu.pc = 0;

    bus.write32::<BusRead>(0x0000, 0x38 << 26);

    cpu.step(&mut bus);

    let cause = cpu.cop0[13];
    assert_eq!(
        cause & 0x7C,
        0x0B << 2,
        "A2: SWC0 → CAUSE.ExcCode = 0Bh (CpU), nao 0Ah. CAUSE=0x{:08X}",
        cause
    );
    assert_eq!(cpu.cop0[14], 0x0000_0000);
    assert_eq!(cpu.pc, 0x8000_0080);
}

#[test]
fn lwc0_gera_cpu() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    cpu.pc = 0;

    bus.write32::<BusRead>(0x0000, 0x30 << 26);

    cpu.step(&mut bus);

    let cause = cpu.cop0[13];
    assert_eq!(
        cause & 0x7C,
        0x0B << 2,
        "A2b: LWC0 → CAUSE.ExcCode = 0Bh. CAUSE=0x{:08X}",
        cause
    );
}

#[test]
fn cop1_gera_cpu() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    cpu.pc = 0;

    bus.write32::<BusRead>(0x0000, 0x11 << 26);

    cpu.step(&mut bus);

    let cause = cpu.cop0[13];
    assert_eq!(
        cause & 0x7C,
        0x0B << 2,
        "A2c: COP1 (0x11) → CpU 0Bh. CAUSE=0x{:08X}",
        cause
    );
}

#[test]
fn cop3_gera_cpu() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    cpu.pc = 0;

    bus.write32::<BusRead>(0x0000, 0x13 << 26);

    cpu.step(&mut bus);

    let cause = cpu.cop0[13];
    assert_eq!(
        cause & 0x7C,
        0x0B << 2,
        "A2d: COP3 (0x13) → CpU 0Bh. CAUSE=0x{:08X}",
        cause
    );
}

#[test]
fn opcode_reservado_em_delay_slot_seta_bd_e_epc_no_branch() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    cpu.pc = 0;

    bus.write32::<BusRead>(0x0000, encode_j_type(0x03, 2));
    bus.write32::<BusRead>(0x0004, 0x3F << 26);

    cpu.step(&mut bus);
    cpu.step(&mut bus);

    let cause = cpu.cop0[13];
    assert_eq!(
        cause & 0x7C,
        0x0A << 2,
        "A3: CAUSE.ExcCode = 0Ah (RI). CAUSE=0x{:08X}",
        cause
    );
    assert!(
        cause & (1 << 31) != 0,
        "A3: CAUSE.BD = 1 (excecao em delay slot). CAUSE=0x{:08X}",
        cause
    );
    assert!(
        cause & (1 << 30) != 0,
        "A3: CAUSE.BT = 1 (branch tomado). CAUSE=0x{:08X}",
        cause
    );
    assert_eq!(
        cpu.cop0[14], 0x0000_0000,
        "A3: EPC = endereco do branch (0x00000000), nao do delay slot (0x00000004)"
    );
    assert_eq!(cpu.pc, 0x8000_0080);
}

#[test]
fn swc0_em_delay_slot_gera_cpu_com_bd() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    cpu.pc = 0;

    bus.write32::<BusRead>(0x0000, encode_j_type(0x03, 2));
    bus.write32::<BusRead>(0x0004, 0x38 << 26);

    cpu.step(&mut bus);
    cpu.step(&mut bus);

    let cause = cpu.cop0[13];
    assert_eq!(
        cause & 0x7C,
        0x0B << 2,
        "A3b: SWC0 em delay slot → CpU=0Bh. CAUSE=0x{:08X}",
        cause
    );
    assert!(cause & (1 << 31) != 0, "A3b: BD = 1");
    assert_eq!(cpu.cop0[14], 0x0000_0000);
    assert_eq!(cpu.pc, 0x8000_0080);
}

#[test]
fn tlb_op_gera_ri() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    cpu.pc = 0;

    let tlbr = (0x10 << 26) | (1 << 25) | 0x01;
    bus.write32::<BusRead>(0x0000, tlbr);

    cpu.step(&mut bus);

    let cause = cpu.cop0[13];
    assert_eq!(
        cause & 0x7C,
        0x0A << 2,
        "TLB: TLBR → ExcCode=0Ah (RI). CAUSE=0x{:08X}",
        cause
    );
    assert_eq!(cpu.cop0[14], 0x0000_0000);
    assert_eq!(cpu.pc, 0x8000_0080);
}

#[test]
fn varios_primarios_reservados_nao_panicam() {
    let mut bus = bus_with_bios_empty();

    let reservados = [0x14u32, 0x15, 0x2F, 0x34, 0x3C, 0x3F];

    for &primary in &reservados {
        let mut cpu = Cpu::new();
        cpu.pc = 0;
        bus.write32::<BusRead>(0x0000, primary << 26);
        cpu.step(&mut bus);

        let exc = (cpu.cop0[13] >> 2) & 0x1F;
        assert_eq!(
            exc, 0x0A,
            "primary=0x{:02X}: ExcCode=0Ah esperado, veio 0x{:1X}",
            primary, exc
        );
        assert_eq!(
            cpu.pc, 0x8000_0080,
            "primary=0x{:02X}: PC nao foi para vetor",
            primary
        );
    }
}

#[test]
fn todos_primarios_cpu_geram_cpu() {
    let mut bus = bus_with_bios_empty();

    let cpu_primaries = [
        0x11u32, 0x12, 0x13, 0x30, 0x31, 0x32, 0x33, 0x38, 0x39, 0x3A, 0x3B,
    ];

    for &primary in &cpu_primaries {
        let mut cpu = Cpu::new();
        cpu.pc = 0;
        bus.write32::<BusRead>(0x0000, primary << 26);
        cpu.step(&mut bus);

        let exc = (cpu.cop0[13] >> 2) & 0x1F;
        assert_eq!(
            exc, 0x0B,
            "primary=0x{:02X}: ExcCode=0Bh (CpU) esperado, veio 0x{:1X}",
            primary, exc
        );
        assert_eq!(
            cpu.pc, 0x8000_0080,
            "primary=0x{:02X}: PC nao foi para vetor de excecao",
            primary
        );
    }
}
