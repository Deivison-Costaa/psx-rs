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

fn cop2_cmd(real_cmd: u32, sf: bool, lm: bool) -> u32 {
    (0x12 << 26) | (1 << 25) | (real_cmd & 0x3F) | ((sf as u32) << 19) | ((lm as u32) << 10)
}

fn escreve_e_executa(cpu: &mut Cpu, bus: &mut psx_core::bus::Bus, addr: u32, instr: u32) {
    bus.write32::<BusRead>(addr, instr);
    cpu.pc = addr;
    cpu.step(bus);
}

fn mtc2_r8(cpu: &mut Cpu, bus: &mut psx_core::bus::Bus, addr: u32, rd: u32, val: u32) {
    cpu.regs[8] = val;
    escreve_e_executa(cpu, bus, addr, mtc2(8, rd));
}

fn ctc2_r8(cpu: &mut Cpu, bus: &mut psx_core::bus::Bus, addr: u32, rd: u32, val: u32) {
    cpu.regs[8] = val;
    escreve_e_executa(cpu, bus, addr, ctc2(8, rd));
}

fn le_mfc2(cpu: &mut Cpu, bus: &mut psx_core::bus::Bus, a: u32, dst: usize, rd: u32) {
    escreve_e_executa(cpu, bus, a, mfc2(dst as u32, rd));
    escreve_e_executa(cpu, bus, a, nop());
}

fn le_cfc2(cpu: &mut Cpu, bus: &mut psx_core::bus::Bus, a: u32, dst: usize, rd: u32) {
    escreve_e_executa(cpu, bus, a, cfc2(dst as u32, rd));
    escreve_e_executa(cpu, bus, a, nop());
}

// ── NCLIP ──

#[test]
fn nclip_poligono_horario_mac0_positivo() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    let a = 0x0000u32;

    mtc2_r8(&mut cpu, &mut bus, a, 12, 0x0000_0000);
    mtc2_r8(&mut cpu, &mut bus, a, 13, 0x0000_0064);
    mtc2_r8(&mut cpu, &mut bus, a, 14, 0x0064_0064);

    escreve_e_executa(&mut cpu, &mut bus, a, cop2_cmd(0x06, false, false));
    escreve_e_executa(&mut cpu, &mut bus, a, nop());

    le_mfc2(&mut cpu, &mut bus, a, 10, 24);

    assert_eq!(
        cpu.regs[10] as i32, 10000,
        "NCLIP horario: MAC0 = SX0*SY1 + SX1*SY2 + SX2*SY0 - SX0*SY2 - SX1*SY0 - SX2*SY1 = 10000"
    );
}

#[test]
fn nclip_poligono_anti_horario_mac0_negativo() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    let a = 0x0000u32;

    mtc2_r8(&mut cpu, &mut bus, a, 12, 0x0000_0000);
    mtc2_r8(&mut cpu, &mut bus, a, 13, 0x0064_0000);
    mtc2_r8(&mut cpu, &mut bus, a, 14, 0x0000_0064);

    escreve_e_executa(&mut cpu, &mut bus, a, cop2_cmd(0x06, false, false));
    escreve_e_executa(&mut cpu, &mut bus, a, nop());

    le_mfc2(&mut cpu, &mut bus, a, 10, 24);

    assert_eq!(
        cpu.regs[10] as i32, -10000,
        "NCLIP anti-horario: MAC0 = -10000"
    );
}

#[test]
fn nclip_colinear_mac0_zero() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    let a = 0x0000u32;

    mtc2_r8(&mut cpu, &mut bus, a, 12, 0x0000_0000);
    mtc2_r8(&mut cpu, &mut bus, a, 13, 0x0032_0032);
    mtc2_r8(&mut cpu, &mut bus, a, 14, 0x0064_0064);

    escreve_e_executa(&mut cpu, &mut bus, a, cop2_cmd(0x06, false, false));
    escreve_e_executa(&mut cpu, &mut bus, a, nop());

    le_mfc2(&mut cpu, &mut bus, a, 10, 24);

    assert_eq!(
        cpu.regs[10] as i32, 0,
        "NCLIP colinear: pontos em linha reta devem gerar MAC0=0"
    );
}

// ── AVSZ3 ──

#[test]
fn avsz3_media_ponderada_de_tres_z() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    let a = 0x0000u32;

    ctc2_r8(&mut cpu, &mut bus, a, 29, 0x0000_0068);
    ctc2_r8(&mut cpu, &mut bus, a, 26, 0x0000_0100);
    ctc2_r8(&mut cpu, &mut bus, a, 24, 0);
    ctc2_r8(&mut cpu, &mut bus, a, 25, 0);
    ctc2_r8(&mut cpu, &mut bus, a, 27, 0);
    ctc2_r8(&mut cpu, &mut bus, a, 28, 0);

    mtc2_r8(&mut cpu, &mut bus, a, 17, 100);
    mtc2_r8(&mut cpu, &mut bus, a, 18, 200);
    mtc2_r8(&mut cpu, &mut bus, a, 19, 300);

    escreve_e_executa(&mut cpu, &mut bus, a, cop2_cmd(0x2D, false, false));
    escreve_e_executa(&mut cpu, &mut bus, a, nop());

    le_mfc2(&mut cpu, &mut bus, a, 10, 7);
    le_mfc2(&mut cpu, &mut bus, a, 11, 24);

    assert_eq!(
        cpu.regs[10], 15,
        "AVSZ3 OTZ: ZSF3*(SZ1+SZ2+SZ3)/1000h = 0x68*600/4096 = 15 (truncado)"
    );
    assert_eq!(
        cpu.regs[11] as i32,
        0x68 * 600,
        "AVSZ3 MAC0: ZSF3*(SZ1+SZ2+SZ3) = 0x68*600 = 62400"
    );
}

#[test]
fn avsz3_otz_saturado_em_maximo() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    let a = 0x0000u32;

    ctc2_r8(&mut cpu, &mut bus, a, 29, (0x1000i32) as u32);
    ctc2_r8(&mut cpu, &mut bus, a, 26, 0x0000_0100);
    ctc2_r8(&mut cpu, &mut bus, a, 24, 0);
    ctc2_r8(&mut cpu, &mut bus, a, 25, 0);
    ctc2_r8(&mut cpu, &mut bus, a, 27, 0);
    ctc2_r8(&mut cpu, &mut bus, a, 28, 0);

    mtc2_r8(&mut cpu, &mut bus, a, 17, 0xFFFF);
    mtc2_r8(&mut cpu, &mut bus, a, 18, 0xFFFF);
    mtc2_r8(&mut cpu, &mut bus, a, 19, 0xFFFF);

    escreve_e_executa(&mut cpu, &mut bus, a, cop2_cmd(0x2D, false, false));
    escreve_e_executa(&mut cpu, &mut bus, a, nop());

    le_mfc2(&mut cpu, &mut bus, a, 10, 7);
    le_cfc2(&mut cpu, &mut bus, a, 11, 31);

    assert_eq!(
        cpu.regs[10], 0xFFFF,
        "AVSZ3 OTZ saturado em 0xFFFF quando MAC0/1000h > 0xFFFF"
    );
    assert_ne!(
        cpu.regs[11] & (1 << 18),
        0,
        "FLAG.18 (SZ3/OTZ) acionado por saturacao"
    );
}

// ── AVSZ4 ──

#[test]
fn avsz4_media_ponderada_de_quatro_z() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    let a = 0x0000u32;

    ctc2_r8(&mut cpu, &mut bus, a, 30, 0x0000_0040);
    ctc2_r8(&mut cpu, &mut bus, a, 26, 0x0000_0100);
    ctc2_r8(&mut cpu, &mut bus, a, 24, 0);
    ctc2_r8(&mut cpu, &mut bus, a, 25, 0);
    ctc2_r8(&mut cpu, &mut bus, a, 27, 0);
    ctc2_r8(&mut cpu, &mut bus, a, 28, 0);

    mtc2_r8(&mut cpu, &mut bus, a, 16, 100);
    mtc2_r8(&mut cpu, &mut bus, a, 17, 200);
    mtc2_r8(&mut cpu, &mut bus, a, 18, 300);
    mtc2_r8(&mut cpu, &mut bus, a, 19, 400);

    escreve_e_executa(&mut cpu, &mut bus, a, cop2_cmd(0x2E, false, false));
    escreve_e_executa(&mut cpu, &mut bus, a, nop());

    le_mfc2(&mut cpu, &mut bus, a, 10, 7);
    le_mfc2(&mut cpu, &mut bus, a, 11, 24);

    assert_eq!(
        cpu.regs[10], 15,
        "AVSZ4 OTZ: ZSF4*(SZ0+SZ1+SZ2+SZ3)/1000h = 0x40*1000/4096 = 15 (truncado)"
    );
    assert_eq!(
        cpu.regs[11] as i32,
        0x40 * 1000,
        "AVSZ4 MAC0: ZSF4*(SZ0+SZ1+SZ2+SZ3) = 0x40*1000 = 64000"
    );
}

// ── SQR ──

#[test]
fn sqr_sf0_quadrado_de_ir_produz_mac_e_ir() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    let a = 0x0000u32;

    mtc2_r8(&mut cpu, &mut bus, a, 9, 1);
    mtc2_r8(&mut cpu, &mut bus, a, 10, 2);
    mtc2_r8(&mut cpu, &mut bus, a, 11, 3);

    escreve_e_executa(&mut cpu, &mut bus, a, cop2_cmd(0x28, false, false));
    escreve_e_executa(&mut cpu, &mut bus, a, nop());

    le_mfc2(&mut cpu, &mut bus, a, 4, 9);
    le_mfc2(&mut cpu, &mut bus, a, 5, 10);
    le_mfc2(&mut cpu, &mut bus, a, 6, 11);

    assert_eq!(cpu.regs[4] as i32, 1, "SQR sf=0: IR1 = 1*1 = 1");
    assert_eq!(cpu.regs[5] as i32, 4, "SQR sf=0: IR2 = 2*2 = 4");
    assert_eq!(cpu.regs[6] as i32, 9, "SQR sf=0: IR3 = 3*3 = 9");
}

#[test]
fn sqr_sf1_desloca_12_bits_para_direita() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    let a = 0x0000u32;

    mtc2_r8(&mut cpu, &mut bus, a, 9, 0x0100u32);
    mtc2_r8(&mut cpu, &mut bus, a, 10, 0x0100u32 as i16 as u32);
    mtc2_r8(&mut cpu, &mut bus, a, 11, 0x0080u32 as i16 as u32);

    escreve_e_executa(&mut cpu, &mut bus, a, cop2_cmd(0x28, true, false));
    escreve_e_executa(&mut cpu, &mut bus, a, nop());

    le_mfc2(&mut cpu, &mut bus, a, 4, 9);

    assert_eq!(
        cpu.regs[4], 0x10,
        "SQR sf=1: IR1 = (0x100*0x100) >> 12 = 0x10000 >> 12 = 0x10"
    );
}

#[test]
fn sqr_saturacao_ir_em_7fff() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    let a = 0x0000u32;

    mtc2_r8(&mut cpu, &mut bus, a, 9, 0x4000u32);

    escreve_e_executa(&mut cpu, &mut bus, a, cop2_cmd(0x28, false, false));
    escreve_e_executa(&mut cpu, &mut bus, a, nop());

    le_mfc2(&mut cpu, &mut bus, a, 4, 9);
    le_cfc2(&mut cpu, &mut bus, a, 5, 31);

    assert_eq!(
        cpu.regs[4], 0x7FFF,
        "SQR: 0x4000*0x4000 = 0x1000_0000 > 0x7FFF, saturado"
    );
    assert_ne!(
        cpu.regs[5] & (1 << 24),
        0,
        "FLAG.24 acionado por saturacao IR1"
    );
}

// ── OP ──

#[test]
fn op_produto_vetorial_sf0_lm0() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    let a = 0x0000u32;

    mtc2_r8(&mut cpu, &mut bus, a, 9, 1);
    mtc2_r8(&mut cpu, &mut bus, a, 10, 2);
    mtc2_r8(&mut cpu, &mut bus, a, 11, 3);

    ctc2_r8(&mut cpu, &mut bus, a, 0, 0x0000_0005);
    ctc2_r8(&mut cpu, &mut bus, a, 1, 0);
    ctc2_r8(&mut cpu, &mut bus, a, 2, 0x0000_0007);
    ctc2_r8(&mut cpu, &mut bus, a, 3, 0);
    ctc2_r8(&mut cpu, &mut bus, a, 4, 0x0000_000B);

    escreve_e_executa(&mut cpu, &mut bus, a, cop2_cmd(0x0C, false, false));
    escreve_e_executa(&mut cpu, &mut bus, a, nop());

    le_mfc2(&mut cpu, &mut bus, a, 4, 9);
    le_mfc2(&mut cpu, &mut bus, a, 5, 10);
    le_mfc2(&mut cpu, &mut bus, a, 6, 11);

    assert_eq!(
        cpu.regs[4] as i32,
        3 * 7 - 2 * 11,
        "OP MAC1/IR1 = IR3*D2 - IR2*D3 = 21 - 22 = -1"
    );
    assert_eq!(
        cpu.regs[5] as i32,
        11 - 3 * 5,
        "OP MAC2/IR2 = IR1*D3 - IR3*D1 = 11 - 15 = -4"
    );
    assert_eq!(
        cpu.regs[6] as i32,
        2 * 5 - 7,
        "OP MAC3/IR3 = IR2*D1 - IR1*D2 = 10 - 7 = 3"
    );
}

#[test]
fn op_sf1_desloca_12_bits() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    let a = 0x0000u32;

    mtc2_r8(&mut cpu, &mut bus, a, 9, 0x0100u32 as i16 as u32);
    mtc2_r8(&mut cpu, &mut bus, a, 10, 0x0200u32 as i16 as u32);
    mtc2_r8(&mut cpu, &mut bus, a, 11, 0x0080u32 as i16 as u32);

    ctc2_r8(&mut cpu, &mut bus, a, 0, 0x0000_0010);
    ctc2_r8(&mut cpu, &mut bus, a, 1, 0);
    ctc2_r8(&mut cpu, &mut bus, a, 2, 0x0000_0020);
    ctc2_r8(&mut cpu, &mut bus, a, 3, 0);
    ctc2_r8(&mut cpu, &mut bus, a, 4, 0x0000_0008);

    escreve_e_executa(&mut cpu, &mut bus, a, cop2_cmd(0x0C, true, false));
    escreve_e_executa(&mut cpu, &mut bus, a, nop());

    le_mfc2(&mut cpu, &mut bus, a, 4, 9);

    let expected_ir3_d2 = (0x80i32).wrapping_mul(32i32);
    let expected_ir2_d3 = (0x200i32).wrapping_mul(8i32);
    let mac1 = expected_ir3_d2 - expected_ir2_d3;
    let ir1_expected = mac1 >> 12;

    assert_eq!(
        cpu.regs[4] as i32, ir1_expected,
        "OP sf=1: MAC1 = IR3*D2 - IR2*D3 >> 12"
    );
}
