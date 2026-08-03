use psx_core::bus::BusRead;
use psx_core::cpu::Cpu;

mod support;
use support::asm::{bus_with_bios_empty, nop};

// A partir da iteracao 0171 o GTE exige CU2 ligado no SR (§ cop0r12 - SR (L746) de
// docs/reference/02-cpu.md); o hardware lanca 0Bh sem ele. Codigo real liga antes de usar.

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

fn lwc2(rt: u32, rs: u32, imm: u16) -> u32 {
    (0x32 << 26) | (rs << 21) | (rt << 16) | (imm as u32)
}

fn swc2(rt: u32, rs: u32, imm: u16) -> u32 {
    (0x3A << 26) | (rs << 21) | (rt << 16) | (imm as u32)
}

#[test]
fn mtc2_escreve_registro_de_dados_e_mfc2_le_de_volta() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    cpu.set_sr(1 << 30);
    cpu.pc = 0;

    cpu.regs[8] = 0xDEAD_BEEF;
    bus.write32::<BusRead>(0x0000, mtc2(8, 5));
    bus.write32::<BusRead>(0x0004, mfc2(10, 5));
    bus.write32::<BusRead>(0x0008, nop());

    cpu.step(&mut bus);
    cpu.step(&mut bus);
    cpu.step(&mut bus);

    assert_eq!(
        cpu.regs[10], 0xDEAD_BEEF,
        "MFC2 deve ler o valor escrito por MTC2 no registrador r5"
    );
}

#[test]
fn ctc2_escreve_registro_de_controle_e_cfc2_le_de_volta() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    cpu.set_sr(1 << 30);
    cpu.pc = 0;

    cpu.regs[8] = 0x1234_5678;
    bus.write32::<BusRead>(0x0000, ctc2(8, 5));
    bus.write32::<BusRead>(0x0004, cfc2(10, 5));
    bus.write32::<BusRead>(0x0008, nop());

    cpu.step(&mut bus);
    cpu.step(&mut bus);
    cpu.step(&mut bus);

    assert_eq!(
        cpu.regs[10], 0x1234_5678,
        "CFC2 de registrador 32-bit (ctrl r5 = TRX) deve retornar valor integral"
    );
}

#[test]
fn mfc2_tem_load_delay_de_uma_instrucao() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    cpu.set_sr(1 << 30);
    cpu.pc = 0;

    bus.write32::<BusRead>(0x0000, mtc2(8, 3));
    bus.write32::<BusRead>(0x0004, mfc2(9, 3));
    bus.write32::<BusRead>(0x0008, nop());
    bus.write32::<BusRead>(0x000C, nop());

    cpu.regs[8] = 0xCAFE_0000;
    cpu.step(&mut bus);
    assert_eq!(
        cpu.regs[9], 0,
        "MFC2: load delay — valor ainda nao disponivel"
    );
    cpu.step(&mut bus);
    assert_eq!(
        cpu.regs[9], 0,
        "MFC2: delay slot executa, gpr ainda nao atualizado"
    );
    cpu.step(&mut bus);
    assert_eq!(
        cpu.regs[9], 0xCAFE_0000,
        "MFC2: apos o delay slot, gpr deve conter o valor lido"
    );
}

#[test]
fn cfc2_tem_load_delay_de_uma_instrucao() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    cpu.set_sr(1 << 30);
    cpu.pc = 0;

    bus.write32::<BusRead>(0x0000, ctc2(8, 0));
    bus.write32::<BusRead>(0x0004, cfc2(9, 0));
    bus.write32::<BusRead>(0x0008, nop());
    bus.write32::<BusRead>(0x000C, nop());

    cpu.regs[8] = 0x0000_0123;
    cpu.step(&mut bus);
    cpu.step(&mut bus);
    assert_eq!(
        cpu.regs[9], 0,
        "CFC2: load delay — valor ainda nao disponivel"
    );
    cpu.step(&mut bus);
    assert_eq!(
        cpu.regs[9], 0x0000_0123,
        "CFC2: apos o delay slot, gpr deve conter o valor lido"
    );
}

#[test]
fn mtc2_32bits_para_reg_16bits_trunca_e_sign_extende_na_leitura() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    cpu.set_sr(1 << 30);
    cpu.pc = 0;

    cpu.regs[8] = 0x1200_8900;
    bus.write32::<BusRead>(0x0000, ctc2(8, 29));
    bus.write32::<BusRead>(0x0004, cfc2(10, 29));
    bus.write32::<BusRead>(0x0008, nop());

    cpu.step(&mut bus);
    cpu.step(&mut bus);
    cpu.step(&mut bus);

    let val = cpu.regs[10] as i32;
    assert!(
        val < 0,
        "CTC2 de 0x12008900 em ZSF3 (16-bit) -> CFC2 sign-extende; valor={:#010x}",
        cpu.regs[10]
    );
}

#[test]
fn lwc2_carrega_palavra_da_memoria_para_registro_de_dados() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    cpu.set_sr(1 << 30);
    cpu.pc = 0;

    cpu.regs[1] = 0x0000_0100;
    bus.write32::<BusRead>(0x0108, 0xBABE_FACE);

    bus.write32::<BusRead>(0x0000, lwc2(2, 1, 8));
    bus.write32::<BusRead>(0x0004, nop());
    bus.write32::<BusRead>(0x0008, nop());
    bus.write32::<BusRead>(0x000C, mfc2(10, 2));
    bus.write32::<BusRead>(0x0010, nop());

    cpu.step(&mut bus);
    cpu.step(&mut bus);
    cpu.step(&mut bus);
    cpu.step(&mut bus);
    cpu.step(&mut bus);

    assert_eq!(
        cpu.regs[10], 0xBABE_FACE,
        "LWC2 deve carregar a palavra da memoria para o registrador de dados r2"
    );
}

#[test]
fn swc2_armazena_registro_de_dados_na_memoria() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    cpu.set_sr(1 << 30);
    cpu.pc = 0;

    cpu.regs[1] = 0x0000_0200;
    cpu.regs[8] = 0xFEED_C0DE;
    bus.write32::<BusRead>(0x0000, mtc2(8, 3));
    bus.write32::<BusRead>(0x0004, swc2(3, 1, 0x10));
    bus.write32::<BusRead>(0x0008, nop());

    cpu.step(&mut bus);
    cpu.step(&mut bus);
    cpu.step(&mut bus);

    let stored = bus.read32::<BusRead>(0x0210);
    assert_eq!(
        stored, 0xFEED_C0DE,
        "SWC2 deve armazenar o registrador de dados r3 na memoria"
    );
}

#[test]
fn gte_tem_64_registradores_indexaveis() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    cpu.set_sr(1 << 30);
    cpu.pc = 0;

    cpu.regs[8] = 0xAAAA_BBBB;
    bus.write32::<BusRead>(0x0000, mtc2(8, 31));
    bus.write32::<BusRead>(0x0004, mfc2(10, 31));
    bus.write32::<BusRead>(0x0008, nop());

    cpu.step(&mut bus);
    cpu.step(&mut bus);
    cpu.step(&mut bus);

    assert_eq!(
        cpu.regs[10], 0xAAAA_BBBB,
        "MFC2 r31 (ultimo registrador de dados) deve ser acessivel"
    );

    cpu.regs[8] = 0xCCCC_DDDD;
    bus.write32::<BusRead>(0x000C, ctc2(8, 31));
    bus.write32::<BusRead>(0x0010, cfc2(11, 31));
    bus.write32::<BusRead>(0x0014, nop());

    cpu.step(&mut bus);
    cpu.step(&mut bus);
    cpu.step(&mut bus);

    assert_ne!(
        cpu.regs[11], 0,
        "CFC2 r63 (ultimo registrador de controle) deve ser acessivel"
    );
}

#[test]
fn cfc2_reg16_sign_extende_valor_com_bit15_ligado() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    cpu.set_sr(1 << 30);
    cpu.pc = 0;

    cpu.regs[8] = 0x0000_8000;
    bus.write32::<BusRead>(0x0000, ctc2(8, 29));
    bus.write32::<BusRead>(0x0004, cfc2(10, 29));
    bus.write32::<BusRead>(0x0008, nop());

    cpu.step(&mut bus);
    cpu.step(&mut bus);
    cpu.step(&mut bus);

    assert_eq!(
        cpu.regs[10], 0xFFFF_8000,
        "CFC2 r29 (ZSF3, 16-bit): 0x8000 deve sign-extender para 0xFFFF8000"
    );
}

#[test]
fn mtc2_nao_dispara_saturacao_em_reg_16bit() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    cpu.set_sr(1 << 30);
    cpu.pc = 0;

    cpu.regs[8] = 0x1200_8900;
    bus.write32::<BusRead>(0x0000, ctc2(8, 3));
    bus.write32::<BusRead>(0x0004, nop());

    cpu.step(&mut bus);
    cpu.step(&mut bus);

    let gte = bus.gte();
    let flag = gte.regs[63];
    assert_eq!(
        flag & 0x7FFF_F000,
        0,
        "MTC2 nao deve disparar flag de saturacao (FLAG limpo apos escrita)"
    );
}

#[test]
fn flag_bit11_read_only_escrita_com_bit11_retorna_0() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    cpu.set_sr(1 << 30);

    cpu.regs[8] = 0x0000_0800;
    bus.write32::<BusRead>(0x0000, ctc2(8, 31));
    bus.write32::<BusRead>(0x0004, nop());

    cpu.step(&mut bus);
    cpu.step(&mut bus);

    bus.write32::<BusRead>(0x0008, cfc2(9, 31));
    bus.write32::<BusRead>(0x000C, nop());

    cpu.step(&mut bus);
    cpu.step(&mut bus);

    assert_eq!(
        cpu.regs[9] & 0x800,
        0,
        "FLAG bit 11 deve ser sempre zero (bits 0-11 sao read-only)"
    );
}
