use psx_core::bus::BusRead;
use psx_core::cpu::Cpu;

// Sem kernel montado, o vetor de syscall e um `jr $ra` — e nesse ambiente que o hook de
// alto nivel de TTY emite. Com kernel real quem emite e a BIOS (ver cpu_tty_sem_duplicar).
const JR_RA_STUB: u32 = (31 << 21) | 0x08;

mod support;
use support::asm::{bus_with_bios_empty, encode_j_type, nop};

fn jal(addr: u32) -> u32 {
    encode_j_type(0x03, addr >> 2)
}

fn jalr(rs: u32, rd: u32) -> u32 {
    (rs << 21) | (rd << 11) | 0x09
}

fn lw(rt: u32, rs: u32, imm: i16) -> u32 {
    (0x23 << 26) | (rs << 21) | (rt << 16) | (imm as u16 as u32)
}

fn step_n(cpu: &mut Cpu, bus: &mut psx_core::bus::Bus, n: usize) {
    for _ in 0..n {
        cpu.step(bus);
    }
}

// ===== D1 — putchar por A0h =====

#[test]
fn putchar_por_a0h() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    bus.write32::<BusRead>(0xA0, JR_RA_STUB);
    bus.write32::<BusRead>(0xB0, JR_RA_STUB);
    cpu.pc = 0x0000_0000;
    cpu.regs[9] = 0x3C;
    cpu.regs[4] = b'X' as u32;

    bus.write32::<BusRead>(0x0000_0000, jal(0x0000_00A0));
    bus.write32::<BusRead>(0x0000_0004, nop());

    step_n(&mut cpu, &mut bus, 3);

    assert_eq!(
        bus.take_tty(),
        b"X",
        "D1: putchar via A0h com R9=3Ch deve emitir 'X'"
    );
}

// ===== D2 — putchar por B0h usa outro número =====

#[test]
fn putchar_por_b0h() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    bus.write32::<BusRead>(0xA0, JR_RA_STUB);
    bus.write32::<BusRead>(0xB0, JR_RA_STUB);
    cpu.pc = 0x0000_0000;
    cpu.regs[9] = 0x3D;
    cpu.regs[4] = b'Y' as u32;

    bus.write32::<BusRead>(0x0000_0000, jal(0x0000_00B0));
    bus.write32::<BusRead>(0x0000_0004, nop());

    step_n(&mut cpu, &mut bus, 3);

    assert_eq!(
        bus.take_tty(),
        b"Y",
        "D2a: putchar via B0h com R9=3Dh deve emitir 'Y'"
    );
}

#[test]
fn b0h_com_numero_de_a0h_ignorado() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    bus.write32::<BusRead>(0xA0, JR_RA_STUB);
    bus.write32::<BusRead>(0xB0, JR_RA_STUB);
    cpu.pc = 0x0000_0000;
    cpu.regs[9] = 0x3C;
    cpu.regs[4] = b'Z' as u32;

    bus.write32::<BusRead>(0x0000_0000, jal(0x0000_00B0));
    bus.write32::<BusRead>(0x0000_0004, nop());

    step_n(&mut cpu, &mut bus, 3);

    assert!(
        bus.take_tty().is_empty(),
        "D2b: B0h com R9=3Ch (numero de A0h) nao deve emitir nada"
    );
}

// ===== D3 — puts lê até o 00h =====

#[test]
fn puts_le_ate_zero() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    bus.write32::<BusRead>(0xA0, JR_RA_STUB);
    bus.write32::<BusRead>(0xB0, JR_RA_STUB);
    cpu.pc = 0x0000_0000;
    cpu.regs[9] = 0x3E;
    cpu.regs[4] = 0x100;

    bus.write8::<BusRead>(0x100, b'o');
    bus.write8::<BusRead>(0x101, b'i');
    bus.write8::<BusRead>(0x102, 0x00);

    bus.write32::<BusRead>(0x0000_0000, jal(0x0000_00A0));
    bus.write32::<BusRead>(0x0000_0004, nop());

    step_n(&mut cpu, &mut bus, 3);

    assert_eq!(
        bus.take_tty(),
        b"oi",
        "D3: puts deve ler ate o terminador 00h, sem emiti-lo"
    );
}

// ===== D4 — puts(0) emite <NULL> =====

#[test]
fn puts_null_emite_texto_null() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    bus.write32::<BusRead>(0xA0, JR_RA_STUB);
    bus.write32::<BusRead>(0xB0, JR_RA_STUB);
    cpu.pc = 0x0000_0000;
    cpu.regs[9] = 0x3E;
    cpu.regs[4] = 0x0000_0000;

    bus.write32::<BusRead>(0x0000_0000, jal(0x0000_00A0));
    bus.write32::<BusRead>(0x0000_0004, nop());

    step_n(&mut cpu, &mut bus, 3);

    assert_eq!(
        bus.take_tty(),
        b"<NULL>",
        "D4: puts(0) deve emitir '<NULL>' sem CR/LF"
    );
}

// ===== D5 — Número desconhecido é ignorado =====

#[test]
fn numero_desconhecido_ignorado_sem_panico() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    bus.write32::<BusRead>(0xA0, JR_RA_STUB);
    bus.write32::<BusRead>(0xB0, JR_RA_STUB);
    cpu.pc = 0x0000_0000;
    cpu.regs[9] = 0xFF;
    cpu.regs[4] = 0x42;

    bus.write32::<BusRead>(0x0000_0000, jal(0x0000_00A0));
    bus.write32::<BusRead>(0x0000_0004, nop());

    step_n(&mut cpu, &mut bus, 3);

    assert!(
        bus.take_tty().is_empty(),
        "D5: R9=FFh desconhecido deve ser ignorado, buffer vazio"
    );
    assert_eq!(
        cpu.pc, 0x0000_00A4,
        "D5: execucao deve continuar apos o hook"
    );
    assert_eq!(
        cpu.regs[9], 0xFF,
        "D5: registradores nao devem ser alterados pelo hook"
    );
    assert_eq!(
        cpu.regs[4], 0x42,
        "D5: registradores nao devem ser alterados pelo hook"
    );
}

// ===== D6 — Espelho KSEG0 =====

#[test]
fn espelho_kseg0_dispara_hook() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    bus.write32::<BusRead>(0xA0, JR_RA_STUB);
    bus.write32::<BusRead>(0xB0, JR_RA_STUB);
    cpu.pc = 0x0000_0000;
    cpu.regs[9] = 0x3C;
    cpu.regs[4] = b'K' as u32;
    cpu.regs[5] = 0x8000_00A0;

    bus.write32::<BusRead>(0x0000_0000, jalr(5, 31));
    bus.write32::<BusRead>(0x0000_0004, nop());

    step_n(&mut cpu, &mut bus, 3);

    assert_eq!(
        bus.take_tty(),
        b"K",
        "D6: salto para 0x800000A0 (fisico = 0xA0) deve disparar o hook"
    );
}

// ===== F1 — puts exige (0x3E,0xA0)|(0x3F,0xB0) estrito =====

#[test]
fn puts_b0h_com_numero_de_a0h_ignorado() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    bus.write32::<BusRead>(0xA0, JR_RA_STUB);
    bus.write32::<BusRead>(0xB0, JR_RA_STUB);
    cpu.pc = 0x0000_0000;
    cpu.regs[9] = 0x3E;
    cpu.regs[4] = 0x100;

    bus.write8::<BusRead>(0x100, b'h');
    bus.write8::<BusRead>(0x101, b'i');
    bus.write8::<BusRead>(0x102, 0x00);

    bus.write32::<BusRead>(0x0000_0000, jal(0x0000_00B0));
    bus.write32::<BusRead>(0x0000_0004, nop());

    step_n(&mut cpu, &mut bus, 3);

    assert!(
        bus.take_tty().is_empty(),
        "F1: puts via B0h com R9=3Eh (numero de A0h) nao deve emitir nada"
    );
}

// ===== F2 — hook deve consultar load delay pendente =====

#[test]
fn putchar_com_lw_no_delay_slot_do_jal() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    bus.write32::<BusRead>(0xA0, JR_RA_STUB);
    bus.write32::<BusRead>(0xB0, JR_RA_STUB);
    cpu.pc = 0x0000_0000;
    cpu.regs[9] = 0x3C;
    cpu.regs[16] = 0x200;

    bus.write32::<BusRead>(0x200, b'Z' as u32);

    bus.write32::<BusRead>(0x0000_0000, jal(0x0000_00A0));
    bus.write32::<BusRead>(0x0000_0004, lw(4, 16, 0));

    step_n(&mut cpu, &mut bus, 3);

    assert_eq!(
        bus.take_tty(),
        b"Z",
        "F2: lw $a0 no delay slot do jal deve carregar antes do hook ler R4"
    );
}
