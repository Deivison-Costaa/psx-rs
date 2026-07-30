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

// ── MVMVA: RT/V0/TR ──

#[test]
fn mvmva_rt_v0_tr_sf0_lm0() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    let a = 0x0000u32;

    ctc2_r8(&mut cpu, &mut bus, a, 0, 0x0000_1000);
    ctc2_r8(&mut cpu, &mut bus, a, 1, 0);
    ctc2_r8(&mut cpu, &mut bus, a, 2, 0x0000_1000);
    ctc2_r8(&mut cpu, &mut bus, a, 3, 0);
    ctc2_r8(&mut cpu, &mut bus, a, 4, 0x1000);

    ctc2_r8(&mut cpu, &mut bus, a, 5, 0x0000_0064);
    ctc2_r8(&mut cpu, &mut bus, a, 6, 0x0000_00C8);
    ctc2_r8(&mut cpu, &mut bus, a, 7, 0x0000_012C);

    mtc2_r8(&mut cpu, &mut bus, a, 0, 0x0001_0002);
    mtc2_r8(&mut cpu, &mut bus, a, 1, 0x0003);

    escreve_e_executa(&mut cpu, &mut bus, a, cop2_mvmva(false, 0, 0, 0, false));
    escreve_e_executa(&mut cpu, &mut bus, a, nop());

    le_mfc2(&mut cpu, &mut bus, a, 10, 9);
    le_mfc2(&mut cpu, &mut bus, a, 11, 10);
    le_mfc2(&mut cpu, &mut bus, a, 12, 11);

    assert_eq!(
        cpu.regs[10] as i32,
        0x1000 * 1 + 0 * 2 + 0 * 3 + 100,
        "MVMVA RT,V0,TR sf=0: IR1 = RT11*Vx + RT12*Vy + RT13*Vz + TRX = 0x1000*1+0*2+0*3+100 = 4196"
    );
    assert_eq!(
        cpu.regs[11] as i32,
        0 * 1 + 0x1000 * 2 + 0 * 3 + 200,
        "MVMVA RT,V0,TR sf=0: IR2 = RT21*Vx + RT22*Vy + RT23*Vz + TRY = 0*1+0x1000*2+0*3+200 = 8392"
    );
    assert_eq!(
        cpu.regs[12] as i32,
        0 * 1 + 0 * 2 + 0x1000 * 3 + 300,
        "MVMVA RT,V0,TR sf=0: IR3 = RT31*Vx + RT32*Vy + RT33*Vz + TRZ = 0*1+0*2+0x1000*3+300 = 12588"
    );
}

// ── MVMVA: RT/V0/TR sf=1 ──

#[test]
fn mvmva_rt_v0_tr_sf1_lm0() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    let a = 0x0000u32;

    ctc2_r8(&mut cpu, &mut bus, a, 0, 0x0000_1000);
    ctc2_r8(&mut cpu, &mut bus, a, 1, 0);
    ctc2_r8(&mut cpu, &mut bus, a, 2, 0x0000_1000);
    ctc2_r8(&mut cpu, &mut bus, a, 3, 0);
    ctc2_r8(&mut cpu, &mut bus, a, 4, 0x1000);

    ctc2_r8(&mut cpu, &mut bus, a, 5, 0x0000_1000);
    ctc2_r8(&mut cpu, &mut bus, a, 6, 0x0000_2000);
    ctc2_r8(&mut cpu, &mut bus, a, 7, 0x0000_3000);

    mtc2_r8(&mut cpu, &mut bus, a, 0, 0x0002_0001);
    mtc2_r8(&mut cpu, &mut bus, a, 1, 0);

    escreve_e_executa(&mut cpu, &mut bus, a, cop2_mvmva(true, 0, 0, 0, false));
    escreve_e_executa(&mut cpu, &mut bus, a, nop());

    le_mfc2(&mut cpu, &mut bus, a, 10, 9);
    le_mfc2(&mut cpu, &mut bus, a, 11, 10);
    le_mfc2(&mut cpu, &mut bus, a, 12, 11);

    let raw1 = 0x1000 * 2 + 0 * 1 + 0 * 0 + 0x1000;
    let raw2 = 0 * 2 + 0x1000 * 1 + 0 * 0 + 0x2000;
    let raw3 = 0 * 2 + 0 * 1 + 0x1000 * 0 + 0x3000;

    assert_eq!(
        cpu.regs[10] as i32,
        raw1 >> 12,
        "MVMVA sf=1: IR1 = (TRX*1000h + RT11*Vx + RT12*Vy + RT13*Vz) >> 12"
    );
    assert_eq!(
        cpu.regs[11] as i32,
        raw2 >> 12,
        "MVMVA sf=1: IR2"
    );
    assert_eq!(
        cpu.regs[12] as i32,
        raw3 >> 12,
        "MVMVA sf=1: IR3"
    );
}

// ── MVMVA: LLM/V1/BK ──

#[test]
fn mvmva_llm_v1_bk_sf0_lm0() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    let a = 0x0000u32;

    ctc2_r8(&mut cpu, &mut bus, a, 8, 0x0000_0100);
    ctc2_r8(&mut cpu, &mut bus, a, 9, 0);
    ctc2_r8(&mut cpu, &mut bus, a, 10, 0x0000_0200);
    ctc2_r8(&mut cpu, &mut bus, a, 11, 0);
    ctc2_r8(&mut cpu, &mut bus, a, 12, 0x0300);

    ctc2_r8(&mut cpu, &mut bus, a, 13, 0x0000_000A);
    ctc2_r8(&mut cpu, &mut bus, a, 14, 0x0000_0014);
    ctc2_r8(&mut cpu, &mut bus, a, 15, 0x0000_001E);

    mtc2_r8(&mut cpu, &mut bus, a, 2, 0x0001_0002);
    mtc2_r8(&mut cpu, &mut bus, a, 3, 0x0003);

    escreve_e_executa(&mut cpu, &mut bus, a, cop2_mvmva(false, 1, 1, 1, false));
    escreve_e_executa(&mut cpu, &mut bus, a, nop());

    le_mfc2(&mut cpu, &mut bus, a, 10, 9);
    le_mfc2(&mut cpu, &mut bus, a, 11, 10);
    le_mfc2(&mut cpu, &mut bus, a, 12, 11);

    assert_eq!(
        cpu.regs[10] as i32,
        0x100 * 1 + 0 * 2 + 0 * 3 + 10,
        "MVMVA LLM,V1,BK: IR1 = L11*Vx + L12*Vy + L13*Vz + RBK"
    );
    assert_eq!(
        cpu.regs[11] as i32,
        0 * 1 + 0x200 * 2 + 0 * 3 + 20,
        "MVMVA LLM,V1,BK: IR2"
    );
    assert_eq!(
        cpu.regs[12] as i32,
        0 * 1 + 0 * 2 + 0x300 * 3 + 30,
        "MVMVA LLM,V1,BK: IR3"
    );
}

// ── MVMVA: LCM/V2/TR ──

#[test]
fn mvmva_lcm_v2_tr_sf0_lm0() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    let a = 0x0000u32;

    ctc2_r8(&mut cpu, &mut bus, a, 16, 0x0000_0010);
    ctc2_r8(&mut cpu, &mut bus, a, 17, 0x0000_0020);
    ctc2_r8(&mut cpu, &mut bus, a, 18, 0x0000_0030);
    ctc2_r8(&mut cpu, &mut bus, a, 19, 0);
    ctc2_r8(&mut cpu, &mut bus, a, 20, 0x0040);

    ctc2_r8(&mut cpu, &mut bus, a, 5, 0x0000_0064);
    ctc2_r8(&mut cpu, &mut bus, a, 6, 0);
    ctc2_r8(&mut cpu, &mut bus, a, 7, 0);

    mtc2_r8(&mut cpu, &mut bus, a, 4, 0x0003_0002);
    mtc2_r8(&mut cpu, &mut bus, a, 5, 0x0001);

    escreve_e_executa(&mut cpu, &mut bus, a, cop2_mvmva(false, 2, 2, 0, false));
    escreve_e_executa(&mut cpu, &mut bus, a, nop());

    le_mfc2(&mut cpu, &mut bus, a, 10, 9);
    le_mfc2(&mut cpu, &mut bus, a, 11, 10);
    le_mfc2(&mut cpu, &mut bus, a, 12, 11);

    assert_eq!(
        cpu.regs[10] as i32,
        0x10 * 3 + 0x20 * 2 + 0 * 1 + 100,
        "MVMVA LCM,V2,TR: IR1 = LR1*Vx + LR2*Vy + LR3*Vz + TRX"
    );
    assert_eq!(
        cpu.regs[11] as i32,
        0x30 * 3 + 0 * 2 + 0 * 1 + 0,
        "MVMVA LCM,V2,TR: IR2"
    );
    assert_eq!(
        cpu.regs[12] as i32,
        0 * 3 + 0 * 2 + 0x40 * 1 + 0,
        "MVMVA LCM,V2,TR: IR3"
    );
}

// ── MVMVA: RT/IR/None ──

#[test]
fn mvmva_rt_ir_none_sf0_lm0() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    let a = 0x0000u32;

    ctc2_r8(&mut cpu, &mut bus, a, 0, 0x0000_0002);
    ctc2_r8(&mut cpu, &mut bus, a, 1, 0);
    ctc2_r8(&mut cpu, &mut bus, a, 2, 0x0000_0003);
    ctc2_r8(&mut cpu, &mut bus, a, 3, 0);
    ctc2_r8(&mut cpu, &mut bus, a, 4, 0x0005);

    mtc2_r8(&mut cpu, &mut bus, a, 9, 0x000A);
    mtc2_r8(&mut cpu, &mut bus, a, 10, 0x0014);
    mtc2_r8(&mut cpu, &mut bus, a, 11, 0x001E);

    escreve_e_executa(&mut cpu, &mut bus, a, cop2_mvmva(false, 0, 3, 3, false));
    escreve_e_executa(&mut cpu, &mut bus, a, nop());

    le_mfc2(&mut cpu, &mut bus, a, 12, 9);
    le_mfc2(&mut cpu, &mut bus, a, 13, 10);
    le_mfc2(&mut cpu, &mut bus, a, 14, 11);

    assert_eq!(
        cpu.regs[12] as i32,
        0x2 * 10 + 0 * 20 + 0 * 30,
        "MVMVA RT,IR,None: IR1 = RT11*IR1 + RT12*IR2 + RT13*IR3, sem translacao"
    );
    assert_eq!(
        cpu.regs[13] as i32,
        0 * 10 + 0x3 * 20 + 0 * 30,
        "MVMVA RT,IR,None: IR2"
    );
    assert_eq!(
        cpu.regs[14] as i32,
        0 * 10 + 0 * 20 + 0x5 * 30,
        "MVMVA RT,IR,None: IR3"
    );
}

// ── MVMVA: saturacao ──

#[test]
fn mvmva_saturacao_ir_lm1() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    let a = 0x0000u32;

    ctc2_r8(&mut cpu, &mut bus, a, 0, 0x7FFF);
    ctc2_r8(&mut cpu, &mut bus, a, 1, 0);
    ctc2_r8(&mut cpu, &mut bus, a, 2, 0x7FFF);
    ctc2_r8(&mut cpu, &mut bus, a, 3, 0);
    ctc2_r8(&mut cpu, &mut bus, a, 4, 0x7FFF);

    mtc2_r8(&mut cpu, &mut bus, a, 0, 0x7FFF_7FFF);
    mtc2_r8(&mut cpu, &mut bus, a, 1, 0x7FFF);

    escreve_e_executa(&mut cpu, &mut bus, a, cop2_mvmva(false, 0, 0, 3, true));
    escreve_e_executa(&mut cpu, &mut bus, a, nop());

    le_mfc2(&mut cpu, &mut bus, a, 10, 9);
    le_cfc2(&mut cpu, &mut bus, a, 11, 31);

    assert_eq!(
        cpu.regs[10] as i32, 0x7FFF,
        "MVMVA lm=1: IR1 saturado em 0x7FFF quando resultado excede"
    );
    assert_ne!(
        cpu.regs[11] & (1 << 24),
        0,
        "FLAG.24 acionado por saturacao IR1"
    );
}

// ── MVMVA: V1 ──

#[test]
fn mvmva_rt_v1_tr_sf0_lm0() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    let a = 0x0000u32;

    ctc2_r8(&mut cpu, &mut bus, a, 0, 0x0010);
    ctc2_r8(&mut cpu, &mut bus, a, 1, 0);
    ctc2_r8(&mut cpu, &mut bus, a, 2, 0x0000_0020);
    ctc2_r8(&mut cpu, &mut bus, a, 3, 0);
    ctc2_r8(&mut cpu, &mut bus, a, 4, 0x0030);

    mtc2_r8(&mut cpu, &mut bus, a, 2, 0x0001_0001);
    mtc2_r8(&mut cpu, &mut bus, a, 3, 0x0001);

    escreve_e_executa(&mut cpu, &mut bus, a, cop2_mvmva(false, 0, 1, 3, false));
    escreve_e_executa(&mut cpu, &mut bus, a, nop());

    le_mfc2(&mut cpu, &mut bus, a, 10, 9);

    assert_eq!(
        cpu.regs[10] as i32,
        0x10 * 1 + 0 * 1 + 0 * 1,
        "MVMVA RT,V1,None: usa V1 em vez de V0"
    );
}
