use psx_core::bus::{Bios, Bus, BusRead, Ram};
use psx_core::cdrom_bin_cue::{DiscLayout, parse_cue};
use psx_core::cpu::Cpu;
use std::collections::HashSet;
use std::io::Read;
use std::io::Write;

const RUNNER_MAX_STEPS: usize = 50_000_000;

fn run(cpu: &mut Cpu, bus: &mut Bus, max_steps: usize, trace_pcs: &HashSet<u32>) -> usize {
    let mut steps = 0;
    while steps < max_steps {
        cpu.step(bus);
        steps += 1;

        if !trace_pcs.is_empty() && trace_pcs.contains(&cpu.pc) {
            let instr = bus.read32::<BusRead>(cpu.pc);
            let _rs = ((instr >> 21) & 0x1F) as usize;
            let _rt = ((instr >> 16) & 0x1F) as usize;
            eprintln!(
                "trace pc=0x{:08X} step={} instr=0x{:08X} \
                 regs: a0($4)=0x{:08X} t1($9)=0x{:08X} s1($17)=0x{:08X} v0($2)=0x{:08X} t4($12)=0x{:08X} t5($13)=0x{:08X}",
                cpu.pc,
                steps,
                instr,
                cpu.regs[4],
                cpu.regs[9],
                cpu.regs[17],
                cpu.regs[2],
                cpu.regs[12],
                cpu.regs[13],
            );
            eprintln!(
                "     mem[t1*4]=0x{:08X} mem[s1*4]=0x{:08X}",
                bus.read32::<BusRead>(cpu.regs[9].wrapping_mul(4)),
                bus.read32::<BusRead>(cpu.regs[17].wrapping_mul(4)),
            );
            let _ = std::io::stderr().flush();
        }
    }
    steps
}

fn load_disc(disc_path: &str) -> (DiscLayout, Vec<u8>) {
    let cue_text = match std::fs::read_to_string(disc_path) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("Erro: nao foi possivel ler CUE '{}': {}", disc_path, e);
            std::process::exit(1);
        }
    };

    let layout = parse_cue(&cue_text);
    let cue_dir = std::path::Path::new(disc_path)
        .parent()
        .unwrap_or_else(|| std::path::Path::new("."));

    let bin_path = cue_dir.join(&layout.bin_path);
    let bin_data = match std::fs::read(&bin_path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!(
                "Erro: nao foi possivel ler BIN '{}': {}",
                bin_path.display(),
                e
            );
            std::process::exit(1);
        }
    };

    (layout, bin_data)
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() == 1 || (args.len() == 2 && args[1] == "--version") {
        println!("psx-cli {}", env!("CARGO_PKG_VERSION"));
        return;
    }

    let mut bios_arg: Option<String> = None;
    let mut exe_arg: Option<String> = None;
    let mut disc_arg: Option<String> = None;
    let mut max_steps: Option<usize> = None;
    let mut trace_pcs: HashSet<u32> = HashSet::new();
    let mut dump_mem: Vec<(u32, usize)> = Vec::new();
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--bios" if i + 1 < args.len() => {
                bios_arg = Some(args[i + 1].clone());
                i += 2;
            }
            "--exe" if i + 1 < args.len() => {
                exe_arg = Some(args[i + 1].clone());
                i += 2;
            }
            "--disc" if i + 1 < args.len() => {
                disc_arg = Some(args[i + 1].clone());
                i += 2;
            }
            "--max-steps" if i + 1 < args.len() => match args[i + 1].parse::<usize>() {
                Ok(n) => {
                    max_steps = Some(n);
                    i += 2;
                }
                Err(e) => {
                    eprintln!(
                        "Erro: '--max-steps' espera um numero, '{}': {}",
                        args[i + 1],
                        e
                    );
                    std::process::exit(1);
                }
            },
            "--trace-pcs" if i + 1 < args.len() => {
                for piece in args[i + 1].split(',') {
                    let piece = piece.trim();
                    match u32::from_str_radix(piece.trim_start_matches("0x"), 16) {
                        Ok(addr) => {
                            trace_pcs.insert(addr);
                        }
                        Err(e) => {
                            eprintln!(
                                "Erro: '--trace-pcs' espera enderecos hex, '{}': {}",
                                piece, e
                            );
                            std::process::exit(1);
                        }
                    }
                }
                i += 2;
            }
            "--bios" | "--exe" | "--disc" => {
                eprintln!("Erro: '{}' requer um caminho", args[i]);
                std::process::exit(1);
            }
            "--dump-mem" if i + 2 < args.len() => {
                let addr = match u32::from_str_radix(args[i + 1].trim_start_matches("0x"), 16) {
                    Ok(a) => a,
                    Err(e) => {
                        eprintln!(
                            "Erro: '--dump-mem' espera endereco hex, '{}': {}",
                            args[i + 1],
                            e
                        );
                        std::process::exit(1);
                    }
                };
                let len = match usize::from_str_radix(args[i + 2].trim_start_matches("0x"), 16) {
                    Ok(n) => n,
                    Err(e) => {
                        eprintln!(
                            "Erro: '--dump-mem' espera um comprimento hex, '{}': {}",
                            args[i + 2],
                            e
                        );
                        std::process::exit(1);
                    }
                };
                dump_mem.push((addr, len));
                i += 3;
            }
            "--max-steps" | "--trace-pcs" => {
                eprintln!("Erro: '{}' requer um valor", args[i]);
                std::process::exit(1);
            }
            arg => {
                eprintln!("Erro: argumento desconhecido: '{}'", arg);
                std::process::exit(1);
            }
        }
    }
    let max_steps = max_steps.unwrap_or(RUNNER_MAX_STEPS);

    if disc_arg.is_some() && bios_arg.is_none() {
        eprintln!("Erro: --disc requer --bios <caminho_da_BIOS>");
        std::process::exit(1);
    }

    if exe_arg.is_some() && bios_arg.is_none() {
        eprintln!(
            "Erro: --exe requer --bios <caminho_da_BIOS>. Use: psx-cli --bios <BIOS> --exe <PS-EXE>"
        );
        std::process::exit(1);
    }

    match (bios_arg.take(), exe_arg.take(), disc_arg.take()) {
        (Some(bios_path), Some(exe_path), disc_path) => {
            let bios_data = match std::fs::read(&bios_path) {
                Ok(d) => d,
                Err(e) => {
                    eprintln!("Erro: nao foi possivel ler BIOS '{}': {}", bios_path, e);
                    std::process::exit(1);
                }
            };
            let bios = match Bios::from_bytes(bios_data) {
                Ok(b) => b,
                Err(e) => {
                    eprintln!("Erro: BIOS invalida: {}", e);
                    std::process::exit(1);
                }
            };

            let exe_data = match std::fs::read(&exe_path) {
                Ok(d) => d,
                Err(e) => {
                    eprintln!("Erro: nao foi possivel ler EXE '{}': {}", exe_path, e);
                    std::process::exit(1);
                }
            };

            let ram = Ram::new();
            let mut bus = Bus::new(ram, bios);
            let mut cpu = Cpu::new();

            if let Some(disc_path) = disc_path {
                let (layout, bin_data) = load_disc(&disc_path);
                bus.inject_disc(layout, bin_data);
            }

            if let Err(e) = psx_core::psexe::load_psexe(&exe_data, &mut bus, &mut cpu) {
                eprintln!("Erro: falha ao carregar PS-EXE '{}': {}", exe_path, e);
                std::process::exit(1);
            }

            psx_core::psexe::install_return_stubs(&mut bus);

            let steps = run(&mut cpu, &mut bus, max_steps, &trace_pcs);

            let tty = bus.take_tty();
            if !tty.is_empty() {
                let _ = std::io::stdout().write_all(&tty);
                std::io::stdout().flush().ok();
            }

            eprintln!("Runner: {} passos, TTY: {} bytes", steps, tty.len());

            for &(addr, len) in &dump_mem {
                eprintln!("dump {:08X}:", addr);
                for off in (0..len).step_by(4) {
                    let word = bus.read32::<BusRead>(addr.wrapping_add(off as u32));
                    eprintln!("  {:08X}: {:08X}", addr.wrapping_add(off as u32), word);
                }
            }

            return;
        }
        (Some(bios_path), None, disc_path) => {
            let bios = {
                let mut file = match std::fs::File::open(&bios_path) {
                    Ok(f) => f,
                    Err(e) => {
                        eprintln!(
                            "Erro: nao foi possivel ler o arquivo BIOS '{}': {}",
                            bios_path, e
                        );
                        std::process::exit(1);
                    }
                };
                let mut data = Vec::new();
                if let Err(e) = file.read_to_end(&mut data) {
                    eprintln!("Erro: falha ao ler '{}': {}", bios_path, e);
                    std::process::exit(1);
                }
                match Bios::from_bytes(data) {
                    Ok(b) => b,
                    Err(e) => {
                        eprintln!("Erro: BIOS invalida: {}", e);
                        std::process::exit(1);
                    }
                }
            };

            let ram = Ram::new();
            let mut bus = Bus::new(ram, bios);
            let mut cpu = Cpu::new();

            if let Some(disc_path) = disc_path {
                let (layout, bin_data) = load_disc(&disc_path);
                bus.inject_disc(layout, bin_data);
                bus.cdrom_mut().insert_disc();
            }

            let steps = run(&mut cpu, &mut bus, max_steps, &trace_pcs);

            let tty = bus.take_tty();
            if !tty.is_empty() {
                let _ = std::io::stdout().write_all(&tty);
                std::io::stdout().flush().ok();
            }

            eprintln!("Runner: {} passos, TTY: {} bytes", steps, tty.len());

            for &(addr, len) in &dump_mem {
                eprintln!("dump {:08X}:", addr);
                for off in (0..len).step_by(4) {
                    let word = bus.read32::<BusRead>(addr.wrapping_add(off as u32));
                    eprintln!("  {:08X}: {:08X}", addr.wrapping_add(off as u32), word);
                }
            }

            return;
        }
        (bios_restored, _exe_restored, _disc_restored) => {
            bios_arg = bios_restored;
        }
    }

    if let Some(path) = bios_arg {
        let mut file = match std::fs::File::open(&path) {
            Ok(f) => f,
            Err(e) => {
                eprintln!(
                    "Erro: nao foi possivel ler o arquivo BIOS '{}': {}",
                    path, e
                );
                std::process::exit(1);
            }
        };
        let mut data = Vec::new();
        if let Err(e) = file.read_to_end(&mut data) {
            eprintln!("Erro: falha ao ler '{}': {}", path, e);
            std::process::exit(1);
        }
        match Bios::from_bytes(data) {
            Ok(bios) => {
                use sha2::Digest;
                let hash = sha2::Sha256::digest(bios.raw());
                println!("BIOS: {} bytes, SHA-256: {:x}", bios.size(), hash);
            }
            Err(e) => {
                eprintln!("Erro: BIOS invalida: {}", e);
                std::process::exit(1);
            }
        }
        return;
    }

    eprintln!("Uso: psx-cli [--version | --bios <caminho> [--exe <caminho>] [--disc <caminho>]]");
    std::process::exit(1);
}
