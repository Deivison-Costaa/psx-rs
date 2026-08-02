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

fn b_table_base(bus: &Bus) -> Option<u32> {
    let w_lui = bus.read32::<BusRead>(0x0000_00B0);
    let w_addiu = bus.read32::<BusRead>(0x0000_00B4);
    if w_lui >> 26 != 0b001111 || w_addiu >> 26 != 0b001001 {
        return None;
    }
    let dispatch = ((w_lui & 0xFFFF) << 16).wrapping_add(w_addiu & 0xFFFF);
    let w_lui = bus.read32::<BusRead>(dispatch);
    let w_addiu = bus.read32::<BusRead>(dispatch + 4);
    if w_lui >> 26 != 0b001111 || w_addiu >> 26 != 0b001001 {
        return None;
    }
    let base = ((w_lui & 0xFFFF) << 16).wrapping_add(w_addiu & 0xFFFF);
    (base != 0).then_some(base)
}

fn b_table_entry(bus: &Bus, index: u32) -> Option<u32> {
    let base = b_table_base(bus)?;
    let target = bus.read32::<BusRead>(base + index * 4);
    (target != 0).then_some(target)
}

fn c_table_base(bus: &Bus) -> Option<u32> {
    let w_lui = bus.read32::<BusRead>(0x0000_00C0);
    let w_addiu = bus.read32::<BusRead>(0x0000_00C4);
    if w_lui >> 26 != 0b001111 || w_addiu >> 26 != 0b001001 {
        return None;
    }
    let dispatch = ((w_lui & 0xFFFF) << 16).wrapping_add(w_addiu & 0xFFFF);
    let w_lui = bus.read32::<BusRead>(dispatch);
    let w_addiu = bus.read32::<BusRead>(dispatch + 4);
    if w_lui >> 26 != 0b001111 || w_addiu >> 26 != 0b001001 {
        return None;
    }
    let base = ((w_lui & 0xFFFF) << 16).wrapping_add(w_addiu & 0xFFFF);
    (base != 0).then_some(base)
}

fn c_table_entry(bus: &Bus, index: u32) -> Option<u32> {
    let base = c_table_base(bus)?;
    let target = bus.read32::<BusRead>(base + index * 4);
    (target != 0).then_some(target)
}

fn counter_accesses(bus: &Bus) -> Vec<(u32, u32)> {
    let mut accesses = Vec::new();
    for offset in (0..0x0020_0000u32).step_by(4) {
        let addr = 0x8000_0000u32 + offset;
        let instr = bus.read32::<BusRead>(addr);
        let op = instr >> 26;
        if (0x20..=0x2E).contains(&op) && instr & 0xFFFF == 0xF2CC {
            accesses.push((addr, instr));
        }
    }
    accesses
}

fn sign_extend_immediate(instr: u32) -> u32 {
    (instr as i16 as i32) as u32
}

fn is_game_pc(pc: u32) -> bool {
    let phys = pc & 0x1FFF_FFFF;
    (0x0001_0000..0x0020_0000).contains(&phys)
}

struct StoreCandidate {
    addr: u32,
    op: u32,
    value: u32,
}

fn watched_store(cpu: &Cpu, bus: &Bus) -> Option<StoreCandidate> {
    let instr = bus.read32::<BusRead>(cpu.pc);
    let op = instr >> 26;
    if !matches!(op, 0x28 | 0x29 | 0x2A | 0x2B | 0x2E) {
        return None;
    }
    let rs = ((instr >> 21) & 0x1F) as usize;
    let addr = cpu.regs[rs].wrapping_add(sign_extend_immediate(instr));
    let phys = addr & 0x1FFF_FFFF;
    let watched = (0x80..0x90).contains(&phys)
        || (0x1F80_1110..=0x1F80_111B).contains(&phys)
        || (0x001D_F2CC..=0x001D_F2CF).contains(&phys);
    if !watched {
        return None;
    }
    let rt = ((instr >> 16) & 0x1F) as usize;
    let value = if op == 0x29 {
        cpu.regs[rt] & 0xFFFF
    } else {
        cpu.regs[rt]
    };
    Some(StoreCandidate { addr, op, value })
}

#[derive(Debug)]
struct BiosCall {
    table: char,
    index: u32,
    step: usize,
    pc: u32,
    args: [u32; 4],
    chain_words: Option<[u32; 4]>,
    hook_words: Option<[u32; 12]>,
    counter_accesses: Option<Vec<(u32, u32)>>,
}

#[derive(Debug)]
struct StoreObservation {
    step: usize,
    pc: u32,
    addr: u32,
    op: u32,
    value: u32,
    before: u32,
    after: u32,
}

#[derive(Debug, Default)]
struct InstallationTrace {
    bios_calls: Vec<BiosCall>,
    stores: Vec<StoreObservation>,
    hook_entries: Vec<(usize, u32)>,
    first_spin_step: Option<usize>,
}

fn trace_rayman_installation(mut bus: Bus, mut cpu: Cpu) -> InstallationTrace {
    const MAX_STEPS: usize = 500_000_000;
    const VSYNC_SPIN: u32 = 0x801B_958C;
    const REFRESH_INTERVAL: usize = 4096;
    let tracked = [0x02u32, 0x08, 0x0C, 0x0D, 0x18, 0x19];
    let mut targets = [None; 6];
    let tracked_c = [0x00u32, 0x02, 0x03, 0x0C, 0x0D];
    let mut c_targets = [None; 5];
    let mut hook_target = None;
    let mut trace = InstallationTrace::default();

    for step in 1..=MAX_STEPS {
        if step == 1 || step % REFRESH_INTERVAL == 0 {
            for (slot, index) in tracked.iter().enumerate() {
                if targets[slot].is_none() {
                    targets[slot] = b_table_entry(&bus, *index);
                }
            }
            for (slot, index) in tracked_c.iter().enumerate() {
                if c_targets[slot].is_none() {
                    c_targets[slot] = c_table_entry(&bus, *index);
                }
            }
        }

        let executed_pc = cpu.pc;
        if hook_target == Some(executed_pc) {
            trace.hook_entries.push((step, executed_pc));
        }
        let watched = watched_store(&cpu, &bus);
        let before = watched
            .as_ref()
            .map(|store| bus.read32::<BusRead>(store.addr & !3));

        for (slot, target) in targets.iter().enumerate() {
            if target == &Some(executed_pc) {
                trace.bios_calls.push(BiosCall {
                    table: 'B',
                    index: tracked[slot],
                    step,
                    pc: executed_pc,
                    args: [cpu.regs[4], cpu.regs[5], cpu.regs[6], cpu.regs[7]],
                    chain_words: None,
                    hook_words: if tracked[slot] == 0x19 {
                        let mut words = [0u32; 12];
                        for (index, word) in words.iter_mut().enumerate() {
                            *word =
                                bus.read32::<BusRead>(cpu.regs[4].wrapping_add(index as u32 * 4));
                        }
                        hook_target = Some(words[0]);
                        Some(words)
                    } else {
                        None
                    },
                    counter_accesses: if tracked[slot] == 0x19 {
                        Some(counter_accesses(&bus))
                    } else {
                        None
                    },
                });
            }
        }

        for (slot, target) in c_targets.iter().enumerate() {
            if target == &Some(executed_pc) {
                let chain_words = if tracked_c[slot] == 0x02 || tracked_c[slot] == 0x03 {
                    let addr = cpu.regs[5];
                    Some([
                        bus.read32::<BusRead>(addr),
                        bus.read32::<BusRead>(addr + 4),
                        bus.read32::<BusRead>(addr + 8),
                        bus.read32::<BusRead>(addr + 12),
                    ])
                } else {
                    None
                };
                trace.bios_calls.push(BiosCall {
                    table: 'C',
                    index: tracked_c[slot],
                    step,
                    pc: executed_pc,
                    args: [cpu.regs[4], cpu.regs[5], cpu.regs[6], cpu.regs[7]],
                    chain_words,
                    hook_words: None,
                    counter_accesses: None,
                });
            }
        }

        cpu.step(&mut bus);
        if let Some(store) = watched {
            let after = bus.read32::<BusRead>(store.addr & !3);
            trace.stores.push(StoreObservation {
                step,
                pc: executed_pc,
                addr: store.addr,
                op: store.op,
                value: store.value,
                before: before.unwrap_or(0),
                after,
            });
        }
        if cpu.pc == VSYNC_SPIN {
            trace.first_spin_step = Some(step);
            break;
        }
    }

    trace
}

#[test]
fn diagnostico_instalacao_vsync_rayman() {
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

    let trace = trace_rayman_installation(bus, Cpu::new());
    eprintln!("=== Instalacao do VSync do Rayman ===");
    eprintln!("primeiro_spin={:?}", trace.first_spin_step);
    for call in &trace.bios_calls {
        eprintln!(
            "{}({:02X}h) step={} pc=0x{:08X} a0=0x{:08X} a1=0x{:08X} \
             a2=0x{:08X} a3=0x{:08X}",
            call.table,
            call.index,
            call.step,
            call.pc,
            call.args[0],
            call.args[1],
            call.args[2],
            call.args[3]
        );
        if let Some(words) = call.chain_words {
            eprintln!(
                "     chain[0..3]=0x{:08X} 0x{:08X} 0x{:08X} 0x{:08X}",
                words[0], words[1], words[2], words[3]
            );
        }
        if let Some(words) = call.hook_words {
            eprintln!("     hook structure: {:08X?}", words);
        }
        if let Some(accesses) = &call.counter_accesses {
            eprintln!("     counter accesses: {:08X?}", accesses);
        }
    }
    eprintln!("hook entries: {}", trace.hook_entries.len());
    for store in &trace.stores {
        eprintln!(
            "store op=0x{:02X} step={} pc=0x{:08X} addr=0x{:08X}",
            store.op, store.step, store.pc, store.addr
        );
        eprintln!(
            "     value=0x{:08X} before=0x{:08X} after=0x{:08X}",
            store.value, store.before, store.after
        );
    }

    assert!(
        trace.first_spin_step.is_some(),
        "o Rayman deve alcancar o spin de VSync em 0x801B958C antes do limite"
    );
    let has_vsync_event = trace
        .bios_calls
        .iter()
        .any(|call| call.table == 'B' && call.index == 8 && call.args[0] == 0xF000_0001);
    let has_timer_vblank = trace.stores.iter().any(|store| {
        let phys = store.addr & 0x1FFF_FFFF;
        phys == 0x1F80_1114 && store.value & 1 != 0
    });
    let has_game_vector = trace.stores.iter().any(|store| {
        let phys = store.addr & 0x1FFF_FFFF;
        (0x80..0x90).contains(&phys) && is_game_pc(store.pc)
    });
    let hook_install = trace.bios_calls.iter().any(|call| {
        call.table == 'B'
            && call.index == 0x19
            && call.hook_words.is_some_and(|words| words[0] == 0x801B_8E60)
    });
    assert!(
        hook_install,
        "Rayman deve instalar o hook de excecao 0x801B8E60 via B(19h) antes do spin"
    );
    assert!(
        !has_vsync_event,
        "VSyncCallback nao deve abrir EvCB F0000001"
    );
    assert!(
        !has_timer_vblank,
        "SetRCnt nao deve habilitar sincronia de Timer1 com VBlank"
    );
    assert!(
        !has_game_vector,
        "Rayman nao deve substituir diretamente o vetor 0x80000080"
    );
    assert!(
        !trace.hook_entries.is_empty(),
        "o hook instalado deve ser executado"
    );
    assert!(
        !trace
            .stores
            .iter()
            .any(|store| { (store.addr & 0x1FFF_FFFF) == 0x001D_F2CC && store.value != 0 }),
        "nenhuma execucao antes do spin deve incrementar 0x801DF2CC"
    );
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
