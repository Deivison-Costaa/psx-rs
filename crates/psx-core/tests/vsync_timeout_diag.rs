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

fn rayman_disc_path() -> Option<String> {
    for candidate in &[
        "../../../roms/extraido/Rayman (USA) DADOS.cue",
        "../../roms/extraido/Rayman (USA) DADOS.cue",
        "../roms/extraido/Rayman (USA) DADOS.cue",
    ] {
        if std::path::Path::new(candidate).exists() {
            return Some(candidate.to_string());
        }
    }
    None
}

#[test]
fn diagnostico_vsync_timeout_rayman() {
    if !bios_existe() {
        eprintln!("BIOS nao encontrada — teste ignorado");
        return;
    }

    let disc = match rayman_disc_path() {
        Some(p) => p,
        None => {
            eprintln!("disco Rayman nao encontrado — teste ignorado");
            return;
        }
    };

    let bios = carregar_bios().expect("BIOS invalida");
    let ram = Ram::new();
    let mut bus = Bus::new(ram, bios);

    let cue_text = std::fs::read_to_string(&disc).expect("ler CUE");
    let layout = psx_core::cdrom_bin_cue::parse_cue(&cue_text);
    let bin_dir = std::path::Path::new(&disc)
        .parent()
        .unwrap_or_else(|| std::path::Path::new("."));
    let bin_data = std::fs::read(bin_dir.join(&layout.bin_path)).expect("ler BIN");

    bus.inject_disc(layout, bin_data);
    bus.cdrom_mut().insert_disc();

    let mut cpu = Cpu::new();

    let max_steps = 500_000_000usize;
    let mut tty = Vec::new();
    let mut timeout = false;

    for _ in 0..max_steps {
        cpu.step(&mut bus);
        tty.extend_from_slice(&bus.take_tty());
        if String::from_utf8_lossy(&tty).contains("VSync: timeout") {
            timeout = true;
            break;
        }
    }

    let irq0_total = bus.irq().raise_count(0);
    let handler_total = cpu.irq_handler_entries;
    let mask = bus.irq().read_mask();
    let tmr1_mode = bus.read32::<BusRead>(0x1F80_1114);
    let tmr1_sync = (tmr1_mode & 1) != 0;
    let counter = bus.read32::<BusRead>(0x801D_F2CC);

    eprintln!("=== Diagnostico VSync timeout Rayman ===");
    eprintln!(
        "timeout={timeout} irq0={irq0_total} handlers={handler_total} \
         mask=0x{mask:04X} tmr1_sync={tmr1_sync} counter=0x{counter:08X}"
    );

    assert!(
        irq0_total > 0,
        "IRQ0 deve ser levantada durante o boot (contagem=0 em ate {max_steps} passos). \
         Sem IRQ0, o scheduler de VBlank nao funciona."
    );

    assert!(
        handler_total > 0,
        "CPU deve vetorizar para 0x80000080 ao menos uma vez (entries=0)."
    );

    assert!(
        timeout,
        "'VSync: timeout' deve ser detectado na TTY ate {max_steps} passos. \
         Se nao, o jogo passou do VSync ou a janela e curta demais."
    );

    // Hipotese (c): I_MASK bit 0 (VBlank) nao habilitado
    assert_ne!(
        mask & 1,
        0,
        "I_MASK bit0 deve estar habilitado no momento do timeout. \
         I_MASK=0x{mask:04X}. Com bit0=0, a hipotese (c) seria verdadeira."
    );

    // Hipotese (b): Timer1 sincronizado com VBlank
    assert!(
        !tmr1_sync,
        "Timer1 NAO deve estar sincronizado com VBlank no momento do timeout. \
         mode=0x{tmr1_mode:08X}. O jogo nao usa RCnt1+VBlank — hipotese (b) refutada."
    );

    // Hipotese (a): contador de VBlank incrementado por handler via IRQ0
    assert_eq!(
        counter, 0,
        "Contador de VBlank do jogo em 0x801DF2CC deve ser 0 (nunca incrementado). \
         counter=0x{counter:08X}. A hipotese (a) esta confirmada: \
         IRQ0 e levantada ({irq0_total}x), a CPU entra no handler ({handler_total}x), \
         I_MASK tem bit0 habilitado, mas o handler do jogo nunca incrementou o contador. \
         O defeito esta na cadeia de dispatch do handler: a BIOS vetoriza para 0x80000080 \
         mas o handler registrado pelo jogo (que incrementa 0x801DF2CC) nao e alcancado. \
         A cadeia ExCB/EvCB nao contem entrada para classe F0000001 (VBlank callback)."
    );
}
