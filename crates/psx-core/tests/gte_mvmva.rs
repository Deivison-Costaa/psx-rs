#![allow(clippy::erasing_op, clippy::identity_op)]

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

fn cop2_mvmva(sf: bool, mx: u32, v: u32, cv: u32, lm: bool) -> u32 {
    (0x12 << 26)
        | (1 << 25)
        | 0x12
        | ((sf as u32) << 19)
        | ((mx & 3) << 17)
        | ((v & 3) << 15)
        | ((cv & 3) << 13)
        | ((lm as u32) << 10)
}

fn escreve_e_executa(cpu: &mut Cpu, bus: &mut psx_core::bus::Bus, addr: u32, instr: u32) {
    bus.write32::<BusRead>(addr, instr);
    cpu.pc = addr;
    cpu.step(bus);
}

fn ctc2_r8(cpu: &mut Cpu, bus: &mut psx_core::bus::Bus, addr: u32, rd: u32, val: u32) {
    cpu.regs[8] = val;
    escreve_e_executa(cpu, bus, addr, ctc2(8, rd));
}

fn mtc2_r8(cpu: &mut Cpu, bus: &mut psx_core::bus::Bus, addr: u32, rd: u32, val: u32) {
    cpu.regs[8] = val;
    escreve_e_executa(cpu, bus, addr, mtc2(8, rd));
}

fn le_mfc2(cpu: &mut Cpu, bus: &mut psx_core::bus::Bus, a: u32, dst: usize, rd: u32) {
    escreve_e_executa(cpu, bus, a, mfc2(dst as u32, rd));
    escreve_e_executa(cpu, bus, a, nop());
}

fn le_cfc2(cpu: &mut Cpu, bus: &mut psx_core::bus::Bus, a: u32, dst: usize, rd: u32) {
    escreve_e_executa(cpu, bus, a, cfc2(dst as u32, rd));
    escreve_e_executa(cpu, bus, a, nop());
}

fn setup_rt_identity(cpu: &mut Cpu, bus: &mut psx_core::bus::Bus, a: u32) {
    ctc2_r8(cpu, bus, a, 0, 0x0000_1000);
    ctc2_r8(cpu, bus, a, 1, 0);
    ctc2_r8(cpu, bus, a, 2, 0x0000_1000);
    ctc2_r8(cpu, bus, a, 3, 0);
    ctc2_r8(cpu, bus, a, 4, 0x1000);
}

fn setup_light_matrix(cpu: &mut Cpu, bus: &mut psx_core::bus::Bus, a: u32) {
    ctc2_r8(cpu, bus, a, 8, 0x0000_0800);
    ctc2_r8(cpu, bus, a, 9, 0x0000_0400);
    ctc2_r8(cpu, bus, a, 10, 0x0000_0C00);
    ctc2_r8(cpu, bus, a, 11, 0x0000_0200);
    ctc2_r8(cpu, bus, a, 12, 0x1000);
}

// ── MVMVA: RT/V0/TR sf=1 ──

#[test]
fn mvmva_rt_v0_tr_sf1_lm0() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    let a = 0x0000u32;

    setup_rt_identity(&mut cpu, &mut bus, a);

    ctc2_r8(&mut cpu, &mut bus, a, 5, 10);
    ctc2_r8(&mut cpu, &mut bus, a, 6, 20);
    ctc2_r8(&mut cpu, &mut bus, a, 7, 30);

    mtc2_r8(&mut cpu, &mut bus, a, 0, 0x0064_0032);
    mtc2_r8(&mut cpu, &mut bus, a, 1, 0x0000_000A);

    escreve_e_executa(&mut cpu, &mut bus, a, cop2_mvmva(true, 0, 0, 0, false));
    escreve_e_executa(&mut cpu, &mut bus, a, nop());

    le_mfc2(&mut cpu, &mut bus, a, 10, 9);
    le_mfc2(&mut cpu, &mut bus, a, 11, 10);
    le_mfc2(&mut cpu, &mut bus, a, 12, 11);

    assert_eq!(
        cpu.regs[10] as i32,
        10 + 50,
        "MVMVA RT,V0,TR sf=1: IR1 = TRX + VX = 10 + 50 = 60"
    );
    assert_eq!(
        cpu.regs[11] as i32,
        20 + 100,
        "MVMVA RT,V0,TR sf=1: IR2 = TRY + VY = 20 + 100 = 120"
    );
    assert_eq!(
        cpu.regs[12] as i32,
        30 + 10,
        "MVMVA RT,V0,TR sf=1: IR3 = TRZ + VZ = 30 + 10 = 40"
    );
}

// ── MVMVA: RT/V0/TR sf=0 (resultados em 1.3.12, saturam para IR 16-bit) ──

#[test]
fn mvmva_rt_v0_tr_sf0_lm0() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    let a = 0x0000u32;

    ctc2_r8(&mut cpu, &mut bus, a, 0, 0x0000_0001);
    ctc2_r8(&mut cpu, &mut bus, a, 1, 0);
    ctc2_r8(&mut cpu, &mut bus, a, 2, 0x0000_0002);
    ctc2_r8(&mut cpu, &mut bus, a, 3, 0);
    ctc2_r8(&mut cpu, &mut bus, a, 4, 0x0003);

    ctc2_r8(&mut cpu, &mut bus, a, 5, 0);
    ctc2_r8(&mut cpu, &mut bus, a, 6, 0);
    ctc2_r8(&mut cpu, &mut bus, a, 7, 0);

    mtc2_r8(&mut cpu, &mut bus, a, 0, 0x0010_0008);
    mtc2_r8(&mut cpu, &mut bus, a, 1, 0x0020);

    escreve_e_executa(&mut cpu, &mut bus, a, cop2_mvmva(false, 0, 0, 3, false));
    escreve_e_executa(&mut cpu, &mut bus, a, nop());

    le_mfc2(&mut cpu, &mut bus, a, 10, 9);
    le_mfc2(&mut cpu, &mut bus, a, 11, 10);
    le_mfc2(&mut cpu, &mut bus, a, 12, 11);

    assert_eq!(
        cpu.regs[10] as i32,
        1 * 8 + 0 * 16 + 0 * 32,
        "MVMVA sf=0: IR1 = RT11*VX = 8, sem saturacao"
    );
    assert_eq!(
        cpu.regs[11] as i32,
        0 * 8 + 2 * 16 + 0 * 32,
        "MVMVA sf=0: IR2 = RT22*VY = 32"
    );
    assert_eq!(
        cpu.regs[12] as i32,
        0 * 8 + 0 * 16 + 3 * 32,
        "MVMVA sf=0: IR3 = RT33*VZ = 96"
    );
}

// ── MVMVA: LLM/V1/BK sf=1 ──

#[test]
fn mvmva_llm_v1_bk_sf1_lm0() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    let a = 0x0000u32;

    setup_light_matrix(&mut cpu, &mut bus, a);

    ctc2_r8(&mut cpu, &mut bus, a, 13, 0);
    ctc2_r8(&mut cpu, &mut bus, a, 14, 0);
    ctc2_r8(&mut cpu, &mut bus, a, 15, 0);

    mtc2_r8(&mut cpu, &mut bus, a, 2, 0x0004_0003);
    mtc2_r8(&mut cpu, &mut bus, a, 3, 0x0005);

    escreve_e_executa(&mut cpu, &mut bus, a, cop2_mvmva(true, 1, 1, 1, false));
    escreve_e_executa(&mut cpu, &mut bus, a, nop());

    le_mfc2(&mut cpu, &mut bus, a, 10, 9);
    le_mfc2(&mut cpu, &mut bus, a, 11, 10);
    le_mfc2(&mut cpu, &mut bus, a, 12, 11);

    assert_eq!(
        cpu.regs[10] as i32,
        (0x0800i32 * 3 + 0 * 4 + 0x0400i32 * 5) / 0x1000,
        "MVMVA LLM,V1,BK sf=1: IR1 = (L11*VX+L12*VY+L13*VZ)>>12 = (6144+5120)/4096 = 2"
    );
    assert_eq!(
        cpu.regs[11] as i32,
        (0 * 3 + 0x0C00i32 * 4 + 0 * 5) / 0x1000,
        "MVMVA LLM,V1,BK sf=1: IR2 = (L21*VX+L22*VY+L23*VZ)>>12 = 12288/4096 = 3"
    );
    assert_eq!(
        cpu.regs[12] as i32,
        (0x0200i32 * 3 + 0 * 4 + 0x1000i32 * 5) / 0x1000,
        "MVMVA LLM,V1,BK sf=1: IR3 = (L31*VX+L32*VY+L33*VZ)>>12 = (1536+20480)/4096 = 5"
    );
}

// ── MVMVA: LCM/V2/TR sf=1 ──

#[test]
fn mvmva_lcm_v2_tr_sf1_lm0() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    let a = 0x0000u32;

    ctc2_r8(&mut cpu, &mut bus, a, 16, 0x0000_0800);
    ctc2_r8(&mut cpu, &mut bus, a, 17, 0);
    ctc2_r8(&mut cpu, &mut bus, a, 18, 0x0000_0400);
    ctc2_r8(&mut cpu, &mut bus, a, 19, 0);
    ctc2_r8(&mut cpu, &mut bus, a, 20, 0x0C00);

    ctc2_r8(&mut cpu, &mut bus, a, 5, 0);
    ctc2_r8(&mut cpu, &mut bus, a, 6, 0);
    ctc2_r8(&mut cpu, &mut bus, a, 7, 0);

    mtc2_r8(&mut cpu, &mut bus, a, 4, 0x0020_0010);
    mtc2_r8(&mut cpu, &mut bus, a, 5, 0x0008);

    escreve_e_executa(&mut cpu, &mut bus, a, cop2_mvmva(true, 2, 2, 0, false));
    escreve_e_executa(&mut cpu, &mut bus, a, nop());

    le_mfc2(&mut cpu, &mut bus, a, 10, 9);
    le_mfc2(&mut cpu, &mut bus, a, 11, 10);
    le_mfc2(&mut cpu, &mut bus, a, 12, 11);

    assert_eq!(
        cpu.regs[10] as i32,
        (0x0800i32 * 16 + 0 * 32 + 0 * 8) / 0x1000,
        "MVMVA LCM,V2,TR sf=1: IR1 = (LR1*VX+LR2*VY+LR3*VZ)>>12 = 32768/4096 = 8"
    );
    assert_eq!(
        cpu.regs[11] as i32,
        (0 * 16 + 0x0400i32 * 32 + 0 * 8) / 0x1000,
        "MVMVA LCM,V2,TR sf=1: IR2 = (LG1*VX+LG2*VY+LG3*VZ)>>12 = 32768/4096 = 8"
    );
    assert_eq!(
        cpu.regs[12] as i32,
        (0 * 16 + 0 * 32 + 0x0C00i32 * 8) / 0x1000,
        "MVMVA LCM,V2,TR sf=1: IR3 = (LB1*VX+LB2*VY+LB3*VZ)>>12 = 24576/4096 = 6"
    );
}

// ── MVMVA: RT/IR/None sf=1 ──

#[test]
fn mvmva_rt_ir_none_sf1_lm0() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    let a = 0x0000u32;

    setup_rt_identity(&mut cpu, &mut bus, a);

    mtc2_r8(&mut cpu, &mut bus, a, 9, 0x000A);
    mtc2_r8(&mut cpu, &mut bus, a, 10, 0x0014);
    mtc2_r8(&mut cpu, &mut bus, a, 11, 0x001E);

    escreve_e_executa(&mut cpu, &mut bus, a, cop2_mvmva(true, 0, 3, 3, false));
    escreve_e_executa(&mut cpu, &mut bus, a, nop());

    le_mfc2(&mut cpu, &mut bus, a, 12, 9);
    le_mfc2(&mut cpu, &mut bus, a, 13, 10);
    le_mfc2(&mut cpu, &mut bus, a, 14, 11);

    assert_eq!(
        cpu.regs[12] as i32, 10,
        "MVMVA RT,IR,None sf=1: IR1 = 1.0*IR1+0+0 = 10"
    );
    assert_eq!(cpu.regs[13] as i32, 20, "MVMVA RT,IR,None sf=1: IR2 = 20");
    assert_eq!(cpu.regs[14] as i32, 30, "MVMVA RT,IR,None sf=1: IR3 = 30");
}

// ── MVMVA: saturacao IR com lm=1 ──

#[test]
fn mvmva_saturacao_ir_lm1_negativo_saturado_em_zero() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    let a = 0x0000u32;

    ctc2_r8(&mut cpu, &mut bus, a, 0, 0x8000u32);
    ctc2_r8(&mut cpu, &mut bus, a, 1, 0);
    ctc2_r8(&mut cpu, &mut bus, a, 2, 0x0000_1000);
    ctc2_r8(&mut cpu, &mut bus, a, 3, 0);
    ctc2_r8(&mut cpu, &mut bus, a, 4, 0x1000);

    mtc2_r8(&mut cpu, &mut bus, a, 0, 0x0001_0001);
    mtc2_r8(&mut cpu, &mut bus, a, 1, 0);

    escreve_e_executa(&mut cpu, &mut bus, a, cop2_mvmva(true, 0, 0, 3, true));
    escreve_e_executa(&mut cpu, &mut bus, a, nop());

    le_mfc2(&mut cpu, &mut bus, a, 10, 9);
    le_cfc2(&mut cpu, &mut bus, a, 11, 31);

    assert_eq!(
        cpu.regs[10] as i32, 0,
        "MVMVA lm=1: IR1 de -1 saturado em 0 com lm=1"
    );
    assert_ne!(
        cpu.regs[11] & (1 << 24),
        0,
        "FLAG.24 acionado por saturacao IR1"
    );
}

// ── MVMVA: RT/V1/None sf=1 ──

#[test]
fn mvmva_rt_v1_tr_sf1_lm0() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    let a = 0x0000u32;

    setup_rt_identity(&mut cpu, &mut bus, a);

    mtc2_r8(&mut cpu, &mut bus, a, 2, 0x0004_0003);
    mtc2_r8(&mut cpu, &mut bus, a, 3, 0x0005);

    escreve_e_executa(&mut cpu, &mut bus, a, cop2_mvmva(true, 0, 1, 3, false));
    escreve_e_executa(&mut cpu, &mut bus, a, nop());

    le_mfc2(&mut cpu, &mut bus, a, 10, 9);
    le_mfc2(&mut cpu, &mut bus, a, 11, 10);
    le_mfc2(&mut cpu, &mut bus, a, 12, 11);

    assert_eq!(
        cpu.regs[10] as i32, 3,
        "MVMVA RT,V1,None sf=1: IR1 = V1_X = 3"
    );
    assert_eq!(
        cpu.regs[11] as i32, 4,
        "MVMVA RT,V1,None sf=1: IR2 = V1_Y = 4"
    );
    assert_eq!(
        cpu.regs[12] as i32, 5,
        "MVMVA RT,V1,None sf=1: IR3 = V1_Z = 5"
    );
}

// ── MVMVA: m33 negativo (sign-extend) ──

#[test]
fn mvmva_lcm_m33_negativo_sign_extend() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    let a = 0x0000u32;

    ctc2_r8(&mut cpu, &mut bus, a, 16, 0);
    ctc2_r8(&mut cpu, &mut bus, a, 17, 0);
    ctc2_r8(&mut cpu, &mut bus, a, 18, 0);
    ctc2_r8(&mut cpu, &mut bus, a, 19, 0);
    ctc2_r8(&mut cpu, &mut bus, a, 20, 0xC000u32);

    mtc2_r8(&mut cpu, &mut bus, a, 4, 0);
    mtc2_r8(&mut cpu, &mut bus, a, 5, 0x0010);

    escreve_e_executa(&mut cpu, &mut bus, a, cop2_mvmva(true, 2, 2, 3, false));
    escreve_e_executa(&mut cpu, &mut bus, a, nop());

    le_mfc2(&mut cpu, &mut bus, a, 10, 11);

    assert_eq!(
        cpu.regs[10] as i32, -64,
        "MVMVA LCM,V2,None sf=1: LB3=0xC000 sign-extend=-16384, IR3=-16384*16>>12=-64"
    );
}
