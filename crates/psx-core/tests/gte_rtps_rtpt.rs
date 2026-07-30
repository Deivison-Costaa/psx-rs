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

fn ctc2_r8(cpu: &mut Cpu, bus: &mut psx_core::bus::Bus, addr: u32, rd: u32, val: u32) {
    cpu.regs[8] = val;
    escreve_e_executa(cpu, bus, addr, ctc2(8, rd));
}

fn mtc2_r8(cpu: &mut Cpu, bus: &mut psx_core::bus::Bus, addr: u32, rd: u32, val: u32) {
    cpu.regs[8] = val;
    escreve_e_executa(cpu, bus, addr, mtc2(8, rd));
}

#[test]
fn rtps_perspectiva_simples_sem_saturacao() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    let a = 0x0000u32;

    ctc2_r8(&mut cpu, &mut bus, a, 0, 0x0000_1000);
    ctc2_r8(&mut cpu, &mut bus, a, 1, 0);
    ctc2_r8(&mut cpu, &mut bus, a, 2, 0x0000_1000);
    ctc2_r8(&mut cpu, &mut bus, a, 3, 0);
    ctc2_r8(&mut cpu, &mut bus, a, 4, 0x1000);

    ctc2_r8(&mut cpu, &mut bus, a, 5, 0);
    ctc2_r8(&mut cpu, &mut bus, a, 6, 0);
    ctc2_r8(&mut cpu, &mut bus, a, 7, 256);

    ctc2_r8(&mut cpu, &mut bus, a, 26, 0x0000_0100);
    ctc2_r8(&mut cpu, &mut bus, a, 24, 0);
    ctc2_r8(&mut cpu, &mut bus, a, 25, 0);
    ctc2_r8(&mut cpu, &mut bus, a, 27, 0);
    ctc2_r8(&mut cpu, &mut bus, a, 28, 0);

    mtc2_r8(&mut cpu, &mut bus, a, 0, 0x0032_0064);
    mtc2_r8(&mut cpu, &mut bus, a, 1, 0);

    escreve_e_executa(&mut cpu, &mut bus, a, cop2_cmd(0x01, true, false));
    escreve_e_executa(&mut cpu, &mut bus, a, nop());

    escreve_e_executa(&mut cpu, &mut bus, a, mfc2(10, 9));
    escreve_e_executa(&mut cpu, &mut bus, a, nop());
    escreve_e_executa(&mut cpu, &mut bus, a, mfc2(11, 10));
    escreve_e_executa(&mut cpu, &mut bus, a, nop());
    escreve_e_executa(&mut cpu, &mut bus, a, mfc2(12, 11));
    escreve_e_executa(&mut cpu, &mut bus, a, nop());
    escreve_e_executa(&mut cpu, &mut bus, a, mfc2(13, 14));
    escreve_e_executa(&mut cpu, &mut bus, a, nop());
    escreve_e_executa(&mut cpu, &mut bus, a, mfc2(14, 19));
    escreve_e_executa(&mut cpu, &mut bus, a, nop());
    escreve_e_executa(&mut cpu, &mut bus, a, cfc2(15, 31));
    escreve_e_executa(&mut cpu, &mut bus, a, nop());

    assert_eq!(cpu.regs[10], 100, "IR1 apos RTPS: VX=100");
    assert_eq!(cpu.regs[11], 50, "IR2 apos RTPS: VY=50");
    assert_eq!(cpu.regs[12], 256, "IR3 apos RTPS: 256");
    assert_eq!(cpu.regs[13], 0x0032_0064, "SXY2: SY2=50 SX2=100");
    assert_eq!(cpu.regs[14], 256, "SZ3 apos RTPS: 256");
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
    let a = 0x0000u32;

    ctc2_r8(&mut cpu, &mut bus, a, 0, 0x0000_1000);
    ctc2_r8(&mut cpu, &mut bus, a, 1, 0);
    ctc2_r8(&mut cpu, &mut bus, a, 2, 0x0000_1000);
    ctc2_r8(&mut cpu, &mut bus, a, 3, 0);
    ctc2_r8(&mut cpu, &mut bus, a, 4, 0x1000);

    ctc2_r8(&mut cpu, &mut bus, a, 5, 0);
    ctc2_r8(&mut cpu, &mut bus, a, 6, 0);
    ctc2_r8(&mut cpu, &mut bus, a, 7, 0x100);

    ctc2_r8(&mut cpu, &mut bus, a, 26, 0x0000_0100);
    ctc2_r8(&mut cpu, &mut bus, a, 24, 0);
    ctc2_r8(&mut cpu, &mut bus, a, 25, 0);
    ctc2_r8(&mut cpu, &mut bus, a, 27, 0);
    ctc2_r8(&mut cpu, &mut bus, a, 28, 0);

    mtc2_r8(&mut cpu, &mut bus, a, 0, 0x0032_0064);
    mtc2_r8(&mut cpu, &mut bus, a, 1, 0);

    mtc2_r8(&mut cpu, &mut bus, a, 2, 0x0064_00C8);
    mtc2_r8(&mut cpu, &mut bus, a, 3, 0);

    mtc2_r8(&mut cpu, &mut bus, a, 4, 0x0064_00C8);
    mtc2_r8(&mut cpu, &mut bus, a, 5, 0);

    escreve_e_executa(&mut cpu, &mut bus, a, cop2_cmd(0x30, true, false));
    escreve_e_executa(&mut cpu, &mut bus, a, nop());

    escreve_e_executa(&mut cpu, &mut bus, a, mfc2(10, 12));
    escreve_e_executa(&mut cpu, &mut bus, a, nop());
    escreve_e_executa(&mut cpu, &mut bus, a, mfc2(11, 13));
    escreve_e_executa(&mut cpu, &mut bus, a, nop());
    escreve_e_executa(&mut cpu, &mut bus, a, mfc2(12, 14));
    escreve_e_executa(&mut cpu, &mut bus, a, nop());
    escreve_e_executa(&mut cpu, &mut bus, a, mfc2(13, 16));
    escreve_e_executa(&mut cpu, &mut bus, a, nop());
    escreve_e_executa(&mut cpu, &mut bus, a, mfc2(14, 17));
    escreve_e_executa(&mut cpu, &mut bus, a, nop());
    escreve_e_executa(&mut cpu, &mut bus, a, mfc2(15, 18));
    escreve_e_executa(&mut cpu, &mut bus, a, nop());
    escreve_e_executa(&mut cpu, &mut bus, a, mfc2(16, 19));
    escreve_e_executa(&mut cpu, &mut bus, a, nop());
    escreve_e_executa(&mut cpu, &mut bus, a, cfc2(17, 31));
    escreve_e_executa(&mut cpu, &mut bus, a, nop());

    assert_eq!(
        cpu.regs[10], 0x0032_0064,
        "SXY0: resultado de V0 (SY=50, SX=100)"
    );
    assert_eq!(
        cpu.regs[11], 0x0064_00C8,
        "SXY1: resultado de V1 (SY=100, SX=200)"
    );
    assert_eq!(
        cpu.regs[12], 0x0064_00C8,
        "SXY2: resultado de V2 (SY=100, SX=200)"
    );
    assert_eq!(
        cpu.regs[13], 0,
        "SZ0: valor previo (0) — FIFO tem 4 estagios"
    );
    assert_eq!(cpu.regs[14], 0x100, "SZ1: SZ de V0 deslocado (256)");
    assert_eq!(cpu.regs[15], 0x100, "SZ2: SZ de V1 deslocado (256)");
    assert_eq!(cpu.regs[16], 0x100, "SZ3: SZ de V2 — ultimo (256)");
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
    let a = 0x0000u32;

    ctc2_r8(&mut cpu, &mut bus, a, 0, 0x0000_1000);
    ctc2_r8(&mut cpu, &mut bus, a, 1, 0);
    ctc2_r8(&mut cpu, &mut bus, a, 2, 0x0000_1000);
    ctc2_r8(&mut cpu, &mut bus, a, 3, 0);
    ctc2_r8(&mut cpu, &mut bus, a, 4, 0x1000);

    ctc2_r8(&mut cpu, &mut bus, a, 5, 0);
    ctc2_r8(&mut cpu, &mut bus, a, 6, 0);
    ctc2_r8(&mut cpu, &mut bus, a, 7, 0);

    ctc2_r8(&mut cpu, &mut bus, a, 26, 0);
    ctc2_r8(&mut cpu, &mut bus, a, 24, 0);
    ctc2_r8(&mut cpu, &mut bus, a, 25, 0);

    mtc2_r8(&mut cpu, &mut bus, a, 0, 5);
    mtc2_r8(&mut cpu, &mut bus, a, 1, 4);

    escreve_e_executa(&mut cpu, &mut bus, a, cop2_cmd(0x01, false, false));
    escreve_e_executa(&mut cpu, &mut bus, a, nop());

    escreve_e_executa(&mut cpu, &mut bus, a, mfc2(10, 9));
    escreve_e_executa(&mut cpu, &mut bus, a, nop());
    escreve_e_executa(&mut cpu, &mut bus, a, mfc2(11, 11));
    escreve_e_executa(&mut cpu, &mut bus, a, nop());
    escreve_e_executa(&mut cpu, &mut bus, a, mfc2(12, 19));
    escreve_e_executa(&mut cpu, &mut bus, a, nop());
    escreve_e_executa(&mut cpu, &mut bus, a, cfc2(13, 31));
    escreve_e_executa(&mut cpu, &mut bus, a, nop());

    assert_eq!(
        cpu.regs[10], 0x5000,
        "IR1 com sf=0: 4096*5 = 20480 = 0x5000"
    );
    assert_eq!(
        cpu.regs[11], 0x4000,
        "IR3 com sf=0: 4096*4 = 16384 = 0x4000"
    );
    assert_eq!(cpu.regs[12], 4, "SZ3 com sf=0: raw3>>12 = 0x4000>>12 = 4");
    assert_eq!(
        cpu.regs[13] & 0x7FFF_F800,
        0,
        "FLAG limpo apos RTPS sf=0 sem saturacao"
    );
}
