use psx_core::bus::BusRead;
use psx_core::cpu::Cpu;

// Sem kernel montado, o vetor de syscall e um `jr $ra` — e nesse ambiente que o hook de
// alto nivel de TTY emite. Com kernel real quem emite e a BIOS (ver cpu_tty_sem_duplicar).
const JR_RA_STUB: u32 = (31 << 21) | 0x08;

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
    bus.write32::<BusRead>(0xA0, JR_RA_STUB);
    bus.write32::<BusRead>(0xB0, JR_RA_STUB);
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

    bus.write32::<BusRead>(0xA0, JR_RA_STUB);
    bus.write32::<BusRead>(0xB0, JR_RA_STUB);
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

// ===== A11 — string de formato sem terminador (teto de 1 MiB) =====

#[test]
fn printf_fmt_sem_terminador_teto_1mib_evita_laco_infinito() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();

    for addr in (0x0000_0000u32..0x0020_0000u32).step_by(4) {
        bus.write32::<BusRead>(addr, 0x41414141u32);
    }

    bus.write32::<BusRead>(0x0000_0000, jal(0x0000_00A0));
    bus.write32::<BusRead>(0x0000_0004, nop());

    bus.write32::<BusRead>(0xA0, JR_RA_STUB);
    bus.write32::<BusRead>(0xB0, JR_RA_STUB);
    cpu.pc = 0x0000_0000;
    cpu.regs[9] = 0x3F;
    cpu.regs[4] = 0x100;
    cpu.regs[29] = 0x1FF0;

    step_n(&mut cpu, &mut bus, 3);

    let tty = bus.take_tty();
    assert_eq!(
        tty.len(),
        1_048_576,
        "A11: teto de 1 MiB — TTY deve ter exatamente 1_048_576 bytes"
    );
    assert!(
        tty.iter().all(|&b| b == b'A'),
        "A11: todos os bytes emitidos devem ser 'A'"
    );
}

// ===== A10 — percentual no final da string =====

#[test]
fn printf_percent_no_final_da_string() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();

    write_str(&mut bus, 0x100, "x%");

    bus.write32::<BusRead>(0xA0, JR_RA_STUB);
    bus.write32::<BusRead>(0xB0, JR_RA_STUB);
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

#[test]
fn printf_08x_zero_pad_hex_minusculo() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();

    setup_printf(&mut cpu, &mut bus, "v=%08x\n", 0x100, &[0x1F], 0x1FF0);

    step_n(&mut cpu, &mut bus, 3);

    assert_eq!(
        bus.take_tty(),
        b"v=0000001f\n",
        "%%08x com 0x1F deve gerar '0000001f' (zero-pad, 8 digitos)"
    );
}

#[test]
fn printf_08x_maiusculo_zero_pad_hex() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();

    setup_printf(&mut cpu, &mut bus, "v=%08X\n", 0x100, &[0x1F], 0x1FF0);

    step_n(&mut cpu, &mut bus, 3);

    assert_eq!(
        bus.take_tty(),
        b"v=0000001F\n",
        "%%08X com 0x1F deve gerar '0000001F' (zero-pad maiusculo)"
    );
}

#[test]
fn printf_2d_largura_minima_dois() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();

    setup_printf(&mut cpu, &mut bus, "v=%2d\n", 0x100, &[7], 0x1FF0);

    step_n(&mut cpu, &mut bus, 3);

    assert_eq!(
        bus.take_tty(),
        b"v= 7\n",
        "%%2d com 7 deve gerar ' 7' (largura minima 2, alinhado a direita)"
    );
}

#[test]
fn printf_4d_largura_quatro() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();

    setup_printf(&mut cpu, &mut bus, "v=%4d\n", 0x100, &[42], 0x1FF0);

    step_n(&mut cpu, &mut bus, 3);

    assert_eq!(
        bus.take_tty(),
        b"v=  42\n",
        "%%4d com 42 deve gerar '  42' (largura minima 4)"
    );
}

#[test]
fn printf_3d_nao_trunca_valor_maior_que_largura() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();

    setup_printf(&mut cpu, &mut bus, "v=%3d\n", 0x100, &[12345], 0x1FF0);

    step_n(&mut cpu, &mut bus, 3);

    assert_eq!(
        bus.take_tty(),
        b"v=12345\n",
        "%%3d com 12345 deve gerar '12345' (largura minima, nao trunca)"
    );
}

#[test]
fn printf_04x_zero_pad_largura_4() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();

    setup_printf(&mut cpu, &mut bus, "v=%04x\n", 0x100, &[0xAB], 0x1FF0);

    step_n(&mut cpu, &mut bus, 3);

    assert_eq!(
        bus.take_tty(),
        b"v=00ab\n",
        "%%04x com 0xAB deve gerar '00ab' (zero-pad, largura 4)"
    );
}

#[test]
fn printf_4u_largura_unsigned() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();

    setup_printf(&mut cpu, &mut bus, "v=%4u\n", 0x100, &[7], 0x1FF0);

    step_n(&mut cpu, &mut bus, 3);

    assert_eq!(
        bus.take_tty(),
        b"v=   7\n",
        "%%4u com 7 deve gerar '   7' (largura minima 4, sem zero-pad)"
    );
}

#[test]
fn printf_0_flag_sem_largura_nao_altera_saida() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();

    setup_printf(&mut cpu, &mut bus, "v=%0d\n", 0x100, &[42], 0x1FF0);

    step_n(&mut cpu, &mut bus, 3);

    assert_eq!(
        bus.take_tty(),
        b"v=42\n",
        "%%0d sem largura deve gerar '42' (flag sem efeito visivel)"
    );
}
