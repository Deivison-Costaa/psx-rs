use psx_core::bus::BusRead;
use psx_core::cpu::Cpu;

mod support;
use support::asm::{bus_with_bios_empty, encode_i_type, encode_j_type};

fn add(rd: u32, rt: u32, rs: u32) -> u32 {
    (rs << 21) | (rt << 16) | (rd << 11) | 0x20
}

fn syscall() -> u32 {
    0x0000_000C
}

fn break_op() -> u32 {
    0x0000_000D
}

fn mtc0(rt: u32, cop0_reg: u32) -> u32 {
    (0x10 << 26) | (0x04 << 21) | (rt << 16) | (cop0_reg << 11)
}

fn rfe() -> u32 {
    (0x10 << 26) | (1 << 25) | 0x10
}

// ============================================================================
// B1 — Overflow em ADD: ExcCode=0Ch, rt inalterado, PC=vetor geral
// Spec: ADD rd,rs,rt com overflow deve disparar excecao Ovf e deixar rd
//       inalterado.
// ============================================================================
#[test]
fn overflow_em_add_seta_cause_ovf_e_nao_escreve_rt() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    cpu.pc = 0;

    cpu.regs[8] = 0x7FFF_FFFFu32;
    cpu.regs[9] = 0x7FFF_FFFFu32;
    cpu.regs[10] = 0x1234_5678;

    bus.write32::<BusRead>(0x0000, add(10, 9, 8));

    cpu.step(&mut bus);

    assert_eq!(
        cpu.regs[10], 0x1234_5678,
        "B1: rt (r10) deve ficar inalterado no overflow"
    );
    assert_eq!(
        cpu.cop0[13], 0x0000_0030,
        "B1: CAUSE = ExcCode(Ovf=0Ch) << 2 = 0x30"
    );
    assert_eq!(
        cpu.cop0[14], 0x0000_0000,
        "B1: EPC = endereco do ADD (0x00000000)"
    );
    assert_eq!(
        cpu.pc, 0x8000_0080,
        "B1: PC = vetor geral de excecao (0x80000080)"
    );
}

// ============================================================================
// B2 — syscall: ExcCode=08h, PC=vetor geral
// Spec: syscall gera Sys exception (ExcCode=08h).
// ============================================================================
#[test]
fn syscall_seta_cause_sys_e_desvia_para_vetor() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    cpu.pc = 0;

    bus.write32::<BusRead>(0x0000, syscall());

    cpu.step(&mut bus);

    assert_eq!(
        cpu.cop0[13], 0x0000_0020,
        "B2: CAUSE = ExcCode(Sys=08h) << 2 = 0x20"
    );
    assert_eq!(
        cpu.cop0[14], 0x0000_0000,
        "B2: EPC = endereco do syscall (0x00000000)"
    );
    assert_eq!(
        cpu.pc, 0x8000_0080,
        "B2: PC = vetor geral de excecao (0x80000080)"
    );
}

// ============================================================================
// B3 — break vai para OUTRO vetor (0x80000040), nao o geral
// Spec: Exception Vectors — "COP0 Break" tem vetor proprio em 80000040h.
// ============================================================================
#[test]
fn break_desvia_para_vetor_cop0_break_nao_para_vetor_geral() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    cpu.pc = 0;

    bus.write32::<BusRead>(0x0000, break_op());

    cpu.step(&mut bus);

    assert_eq!(
        cpu.cop0[13], 0x0000_0024,
        "B3: CAUSE = ExcCode(Bp=09h) << 2 = 0x24"
    );
    assert_eq!(
        cpu.pc, 0x8000_0040,
        "B3: PC = vetor COP0 Break (0x80000040), NAO o geral (0x80000080)"
    );
}

// ============================================================================
// B4 — BD no delay slot: JAL + syscall no delay slot
// Spec: BD=1, EPC aponta para o branch (JAL), nao para o delay slot.
//       CAUSE = BD(bit31) | BT(bit30) | ExcCode(08h no bits 2-6) = 0xC0000020.
// ============================================================================
#[test]
fn excecao_em_delay_slot_seta_bd_e_epc_aponta_para_o_branch() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    cpu.pc = 0;

    // JAL 0x100 → delay slot contem syscall
    let target = 0x100 >> 2;
    bus.write32::<BusRead>(0x0000, encode_j_type(0x03, target));
    bus.write32::<BusRead>(0x0004, syscall());

    cpu.step(&mut bus); // JAL: seta link em r31, branch_target=0x100, PC avanca para 0x0004
    cpu.step(&mut bus); // delay slot: syscall em pc=0x0004 (delay do JAL)

    assert_eq!(
        cpu.cop0[13], 0xC000_0020u32,
        "B4: CAUSE = BD(bit31) | BT(bit30) | ExcCode(08h nos bits 2-6) = 0xC0000020"
    );
    assert_eq!(
        cpu.cop0[14], 0x0000_0000,
        "B4: EPC = endereco do JAL (0x00000000), NAO do delay slot (0x00000004)"
    );
    assert_eq!(
        cpu.pc, 0x8000_0080,
        "B4: PC = vetor geral (syscall vai para o vetor geral)"
    );
}

// ============================================================================
// B5a — Load desalinhado dispara AdEL (ExcCode=04h)
// Spec: LW em endereco nao multiplo de 4 → AdEL, rt inalterado, BadVaddr escrito.
// ============================================================================
#[test]
fn load_word_desalinhado_dispara_adel() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    cpu.pc = 0;

    cpu.regs[4] = 0x0000_0001; // base = endereco desalinhado
    cpu.regs[5] = 0xDEAD_BEEF; // rt antes do load

    // LW r5, 0(r4) → addr = 0x00000001 (desalinhado para word)
    let lw_instr = 0x8C850000u32; // LW r5, 0(r4)
    bus.write32::<BusRead>(0x0000, lw_instr);

    cpu.step(&mut bus);

    assert_eq!(
        cpu.regs[5], 0xDEAD_BEEF,
        "B5a: rt (r5) deve ficar inalterado"
    );
    assert_eq!(
        cpu.cop0[13], 0x0000_0010,
        "B5a: CAUSE = ExcCode(AdEL=04h) << 2 = 0x10"
    );
    assert_eq!(
        cpu.cop0[8], 0x0000_0001,
        "B5a: BadVaddr = endereco desalinhado (0x00000001)"
    );
    assert_eq!(cpu.pc, 0x8000_0080, "B5a: PC = vetor geral de excecao");
}

// ============================================================================
// B5b — Store desalinhado dispara AdES (ExcCode=05h)
// Spec: SW em endereco nao multiplo de 4 → AdES, BadVaddr escrito.
// ============================================================================
#[test]
fn store_word_desalinhado_dispara_ades() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    cpu.pc = 0;

    cpu.regs[4] = 0x0000_0003; // base = endereco desalinhado
    cpu.regs[5] = 0xCAFE_F00D; // valor a ser escrito

    // SW r5, 0(r4) → addr = 0x00000003 (desalinhado para word)
    let sw_instr = 0xAC850000u32; // SW r5, 0(r4)
    bus.write32::<BusRead>(0x0000, sw_instr);

    cpu.step(&mut bus);

    assert_eq!(
        cpu.cop0[13], 0x0000_0014,
        "B5b: CAUSE = ExcCode(AdES=05h) << 2 = 0x14"
    );
    assert_eq!(
        cpu.cop0[8], 0x0000_0003,
        "B5b: BadVaddr = endereco desalinhado (0x00000003)"
    );
    assert_eq!(cpu.pc, 0x8000_0080, "B5b: PC = vetor geral de excecao");
}

// ============================================================================
// Verificacao: BadVaddr NAO e alterado por excecoes que nao sejam AdEL/AdES
// Spec: "BadVaddr is updated ONLY by Address errors"
// ============================================================================
#[test]
fn badvaddr_inalterado_por_excecoes_que_nao_sejam_addr_error() {
    {
        // syscall nao mexe em BadVaddr
        let mut bus = bus_with_bios_empty();
        let mut cpu = Cpu::new();
        cpu.pc = 0;
        cpu.cop0[8] = 0xCAFE_0000;

        bus.write32::<BusRead>(0x0000, syscall());

        cpu.step(&mut bus);

        assert_eq!(cpu.cop0[8], 0xCAFE_0000, "BadVaddr inalterado por syscall");
    }
    {
        // break nao mexe em BadVaddr
        let mut bus = bus_with_bios_empty();
        let mut cpu = Cpu::new();
        cpu.pc = 0;
        cpu.cop0[8] = 0xCAFE_0000;

        bus.write32::<BusRead>(0x0000, break_op());

        cpu.step(&mut bus);

        assert_eq!(cpu.cop0[8], 0xCAFE_0000, "BadVaddr inalterado por break");
    }
    {
        // overflow nao mexe em BadVaddr
        let mut bus = bus_with_bios_empty();
        let mut cpu = Cpu::new();
        cpu.pc = 0;
        cpu.cop0[8] = 0xCAFE_0000;
        cpu.regs[8] = 0x7FFF_FFFFu32;
        cpu.regs[9] = 0x7FFF_FFFFu32;

        bus.write32::<BusRead>(0x0000, add(10, 9, 8));

        cpu.step(&mut bus);

        assert_eq!(cpu.cop0[8], 0xCAFE_0000, "BadVaddr inalterado por overflow");
    }
}

// ============================================================================
// ADDI overflow (dívida nota 2 do STATUS)
// Spec: ADDI rt,rs,imm com overflow → Ovf, rt inalterado.
// ============================================================================
#[test]
fn addi_overflow_seta_cause_ovf_e_nao_escreve_rt() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    cpu.pc = 0;

    cpu.regs[8] = 0x7FFF_FFFFu32;
    cpu.regs[9] = 0x5555_5555;

    // ADDI r9, r8, 0x0001 → 0x7FFFFFFF + 1 = overflow
    bus.write32::<BusRead>(0x0000, encode_i_type(0x08, 9, 8, 0x0001));

    cpu.step(&mut bus);

    assert_eq!(
        cpu.regs[9], 0x5555_5555,
        "rt (r9) deve ficar inalterado no overflow do ADDI"
    );
    assert_eq!(
        cpu.cop0[13], 0x0000_0030,
        "CAUSE = ExcCode(Ovf=0Ch) << 2 = 0x30"
    );
    assert_eq!(
        cpu.cop0[14], 0x0000_0000,
        "EPC = endereco do ADDI (0x00000000)"
    );
    assert_eq!(cpu.pc, 0x8000_0080, "PC = vetor geral de excecao");
}

// ============================================================================
// LH desalinhado (halfword boundary) dispara AdEL
// ============================================================================
#[test]
fn load_halfword_desalinhado_dispara_adel() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    cpu.pc = 0;

    cpu.regs[4] = 0x0000_0001; // nao multiplo de 2

    // LH r5, 0(r4) = primary 0x21
    bus.write32::<BusRead>(0x0000, encode_i_type(0x21, 5, 4, 0x0000));

    cpu.step(&mut bus);

    assert_eq!(
        cpu.cop0[13], 0x0000_0010,
        "LH desalinhado: AdEL (ExcCode=04h)"
    );
    assert_eq!(
        cpu.cop0[8], 0x0000_0001,
        "LH desalinhado: BadVaddr = endereco desalinhado"
    );
    assert_eq!(cpu.pc, 0x8000_0080, "LH desalinhado: PC = vetor geral");
}

// ============================================================================
// SH desalinhado (halfword boundary) dispara AdES
// ============================================================================
#[test]
fn store_halfword_desalinhado_dispara_ades() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    cpu.pc = 0;

    cpu.regs[4] = 0x0000_0003; // nao multiplo de 2

    // SH r5, 0(r4) = primary 0x29
    bus.write32::<BusRead>(0x0000, encode_i_type(0x29, 5, 4, 0x0000));

    cpu.step(&mut bus);

    assert_eq!(
        cpu.cop0[13], 0x0000_0014,
        "SH desalinhado: AdES (ExcCode=05h)"
    );
    assert_eq!(
        cpu.cop0[8], 0x0000_0003,
        "SH desalinhado: BadVaddr = endereco desalinhado"
    );
    assert_eq!(cpu.pc, 0x8000_0080, "SH desalinhado: PC = vetor geral");
}

// ============================================================================
// E1 — Corrigir: excecao NAO apaga os bits Sw (8-9) do CAUSE.
// A spec diz que os bits Sw sao R/W e devem ser limpos por software
// (RFE handler), nao pelo hardware na entrada da excecao.
// O bug: self.cop0[13] = cause sobrescreve o registrador inteiro.
// ============================================================================
#[test]
fn excecao_preserva_bits_sw_do_cause() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    cpu.pc = 0;

    cpu.regs[8] = 0x0000_0300;
    bus.write32::<BusRead>(0x0000, mtc0(8, 13));
    bus.write32::<BusRead>(0x0004, syscall());

    cpu.step(&mut bus); // MTC0 → cop0[13] = 0x300
    cpu.step(&mut bus); // syscall → excecao

    assert_eq!(
        cpu.cop0[13], 0x0000_0320,
        "E1: CAUSE = ExcCode(Sys=08h) << 2 | Sw=0x300 preservado = 0x320"
    );
}

// ============================================================================
// E2 — Empilhamento de SR na entrada da excecao.
// Comportamento ASSUMIDO (nao documentado na spec local, apenas o RFE).
// O inverso exato do RFE: bits 0-1 → 2-3, bits 2-3 → 4-5, bits 0-1 = 0.
// Ponto de resolucao: Amidog psxtest_cpu (item 1.11).
// ============================================================================
#[test]
fn sr_e_empilhado_na_entrada_da_excecao() {
    {
        let mut bus = bus_with_bios_empty();
        let mut cpu = Cpu::new();
        cpu.pc = 0;

        cpu.regs[8] = 0x0000_0003;
        bus.write32::<BusRead>(0x0000, mtc0(8, 12));
        bus.write32::<BusRead>(0x0004, syscall());

        cpu.step(&mut bus); // MTC0 r8 → SR=0x03
        cpu.step(&mut bus); // syscall → excecao (push)

        assert_eq!(
            cpu.cop0[12], 0x0000_000C,
            "E2: SR=0x03 antes do syscall deve virar 0x0C (bits 0-1→2-3, bits 0-1 zerados). \
             Comportamento ASSUMIDO — verificar com Amidog psxtest_cpu no item 1.11 (nota 10 do STATUS)."
        );
    }
    {
        let mut bus = bus_with_bios_empty();
        let mut cpu = Cpu::new();
        cpu.pc = 0;

        cpu.regs[8] = 0x0040_0031;
        bus.write32::<BusRead>(0x0000, mtc0(8, 12));
        bus.write32::<BusRead>(0x0004, syscall());

        cpu.step(&mut bus); // MTC0 r8 → SR=0x0040_0031
        cpu.step(&mut bus); // syscall → excecao (push)

        assert_eq!(
            cpu.cop0[12], 0x0040_0004,
            "E2: SR=0x0040_0031 deve virar 0x0040_0004 (bits 4-5 sobrescritos por zero, \
             bit 22 intacto). Pega mutante que limpa so bits 0-1 em vez de 2-5."
        );
    }
}

// ============================================================================
// E2b — Round-trip: SR empilhado + RFE restaura os bits 0-3.
// Verifica que o empilhamento e o RFE sao inversos exatos.
// ============================================================================
#[test]
fn sr_push_seguido_de_rfe_restaura_os_bits_0_3() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    cpu.pc = 0;

    cpu.regs[8] = 0x0000_003F;
    bus.write32::<BusRead>(0x0000, mtc0(8, 12));
    bus.write32::<BusRead>(0x0004, syscall());
    bus.write32::<BusRead>(0x0080, rfe());

    cpu.step(&mut bus); // MTC0 → SR=0x3F
    cpu.step(&mut bus); // syscall → excecao, push, PC=0x8000_0080

    assert_eq!(
        cpu.cop0[12], 0x0000_003C,
        "E2b: push de 0x3F deve dar 0x3C (bits 0-1→2-3=0xC, bits 2-3→4-5=0x30)"
    );

    cpu.step(&mut bus); // RFE no vetor

    assert_eq!(
        cpu.cop0[12] & 0x3F,
        0x0000_003F,
        "E2b: round-trip SR=0x3F → push → RFE → 0x3F nos bits 0-5. \
         Confirma que empilhar e desempilhar sao simetricos."
    );
}

// ============================================================================
// E3 — Load delay nao e descartado pela excecao.
// Comportamento ASSUMIDO: o acesso a memoria do lw ja ocorreu quando
// a excecao da instrucao seguinte e reconhecida, entao o valor
// pendente deve ser commitado antes de entrar na excecao.
// Ponto de resolucao: Amidog psxtest_cpu (item 1.11).
// ============================================================================
#[test]
fn load_pendente_e_commitado_antes_da_excecao_comportamento_assumido() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    cpu.pc = 0;

    bus.write32::<BusRead>(0x1000, 0xCAFE_BABEu32);
    cpu.regs[4] = 0x1000;

    // LW r5, 0(r4) → carrega 0xCAFE_BABE da RAM (pendente)
    bus.write32::<BusRead>(0x0000, encode_i_type(0x23, 5, 4, 0x0000));
    // syscall (dispara excecao)
    bus.write32::<BusRead>(0x0004, syscall());

    cpu.step(&mut bus); // LW → load_delay = Some((5, 0xCAFE_BABE))
    cpu.step(&mut bus); // syscall → excecao

    assert_eq!(
        cpu.regs[5], 0xCAFE_BABEu32,
        "E3: r5 deve valer 0xCAFE_BABE apos a excecao. \
         Comportamento ASSUMIDO — o load delay e commitado antes do desvio para o handler. \
         Verificar com Amidog psxtest_cpu no item 1.11 (nota 11 do STATUS)."
    );
    assert_eq!(
        cpu.cop0[13], 0x0000_0020,
        "E3: CAUSE = ExcCode(Sys=08h) << 2 = 0x20 (syscall no delay slot?)"
    );
}
