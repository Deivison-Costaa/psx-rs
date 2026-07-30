use psx_core::bus::{Bios, Bus, BusRead, Ram};
use psx_core::cpu::Cpu;

fn bus_with_bios_empty() -> Bus {
    let ram = Ram::new();
    let bios_bytes = vec![0u8; 0x80000];
    let bios = Bios::from_bytes(bios_bytes).unwrap();
    Bus::new(ram, bios)
}

fn encode_special(secondary: u32, rd: u32, rt: u32, rs: u32) -> u32 {
    (rs << 21) | (rt << 16) | (rd << 11) | secondary
}

fn encode_load_store(primary: u32, rt: u32, rs: u32, imm: u16) -> u32 {
    (primary << 26) | (rs << 21) | (rt << 16) | (imm as u32)
}

fn nop() -> u32 {
    encode_special(0x00, 0, 0, 0)
}

fn setup_test_patterns(bus: &mut Bus) {
    bus.write32::<BusRead>(0x1000, 0xDEAD_BEEF);
    bus.write32::<BusRead>(0x1004, 0x12345678);
    bus.write32::<BusRead>(0x1008, 0x00000080);
    bus.write32::<BusRead>(0x100C, 0x00008000);
}

// LW: load word
#[test]
fn lw_carrega_palavra() {
    let mut bus = bus_with_bios_empty();
    setup_test_patterns(&mut bus);
    let mut cpu = Cpu::new();
    cpu.pc = 0;
    cpu.regs[8] = 0x1000;
    bus.write32::<BusRead>(0, encode_load_store(0x23, 10, 8, 0x0000));
    bus.write32::<BusRead>(4, nop());
    cpu.step(&mut bus);
    cpu.step(&mut bus);
    assert_eq!(cpu.regs[10], 0xDEAD_BEEF);
}

// LB: load byte sign-extended
#[test]
fn lb_carrega_byte_signed() {
    let mut bus = bus_with_bios_empty();
    setup_test_patterns(&mut bus);
    let mut cpu = Cpu::new();
    cpu.pc = 0;
    cpu.regs[8] = 0x1000;
    // LB r10, 0(r8) — carrega byte em 0x1000 = 0xEF, sign-extend → 0xFFFFFFEF
    bus.write32::<BusRead>(0, encode_load_store(0x20, 10, 8, 0x0000));
    bus.write32::<BusRead>(4, nop());
    cpu.step(&mut bus);
    cpu.step(&mut bus);
    assert_eq!(cpu.regs[10], 0xFFFF_FFEF, "LB: byte 0xEF sign-extended");
}

#[test]
fn lb_carrega_byte_signed_80() {
    let mut bus = bus_with_bios_empty();
    setup_test_patterns(&mut bus);
    let mut cpu = Cpu::new();
    cpu.pc = 0;
    cpu.regs[8] = 0x1008;
    // LB r10, 0(r8) — byte em 0x1008 = 0x80, sign-extend → 0xFFFFFF80
    bus.write32::<BusRead>(0, encode_load_store(0x20, 10, 8, 0x0000));
    bus.write32::<BusRead>(4, nop());
    cpu.step(&mut bus);
    cpu.step(&mut bus);
    assert_eq!(cpu.regs[10], 0xFFFF_FF80, "LB: byte 0x80 sign-extended");
}

// LBU: load byte unsigned
#[test]
fn lbu_carrega_byte_unsigned() {
    let mut bus = bus_with_bios_empty();
    setup_test_patterns(&mut bus);
    let mut cpu = Cpu::new();
    cpu.pc = 0;
    cpu.regs[8] = 0x1000;
    // LBU r10, 0(r8) — byte em 0x1000 = 0xEF, zero-extend → 0x000000EF
    bus.write32::<BusRead>(0, encode_load_store(0x24, 10, 8, 0x0000));
    bus.write32::<BusRead>(4, nop());
    cpu.step(&mut bus);
    cpu.step(&mut bus);
    assert_eq!(cpu.regs[10], 0x0000_00EF, "LBU: byte 0xEF zero-extended");
}

#[test]
fn lbu_nao_sign_extende() {
    let mut bus = bus_with_bios_empty();
    setup_test_patterns(&mut bus);
    let mut cpu = Cpu::new();
    cpu.pc = 0;
    cpu.regs[8] = 0x1008;
    // LBU r10, 0(r8) — byte 0x80, zero-extend → 0x00000080 (positivo)
    bus.write32::<BusRead>(0, encode_load_store(0x24, 10, 8, 0x0000));
    bus.write32::<BusRead>(4, nop());
    cpu.step(&mut bus);
    cpu.step(&mut bus);
    assert_eq!(cpu.regs[10], 0x0000_0080, "LBU: byte 0x80 zero-extended");
}

// LH: load halfword sign-extended
#[test]
fn lh_carrega_half_signed() {
    let mut bus = bus_with_bios_empty();
    setup_test_patterns(&mut bus);
    let mut cpu = Cpu::new();
    cpu.pc = 0;
    cpu.regs[8] = 0x1000;
    // LH r10, 0(r8) — halfword em 0x1000 (LE) = 0xADEF → sign-extend if bit15 set
    // Wait: 0xDEAD_BEEF em LE = EF BE AD DE
    // Halfword em 0x1000 = 0xBEEF, sign-extend → 0xFFFF_BEEF (bit15=1)
    bus.write32::<BusRead>(0, encode_load_store(0x21, 10, 8, 0x0000));
    bus.write32::<BusRead>(4, nop());
    cpu.step(&mut bus);
    cpu.step(&mut bus);
    assert_eq!(
        cpu.regs[10], 0xFFFF_BEEF,
        "LH: halfword 0xBEEF sign-extended"
    );
}

#[test]
fn lh_saca_halfword_alinhado() {
    let mut bus = bus_with_bios_empty();
    setup_test_patterns(&mut bus);
    let mut cpu = Cpu::new();
    cpu.pc = 0;
    cpu.regs[8] = 0x1004;
    // Halfword em 0x1004 = 0x5678, sign-extend → 0x0000_5678 (bit15=0)
    bus.write32::<BusRead>(0, encode_load_store(0x21, 10, 8, 0x0000));
    bus.write32::<BusRead>(4, nop());
    cpu.step(&mut bus);
    cpu.step(&mut bus);
    assert_eq!(cpu.regs[10], 0x0000_5678, "LH: halfword 0x5678 positivo");
}

// LHU: load halfword unsigned
#[test]
fn lhu_carrega_half_unsigned() {
    let mut bus = bus_with_bios_empty();
    setup_test_patterns(&mut bus);
    let mut cpu = Cpu::new();
    cpu.pc = 0;
    cpu.regs[8] = 0x100C;
    // LHU r10, 0(r8) — halfword em 0x100C (LE) = 0x8000, zero-extend → 0x00008000
    bus.write32::<BusRead>(0, encode_load_store(0x25, 10, 8, 0x0000));
    bus.write32::<BusRead>(4, nop());
    cpu.step(&mut bus);
    cpu.step(&mut bus);
    assert_eq!(
        cpu.regs[10], 0x0000_8000,
        "LHU: halfword 0x8000 zero-extended"
    );
}

// SB: store byte
#[test]
fn sb_armazena_byte() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    cpu.pc = 0;
    cpu.regs[8] = 0x2000;
    cpu.regs[10] = 0x1234_56FF;
    bus.write32::<BusRead>(0, encode_load_store(0x28, 10, 8, 0x0000));
    cpu.step(&mut bus);
    let written = bus.read32::<BusRead>(0x2000) & 0xFF;
    assert_eq!(written, 0xFF, "SB: armazena byte baixo");
}

#[test]
fn sb_nao_afeta_bytes_vizinhos() {
    let mut bus = bus_with_bios_empty();
    bus.write32::<BusRead>(0x2000, 0xDEAD_BEEF);
    let mut cpu = Cpu::new();
    cpu.pc = 0;
    cpu.regs[8] = 0x2000;
    cpu.regs[10] = 0x0000_00AA;
    bus.write32::<BusRead>(0, encode_load_store(0x28, 10, 8, 0x0000));
    cpu.step(&mut bus);
    let written = bus.read32::<BusRead>(0x2000);
    assert_eq!(written, 0xDEAD_BEAA, "SB: só byte baixo alterado");
}

// SH: store halfword
#[test]
fn sh_armazena_halfword() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    cpu.pc = 0;
    cpu.regs[8] = 0x2000;
    cpu.regs[10] = 0x1234_DCBE;
    bus.write32::<BusRead>(0, encode_load_store(0x29, 10, 8, 0x0000));
    cpu.step(&mut bus);
    let written = bus.read32::<BusRead>(0x2000) & 0xFFFF;
    assert_eq!(written, 0xDCBE, "SH: armazena halfword baixo");
}

#[test]
fn sh_nao_afeta_halfword_alto() {
    let mut bus = bus_with_bios_empty();
    bus.write32::<BusRead>(0x2000, 0xDEAD_BEEF);
    let mut cpu = Cpu::new();
    cpu.pc = 0;
    cpu.regs[8] = 0x2000;
    cpu.regs[10] = 0x0000_00AA;
    bus.write32::<BusRead>(0, encode_load_store(0x29, 10, 8, 0x0000));
    cpu.step(&mut bus);
    let written = bus.read32::<BusRead>(0x2000);
    assert_eq!(written, 0xDEAD_00AA, "SH: só halfword baixo alterado");
}

// Load delay: testa que o registrador alvo do load NÃO é atualizado
// na instrução imediatamente seguinte
#[test]
fn load_delay_basico() {
    let mut bus = bus_with_bios_empty();
    setup_test_patterns(&mut bus);
    let mut cpu = Cpu::new();
    cpu.pc = 0;
    cpu.regs[8] = 0x1000;
    cpu.regs[11] = 0x0000_0001;
    // LW r10, 0(r8)   → carrega 0xDEAD_BEEF, mas com load delay
    // ADDU r12, r11, r10 → r12 = r11 + r10 (r10 é o OLD = 0)
    bus.write32::<BusRead>(0, encode_load_store(0x23, 10, 8, 0x0000));
    bus.write32::<BusRead>(4, encode_special(0x21, 12, 10, 11));
    cpu.step(&mut bus); // LW
    cpu.step(&mut bus); // ADDU — deve ver r10 = 0 (OLD)
    assert_eq!(
        cpu.regs[12], 0x0000_0001,
        "Load delay: ADDU deve ver OLD r10=0, entao 1+0=1"
    );
    assert_eq!(
        cpu.regs[10], 0xDEAD_BEEF,
        "Load delay: r10 atualizado APOS o delay slot"
    );
}

#[test]
fn load_delay_back_to_back() {
    let mut bus = bus_with_bios_empty();
    setup_test_patterns(&mut bus);
    let mut cpu = Cpu::new();
    cpu.pc = 0;
    cpu.regs[8] = 0x1000;
    // LW r10, 0(r8)   → r10 deve receber 0xDEAD_BEEF (com delay)
    // LW r12, 4(r8)   → r12 deve receber 0x12345678 (com delay)
    // LW r14, 8(r8)   → r14 deve receber 0x00000080 (com delay)
    // NOP              → delay slot do ultimo LW, commits todos os anteriores
    bus.write32::<BusRead>(0, encode_load_store(0x23, 10, 8, 0x0000));
    bus.write32::<BusRead>(4, encode_load_store(0x23, 12, 8, 0x0004));
    bus.write32::<BusRead>(8, encode_load_store(0x23, 14, 8, 0x0008));
    bus.write32::<BusRead>(12, nop());
    cpu.step(&mut bus); // LW r10
    cpu.step(&mut bus); // LW r12 — r10 ainda OLD
    cpu.step(&mut bus); // LW r14 — r10, r12 ainda OLD
    cpu.step(&mut bus); // NOP   — todos commitam no fim
    assert_eq!(cpu.regs[10], 0xDEAD_BEEF, "r10 carregado");
    assert_eq!(cpu.regs[12], 0x12345678, "r12 carregado");
    assert_eq!(cpu.regs[14], 0x00000080, "r14 carregado");
}

#[test]
fn load_delay_r0_nunca_afetado() {
    let mut bus = bus_with_bios_empty();
    setup_test_patterns(&mut bus);
    let mut cpu = Cpu::new();
    cpu.pc = 0;
    cpu.regs[8] = 0x1000;
    // LW r0, 0(r8) — load para R0, nada deve mudar
    bus.write32::<BusRead>(0, encode_load_store(0x23, 0, 8, 0x0000));
    bus.write32::<BusRead>(4, nop());
    cpu.step(&mut bus);
    cpu.step(&mut bus);
    assert_eq!(cpu.regs[0], 0, "R0 deve permanecer 0 apos load");
}

#[test]
fn load_delay_instrucao_usa_rs_do_load() {
    let mut bus = bus_with_bios_empty();
    setup_test_patterns(&mut bus);
    let mut cpu = Cpu::new();
    cpu.pc = 0;
    cpu.regs[8] = 0x1000;
    // LW r8, 0(r8) — carrega em r8 (= rt) usando r8 como base (= rs)
    //   rs = r8 = 0x1000, valor carregado = 0xDEAD_BEEF
    //   r8 deve receber 0xDEAD_BEEF com delay
    // ADDU r10, r8, r9 — r8 ainda OLD (= 0x1000) durante esta instrução
    cpu.regs[9] = 0x0004;
    bus.write32::<BusRead>(0, encode_load_store(0x23, 8, 8, 0x0000));
    bus.write32::<BusRead>(4, encode_special(0x21, 10, 9, 8));
    cpu.step(&mut bus); // LW r8, 0(r8)
    cpu.step(&mut bus); // ADDU r10, r8, r9 — r8 ainda = 0x1000
    assert_eq!(
        cpu.regs[10], 0x0000_1004,
        "r8 ainda OLD durante delay: 0x1000 + 0x0004"
    );
    assert_eq!(cpu.regs[8], 0xDEAD_BEEF, "r8 atualizado apos delay");
}

#[test]
fn lbu_com_offset_positivo() {
    let mut bus = bus_with_bios_empty();
    setup_test_patterns(&mut bus);
    let mut cpu = Cpu::new();
    cpu.pc = 0;
    cpu.regs[8] = 0x1000;
    // LBU r10, 3(r8) — byte no endereco 0x1003 = 0xDE
    bus.write32::<BusRead>(0, encode_load_store(0x24, 10, 8, 0x0003));
    bus.write32::<BusRead>(4, nop());
    cpu.step(&mut bus);
    cpu.step(&mut bus);
    assert_eq!(cpu.regs[10], 0x0000_00DE, "LBU com offset 3: byte 0xDE");
}

#[test]
fn sb_offset_negativo() {
    let mut bus = bus_with_bios_empty();
    bus.write32::<BusRead>(0x2000, 0xFFFF_FFFF);
    let mut cpu = Cpu::new();
    cpu.pc = 0;
    cpu.regs[8] = 0x2008;
    cpu.regs[10] = 0x0000_0042;
    // SB r10, -8(r8) — escreve byte 0x42 em 0x2000
    bus.write32::<BusRead>(0, encode_load_store(0x28, 10, 8, 0xFFF8));
    cpu.step(&mut bus);
    let written = bus.read32::<BusRead>(0x2000);
    assert_eq!(written & 0xFF, 0x42, "SB com offset -8");
    assert_eq!(written >> 8, 0xFFFFFF, "SB: bytes altos intactos");
}

#[test]
fn load_delay_vs_escrita_no_mesmo_registrador_escrita_vence() {
    let mut bus = bus_with_bios_empty();
    setup_test_patterns(&mut bus);
    let mut cpu = Cpu::new();
    cpu.pc = 0;
    cpu.regs[8] = 0x1000;
    bus.write32::<BusRead>(0, encode_load_store(0x23, 10, 8, 0x0000));
    bus.write32::<BusRead>(4, encode_load_store(0x0D, 10, 0, 0x0005));
    cpu.step(&mut bus);
    cpu.step(&mut bus);
    assert_eq!(
        cpu.regs[10], 0x0000_0005,
        "a versao anterior deste teste ASSUMIA que o load vencia; a BIOS SCPH1001 \
         (beq nao tomado com lw $ra no delay slot, seguido de jal, em 0x8004723C-40) \
         prova que a escrita da instrucao seguinte vence — iter 0111, item 4.4h"
    );
}
