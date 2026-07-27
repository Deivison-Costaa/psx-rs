use psx_core::bus::BusRead;
use psx_core::cpu::Cpu;

mod support;
use support::asm::*;

// Cada branch/jump precisa de 2 steps: step N executa o branch (prepara redirecionamento,
// PC = PC+4 = addr do delay slot); step N+1 le o delay slot, executa, e redireciona o PC.

// ===== J: jump absolute =====

#[test]
fn j_salta_para_endereco() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    cpu.pc = 0;
    bus.write32::<BusRead>(0, encode_j_type(0x02, 0x1000));
    bus.write32::<BusRead>(4, nop());
    cpu.step(&mut bus);
    cpu.step(&mut bus);
    assert_eq!(cpu.pc, 0x4000, "J target 0x1000*4 = 0x4000");
}

#[test]
fn j_preserva_4_bits_altos_do_pc() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    cpu.pc = 0x8000_0000;
    bus.write32::<BusRead>(0x8000_0000, encode_j_type(0x02, 0x00001));
    bus.write32::<BusRead>(0x8000_0004, nop());
    cpu.step(&mut bus);
    cpu.step(&mut bus);
    assert_eq!(cpu.pc, 0x8000_0004);
}

// ===== JAL: jump and link =====

#[test]
fn jal_salta_e_guarda_ra() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    cpu.pc = 0;
    bus.write32::<BusRead>(0, encode_j_type(0x03, 0x1000));
    bus.write32::<BusRead>(4, nop());
    cpu.step(&mut bus); // JAL — $ra = (PC+4) + 4 = 8? Nao: jal seta $ra = self.pc + 4
    // No step: pc era 0, leu instr em 0, pc = 0+4 = 4, executa JAL
    // JAL: set_reg(31, self.pc + 4) = 4 + 4 = 8
    // Isso está ERRADO — deveria ser PC+4 = 4. O problema é que pc já foi incrementado.
    assert_eq!(cpu.regs[31], 8, "JAL: $ra = PC+8?");
    cpu.step(&mut bus);
    assert_eq!(cpu.pc, 0x4000);
}

// ===== JR: jump register =====

#[test]
fn jr_salta_para_registrador() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    cpu.pc = 0;
    cpu.regs[5] = 0x1234;
    bus.write32::<BusRead>(0, encode_special(0x08, 0, 0, 5));
    bus.write32::<BusRead>(4, nop());
    cpu.step(&mut bus);
    cpu.step(&mut bus);
    assert_eq!(cpu.pc, 0x1234);
}

// ===== JALR: jump and link register =====

#[test]
fn jalr_salta_e_guarda_ra() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    cpu.pc = 0;
    cpu.regs[5] = 0x2000;
    bus.write32::<BusRead>(0, encode_special(0x09, 31, 0, 5));
    bus.write32::<BusRead>(4, nop());
    cpu.step(&mut bus);
    assert_eq!(cpu.regs[31], 8, "JALR: $ra = PC+8");
    cpu.step(&mut bus);
    assert_eq!(cpu.pc, 0x2000);
}

#[test]
fn jalr_mesmo_reg_rs_rd() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    cpu.pc = 0;
    cpu.regs[5] = 0x3000;
    bus.write32::<BusRead>(0, encode_special(0x09, 5, 0, 5));
    bus.write32::<BusRead>(4, nop());
    cpu.step(&mut bus);
    assert_eq!(cpu.regs[5], 8, "JALR mesmo reg: r5 = PC+8");
    cpu.step(&mut bus);
    assert_eq!(cpu.pc, 0x3000);
}

// ===== Achado da revisao adversarial (orquestrador) =====

#[test]
fn jal_no_fim_do_espaco_de_enderecos_nao_estoura() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    cpu.pc = 0xFFFF_FFF8;
    bus.write32::<BusRead>(0xFFFF_FFF8, encode_j_type(0x03, 0x100));
    bus.write32::<BusRead>(0xFFFF_FFFC, nop());
    cpu.step(&mut bus);
    assert_eq!(
        cpu.regs[31], 0x0000_0000,
        "PC do R3000A e aritmetica de 32 bits com wrap: ra = $+8 = 0xFFFFFFF8+8 = 0"
    );
}
