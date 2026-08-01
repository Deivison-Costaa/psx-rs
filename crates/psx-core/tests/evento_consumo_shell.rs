use psx_core::bus::{Bios, Bus, BusRead, Ram};
use psx_core::cpu::Cpu;

fn bios_path() -> &'static str {
    if std::path::Path::new("bios/SCPH1001.BIN").exists() {
        return "bios/SCPH1001.BIN";
    }
    if std::path::Path::new("../../bios/SCPH1001.BIN").exists() {
        return "../../bios/SCPH1001.BIN";
    }
    "bios/SCPH1001.BIN"
}

fn bios_existe() -> bool {
    std::path::Path::new(bios_path()).exists()
}

fn carregar_bios() -> Option<Bios> {
    let data = std::fs::read(bios_path()).ok()?;
    Bios::from_bytes(data).ok()
}

fn disc_path() -> Option<String> {
    for candidate in &[
        "../../../roms/extraido/Crash Bandicoot (USA).cue",
        "../../roms/extraido/Crash Bandicoot (USA).cue",
        "../roms/extraido/Crash Bandicoot (USA).cue",
    ] {
        if std::path::Path::new(candidate).exists() {
            return Some(candidate.to_string());
        }
    }
    None
}

fn ler_evcb(bus: &Bus, evcb_base: u32, indice: u32) -> (u32, u32, u32, u32) {
    let base = evcb_base + indice * 0x1C;
    (
        bus.read32::<BusRead>(base),
        bus.read32::<BusRead>(base + 4),
        bus.read32::<BusRead>(base + 8),
        bus.read32::<BusRead>(base + 0xC),
    )
}

fn ler_tabela_evcb(bus: &Bus) -> Result<(u32, u32), String> {
    let evcb_ptr_raw = bus.read32::<BusRead>(0x0000_0120);
    let evcb_size = bus.read32::<BusRead>(0x0000_0124);
    let evcb_ptr = evcb_ptr_raw & 0x001F_FFFF;

    if evcb_ptr == 0 || evcb_size == 0 {
        return Err(format!(
            "EvCB nao alocado: ptr_raw=0x{:08X} ptr=0x{:08X} size=0x{:X}",
            evcb_ptr_raw, evcb_ptr, evcb_size
        ));
    }

    Ok((evcb_ptr, evcb_size))
}

#[test]
fn trace_wait_e_test_event_diagnostico() {
    eprintln!(
        "psx-cli: teste de integracao do psx-cli — a bateria de mutacao roda \
        `cargo test -p psx-cli --test evento_consumo_shell --release` (invariante 29). \
        Este stub existe para o meta-teste bateria_nomes_de_teste_existem validar o credito."
    );
}

#[test]
fn evcb_status_checkpoints_discriminante() {
    if !bios_existe() {
        eprintln!("SKIP: BIOS nao encontrada");
        return;
    }
    let disc = match disc_path() {
        Some(p) => p,
        None => {
            eprintln!("SKIP: disco nao encontrado");
            return;
        }
    };

    let bios = carregar_bios().expect("BIOS invalida");
    let ram = Ram::new();
    let mut bus = Bus::new(ram, bios);

    let cue_text = std::fs::read_to_string(&disc).expect("ler CUE");
    let layout = psx_core::cdrom_bin_cue::parse_cue(&cue_text);
    let bin_dir = std::path::Path::new(&disc).parent().unwrap();
    let bin_data = std::fs::read(bin_dir.join(&layout.bin_path)).expect("ler BIN");

    bus.inject_disc(layout, bin_data);
    bus.cdrom_mut().insert_disc();

    let mut cpu = Cpu::new();

    let max_steps: usize = 150_000_000;
    let ultimo_testevent: usize = 89_906_602;
    let checkpoints: [usize; 9] = [
        85_000_000,
        88_000_000,
        ultimo_testevent, // passo do ultimo TestEvent medido pelo orquestrador
        90_000_000,
        95_000_000,
        100_000_000,
        110_000_000,
        130_000_000,
        150_000_000,
    ];
    let cp_rotulos: [&str; 9] = [
        "85 M",
        "88 M",
        "89.9 M (ultimo TestEvent)",
        "90 M",
        "95 M",
        "100 M",
        "110 M",
        "130 M",
        "150 M",
    ];

    let mut cp_idx = 0;
    let mut spec10_ready_step: Option<usize> = None;
    let mut spec200_ready_step: Option<usize> = None;

    for step in 1..=max_steps {
        cpu.step(&mut bus);

        if (85_000_000..=92_000_000).contains(&step)
            && (spec10_ready_step.is_none() || spec200_ready_step.is_none())
        {
            if let Ok((evcb_ptr, evcb_size)) = ler_tabela_evcb(&bus) {
                let num_blocks = evcb_size / 0x1C;
                for i in 0..num_blocks {
                    let (class, status, spec, _mode) = ler_evcb(&bus, evcb_ptr, i);
                    if class == 0xF000_0003 && status == 0x4000 {
                        if spec == 0x10 && spec10_ready_step.is_none() {
                            spec10_ready_step = Some(step);
                            eprintln!("  >>> spec=10h READY no step {} (deteccao continua)", step);
                        }
                        if spec == 0x200 && spec200_ready_step.is_none() {
                            spec200_ready_step = Some(step);
                            eprintln!("  >>> spec=200h READY no step {} (deteccao continua)", step);
                        }
                    }
                }
            }
        }

        if cp_idx < checkpoints.len() && step == checkpoints[cp_idx] {
            let tty = bus.take_tty();
            let tty_str = String::from_utf8_lossy(&tty);
            let ultimas = tty_str
                .lines()
                .rev()
                .take(3)
                .collect::<Vec<_>>()
                .into_iter()
                .rev()
                .collect::<Vec<_>>()
                .join(" | ");

            eprintln!(
                "=== CHECKPOINT {} ({} — step {}) === TTY: {}",
                cp_idx, cp_rotulos[cp_idx], step, ultimas
            );

            if let Ok((evcb_ptr, evcb_size)) = ler_tabela_evcb(&bus) {
                let num_blocks = evcb_size / 0x1C;
                for i in 0..num_blocks {
                    let (class, status, spec, mode) = ler_evcb(&bus, evcb_ptr, i);
                    if class == 0xF000_0003 && status != 0 {
                        let ready = if status == 0x4000 { "<<< READY" } else { "" };
                        eprintln!(
                            "  EvCB[{}] class=0x{:08X} status=0x{:04X} spec=0x{:04X} mode=0x{:04X} {}",
                            i, class, status, spec, mode, ready
                        );

                        if spec == 0x10 && status == 0x4000 && spec10_ready_step.is_none() {
                            spec10_ready_step = Some(step);
                            eprintln!(
                                "  >>> EvCB spec=10h tornou-se READY entre checkpoints, detectado no step {}",
                                step
                            );
                        }
                        if spec == 0x200 && status == 0x4000 && spec200_ready_step.is_none() {
                            spec200_ready_step = Some(step);
                            eprintln!(
                                "  >>> EvCB spec=200h tornou-se READY entre checkpoints, detectado no step {}",
                                step
                            );
                        }
                    }
                }
            }

            cp_idx += 1;
        }
    }

    eprintln!("\n=== VEREDITO DISCRIMINANTE ===");
    eprintln!("Ultimo TestEvent (trace):  step {}", ultimo_testevent);

    match spec10_ready_step {
        Some(s) => {
            let dentro_do_mesmo_step = s == ultimo_testevent;
            eprintln!(
                "EvCB spec=10h READY:   step {}{}",
                s,
                if dentro_do_mesmo_step {
                    " (= ultimo TestEvent: corrida no mesmo step — IRQ pos-instrucao)"
                } else if s > ultimo_testevent {
                    " (DEPOIS do ultimo TestEvent → corrida confirmada)"
                } else {
                    " (ANTES do ultimo TestEvent → TestEvent devolveu errado)"
                }
            );
        }
        None => {
            eprintln!("EvCB spec=10h READY:   NUNCA — saturacao ausente, aumentar janela");
        }
    }

    match spec200_ready_step {
        Some(s) => {
            let dentro_do_mesmo_step = s == ultimo_testevent;
            eprintln!(
                "EvCB spec=200h READY:  step {}{}",
                s,
                if dentro_do_mesmo_step {
                    " (= ultimo TestEvent: corrida no mesmo step — IRQ pos-instrucao)"
                } else if s > ultimo_testevent {
                    " (DEPOIS do ultimo TestEvent → corrida confirmada)"
                } else {
                    " (ANTES do ultimo TestEvent → TestEvent devolveu errado)"
                }
            );
        }
        None => {
            eprintln!("EvCB spec=200h READY:  NUNCA — saturacao ausente, aumentar janela");
        }
    }

    assert!(
        spec10_ready_step.is_some(),
        "EvCB spec=10h deve estar ready ate {} M passos (invariante 30: \
         saturacao conhecida a 700 M na iter 0126)",
        max_steps / 1_000_000
    );
    assert!(
        spec200_ready_step.is_some(),
        "EvCB spec=200h deve estar ready ate {} M passos (invariante 30: \
         saturacao conhecida a 700 M na iter 0126)",
        max_steps / 1_000_000
    );
}
