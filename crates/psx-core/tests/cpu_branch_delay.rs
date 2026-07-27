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

fn encode_i_type(primary: u32, rt: u32, rs: u32, imm: u16) -> u32 {
    (primary << 26) | (rs << 21) | (rt << 16) | (imm as u32)
}

fn encode_j_type(opcode: u32, target: u32) -> u32 {
    (opcode << 26) | (target & 0x03FF_FFFF)
}

fn nop() -> u32 {
    encode_special(0x00, 0, 0, 0)
}

fn ori(rt: u32, rs: u32, imm: u16) -> u32 {
    encode_i_type(0x0D, rt, rs, imm)
}

fn addiu(rt: u32, rs: u32, imm: u16) -> u32 {
    encode_i_type(0x09, rt, rs, imm)
}

// ===== J: jump absolute =====

#[test]
fn j_salta_para_endereco() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    cpu.pc = 0;
    // J 0x1000 — target = (PC & 0xF000_0000) | (0x1000 * 4)
    // jump target = 0x0000_4000
    bus.write32::<BusRead>(0, encode_j_type(0x02, 0x1000));
    bus.write32::<BusRead>(4, nop()); // delay slot
    cpu.step(&mut bus); // J (prepara branch)
    // delay slot executou (NOP) no mesmo step — agora PC deve estar em 0x4000
    assert_eq!(cpu.pc, 0x4000, "J deve pular para 0x4000");
}

#[test]
fn j_preserva_4_bits_altos_do_pc() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    cpu.pc = 0x8000_0000;
    // J 0x00001 (target = 0x00001*4 = 0x04)
    // resultado: 0x8000_0000 & 0xF000_0000 | 0x04 = 0x8000_0004
    bus.write32::<BusRead>(0x8000_0000, encode_j_type(0x02, 0x00001));
    bus.write32::<BusRead>(0x8000_0004, nop());
    cpu.step(&mut bus);
    assert_eq!(cpu.pc, 0x8000_0004, "J deve preservar os 4 bits altos do PC");
}

// ===== JAL: jump and link =====

#[test]
fn jal_salta_e_guarda_ra() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    cpu.pc = 0;
    // JAL 0x1000 — deve guardar PC+4 (= 0x4) em $ra (r31)
    bus.write32::<BusRead>(0, encode_j_type(0x03, 0x1000));
    bus.write32::<BusRead>(4, nop());
    cpu.step(&mut bus);
    assert_eq!(cpu.regs[31], 0x4, "JAL: $ra deve ser PC+4 = 0x4");
    assert_eq!(cpu.pc, 0x4000, "JAL: PC deve pular para 0x4000");
}

// ===== JR: jump register =====

#[test]
fn jr_salta_para_registrador() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    cpu.pc = 0;
    cpu.regs[5] = 0x1234;
    // JR r5
    bus.write32::<BusRead>(0, encode_special(0x08, 0, 0, 5));
    bus.write32::<BusRead>(4, nop());
    cpu.step(&mut bus);
    assert_eq!(cpu.pc, 0x1234, "JR: PC deve ser r5 = 0x1234");
}

// ===== JALR: jump and link register =====

#[test]
fn jalr_salta_e_guarda_ra() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    cpu.pc = 0;
    cpu.regs[5] = 0x2000;
    // JALR r31, r5 — PC = r5 = 0x2000, r31 = PC+8 = 0x8
    // (o codif: rd=31, rs=5; MIPS32 sintaxe: jalr rd, rs)
    bus.write32::<BusRead>(0, encode_special(0x09, 31, 0, 5));
    bus.write32::<BusRead>(4, nop());
    cpu.step(&mut bus);
    assert_eq!(cpu.regs[31], 0x8, "JALR: $ra deve ser PC+8 = 0x8");
    assert_eq!(cpu.pc, 0x2000, "JALR: PC deve ser r5 = 0x2000");
}

#[test]
fn jalr_mesmo_reg_rs_rd() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    cpu.pc = 0;
    cpu.regs[5] = 0x3000;
    // JALR r5, r5 — rs=r5 (target = 0x3000), rd=r5 (link = PC+8 = 0x8)
    // rs original é lido ANTES de rd ser escrito
    bus.write32::<BusRead>(0, encode_special(0x09, 5, 0, 5));
    bus.write32::<BusRead>(4, nop());
    cpu.step(&mut bus);
    assert_eq!(cpu.regs[5], 0x8, "JALR mesmo reg: r5 deve conter PC+8 = 0x8");
    assert_eq!(cpu.pc, 0x3000, "JALR mesmo reg: PC deve ser target original 0x3000");
}

// ===== BEQ: branch if equal =====

#[test]
fn beq_tomado() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    cpu.pc = 0;
    cpu.regs[5] = 0x10;
    cpu.regs[6] = 0x10;
    // BEQ r5, r6, +8 → se igual, PC = 4 + 8*4 = 0x24
    bus.write32::<BusRead>(0, encode_i_type(0x04, 6, 5, 0x0008));
    bus.write32::<BusRead>(4, nop());
    cpu.step(&mut bus);
    assert_eq!(cpu.pc, 0x24, "BEQ tomado: PC deve ser 4 + 8*4 = 0x24");
}

#[test]
fn beq_nao_tomado() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    cpu.pc = 0;
    cpu.regs[5] = 0x10;
    cpu.regs[6] = 0x20;
    // BEQ r5, r6, +8 → não igual, PC continua = 4 (já incrementado)
    bus.write32::<BusRead>(0, encode_i_type(0x04, 6, 5, 0x0008));
    bus.write32::<BusRead>(4, nop());
    cpu.step(&mut bus);
    assert_eq!(cpu.pc, 0x8, "BEQ nao tomado: PC deve ser 8 (delay slot + fallthrough)");
}

// ===== BNE: branch if not equal =====

#[test]
fn bne_tomado() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    cpu.pc = 0;
    cpu.regs[5] = 0x10;
    cpu.regs[6] = 0x20;
    // BNE r5, r6, -4 → se !=, PC = 4 + (-4)*4 = 4 - 16 = -12 = 0xFFFF_FFF4
    bus.write32::<BusRead>(0, encode_i_type(0x05, 6, 5, 0xFFFC));
    bus.write32::<BusRead>(4, nop());
    cpu.step(&mut bus);
    assert_eq!(cpu.pc, 0xFFFF_FFF4u32, "BNE tomado: PC = 4 + (-4)*4");
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
    assert_eq!(cpu.pc, 0x8, "BNE nao tomado: PC deve ser 8");
}

// ===== BLEZ: branch if less than or equal to zero =====

#[test]
fn blez_tomado_negativo() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    cpu.pc = 0;
    cpu.regs[5] = 0xFFFF_FFFFu32; // -1 signed
    bus.write32::<BusRead>(0, encode_i_type(0x06, 0, 5, 0x0004));
    bus.write32::<BusRead>(4, nop());
    cpu.step(&mut bus);
    assert_eq!(cpu.pc, 0x14, "BLEZ tomado (negativo): PC = 4 + 4*4 = 0x14");
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
    assert_eq!(cpu.pc, 0x14, "BLEZ tomado (zero): PC = 4 + 4*4 = 0x14");
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
    assert_eq!(cpu.pc, 0x8, "BLEZ nao tomado (positivo): PC = 8");
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
    assert_eq!(cpu.pc, 0x14, "BGTZ tomado: PC = 4 + 4*4 = 0x14");
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
    assert_eq!(cpu.pc, 0x8, "BGTZ nao tomado (zero): PC = 8");
}

#[test]
fn bgtz_nao_tomado_negativo() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    cpu.pc = 0;
    cpu.regs[5] = 0xFFFF_FFFFu32; // -1
    bus.write32::<BusRead>(0, encode_i_type(0x07, 0, 5, 0x0004));
    bus.write32::<BusRead>(4, nop());
    cpu.step(&mut bus);
    assert_eq!(cpu.pc, 0x8, "BGTZ nao tomado (negativo): PC = 8");
}

// ===== BLTZ: branch if less than zero (BcondZ, rt=0) =====

#[test]
fn bltz_tomado() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    cpu.pc = 0;
    cpu.regs[5] = 0xFFFF_FFFFu32; // -1
    // BLTZ: primary=0x01, rs=r5, rt=0
    bus.write32::<BusRead>(0, encode_i_type(0x01, 0, 5, 0x0004));
    bus.write32::<BusRead>(4, nop());
    cpu.step(&mut bus);
    assert_eq!(cpu.pc, 0x14, "BLTZ tomado: PC = 4 + 4*4 = 0x14");
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
    assert_eq!(cpu.pc, 0x8, "BLTZ nao tomado (zero): PC = 8");
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
    assert_eq!(cpu.pc, 0x14, "BGEZ tomado (zero): PC = 4 + 4*4 = 0x14");
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
    assert_eq!(cpu.pc, 0x14, "BGEZ tomado (positivo): PC = 4 + 4*4 = 0x14");
}

#[test]
fn bgez_nao_tomado_negativo() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    cpu.pc = 0;
    cpu.regs[5] = 0xFFFF_FFFFu32; // -1
    bus.write32::<BusRead>(0, encode_i_type(0x01, 1, 5, 0x0004));
    bus.write32::<BusRead>(4, nop());
    cpu.step(&mut bus);
    assert_eq!(cpu.pc, 0x8, "BGEZ nao tomado (negativo): PC = 8");
}

// ===== BLTZAL: branch if less than zero and link (BcondZ, rt=16) =====

#[test]
fn bltzal_tomado() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    cpu.pc = 0;
    cpu.regs[5] = 0xFFFF_FFFFu32; // -1
    // BLTZAL: primary=0x01, rt=16
    bus.write32::<BusRead>(0, encode_i_type(0x01, 16, 5, 0x0004));
    bus.write32::<BusRead>(4, nop());
    cpu.step(&mut bus);
    assert_eq!(cpu.regs[31], 0x4, "BLTZAL tomado: $ra deve ser PC+4 = 4");
    assert_eq!(cpu.pc, 0x14, "BLTZAL tomado: PC = 4 + 4*4 = 0x14");
}

#[test]
fn bltzal_nao_tomado_mas_linka() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    cpu.pc = 0;
    cpu.regs[5] = 42; // positivo, não toma
    cpu.regs[31] = 0xDEAD; // valor anterior
    bus.write32::<BusRead>(0, encode_i_type(0x01, 16, 5, 0x0004));
    bus.write32::<BusRead>(4, nop());
    cpu.step(&mut bus);
    assert_eq!(
        cpu.regs[31], 0x4,
        "BLTZAL nao tomado: $ra SEMPRE recebe PC+4 = 4"
    );
    assert_eq!(cpu.pc, 0x8, "BLTZAL nao tomado: PC = 8 (fallthrough)");
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
    assert_eq!(cpu.regs[31], 0x4, "BGEZAL tomado: $ra = PC+4 = 4");
    assert_eq!(cpu.pc, 0x14, "BGEZAL tomado: PC = 4 + 4*4 = 0x14");
}

#[test]
fn bgezal_nao_tomado_mas_linka() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    cpu.pc = 0;
    cpu.regs[5] = 0xFFFF_FFFFu32; // -1, não toma
    cpu.regs[31] = 0xDEAD;
    bus.write32::<BusRead>(0, encode_i_type(0x01, 17, 5, 0x0004));
    bus.write32::<BusRead>(4, nop());
    cpu.step(&mut bus);
    assert_eq!(
        cpu.regs[31], 0x4,
        "BGEZAL nao tomado: $ra SEMPRE recebe PC+4 = 4"
    );
    assert_eq!(cpu.pc, 0x8, "BGEZAL nao tomado: PC = 8");
}

// ===== Branch delay slot: instrucao apos branch sempre executa =====

#[test]
fn delay_slot_executa_antes_do_desvio() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    cpu.pc = 0;
    cpu.regs[5] = 0x10;
    cpu.regs[6] = 0x10;
    // BEQ r5, r6, +8 (tomado → PC = 4+8*4 = 0x24)
    // delay slot: ADDIU r7, r0, 99 → r7 = 99
    // delay slot executa ANTES do desvio
    bus.write32::<BusRead>(0, encode_i_type(0x04, 6, 5, 0x0008));
    bus.write32::<BusRead>(4, addiu(7, 0, 99));
    cpu.step(&mut bus);
    assert_eq!(cpu.regs[7], 99, "Delay slot: instrucao no delay slot executa");
    assert_eq!(cpu.pc, 0x24, "Delay slot: PC vai para o target apos delay");
}

#[test]
fn delay_slot_nao_afeta_fallthrough() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    cpu.pc = 0;
    cpu.regs[5] = 0x10;
    cpu.regs[6] = 0x20;
    // BEQ r5, r6, +8 (nao tomado → PC = 8)
    // delay slot: ADDIU r7, r0, 99
    bus.write32::<BusRead>(0, encode_i_type(0x04, 6, 5, 0x0008));
    bus.write32::<BusRead>(4, addiu(7, 0, 99));
    cpu.step(&mut bus);
    assert_eq!(cpu.regs[7], 99, "Delay slot executa mesmo em fallthrough");
    assert_eq!(cpu.pc, 0x8, "Fallthrough: PC = 8");
}

// ===== BGEZAL com rs=$ra: compara o valor ANTES do link =====

#[test]
fn bgezal_com_rs_ra_compara_valor_antes_do_link() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    cpu.pc = 0;
    cpu.regs[31] = 0xFFFF_FFFFu32; // -1 — não toma BGEZ
    // BGEZAL r31, +4 — compara r31=-1 (<0, não toma), mas $ra sempre recebe PC+4
    bus.write32::<BusRead>(0, encode_i_type(0x01, 17, 31, 0x0004));
    bus.write32::<BusRead>(4, nop());
    cpu.step(&mut bus);
    assert_eq!(cpu.regs[31], 0x4, "BGEZAL rs=$ra: $ra = PC+4");
    assert_eq!(cpu.pc, 0x8, "BGEZAL rs=$ra: fallthrough");
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
    // BEQ r8, r9, +4 (tomado → PC = 4+4*4 = 0x14)
    // delay slot: LW r10, 0x1000(r0) → com load delay
    // No fim: PC=0x14, r10 = ??? (escrita com delay)
    bus.write32::<BusRead>(0, encode_i_type(0x04, 9, 8, 0x0004));
    bus.write32::<BusRead>(4, encode_i_type(0x23, 10, 0, 0x1000));
    cpu.step(&mut bus);
    assert_eq!(cpu.pc, 0x14, "BEQ + LW delay: PC no target");
    // O load foi executado, mas o valor comita depois
    // O delay slot executa, o load é enfileirado; neste step, o commit do
    // load_load antigo (nenhum) não acontece, mas o novo load é enfileirado
    // Na prática, o r10 ainda é 0 (não foi carregado)
    assert_eq!(cpu.regs[10], 0, "LW em delay slot: r10 ainda OLD");
}
