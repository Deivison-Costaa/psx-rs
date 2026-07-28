use psx_core::bus::BusRead;
use psx_core::cpu::Cpu;

mod support;
use support::asm::{bus_with_bios_empty, nop};

fn jal(addr: u32) -> u32 {
    (0x03 << 26) | ((addr >> 2) & 0x03FF_FFFF)
}

fn step_n(cpu: &mut Cpu, bus: &mut psx_core::bus::Bus, n: usize) {
    for _ in 0..n {
        cpu.step(bus);
    }
}

fn write_str(bus: &mut psx_core::bus::Bus, addr: u32, s: &str) {
    for (i, b) in s.bytes().enumerate() {
        bus.write8::<BusRead>(addr + i as u32, b);
    }
    bus.write8::<BusRead>(addr + s.len() as u32, 0);
}

fn setup_printf(
    cpu: &mut Cpu,
    bus: &mut psx_core::bus::Bus,
    fmt_str: &str,
    fmt_addr: u32,
    args: &[u32],
    sp_arg_base: u32,
) {
    cpu.pc = 0x0000_0000;
    cpu.regs[9] = 0x3F;
    cpu.regs[4] = fmt_addr;
    write_str(bus, fmt_addr, fmt_str);

    if !args.is_empty() {
        cpu.regs[5] = args[0];
    }
    if args.len() > 1 {
        cpu.regs[6] = args[1];
    }
    if args.len() > 2 {
        cpu.regs[7] = args[2];
    }
    for (i, &arg) in args.iter().enumerate().skip(3) {
        let addr = sp_arg_base.wrapping_add(0x10 + (i as u32 - 3) * 4);
        bus.write32::<BusRead>(addr, arg);
    }

    cpu.regs[29] = sp_arg_base;

    bus.write32::<BusRead>(0x0000_0000, jal(0x0000_00A0));
    bus.write32::<BusRead>(0x0000_0004, nop());
}

// ===== A1 — %d =====

#[test]
fn printf_d_signed_decimal() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();

    setup_printf(&mut cpu, &mut bus, "n=%d\n", 0x100, &[42], 0x1FF0);

    step_n(&mut cpu, &mut bus, 3);

    assert_eq!(
        bus.take_tty(),
        b"n=42\n",
        "A1: %%d com R5=42 deve gerar 'n=42\\n'"
    );
}

// ===== A2 — sinal de %d e %u =====

#[test]
fn printf_d_negativo() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();

    setup_printf(&mut cpu, &mut bus, "n=%d\n", 0x100, &[0xFFFF_FFFF], 0x1FF0);

    step_n(&mut cpu, &mut bus, 3);

    assert_eq!(
        bus.take_tty(),
        b"n=-1\n",
        "A2a: %%d com 0xFFFFFFFF deve gerar 'n=-1\\n'"
    );
}

#[test]
fn printf_u_unsigned_decimal() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();

    setup_printf(&mut cpu, &mut bus, "n=%u\n", 0x100, &[0xFFFF_FFFF], 0x1FF0);

    step_n(&mut cpu, &mut bus, 3);

    assert_eq!(
        bus.take_tty(),
        b"n=4294967295\n",
        "A2b: %%u com 0xFFFFFFFF deve gerar 'n=4294967295\\n'"
    );
}

// ===== A3 — %s, %c e %% =====

#[test]
fn printf_s_c_percent() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();

    write_str(&mut bus, 0x400, "ok");

    setup_printf(
        &mut cpu,
        &mut bus,
        "%s=%c 100%%\n",
        0x100,
        &[0x400, b'X' as u32],
        0x1FF0,
    );

    step_n(&mut cpu, &mut bus, 3);

    assert_eq!(
        bus.take_tty(),
        b"ok=X 100%\n",
        "A3: %%s=%%c 100%%%%\\n com R5→'ok' e R6='X' deve gerar 'ok=X 100%%\\n'"
    );
}

// ===== A4 — hex %x e %X =====

#[test]
fn printf_x_hexadecimal() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();

    setup_printf(
        &mut cpu,
        &mut bus,
        "%x %X\n",
        0x100,
        &[0xDEAD_BEEF, 0xDEAD_BEEF],
        0x1FF0,
    );

    step_n(&mut cpu, &mut bus, 3);

    assert_eq!(
        bus.take_tty(),
        b"deadbeef DEADBEEF\n",
        "A4: %%x %%X\\n com 0xDEADBEEF deve gerar 'deadbeef DEADBEEF\\n'"
    );
}

// ===== A5 — especificador fora do escopo sai literal =====

#[test]
fn printf_especificador_desconhecido_sai_literal() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();

    setup_printf(&mut cpu, &mut bus, "%o\n", 0x100, &[], 0x1FF0);

    step_n(&mut cpu, &mut bus, 3);

    assert_eq!(
        bus.take_tty(),
        b"%o\n",
        "A5: %%o nao suportado deve emitir a sequencia literal '%%o\\n'"
    );
}

// ===== A7 — argumentos da pilha =====

#[test]
fn printf_argumento_da_pilha() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();

    let sp = 0x1FF0;
    setup_printf(
        &mut cpu,
        &mut bus,
        "%d %d %d %d\n",
        0x100,
        &[10, 20, 30, 40],
        sp,
    );

    step_n(&mut cpu, &mut bus, 3);

    assert_eq!(
        bus.take_tty(),
        b"10 20 30 40\n",
        "A7: quarto argumento (R5,R6,R7,[SP+10h]) deve vir da pilha"
    );
}

// ===== A8 — printf (0x3F,0xA0) distinto de puts (0x3F,0xB0) =====

#[test]
fn printf_a0h_distinto_de_puts_b0h() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();

    cpu.pc = 0x0000_0000;
    cpu.regs[9] = 0x3F;
    cpu.regs[4] = 0x100;
    write_str(&mut bus, 0x100, "oi");

    bus.write32::<BusRead>(0x0000_0000, jal(0x0000_00B0));
    bus.write32::<BusRead>(0x0000_0004, nop());

    step_n(&mut cpu, &mut bus, 3);

    assert_eq!(
        bus.take_tty(),
        b"oi",
        "A8: B0h com R9=3Fh deve disparar puts, nao printf"
    );
}

// ===== A9 — string vazia =====

#[test]
fn printf_string_vazia() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();

    setup_printf(&mut cpu, &mut bus, "", 0x100, &[], 0x1FF0);

    step_n(&mut cpu, &mut bus, 3);

    assert!(
        bus.take_tty().is_empty(),
        "A9: string vazia nao deve emitir nada"
    );
}

// ===== A10 — percentual no final da string =====

#[test]
fn printf_percent_no_final_da_string() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();

    write_str(&mut bus, 0x100, "x%");

    cpu.pc = 0x0000_0000;
    cpu.regs[9] = 0x3F;
    cpu.regs[4] = 0x100;
    cpu.regs[29] = 0x1FF0;

    bus.write32::<BusRead>(0x0000_0000, jal(0x0000_00A0));
    bus.write32::<BusRead>(0x0000_0004, nop());

    step_n(&mut cpu, &mut bus, 3);

    assert_eq!(
        bus.take_tty(),
        b"x%",
        "A10: %% truncado no final da string deve emitir 'x%%'"
    );
}
