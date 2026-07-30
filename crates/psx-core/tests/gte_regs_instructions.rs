use psx_core::bus::BusRead;
use psx_core::cpu::Cpu;

mod support;
use support::asm::{bus_with_bios_empty, nop};

fn mfc2(rt: u32, rd: u32) -> u32 {
    (0x12 << 26) | (rt << 16) | (rd << 11)
}

fn cfc2(rt: u32, rd: u32) -> u32 {
    (0x12 << 26) | (0x02 << 21) | (rt << 16) | (rd << 11)
}

fn mtc2(rt: u32, rd: u32) -> u32 {
    (0x12 << 26) | (0x04 << 21) | (rt << 16) | (rd << 11)
}

fn ctc2(rt: u32, rd: u32) -> u32 {
    (0x12 << 26) | (0x06 << 21) | (rt << 16) | (rd << 11)
}

fn lwc2_enc(rt: u32, rs: u32, imm: u16) -> u32 {
    (0x32 << 26) | (rs << 21) | (rt << 16) | (imm as u32)
}

fn swc2_enc(rt: u32, rs: u32, imm: u16) -> u32 {
    (0x3A << 26) | (rs << 21) | (rt << 16) | (imm as u32)
}

fn enable_cop2(cpu: &mut Cpu) {
    cpu.cop0[12] |= 1 << 30;
}

fn disable_cop2(cpu: &mut Cpu) {
    cpu.cop0[12] &= !(1 << 30);
}

// ============================================================================
// t1 — GTE inicia com 64 registradores zerados
// ============================================================================
#[test]
fn gte_inicia_com_64_registradores_zerados() {
    let cpu = Cpu::new();

    for reg in 0..32 {
        assert_eq!(
            cpu.gte.read_data(reg),
            0,
            "t1: cop2r{reg} (data) deve ser 0 apos construcao"
        );
        assert_eq!(
            cpu.gte.read_ctrl(reg),
            0,
            "t1: cop2r{} (ctrl) deve ser 0 apos construcao",
            reg + 32
        );
    }
}

// ============================================================================
// t2 — MFC2 le registrador de dados (0-31) para GPR
// Spec: 07-gte.md L137 — Data Register Summary cop2r0-31
// ============================================================================
#[test]
fn mfc2_le_registrador_de_dados_para_gpr() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    enable_cop2(&mut cpu);
    cpu.pc = 0;

    // Escreve em cop2r5 via MTC2 e le de volta via MFC2
    cpu.regs[8] = 0xDEAD_BEEF;
    bus.write32::<BusRead>(0x0000, mtc2(8, 5));
    bus.write32::<BusRead>(0x0004, nop());
    bus.write32::<BusRead>(0x0008, mfc2(9, 5));
    bus.write32::<BusRead>(0x000C, nop());

    cpu.step(&mut bus); // mtc2 r5 → r8
    cpu.step(&mut bus); // nop
    cpu.step(&mut bus); // mfc2 r5 → r9
    cpu.step(&mut bus); // nop (load delay)

    assert_eq!(
        cpu.regs[9], 0xDEAD_BEEF,
        "t2: MFC2 r5 → r9 deve ler 0xDEADBEEF"
    );
}

// ============================================================================
// t3 — CFC2 le registrador de controle (32-63) para GPR
// Spec: 07-gte.md L156 — Control Register Summary cop2r32-63
// ============================================================================
#[test]
fn cfc2_le_registrador_de_controle_para_gpr() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    enable_cop2(&mut cpu);
    cpu.pc = 0;

    // cop2r37 = TRX (control register 5, offset 32)
    cpu.regs[8] = 0x1234_5678;
    bus.write32::<BusRead>(0x0000, ctc2(8, 5));
    bus.write32::<BusRead>(0x0004, nop());
    bus.write32::<BusRead>(0x0008, cfc2(9, 5));
    bus.write32::<BusRead>(0x000C, nop());

    cpu.step(&mut bus); // ctc2 r5 → r8
    cpu.step(&mut bus); // nop
    cpu.step(&mut bus); // cfc2 r5 → r9
    cpu.step(&mut bus); // nop (load delay)

    assert_eq!(
        cpu.regs[9], 0x1234_5678,
        "t3: CFC2 r5 (cop2r37) → r9 deve ler 0x12345678"
    );
}

// ============================================================================
// t4 — MTC2 escreve GPR em registrador de dados (0-31)
// ============================================================================
#[test]
fn mtc2_escreve_gpr_em_registrador_de_dados() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    enable_cop2(&mut cpu);
    cpu.pc = 0;

    cpu.regs[8] = 0xCAFE_BABE;
    bus.write32::<BusRead>(0x0000, mtc2(8, 10));

    cpu.step(&mut bus);

    assert_eq!(
        cpu.gte.read_data(10),
        0xCAFE_BABE,
        "t4: MTC2 r8 → cop2r10 deve gravar 0xCAFEBABE"
    );
}

// ============================================================================
// t5 — CTC2 escreve GPR em registrador de controle (32-63)
// ============================================================================
#[test]
fn ctc2_escreve_gpr_em_registrador_de_controle() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    enable_cop2(&mut cpu);
    cpu.pc = 0;

    // cop2r56 = OFX (control register 24, offset 32)
    cpu.regs[8] = 0x00FF_00FF;
    bus.write32::<BusRead>(0x0000, ctc2(8, 24));

    cpu.step(&mut bus);

    assert_eq!(
        cpu.gte.read_ctrl(24),
        0x00FF_00FF,
        "t5: CTC2 r8 → cop2r56 deve gravar 0x00FF00FF"
    );
}

// ============================================================================
// t6 — MFC2 tem load delay de 1 instrucao
// Spec: 07-gte.md L101 — GTE Load Delay Slots: 1 instrucao para CFC2/MFC2
// ============================================================================
#[test]
fn mfc2_tem_load_delay_de_1_instrucao() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    enable_cop2(&mut cpu);
    cpu.pc = 0;

    // r8 comeca com 0, r9 escreve 0x42 via MTC2, r8 recebe via MFC2 com delay.
    // A instrucao seguinte (addiu r9, r8, 0) deve ver r8 = 0 (valor antigo).
    cpu.regs[9] = 0x42;
    // MTC2 r9, r5 (escreve 0x42 em cop2r5)
    bus.write32::<BusRead>(0x0000, mtc2(9, 5));
    // MFC2 r8, r5 (le cop2r5(0x42) → r8 com delay)
    bus.write32::<BusRead>(0x0004, mfc2(8, 5));
    // ADDIU r9, r8, 0 (instrucao no delay slot — usa r8 pendente, deve ver 0)
    bus.write32::<BusRead>(0x0008, (0x09 << 26) | (8 << 21) | (9 << 16));
    // NOP para resolver o load delay
    bus.write32::<BusRead>(0x000C, nop());

    cpu.step(&mut bus); // mtc2 (nada pendente)
    cpu.step(&mut bus); // mfc2 (load delay pendente para r8)
    // No delay slot, r8 ainda tem o valor antigo (0, nao 0x42)
    cpu.step(&mut bus); // addiu r9, r8, 0 → r9 = OLD r8 = 0
    cpu.step(&mut bus); // nop → load delay resolve, r8 = 0x42

    assert_eq!(
        cpu.regs[9], 0,
        "t6: no delay slot do MFC2, r8 tem valor antigo (0), r9 recebe 0"
    );
    assert_eq!(
        cpu.regs[8], 0x42,
        "t6: apos o delay slot, r8 recebe 0x42 do MFC2"
    );
}

// ============================================================================
// t7 — CFC2 tem load delay de 1 instrucao
// ============================================================================
#[test]
fn cfc2_tem_load_delay_de_1_instrucao() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    enable_cop2(&mut cpu);
    cpu.pc = 0;

    // r9 escreve 0x99 via CTC2, r8 le via CFC2 com delay.
    // r8 comeca com 0 — a instrucao seguinte deve ver o valor antigo.
    cpu.regs[9] = 0x99;
    // CTC2 r9, r10 (cop2r42)
    bus.write32::<BusRead>(0x0000, ctc2(9, 10));
    // CFC2 r8, r10 (le cop2r42 → r8 com delay)
    bus.write32::<BusRead>(0x0004, cfc2(8, 10));
    // ADDIU r9, r8, 0 (usa r8 no delay slot, deve ver 0)
    bus.write32::<BusRead>(0x0008, (0x09 << 26) | (8 << 21) | (9 << 16));
    // NOP
    bus.write32::<BusRead>(0x000C, nop());

    cpu.step(&mut bus); // ctc2
    cpu.step(&mut bus); // cfc2 (pendente para r8)
    cpu.step(&mut bus); // addiu (r8 = valor antigo = 0; r9 = 0)
    cpu.step(&mut bus); // nop (r8 = 0x99)

    assert_eq!(cpu.regs[9], 0, "t7: delay slot ve r8 antigo (0)");
    assert_eq!(cpu.regs[8], 0x99, "t7: apos delay, r8 = 0x99");
}

// ============================================================================
// t8 — MTC2 em registrador de 16 bits nao dispara flag nem satura
// Spec: 07-gte.md L379-381 — Writing 32bit values to 16bit GTE registers
//       by software does not trigger any overflow/saturation flags
// ============================================================================
#[test]
fn mtc2_em_registrador_16bits_nao_dispara_flag_nem_satura() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    enable_cop2(&mut cpu);
    cpu.pc = 0;

    // cop2r8 = IR0 (16-bit signed)
    // cop2r63 = FLAG (bit 31 = error flag)
    cpu.regs[8] = 0x8000_0001; // valor de 32 bits, excede 16-bit signed
    bus.write32::<BusRead>(0x0000, mtc2(8, 8));

    cpu.step(&mut bus);

    // FLAG nao deve ter sido alterado (permanece 0)
    assert_eq!(
        cpu.gte.read_ctrl(31),
        0,
        "t8: FLAG deve permanecer 0 — MTC2 nao dispara flag de saturacao"
    );
    // O valor deve ser o que escrevemos, sem saturacao
    assert_eq!(
        cpu.gte.read_data(8),
        0x8000_0001,
        "t8: valor deve ser armazenado raw, sem saturacao"
    );
}

// ============================================================================
// t9 — LWC2 carrega word da memoria para registrador de dados GTE
// ============================================================================
#[test]
fn lwc2_carrega_word_da_memoria_para_registrador_gte() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    enable_cop2(&mut cpu);
    cpu.pc = 0;

    let addr = 0x1000u32;
    bus.write32::<BusRead>(addr, 0xFEED_FACE);
    cpu.regs[4] = addr; // rs = base

    bus.write32::<BusRead>(0x0000, lwc2_enc(7, 4, 0)); // LWC2 r7, 0(r4)

    cpu.step(&mut bus);

    assert_eq!(
        cpu.gte.read_data(7),
        0xFEED_FACE,
        "t9: LWC2 carregou 0xFEEDFACE da memoria para cop2r7"
    );
}

// ============================================================================
// t9b — LWC2 com offset negativo (sign-extend do imediato)
// Spec: 02-cpu.md L207 — imediato de LWCn e sign-extendido de 16 bits
// ============================================================================
#[test]
fn lwc2_com_offset_negativo_sign_extende_imediato() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    enable_cop2(&mut cpu);
    cpu.pc = 0;

    let base = 0x1008u32;
    let target = 0x1000u32;
    bus.write32::<BusRead>(target, 0xCAFE_BABE);
    cpu.regs[4] = base;

    let imm_neg8 = (-8i16) as u16;
    bus.write32::<BusRead>(0x0000, lwc2_enc(7, 4, imm_neg8));

    cpu.step(&mut bus);

    assert_eq!(
        cpu.gte.read_data(7),
        0xCAFE_BABE,
        "t9b: LWC2 com offset -8 deve ler de 0x1000"
    );
}

// ============================================================================
// t10 — SWC2 armazena word de registrador GTE na memoria
// ============================================================================
#[test]
fn swc2_armazena_word_de_registrador_gte_na_memoria() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    enable_cop2(&mut cpu);
    cpu.pc = 0;

    // Escreve em cop2r3 via MTC2
    cpu.regs[8] = 0xBEEF_CAFE;
    bus.write32::<BusRead>(0x0000, mtc2(8, 3));

    cpu.step(&mut bus);

    // SWC2 r3, 0(r10)
    let addr = 0x2000u32;
    cpu.regs[10] = addr;
    bus.write32::<BusRead>(0x0004, swc2_enc(3, 10, 0));

    cpu.step(&mut bus);

    assert_eq!(
        bus.read32::<BusRead>(addr),
        0xBEEF_CAFE,
        "t10: SWC2 armazenou 0xBEEFCAFE de cop2r3 na memoria"
    );
}

// ============================================================================
// t11 — COP2 desabilitado (SR.bit30=0) dispara CpU (exccode=0x0B)
// Spec: 02-cpu.md (instrucoes COP2 sem bit de enable levam excecao CpU)
// ============================================================================
#[test]
fn cop2_desabilitado_dispara_cpu() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    disable_cop2(&mut cpu);
    cpu.pc = 0;

    bus.write32::<BusRead>(0x0000, mfc2(8, 0));

    cpu.step(&mut bus);

    assert_eq!(
        cpu.cop0[13] & 0x7C,
        0x0B << 2,
        "t11: CAUSE.ExcCode = 0Bh (Coprocessor Unusable)"
    );
    assert_eq!(cpu.cop0[14], 0x0, "t11: EPC = endereco da instrucao COP2");
    assert_eq!(cpu.pc, 0x8000_0080, "t11: PC = vetor geral de excecao");
}

// ============================================================================
// t12 — COP2 habilitado (SR.bit30=1) permite acesso normal
// ============================================================================
#[test]
fn cop2_habilitado_permite_acesso_normal() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    enable_cop2(&mut cpu);
    cpu.pc = 0;

    cpu.regs[8] = 0xABCD;
    bus.write32::<BusRead>(0x0000, mtc2(8, 1));
    bus.write32::<BusRead>(0x0004, nop());
    bus.write32::<BusRead>(0x0008, mfc2(9, 1));
    bus.write32::<BusRead>(0x000C, nop());

    cpu.step(&mut bus); // mtc2
    cpu.step(&mut bus); // nop
    cpu.step(&mut bus); // mfc2
    cpu.step(&mut bus); // nop

    assert_eq!(
        cpu.regs[9], 0xABCD,
        "t12: com COP2 habilitado, MFC2 funciona normalmente"
    );
}

// ============================================================================
// t13 — COP2 command (co=0x10..=0x1F) e no-op nao dispara excecao
// ============================================================================
#[test]
fn cop2_command_eh_noop_nao_dispara_excecao() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    enable_cop2(&mut cpu);
    cpu.pc = 0;

    // COP2 imm25 com gte_command=01h (RTPS) — deve ser no-op em 5.1
    let cop2_cmd = (0x12 << 26) | (1 << 25) | 0x01;
    bus.write32::<BusRead>(0x0000, cop2_cmd);

    cpu.step(&mut bus);

    // Nao deve ter excecao pendente; PC avanca normalmente
    assert_eq!(cpu.pc, 4, "t13: COP2 command avanca PC sem excecao");
    assert!(
        cpu.cop0[13] & 0x7C == 0,
        "t13: CAUSE sem ExcCode apos COP2 command"
    );
}

// ============================================================================
// t14 — MTC2/CTC2 nao tem load delay (escrita e imediata)
// Spec: 07-gte.md L101 — load delay so para leitura (MFC2/CFC2)
// ============================================================================
#[test]
fn mtc2_ctc2_nao_tem_load_delay() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    enable_cop2(&mut cpu);
    cpu.pc = 0;

    cpu.regs[8] = 0x1111;
    cpu.regs[9] = 0x2222;
    // MTC2 r8, r5 (escreve 0x1111 em cop2r5)
    bus.write32::<BusRead>(0x0000, mtc2(8, 5));
    // MFC2 r10, r5 (le cop2r5 → r10, DELAY de 1)
    bus.write32::<BusRead>(0x0004, mfc2(10, 5));
    // NOP (resolve delay)
    bus.write32::<BusRead>(0x0008, nop());

    cpu.step(&mut bus); // mtc2 → cop2r5 = 0x1111
    cpu.step(&mut bus); // mfc2 → pendente r10 = 0x1111 (ja ve o valor!)
    cpu.step(&mut bus); // nop → resolve r10

    assert_eq!(
        cpu.regs[10], 0x1111,
        "t14: MTC2 seguido imediatamente de MFC2 le o valor novo (sem store delay)"
    );
}
