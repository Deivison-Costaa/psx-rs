use psx_core::bus::{Bios, Bus, Ram};
use psx_core::cdrom_bin_cue::{DiscLayout, parse_cue};
use psx_core::cpu::Cpu;
use std::io::Read;
use std::io::Write;

const RUNNER_MAX_STEPS: usize = 50_000_000;

fn run(cpu: &mut Cpu, bus: &mut Bus, max_steps: usize) -> usize {
    let mut steps = 0;
    while steps < max_steps {
        cpu.step(bus);
        steps += 1;
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
            "--bios" | "--exe" | "--disc" => {
                eprintln!("Erro: '{}' requer um caminho", args[i]);
                std::process::exit(1);
            }
            arg => {
                eprintln!("Erro: argumento desconhecido: '{}'", arg);
                std::process::exit(1);
            }
        }
    }

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

            let steps = run(&mut cpu, &mut bus, RUNNER_MAX_STEPS);

            let tty = bus.take_tty();
            if !tty.is_empty() {
                let _ = std::io::stdout().write_all(&tty);
                std::io::stdout().flush().ok();
            }

            eprintln!("Runner: {} passos, TTY: {} bytes", steps, tty.len());

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

            if let Some(disc_path) = disc_path {
                let (layout, bin_data) = load_disc(&disc_path);
                println!(
                    "DISCO: {} faixa(s), BIN: {}",
                    layout.tracks.len(),
                    layout.bin_path
                );
                let ram = Ram::new();
                let mut bus = Bus::new(ram, bios);
                bus.inject_disc(layout, bin_data);
                bus.cdrom_mut().insert_disc();
                return;
            }

            {
                use sha2::Digest;
                let hash = sha2::Sha256::digest(bios.raw());
                println!("BIOS: {} bytes, SHA-256: {:x}", bios.size(), hash);
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
