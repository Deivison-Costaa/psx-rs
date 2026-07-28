use psx_core::bus::{Bios, Bus, BusRead, Ram};
use psx_core::cpu::Cpu;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn encode_j(opcode: u32, target_addr: u32) -> u32 {
    (opcode << 26) | ((target_addr >> 2) & 0x03FF_FFFF)
}

fn nop() -> u32 {
    0x0000_0000
}

fn ori(rt: u32, rs: u32, imm: u16) -> u32 {
    (0x0D << 26) | (rs << 21) | (rt << 16) | (imm as u32)
}

struct PsexeConfig {
    dest_addr: u32,
    initial_pc: u32,
    initial_gp: u32,
    sp_fp_base: u32,
    sp_fp_offset: u32,
    bss_addr: u32,
    bss_size: u32,
}

fn build_ps_exe(code: &[u32], cfg: &PsexeConfig) -> Vec<u8> {
    let header_size = 0x800;
    let body_words = code.len();
    let body_size = ((body_words * 4) + 0x7FF) & !0x7FF;

    let mut data = vec![0u8; header_size + body_size];
    data[0..8].copy_from_slice(b"PS-X EXE");

    let mut offset = |pos: usize, val: u32| {
        data[pos..pos + 4].copy_from_slice(&val.to_le_bytes());
    };

    offset(0x10, cfg.initial_pc);
    offset(0x14, cfg.initial_gp);
    offset(0x18, cfg.dest_addr);
    offset(0x1C, body_size as u32);
    offset(0x20, 0);
    offset(0x24, 0);
    offset(0x28, cfg.bss_addr);
    offset(0x2C, cfg.bss_size);
    offset(0x30, cfg.sp_fp_base);
    offset(0x34, cfg.sp_fp_offset);

    for (i, &word) in code.iter().enumerate() {
        let pos = header_size + i * 4;
        data[pos..pos + 4].copy_from_slice(&word.to_le_bytes());
    }

    data
}

fn bus_with_bios_empty() -> Bus {
    let ram = Ram::new();
    let bios_bytes = vec![0u8; 0x80000];
    let bios = Bios::from_bytes(bios_bytes).expect("BIOS de teste vazia");
    Bus::new(ram, bios)
}

fn run(cpu: &mut Cpu, bus: &mut Bus, max_steps: usize) -> usize {
    let mut steps = 0;
    while steps < max_steps {
        cpu.step(bus);
        steps += 1;
    }
    steps
}

fn bins_dir() -> PathBuf {
    let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    d.push("tests");
    d.push("bins");
    d
}

// ===== A1 — Sideload de EXE minimo com JMP $ =====

#[test]
fn sideload_exe_minimo_jmp_self() {
    let code_addr: u32 = 0x8000_0000;
    let jmp_self = encode_j(0x02, code_addr);
    let code = [jmp_self, nop()];

    let cfg = PsexeConfig {
        dest_addr: code_addr,
        initial_pc: code_addr,
        initial_gp: 0,
        sp_fp_base: 0x801F_FFF0,
        sp_fp_offset: 0,
        bss_addr: 0,
        bss_size: 0,
    };
    let exe_data = build_ps_exe(&code, &cfg);

    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();

    let result = psx_core::psexe::load_psexe(&exe_data, &mut bus, &mut cpu);
    assert!(
        result.is_ok(),
        "A1: load_psexe deve retornar Ok; {:?}",
        result.err()
    );

    assert_eq!(cpu.pc, code_addr, "A1: PC inicial deve ser dest_addr");

    let steps = run(&mut cpu, &mut bus, 20);

    assert_eq!(steps, 20, "A1: deve executar os 20 passos ate o step-limit");
    assert_eq!(
        cpu.pc, code_addr,
        "A1: apos JMP $, PC deve estar no endereco de self-loop"
    );
}

// ===== A2 — print 'OK' via TTY =====

#[test]
fn print_ok_via_tty() {
    let code_addr: u32 = 0x8000_0000;
    let tty_addr: u32 = 0x0000_00A0;

    let jmp_self_addr = code_addr + 8 * 4;

    let code = [
        ori(9, 0, 0x3C),
        ori(4, 0, b'O' as u16),
        encode_j(0x03, tty_addr),
        nop(),
        ori(9, 0, 0x3C),
        ori(4, 0, b'K' as u16),
        encode_j(0x03, tty_addr),
        nop(),
        encode_j(0x02, jmp_self_addr),
        nop(),
    ];

    let cfg = PsexeConfig {
        dest_addr: code_addr,
        initial_pc: code_addr,
        initial_gp: 0,
        sp_fp_base: 0x801F_FFF0,
        sp_fp_offset: 0,
        bss_addr: 0,
        bss_size: 0,
    };
    let exe_data = build_ps_exe(&code, &cfg);

    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();

    let result = psx_core::psexe::load_psexe(&exe_data, &mut bus, &mut cpu);
    assert!(
        result.is_ok(),
        "A2: load_psexe deve retornar Ok; {:?}",
        result.err()
    );

    psx_core::psexe::install_return_stubs(&mut bus);

    run(&mut cpu, &mut bus, 100);

    assert_eq!(bus.take_tty(), b"OK", "A2: take_tty() deve devolver b'OK'");
}

// ===== A3 — Zero-fill do BSS =====

#[test]
fn zerofill_bss() {
    let code_addr: u32 = 0x8000_0000;
    let bss_addr: u32 = 0x8000_1000;
    let bss_size: u32 = 8;

    let code = [nop()];

    let cfg = PsexeConfig {
        dest_addr: code_addr,
        initial_pc: code_addr,
        initial_gp: 0,
        sp_fp_base: 0x801F_FFF0,
        sp_fp_offset: 0,
        bss_addr,
        bss_size,
    };
    let exe_data = build_ps_exe(&code, &cfg);

    let mut bus = bus_with_bios_empty();

    bus.write32::<BusRead>(bss_addr, 0xDEAD_BEEF);
    bus.write32::<BusRead>(bss_addr.wrapping_add(4), 0xCAFE_BABE);

    let mut cpu = Cpu::new();

    let result = psx_core::psexe::load_psexe(&exe_data, &mut bus, &mut cpu);
    assert!(
        result.is_ok(),
        "A3: load_psexe deve retornar Ok; {:?}",
        result.err()
    );

    assert_eq!(
        bus.read32::<BusRead>(bss_addr),
        0,
        "A3: primeiros 4 bytes do BSS devem ser zero"
    );
    assert_eq!(
        bus.read32::<BusRead>(bss_addr.wrapping_add(4)),
        0,
        "A3: segundos 4 bytes do BSS devem ser zero"
    );
}

// ===== SP/FP base=0 nao altera os registradores =====

#[test]
fn sp_fp_base_zero_nao_altera_registradores() {
    let code_addr: u32 = 0x8000_0000;

    let cfg = PsexeConfig {
        dest_addr: code_addr,
        initial_pc: code_addr,
        initial_gp: 0,
        sp_fp_base: 0,
        sp_fp_offset: 0,
        bss_addr: 0,
        bss_size: 0,
    };
    let exe_data = build_ps_exe(&[nop()], &cfg);

    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    cpu.regs[29] = 0xDEAD_BEEF;
    cpu.regs[30] = 0xCAFE_BABE;

    let result = psx_core::psexe::load_psexe(&exe_data, &mut bus, &mut cpu);
    assert!(result.is_ok(), "SP/FP base=0: load_psexe deve retornar Ok");

    assert_eq!(
        cpu.regs[29], 0xDEAD_BEEF,
        "SP/FP base=0: SP (R29) nao deve ser alterado"
    );
    assert_eq!(
        cpu.regs[30], 0xCAFE_BABE,
        "SP/FP base=0: FP (R30) nao deve ser alterado"
    );
}

// ===== A4 — psxtest_cpu sideload smoke =====

fn exe_dir() -> PathBuf {
    let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    d.push("..");
    d.push("..");
    d.push("tests");
    d.push("exes");
    d.push("amidog");
    d.push("cpu");
    d
}

#[test]
fn psxtest_cpu_sideload_executa_sem_panico() {
    // TTY depende de A(3Fh) printf, ainda nao implementado (ROADMAP 1.11b)
    let exe_path = exe_dir().join("psxtest_cpu.exe");

    if !exe_path.exists() {
        eprintln!("A4: psxtest_cpu.exe nao encontrado — pule com scripts/fetch-test-exes.ps1");
        return;
    }

    let exe_data = fs::read(&exe_path).expect("ler psxtest_cpu.exe");

    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();

    let result = psx_core::psexe::load_psexe(&exe_data, &mut bus, &mut cpu);
    assert!(
        result.is_ok(),
        "A4: load_psexe do psxtest_cpu deve retornar Ok; {:?}",
        result.err()
    );

    psx_core::psexe::install_return_stubs(&mut bus);

    run(&mut cpu, &mut bus, 50_000_000);

    let tty = bus.take_tty();
    assert!(
        !tty.is_empty(),
        "A4: TTY nao deve estar vazio — printf A(3Fh) deve estar implementado (ROADMAP 1.11b)"
    );
    let tty_str = String::from_utf8_lossy(&tty);
    assert!(
        tty_str.contains("args: 0"),
        "A4: TTY deve conter 'args: 0' apos printf; TTY={:?}",
        tty_str
    );
    assert!(
        cpu.pc >= 0x8000_0000 && cpu.pc <= 0x807F_FFFF,
        "A4: PC {:#x} deve estar em KSEG0 apos execucao; TTY={} bytes",
        cpu.pc,
        tty.len()
    );
    eprintln!(
        "A4: psxtest_cpu PC={:#x} TTY='{}' (printf OK)",
        cpu.pc,
        tty_str.trim_end()
    );
}

// ===== A5 — --bios ausente ou BIOS invalida → erro claro =====

#[test]
fn exe_sem_bios_erro() {
    let _ = fs::create_dir_all(bins_dir());

    let exe_path = bins_dir().join("min.psexe");
    let jmp_self = encode_j(0x02, 0x8000_0000);
    let code = [jmp_self, nop()];
    let cfg = PsexeConfig {
        dest_addr: 0x8000_0000,
        initial_pc: 0x8000_0000,
        initial_gp: 0,
        sp_fp_base: 0,
        sp_fp_offset: 0,
        bss_addr: 0,
        bss_size: 0,
    };
    let exe_data = build_ps_exe(&code, &cfg);
    fs::write(&exe_path, &exe_data).expect("escrever EXE sintetico");

    let bin = env!("CARGO_BIN_EXE_psx-cli");

    let output = Command::new(bin)
        .arg("--exe")
        .arg(&exe_path)
        .output()
        .expect("executar psx-cli --exe");

    assert!(
        !output.status.success(),
        "A5: --exe sem --bios deve sair com codigo != 0"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stderr.is_empty(),
        "A5: deve imprimir mensagem de erro no stderr"
    );

    let _ = fs::remove_file(&exe_path);
}

#[test]
fn bios_invalida_com_exe_erro() {
    let _ = fs::create_dir_all(bins_dir());

    let bios_path = bins_dir().join("bios_bad.psexe");
    fs::write(&bios_path, b"invalida").expect("escrever BIOS invalida");

    let exe_path = bins_dir().join("min2.psexe");
    let cfg = PsexeConfig {
        dest_addr: 0x8000_0000,
        initial_pc: 0x8000_0000,
        initial_gp: 0,
        sp_fp_base: 0,
        sp_fp_offset: 0,
        bss_addr: 0,
        bss_size: 0,
    };
    let exe_data = build_ps_exe(&[nop()], &cfg);
    fs::write(&exe_path, &exe_data).expect("escrever EXE sintetico");

    let bin = env!("CARGO_BIN_EXE_psx-cli");

    let output = Command::new(bin)
        .arg("--bios")
        .arg(&bios_path)
        .arg("--exe")
        .arg(&exe_path)
        .output()
        .expect("executar psx-cli com BIOS invalida");

    assert!(
        !output.status.success(),
        "A5: BIOS invalida deve sair com codigo != 0"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stderr.is_empty(),
        "A5: deve imprimir mensagem de erro no stderr"
    );

    let _ = fs::remove_file(&bios_path);
    let _ = fs::remove_file(&exe_path);
}

// ===== F4 — load_psexe rejeita corpo truncado =====

#[test]
fn load_psexe_rejeita_corpo_nao_multiplo_de_4() {
    let code_addr: u32 = 0x8000_0000;
    let code = [nop()];

    let cfg = PsexeConfig {
        dest_addr: code_addr,
        initial_pc: code_addr,
        initial_gp: 0,
        sp_fp_base: 0,
        sp_fp_offset: 0,
        bss_addr: 0,
        bss_size: 0,
    };
    let mut exe_data = build_ps_exe(&code, &cfg);
    exe_data.resize(0x800 + 5, 0);

    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();

    let result = psx_core::psexe::load_psexe(&exe_data, &mut bus, &mut cpu);
    assert!(
        result.is_err(),
        "F4: corpo nao multiplo de 4 deve retornar Err"
    );
}

// ===== F6 — argumento desconhecido rejeitado =====

#[test]
fn argumento_desconhecido_rejeitado() {
    let bin = env!("CARGO_BIN_EXE_psx-cli");

    let output = Command::new(bin)
        .arg("--bios-file")
        .arg("x")
        .output()
        .expect("executar psx-cli com argumento desconhecido");

    assert!(
        !output.status.success(),
        "F6: argumento desconhecido deve sair com codigo != 0"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("desconhecido"),
        "F6: stderr deve mencionar argumento desconhecido; stderr={:?}",
        stderr
    );
}
