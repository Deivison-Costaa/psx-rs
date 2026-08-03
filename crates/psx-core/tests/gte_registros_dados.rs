use psx_core::bus::BusRead;
use psx_core::cpu::Cpu;

mod support;
use support::asm::{bus_with_bios_empty, nop};

// oraculo/gte/test-all (JaCzekanski/ps1-tests): 997/999 linhas divergiam porque os
// registradores de DADOS (cop2r0-31) eram passthrough puro — sem os formatos por
// registrador que docs/reference/07-gte.md documenta. Uma unica lacuna desalinhava
// os 50 testes de registro do gabarito e, em cascata, todos os 1100 testes de opcode
// que vem depois (o proprio programa aborta os testes de opcode assim que o primeiro
// teste de registro falha).

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

fn escreve_e_executa(cpu: &mut Cpu, bus: &mut psx_core::bus::Bus, addr: u32, instr: u32) {
    bus.write32::<BusRead>(addr, instr);
    cpu.pc = addr;
    cpu.step(bus);
}

fn mtc2_r8(cpu: &mut Cpu, bus: &mut psx_core::bus::Bus, a: u32, rd: u32, val: u32) {
    cpu.regs[8] = val;
    escreve_e_executa(cpu, bus, a, mtc2(8, rd));
}

fn ctc2_r8(cpu: &mut Cpu, bus: &mut psx_core::bus::Bus, a: u32, rd: u32, val: u32) {
    cpu.regs[8] = val;
    escreve_e_executa(cpu, bus, a, ctc2(8, rd));
}

fn le_mfc2(cpu: &mut Cpu, bus: &mut psx_core::bus::Bus, a: u32, dst: usize, rd: u32) {
    escreve_e_executa(cpu, bus, a, mfc2(dst as u32, rd));
    escreve_e_executa(cpu, bus, a, nop());
}

fn le_cfc2(cpu: &mut Cpu, bus: &mut psx_core::bus::Bus, a: u32, dst: usize, rd: u32) {
    escreve_e_executa(cpu, bus, a, cfc2(dst as u32, rd));
    escreve_e_executa(cpu, bus, a, nop());
}

fn novo() -> (psx_core::bus::Bus, Cpu) {
    let bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    cpu.set_sr(1 << 30);
    (bus, cpu)
}

// § 16bit Vectors (R/W) (L270-272) de docs/reference/07-gte.md: VZn ocupa um
// registrador de 32 bits inteiro; a leitura sign-extende os 16 bits para 32.
#[test]
fn vz0_sign_extende_na_leitura() {
    let (mut bus, mut cpu) = novo();
    let a = 0x0000u32;

    mtc2_r8(&mut cpu, &mut bus, a, 1, 0x1234_8765);
    le_mfc2(&mut cpu, &mut bus, a, 10, 1);

    assert_eq!(
        cpu.regs[10], 0xFFFF_8765,
        "VZ0 (cop2r1): MFC2 deve sign-extender os 16 bits baixos (0x8765)"
    );
}

// mesma regra para IR0 (cop2r8, § Interpolation Factor L286-290: formato Sign|IR0).
#[test]
fn ir0_sign_extende_na_leitura() {
    let (mut bus, mut cpu) = novo();
    let a = 0x0000u32;

    mtc2_r8(&mut cpu, &mut bus, a, 8, 0x0000_8001);
    le_mfc2(&mut cpu, &mut bus, a, 10, 8);

    assert_eq!(
        cpu.regs[10], 0xFFFF_8001,
        "IR0 (cop2r8): MFC2 deve sign-extender os 16 bits baixos (0x8001)"
    );
}

// § 16bit Vectors (R/W) (L270-272): IR1-3 tambem sign-extendem, mesmo escritos
// diretamente por MTC2 (sem passar por IRGB).
#[test]
fn ir1_sign_extende_na_leitura_direta() {
    let (mut bus, mut cpu) = novo();
    let a = 0x0000u32;

    mtc2_r8(&mut cpu, &mut bus, a, 9, 0xABCD_F000);
    le_mfc2(&mut cpu, &mut bus, a, 10, 9);

    assert_eq!(
        cpu.regs[10], 0xFFFF_F000,
        "IR1 (cop2r9): MFC2 deve sign-extender os 16 bits baixos (0xF000)"
    );
}

// § GTE Data Register Summary (cop2r0-31) (L143): OTZ e 1xU16, sem sinal.
#[test]
fn otz_mascara_16_bits_sem_sinal_na_leitura() {
    let (mut bus, mut cpu) = novo();
    let a = 0x0000u32;

    mtc2_r8(&mut cpu, &mut bus, a, 7, 0x1234_8765);
    le_mfc2(&mut cpu, &mut bus, a, 10, 7);

    assert_eq!(
        cpu.regs[10], 0x0000_8765,
        "OTZ (cop2r7): MFC2 deve mascarar para 16 bits SEM sign-extend"
    );
}

// § Screen XYZ Coordinate FIFOs (L250-253): SZ0-3 sao 0,16,0 (U16).
#[test]
fn sz3_mascara_16_bits_sem_sinal_na_leitura() {
    let (mut bus, mut cpu) = novo();
    let a = 0x0000u32;

    mtc2_r8(&mut cpu, &mut bus, a, 19, 0x8000_FFFF);
    le_mfc2(&mut cpu, &mut bus, a, 10, 19);

    assert_eq!(
        cpu.regs[10], 0x0000_FFFF,
        "SZ3 (cop2r19): MFC2 deve mascarar para 16 bits SEM sign-extend"
    );
}

// § Screen XYZ Coordinate FIFOs (L249, L257-261): escrever SXYP empurra a FIFO
// (SXY0<-SXY1, SXY1<-SXY2, SXY2<-novo valor); SXY0-2 escritos direto nao empurram.
#[test]
fn sxyp_escrita_empurra_fifo_sxy() {
    let (mut bus, mut cpu) = novo();
    let a = 0x0000u32;

    mtc2_r8(&mut cpu, &mut bus, a, 12, 0x1111_1111);
    mtc2_r8(&mut cpu, &mut bus, a, 13, 0x2222_2222);
    mtc2_r8(&mut cpu, &mut bus, a, 14, 0x3333_3333);
    mtc2_r8(&mut cpu, &mut bus, a, 15, 0x4444_4444);

    le_mfc2(&mut cpu, &mut bus, a, 4, 12);
    le_mfc2(&mut cpu, &mut bus, a, 5, 13);
    le_mfc2(&mut cpu, &mut bus, a, 6, 14);
    le_mfc2(&mut cpu, &mut bus, a, 7, 15);

    assert_eq!(cpu.regs[4], 0x2222_2222, "SXY0 <- SXY1 antigo apos push");
    assert_eq!(cpu.regs[5], 0x3333_3333, "SXY1 <- SXY2 antigo apos push");
    assert_eq!(cpu.regs[6], 0x4444_4444, "SXY2 <- valor escrito em SXYP");
    assert_eq!(
        cpu.regs[7], 0x4444_4444,
        "SXYP e espelho de leitura de SXY2"
    );
}

// § cop2r28 - IRGB - Color conversion Input (R/W) (L304-311): decompoe 5:5:5 em
// IR1,IR2,IR3, cada canal multiplicado por 80h.
#[test]
fn irgb_escrita_decompoe_em_ir1_ir2_ir3() {
    let (mut bus, mut cpu) = novo();
    let a = 0x0000u32;

    // R=0x1E(30) G=0x0A(10) B=0x04(4) -> 0b00100_01010_11110 = 0x115E
    mtc2_r8(&mut cpu, &mut bus, a, 28, 0x0000_115E);

    le_mfc2(&mut cpu, &mut bus, a, 4, 9);
    le_mfc2(&mut cpu, &mut bus, a, 5, 10);
    le_mfc2(&mut cpu, &mut bus, a, 6, 11);

    assert_eq!(cpu.regs[4], 30 * 0x80, "IR1 = R * 80h");
    assert_eq!(cpu.regs[5], 10 * 0x80, "IR2 = G * 80h");
    assert_eq!(cpu.regs[6], 4 * 0x80, "IR3 = B * 80h");
}

// § cop2r29 - ORGB - Color conversion Output (R) (L317-329): espelho somente-leitura
// de IR1,IR2,IR3 (dividido por 80h), saturando negativos em 0 e >1Fh em 1Fh.
#[test]
fn orgb_recompoe_de_ir_com_saturacao() {
    let (mut bus, mut cpu) = novo();
    let a = 0x0000u32;

    mtc2_r8(&mut cpu, &mut bus, a, 9, (-100i32) as u32); // satura em 0
    mtc2_r8(&mut cpu, &mut bus, a, 10, 0x0000_0500); // 0x500/80h = 10
    mtc2_r8(&mut cpu, &mut bus, a, 11, 0x0000_7FFF); // satura em 1Fh

    le_mfc2(&mut cpu, &mut bus, a, 10, 29);

    let esperado = (10u32 << 5) | (0x1F << 10);
    assert_eq!(
        cpu.regs[10], esperado,
        "ORGB: R satura em 0, G=IR2/80h, B satura em 1Fh"
    );
}

// § cop2r30/31 - LZCS/LZCR (L331-334): LZCR conta zeros a esquerda se LZCS >= 0,
// ou uns a esquerda se LZCS < 0.
#[test]
fn lzcr_conta_zeros_a_esquerda_de_lzcs_positivo() {
    let (mut bus, mut cpu) = novo();
    let a = 0x0000u32;

    mtc2_r8(&mut cpu, &mut bus, a, 30, 0x0000_1acd);
    le_mfc2(&mut cpu, &mut bus, a, 10, 31);

    assert_eq!(cpu.regs[10], 19, "LZCR: 0x1acd tem 19 zeros a esquerda");
}

#[test]
fn lzcr_conta_uns_a_esquerda_de_lzcs_negativo() {
    let (mut bus, mut cpu) = novo();
    let a = 0x0000u32;

    mtc2_r8(&mut cpu, &mut bus, a, 30, 0x98bd_0000);
    le_mfc2(&mut cpu, &mut bus, a, 10, 31);

    assert_eq!(cpu.regs[10], 1, "LZCR: 0x98bd0000 tem 1 um a esquerda");
}

// § Screen Offset and Distance (Input, R/W?) (L227-231): BUG documentado — a
// leitura de H (cop2r58) sign-extende um valor que e, por definicao, sem sinal.
#[test]
fn h_sign_extende_na_leitura_bug_documentado() {
    let (mut bus, mut cpu) = novo();
    let a = 0x0000u32;

    ctc2_r8(&mut cpu, &mut bus, a, 26, 0x8057_e810);
    le_cfc2(&mut cpu, &mut bus, a, 10, 26);

    assert_eq!(
        cpu.regs[10], 0xFFFF_E810,
        "H (cop2r58): CFC2 sign-extende os 16 bits baixos, mesmo sendo U16"
    );
}
