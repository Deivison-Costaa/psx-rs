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

fn passo_instrucoes(cpu: &mut Cpu, bus: &mut psx_core::bus::Bus, n: usize) {
    for _ in 0..n {
        cpu.step(bus);
    }
}

fn setup_rot_identidade(bus: &mut psx_core::bus::Bus, cpu: &mut Cpu, addr: &mut u32) {
    cpu.regs[8] = 0x1000;
    bus.write32::<BusRead>(*addr, ctc2(8, 0));
    *addr += 4;
    bus.write32::<BusRead>(*addr, ctc2(8, 1));
    *addr += 4;
    bus.write32::<BusRead>(*addr, ctc2(8, 5));
    *addr += 4;
}

#[test]
fn rtps_perspectiva_simples_sem_saturacao() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    cpu.pc = 0;
    let mut a = 0u32;

    cpu.regs[8] = 0x1000;
    bus.write32::<BusRead>(a, ctc2(8, 0));
    a += 4;
    bus.write32::<BusRead>(a, ctc2(8, 1));
    a += 4;
    bus.write32::<BusRead>(a, ctc2(8, 5));
    a += 4;

    cpu.regs[8] = 0;
    bus.write32::<BusRead>(a, ctc2(8, 5));
    a += 4;
    bus.write32::<BusRead>(a, ctc2(8, 6));
    a += 4;
    cpu.regs[8] = 256;
    bus.write32::<BusRead>(a, ctc2(8, 7));
    a += 4;

    cpu.regs[8] = 0x0000_0100;
    bus.write32::<BusRead>(a, ctc2(8, 26));
    a += 4;
    cpu.regs[8] = 0x0002_0000;
    bus.write32::<BusRead>(a, ctc2(8, 24));
    a += 4;
    bus.write32::<BusRead>(a, ctc2(8, 25));
    a += 4;

    cpu.regs[8] = 100;
    bus.write32::<BusRead>(a, mtc2(8, 0));
    a += 4;
    cpu.regs[8] = 50;
    bus.write32::<BusRead>(a, mtc2(8, 1));
    a += 4;
    cpu.regs[8] = 0;
    bus.write32::<BusRead>(a, mtc2(8, 2));
    a += 4;

    bus.write32::<BusRead>(a, cop2_cmd(0x01, true, false));
    a += 4;
    bus.write32::<BusRead>(a, nop());
    a += 4;

    bus.write32::<BusRead>(a, mfc2(10, 9));
    a += 4;
    bus.write32::<BusRead>(a, nop());
    a += 4;
    bus.write32::<BusRead>(a, mfc2(11, 10));
    a += 4;
    bus.write32::<BusRead>(a, nop());
    a += 4;
    bus.write32::<BusRead>(a, mfc2(12, 11));
    a += 4;
    bus.write32::<BusRead>(a, nop());
    a += 4;
    bus.write32::<BusRead>(a, mfc2(13, 14));
    a += 4;
    bus.write32::<BusRead>(a, nop());
    a += 4;
    bus.write32::<BusRead>(a, mfc2(14, 19));
    a += 4;
    bus.write32::<BusRead>(a, nop());
    a += 4;
    bus.write32::<BusRead>(a, cfc2(15, 31));
    a += 4;
    bus.write32::<BusRead>(a, nop());
    a += 4;

    passo_instrucoes(&mut cpu, &mut bus, (a / 4) as usize);

    assert_eq!(cpu.regs[10], 0xA0, "IR1 apos RTPS: 160");
    assert_eq!(cpu.regs[11], 0x32, "IR2 apos RTPS: 50");
    assert_eq!(cpu.regs[12], 0x100, "IR3 apos RTPS: 256");
    assert_eq!(cpu.regs[13], 0x0032_00A0, "SXY2 apos RTPS: SY2=50 SX2=160");
    assert_eq!(cpu.regs[14], 0x100, "SZ3 apos RTPS: 256");
    assert_eq!(
        cpu.regs[15] & 0x7FFF_F800,
        0,
        "FLAG limpo apos RTPS sem saturacao"
    );
}

#[test]
fn rtpt_processa_tres_vertices_e_desloca_fifos() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    cpu.pc = 0;
    let mut a = 0u32;

    setup_rot_identidade(&mut bus, &mut cpu, &mut a);

    cpu.regs[8] = 0;
    bus.write32::<BusRead>(a, ctc2(8, 5));
    a += 4;
    bus.write32::<BusRead>(a, ctc2(8, 6));
    a += 4;
    cpu.regs[8] = 0x100;
    bus.write32::<BusRead>(a, ctc2(8, 7));
    a += 4;

    cpu.regs[8] = 0x0000_0100;
    bus.write32::<BusRead>(a, ctc2(8, 26));
    a += 4;
    cpu.regs[8] = 0x0001_0000;
    bus.write32::<BusRead>(a, ctc2(8, 24));
    a += 4;
    bus.write32::<BusRead>(a, ctc2(8, 25));
    a += 4;

    cpu.regs[8] = 100;
    bus.write32::<BusRead>(a, mtc2(8, 0));
    a += 4;
    cpu.regs[8] = 50;
    bus.write32::<BusRead>(a, mtc2(8, 1));
    a += 4;
    cpu.regs[8] = 0;
    bus.write32::<BusRead>(a, mtc2(8, 2));
    a += 4;

    cpu.regs[8] = 200;
    bus.write32::<BusRead>(a, mtc2(8, 3));
    a += 4;
    cpu.regs[8] = 100;
    bus.write32::<BusRead>(a, mtc2(8, 4));
    a += 4;
    cpu.regs[8] = 0;
    bus.write32::<BusRead>(a, mtc2(8, 5));
    a += 4;

    bus.write32::<BusRead>(a, cop2_cmd(0x30, true, false));
    a += 4;
    bus.write32::<BusRead>(a, nop());
    a += 4;

    bus.write32::<BusRead>(a, mfc2(10, 12));
    a += 4;
    bus.write32::<BusRead>(a, nop());
    a += 4;
    bus.write32::<BusRead>(a, mfc2(11, 13));
    a += 4;
    bus.write32::<BusRead>(a, nop());
    a += 4;
    bus.write32::<BusRead>(a, mfc2(12, 14));
    a += 4;
    bus.write32::<BusRead>(a, nop());
    a += 4;
    bus.write32::<BusRead>(a, mfc2(13, 16));
    a += 4;
    bus.write32::<BusRead>(a, nop());
    a += 4;
    bus.write32::<BusRead>(a, mfc2(14, 17));
    a += 4;
    bus.write32::<BusRead>(a, nop());
    a += 4;
    bus.write32::<BusRead>(a, mfc2(15, 18));
    a += 4;
    bus.write32::<BusRead>(a, nop());
    a += 4;
    bus.write32::<BusRead>(a, mfc2(16, 19));
    a += 4;
    bus.write32::<BusRead>(a, nop());
    a += 4;
    bus.write32::<BusRead>(a, cfc2(17, 31));
    a += 4;
    bus.write32::<BusRead>(a, nop());
    a += 4;

    passo_instrucoes(&mut cpu, &mut bus, (a / 4) as usize);

    assert_eq!(
        cpu.regs[10], 0x0032_00A0,
        "SXY0: resultado de V0 (SY=50, SX=160) deslocado para SXY0"
    );
    assert_eq!(
        cpu.regs[11], 0x0064_00C8,
        "SXY1: resultado de V1 (SY=100, SX=200) deslocado para SXY1"
    );
    assert_eq!(
        cpu.regs[12], 0x0064_00C8,
        "SXY2: resultado de V2 (SY=100, SX=200)"
    );
    assert_eq!(cpu.regs[13], 0x100, "SZ0: SZ de V0 (256)");
    assert_eq!(cpu.regs[14], 0x100, "SZ1: SZ de V1 (256)");
    assert_eq!(cpu.regs[15], 0x100, "SZ2: SZ de V2 (256)");
    assert_eq!(cpu.regs[16], 0x100, "SZ3: SZ de V2 (256) — ultimo");
    assert_eq!(
        cpu.regs[17] & 0x7FFF_F800,
        0,
        "FLAG limpo apos RTPT sem saturacao"
    );
}

#[test]
fn rtps_com_sf_zero_irao_diferentes_de_sf_um() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    cpu.pc = 0;
    let mut a = 0u32;

    setup_rot_identidade(&mut bus, &mut cpu, &mut a);

    cpu.regs[8] = 0;
    bus.write32::<BusRead>(a, ctc2(8, 5));
    a += 4;
    bus.write32::<BusRead>(a, ctc2(8, 6));
    a += 4;
    bus.write32::<BusRead>(a, ctc2(8, 7));
    a += 4;

    cpu.regs[8] = 0x0000_0100;
    bus.write32::<BusRead>(a, ctc2(8, 26));
    a += 4;
    cpu.regs[8] = 0;
    bus.write32::<BusRead>(a, ctc2(8, 24));
    a += 4;
    bus.write32::<BusRead>(a, ctc2(8, 25));
    a += 4;

    cpu.regs[8] = 5;
    bus.write32::<BusRead>(a, mtc2(8, 0));
    a += 4;
    bus.write32::<BusRead>(a, mtc2(8, 1));
    a += 4;
    cpu.regs[8] = 4;
    bus.write32::<BusRead>(a, mtc2(8, 2));
    a += 4;

    bus.write32::<BusRead>(a, cop2_cmd(0x01, false, false));
    a += 4;
    bus.write32::<BusRead>(a, nop());
    a += 4;

    bus.write32::<BusRead>(a, mfc2(10, 9));
    a += 4;
    bus.write32::<BusRead>(a, nop());
    a += 4;
    bus.write32::<BusRead>(a, mfc2(11, 11));
    a += 4;
    bus.write32::<BusRead>(a, nop());
    a += 4;
    bus.write32::<BusRead>(a, mfc2(12, 19));
    a += 4;
    bus.write32::<BusRead>(a, nop());
    a += 4;
    bus.write32::<BusRead>(a, cfc2(13, 31));
    a += 4;
    bus.write32::<BusRead>(a, nop());
    a += 4;

    passo_instrucoes(&mut cpu, &mut bus, (a / 4) as usize);

    assert_eq!(cpu.regs[10], 0x5000, "IR1 com sf=0: 4096*5 = 20480 = 0x5000");
    assert_eq!(cpu.regs[11], 0x4000, "IR3 com sf=0: 4096*4 = 16384 = 0x4000");
    assert_eq!(cpu.regs[12], 4, "SZ3 com sf=0: raw3>>12 = 0x4000>>12 = 4");
    assert_eq!(
        cpu.regs[13] & 0x7FFF_F800,
        0,
        "FLAG limpo apos RTPS sf=0 sem saturacao"
    );
}
