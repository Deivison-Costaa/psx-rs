use psx_core::bus::BusRead;
use psx_core::cpu::Cpu;

mod support;
use support::asm::*;

// Cada branch/jump precisa de 2 steps: step N executa o branch (prepara redirecionamento,
// PC = PC+4 = addr do delay slot); step N+1 le o delay slot, executa, e redireciona o PC.

// ===== BEQ: branch if equal =====

#[test]
fn beq_tomado() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    cpu.pc = 0;
    cpu.regs[5] = 0x10;
    cpu.regs[6] = 0x10;
    bus.write32::<BusRead>(0, encode_i_type(0x04, 6, 5, 0x0008));
    bus.write32::<BusRead>(4, nop());
    cpu.step(&mut bus);
    cpu.step(&mut bus);
    assert_eq!(cpu.pc, 0x24);
}

#[test]
fn beq_nao_tomado() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    cpu.pc = 0;
    cpu.regs[5] = 0x10;
    cpu.regs[6] = 0x20;
    bus.write32::<BusRead>(0, encode_i_type(0x04, 6, 5, 0x0008));
    bus.write32::<BusRead>(4, nop());
    cpu.step(&mut bus);
    cpu.step(&mut bus);
    assert_eq!(cpu.pc, 0x8);
}

// ===== BNE: branch if not equal =====

#[test]
fn bne_tomado() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    cpu.pc = 0;
    cpu.regs[5] = 0x10;
    cpu.regs[6] = 0x20;
    bus.write32::<BusRead>(0, encode_i_type(0x05, 6, 5, 0xFFFC));
    bus.write32::<BusRead>(4, nop());
    cpu.step(&mut bus);
    cpu.step(&mut bus);
    assert_eq!(cpu.pc, 0xFFFF_FFF4u32);
}

#[test]
fn bne_nao_tomado() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    cpu.pc = 0;
    cpu.regs[5] = 0x10;
    cpu.regs[6] = 0x10;
    bus.write32::<BusRead>(0, encode_i_type(0x05, 6, 5, 0x0008));
    bus.write32::<BusRead>(4, nop());
    cpu.step(&mut bus);
    cpu.step(&mut bus);
    assert_eq!(cpu.pc, 0x8);
}

// ===== BLEZ: branch if less than or equal to zero =====

#[test]
fn blez_tomado_negativo() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    cpu.pc = 0;
    cpu.regs[5] = 0xFFFF_FFFFu32;
    bus.write32::<BusRead>(0, encode_i_type(0x06, 0, 5, 0x0004));
    bus.write32::<BusRead>(4, nop());
    cpu.step(&mut bus);
    cpu.step(&mut bus);
    assert_eq!(cpu.pc, 0x14);
}

#[test]
fn blez_tomado_zero() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    cpu.pc = 0;
    cpu.regs[5] = 0;
    bus.write32::<BusRead>(0, encode_i_type(0x06, 0, 5, 0x0004));
    bus.write32::<BusRead>(4, nop());
    cpu.step(&mut bus);
    cpu.step(&mut bus);
    assert_eq!(cpu.pc, 0x14);
}

#[test]
fn blez_nao_tomado_positivo() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    cpu.pc = 0;
    cpu.regs[5] = 1;
    bus.write32::<BusRead>(0, encode_i_type(0x06, 0, 5, 0x0004));
    bus.write32::<BusRead>(4, nop());
    cpu.step(&mut bus);
    cpu.step(&mut bus);
    assert_eq!(cpu.pc, 0x8);
}

// ===== BGTZ: branch if greater than zero =====

#[test]
fn bgtz_tomado_positivo() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    cpu.pc = 0;
    cpu.regs[5] = 1;
    bus.write32::<BusRead>(0, encode_i_type(0x07, 0, 5, 0x0004));
    bus.write32::<BusRead>(4, nop());
    cpu.step(&mut bus);
    cpu.step(&mut bus);
    assert_eq!(cpu.pc, 0x14);
}

#[test]
fn bgtz_nao_tomado_zero() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    cpu.pc = 0;
    cpu.regs[5] = 0;
    bus.write32::<BusRead>(0, encode_i_type(0x07, 0, 5, 0x0004));
    bus.write32::<BusRead>(4, nop());
    cpu.step(&mut bus);
    cpu.step(&mut bus);
    assert_eq!(cpu.pc, 0x8);
}

#[test]
fn bgtz_nao_tomado_negativo() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    cpu.pc = 0;
    cpu.regs[5] = 0xFFFF_FFFFu32;
    bus.write32::<BusRead>(0, encode_i_type(0x07, 0, 5, 0x0004));
    bus.write32::<BusRead>(4, nop());
    cpu.step(&mut bus);
    cpu.step(&mut bus);
    assert_eq!(cpu.pc, 0x8);
}

// ===== BLTZ: branch if less than zero (BcondZ, rt=0) =====

#[test]
fn bltz_tomado() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    cpu.pc = 0;
    cpu.regs[5] = 0xFFFF_FFFFu32;
    bus.write32::<BusRead>(0, encode_i_type(0x01, 0, 5, 0x0004));
    bus.write32::<BusRead>(4, nop());
    cpu.step(&mut bus);
    cpu.step(&mut bus);
    assert_eq!(cpu.pc, 0x14);
}

#[test]
fn bltz_nao_tomado() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    cpu.pc = 0;
    cpu.regs[5] = 0;
    bus.write32::<BusRead>(0, encode_i_type(0x01, 0, 5, 0x0004));
    bus.write32::<BusRead>(4, nop());
    cpu.step(&mut bus);
    cpu.step(&mut bus);
    assert_eq!(cpu.pc, 0x8);
}

// ===== BGEZ: branch if greater than or equal to zero (BcondZ, rt=1) =====

#[test]
fn bgez_tomado_zero() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    cpu.pc = 0;
    cpu.regs[5] = 0;
    bus.write32::<BusRead>(0, encode_i_type(0x01, 1, 5, 0x0004));
    bus.write32::<BusRead>(4, nop());
    cpu.step(&mut bus);
    cpu.step(&mut bus);
    assert_eq!(cpu.pc, 0x14);
}

#[test]
fn bgez_tomado_positivo() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    cpu.pc = 0;
    cpu.regs[5] = 42;
    bus.write32::<BusRead>(0, encode_i_type(0x01, 1, 5, 0x0004));
    bus.write32::<BusRead>(4, nop());
    cpu.step(&mut bus);
    cpu.step(&mut bus);
    assert_eq!(cpu.pc, 0x14);
}

#[test]
fn bgez_nao_tomado_negativo() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    cpu.pc = 0;
    cpu.regs[5] = 0xFFFF_FFFFu32;
    bus.write32::<BusRead>(0, encode_i_type(0x01, 1, 5, 0x0004));
    bus.write32::<BusRead>(4, nop());
    cpu.step(&mut bus);
    cpu.step(&mut bus);
    assert_eq!(cpu.pc, 0x8);
}

// ===== BLTZAL: branch if less than zero and link (BcondZ, rt=16) =====

#[test]
fn bltzal_tomado() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    cpu.pc = 0;
    cpu.regs[5] = 0xFFFF_FFFFu32;
    bus.write32::<BusRead>(0, encode_i_type(0x01, 16, 5, 0x0004));
    bus.write32::<BusRead>(4, nop());
    cpu.step(&mut bus);
    assert_eq!(cpu.regs[31], 8, "BLTZAL: $ra = PC+8");
    cpu.step(&mut bus);
    assert_eq!(cpu.pc, 0x14);
}

#[test]
fn bltzal_nao_tomado_mas_linka() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    cpu.pc = 0;
    cpu.regs[5] = 42;
    cpu.regs[31] = 0xDEAD;
    bus.write32::<BusRead>(0, encode_i_type(0x01, 16, 5, 0x0004));
    bus.write32::<BusRead>(4, nop());
    cpu.step(&mut bus);
    assert_eq!(cpu.regs[31], 8, "BLTZAL nao tomado: $ra SEMPRE = PC+8");
    cpu.step(&mut bus);
    assert_eq!(cpu.pc, 0x8);
}

// ===== BGEZAL: branch if greater than or equal to zero and link (BcondZ, rt=17) =====

#[test]
fn bgezal_tomado() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    cpu.pc = 0;
    cpu.regs[5] = 42;
    bus.write32::<BusRead>(0, encode_i_type(0x01, 17, 5, 0x0004));
    bus.write32::<BusRead>(4, nop());
    cpu.step(&mut bus);
    assert_eq!(cpu.regs[31], 8, "BGEZAL: $ra = PC+8");
    cpu.step(&mut bus);
    assert_eq!(cpu.pc, 0x14);
}

#[test]
fn bgezal_nao_tomado_mas_linka() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    cpu.pc = 0;
    cpu.regs[5] = 0xFFFF_FFFFu32;
    cpu.regs[31] = 0xDEAD;
    bus.write32::<BusRead>(0, encode_i_type(0x01, 17, 5, 0x0004));
    bus.write32::<BusRead>(4, nop());
    cpu.step(&mut bus);
    assert_eq!(cpu.regs[31], 8, "BGEZAL nao tomado: $ra SEMPRE = PC+8");
    cpu.step(&mut bus);
    assert_eq!(cpu.pc, 0x8);
}

// ===== BGEZAL com rs=$ra: compara o valor ANTES do link =====

#[test]
fn bgezal_com_rs_ra_compara_valor_antes_do_link() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    cpu.pc = 0;
    cpu.regs[31] = 0xFFFF_FFFFu32;
    bus.write32::<BusRead>(0, encode_i_type(0x01, 17, 31, 0x0004));
    bus.write32::<BusRead>(4, nop());
    cpu.step(&mut bus);
    assert_eq!(cpu.regs[31], 8, "BGEZAL rs=$ra: $ra = PC+8");
    cpu.step(&mut bus);
    assert_eq!(cpu.pc, 0x8);
}

// ===== Branch delay slot: instrucao apos branch sempre executa =====

#[test]
fn delay_slot_executa_antes_do_desvio() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    cpu.pc = 0;
    cpu.regs[5] = 0x10;
    cpu.regs[6] = 0x10;
    bus.write32::<BusRead>(0, encode_i_type(0x04, 6, 5, 0x0008));
    bus.write32::<BusRead>(4, addiu(7, 0, 99));
    cpu.step(&mut bus); // BEQ — prepara branch, PC = 4
    cpu.step(&mut bus); // ADDIU r7,99 (delay slot) — r7=99, PC = 0x24
    assert_eq!(cpu.regs[7], 99, "Delay slot executa");
    assert_eq!(cpu.pc, 0x24, "PC no target apos delay slot");
}

#[test]
fn delay_slot_nao_afeta_fallthrough() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    cpu.pc = 0;
    cpu.regs[5] = 0x10;
    cpu.regs[6] = 0x20;
    bus.write32::<BusRead>(0, encode_i_type(0x04, 6, 5, 0x0008));
    bus.write32::<BusRead>(4, addiu(7, 0, 99));
    cpu.step(&mut bus);
    cpu.step(&mut bus);
    assert_eq!(cpu.regs[7], 99, "Delay slot executa em fallthrough");
    assert_eq!(cpu.pc, 0x8, "Fallthrough");
}

// ===== Load delay + branch: load no delay slot =====

#[test]
fn load_em_delay_slot() {
    let mut bus = bus_with_bios_empty();
    bus.write32::<BusRead>(0x1000, 0xDEAD_BEEF);
    let mut cpu = Cpu::new();
    cpu.pc = 0;
    cpu.regs[8] = 0x10;
    cpu.regs[9] = 0x10;
    bus.write32::<BusRead>(0, encode_i_type(0x04, 9, 8, 0x0004));
    bus.write32::<BusRead>(4, encode_i_type(0x23, 10, 0, 0x1000));
    cpu.step(&mut bus); // BEQ
    cpu.step(&mut bus); // LW delay slot — ainda OLD, PC = 0x14
    assert_eq!(cpu.pc, 0x14);
    assert_eq!(cpu.regs[10], 0, "LW em delay slot: r10 ainda OLD");
}

// ===== Achado da revisao adversarial (orquestrador) =====

#[test]
fn bcondz_rt_fora_da_tabela_comportamento_assumido() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    cpu.pc = 0;
    cpu.regs[5] = 0xFFFF_FFFF;
    bus.write32::<BusRead>(0, encode_i_type(0x01, 0x02, 5, 0x0004));
    bus.write32::<BusRead>(4, nop());
    cpu.step(&mut bus);
    cpu.step(&mut bus);
    assert_eq!(
        cpu.pc, 0x8,
        "SUPOSICAO NAO VERIFICADA (nota 4 do STATUS, resolve no item 1.11): a spec local so \
         tabela rt=00/01/10/11 no opcode 01h e nao diz o que rt=02h faz. Assumimos no-op; \
         se o psxtest_cpu reprovar, o criterio vira bit16 = condicao e link por bits 20..17"
    );
}
