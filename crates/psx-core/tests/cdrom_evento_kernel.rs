use psx_core::bus::{Bios, Bus, Ram};
use psx_core::cpu::Cpu;
use psx_core::irq::Irq;

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

#[test]
fn irq_raise_count_incrementa_por_bit() {
    let mut irq = Irq::new();
    assert_eq!(irq.raise_count(0), 0);
    assert_eq!(irq.raise_count(2), 0);

    irq.raise(2);
    assert_eq!(irq.raise_count(2), 1);
    assert_eq!(irq.raise_count(0), 0);

    irq.raise(2);
    assert_eq!(irq.raise_count(2), 2);

    irq.raise(0);
    assert_eq!(irq.raise_count(0), 1);
}

#[test]
fn irq_raise_count_bit_fora_do_alcance_retorna_zero() {
    let mut irq = Irq::new();
    irq.raise(15);
    assert_eq!(irq.raise_count(15), 0);
}

#[test]
fn cpu_conta_entradas_do_handler_de_interrupcao() {
    let cpu = Cpu::new();
    assert_eq!(cpu.irq_handler_entries, 0);
}

#[test]
fn boot_com_disco_produz_irq2_e_handler() {
    if !bios_existe() {
        eprintln!("BIOS nao encontrada — teste ignorado");
        return;
    }

    let disc = match disc_path() {
        Some(p) => p,
        None => {
            eprintln!("disco nao encontrado — teste ignorado");
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

    let max_steps = 150_000_000usize;
    for _ in 0..max_steps {
        cpu.step(&mut bus);
    }

    let irq2_count = bus.irq().raise_count(2);
    let handler_entries = cpu.irq_handler_entries;

    eprintln!(
        "Diagnostico (com disco): IRQ2={}, handler_entries={} ({} passos)",
        irq2_count, handler_entries, max_steps
    );

    assert!(
        irq2_count > 0,
        "o boot com disco tem de levantar IRQ2 ate 150 M passos (medido: 107 entre 80 M e \
         100 M); zero aqui significa janela curta demais ou regressao na cadeia do CD-ROM"
    );
    assert!(
        handler_entries > 0,
        "a CPU tem de vetorizar para 0x80000080 durante o boot"
    );

    eprintln!(
        "Contagem por bit: 0={} 1={} 2={} 3={} 4={} 5={} 6={} 7={} 8={} 9={} 10={}",
        bus.irq().raise_count(0),
        bus.irq().raise_count(1),
        bus.irq().raise_count(2),
        bus.irq().raise_count(3),
        bus.irq().raise_count(4),
        bus.irq().raise_count(5),
        bus.irq().raise_count(6),
        bus.irq().raise_count(7),
        bus.irq().raise_count(8),
        bus.irq().raise_count(9),
        bus.irq().raise_count(10),
    );
}

#[test]
fn tabela_de_tabelas_evcb_esta_presente_apos_o_boot() {
    if !bios_existe() {
        eprintln!("BIOS nao encontrada — teste ignorado");
        return;
    }
    if disc_path().is_none() {
        eprintln!("disco nao encontrado — teste ignorado");
        return;
    }

    let bios = carregar_bios().expect("BIOS invalida");
    let ram = Ram::new();
    let mut bus = Bus::new(ram, bios);

    let disc = disc_path().unwrap();
    let cue_text = std::fs::read_to_string(&disc).expect("ler CUE");
    let layout = psx_core::cdrom_bin_cue::parse_cue(&cue_text);
    let bin_dir = std::path::Path::new(&disc)
        .parent()
        .unwrap_or_else(|| std::path::Path::new("."));
    let bin_data = std::fs::read(bin_dir.join(&layout.bin_path)).expect("ler BIN");

    bus.inject_disc(layout, bin_data);
    bus.cdrom_mut().insert_disc();

    let mut cpu = Cpu::new();
    let max_steps = 150_000_000usize;

    for _ in 0..max_steps {
        cpu.step(&mut bus);
    }

    let evcb_ptr_raw = bus.read32::<psx_core::bus::BusRead>(0x0000_0120);
    let evcb_size = bus.read32::<psx_core::bus::BusRead>(0x0000_0124);
    let evcb_ptr = evcb_ptr_raw & 0x001F_FFFF;

    let tty = bus.take_tty();
    if !tty.is_empty() {
        let tty_str = String::from_utf8_lossy(&tty);
        let last_lines: Vec<&str> = tty_str.lines().rev().take(10).collect();
        eprintln!("TTY (ultimas 10 linhas):");
        for line in last_lines.iter().rev() {
            eprintln!("  {}", line);
        }
    }

    eprintln!(
        "Table of Tables: EvCB ptr=0x{:08X} (fisico=0x{:08X}) size=0x{:X} ({} bytes)",
        evcb_ptr_raw, evcb_ptr, evcb_size, evcb_size
    );

    let mut blocos_registrados = 0u32;
    let mut ready_events = 0u32;
    if evcb_ptr != 0 && evcb_ptr < 0x0020_0000 && evcb_size > 0 && evcb_size <= 0x10000 {
        let num_blocks = evcb_size / 0x1C;
        eprintln!(
            "  EvCB dump ({} blocos, base=0x{:08X}):",
            num_blocks, evcb_ptr
        );
        for i in 0..num_blocks {
            let base = evcb_ptr + i * 0x1C;
            let class = bus.read32::<psx_core::bus::BusRead>(base);
            let status = bus.read32::<psx_core::bus::BusRead>(base + 4);
            let spec = bus.read32::<psx_core::bus::BusRead>(base + 8);
            let mode = bus.read32::<psx_core::bus::BusRead>(base + 0xC);
            if status != 0 {
                blocos_registrados += 1;
                eprintln!(
                    "  [{}] 0x{:08X}: class=0x{:08X} status=0x{:04X} spec=0x{:08X} mode=0x{:04X}",
                    i, base, class, status, spec, mode
                );
                if status == 0x4000 {
                    ready_events += 1;
                }
            }
        }
        eprintln!("  Eventos ready (status=4000h): {}", ready_events);
    } else {
        eprintln!(
            "  EvCB nao alocado (ptr=0x{:08X}, size=0x{:X})",
            evcb_ptr, evcb_size
        );
    }

    assert!(
        blocos_registrados > 0,
        "ate 150 M passos o kernel tem de ter registrado eventos de CD-ROM nos EvCBs \
         (medido: 6 blocos class=F0000003h a 700 M; vazio aqui = janela curta ou regressao)"
    );
}

fn parse_bhandler_addr(bus: &Bus) -> Option<u32> {
    let w_lui = bus.read32::<psx_core::bus::BusRead>(0x0000_00B0);
    let w_addiu = bus.read32::<psx_core::bus::BusRead>(0x0000_00B4);
    let op_lui = w_lui >> 26;
    let op_addiu = w_addiu >> 26;
    if op_lui != 0b001111 || op_addiu != 0b001001 {
        return None;
    }
    let imm_lui = w_lui & 0xFFFF;
    let imm_addiu = w_addiu & 0xFFFF;
    let base = imm_lui << 16;
    let offset = imm_addiu;
    Some(base.wrapping_add(offset))
}

fn parse_ahandler_addr(bus: &Bus) -> Option<u32> {
    let w_lui = bus.read32::<psx_core::bus::BusRead>(0x0000_00A0);
    let w_addiu = bus.read32::<psx_core::bus::BusRead>(0x0000_00A4);
    let op_lui = w_lui >> 26;
    let op_addiu = w_addiu >> 26;
    if op_lui != 0b001111 || op_addiu != 0b001001 {
        return None;
    }
    let imm_lui = w_lui & 0xFFFF;
    let imm_addiu = w_addiu & 0xFFFF;
    let base = imm_lui << 16;
    let offset = imm_addiu;
    Some(base.wrapping_add(offset))
}

#[test]
fn bhandler_addr_e_valido_e_em_ram() {
    if !bios_existe() {
        eprintln!("BIOS nao encontrada — teste ignorado");
        return;
    }

    let bios = carregar_bios().expect("BIOS invalida");
    let ram = Ram::new();
    let mut bus = Bus::new(ram, bios);
    let mut cpu = Cpu::new();

    for _ in 0..5_000_000 {
        cpu.step(&mut bus);
    }

    let b_addr = parse_bhandler_addr(&bus);
    assert!(
        b_addr.is_some(),
        "B-handler deve ser parseavel de RAM[0xB0..0xB7]; RAM[0xB0]=0x{:08X} RAM[0xB4]=0x{:08X}",
        bus.read32::<psx_core::bus::BusRead>(0x0000_00B0),
        bus.read32::<psx_core::bus::BusRead>(0x0000_00B4)
    );

    let b_addr = b_addr.unwrap();
    eprintln!("B-handler target addr = 0x{:08X}", b_addr);
    assert!(
        b_addr > 0 && b_addr < 0x0020_0000,
        "B-handler target 0x{:08X} deve estar em RAM baixa (<2MB)",
        b_addr
    );

    let _dispatch_instr = bus.read32::<psx_core::bus::BusRead>(b_addr);
}

#[test]
fn ahandler_addr_e_valido_e_em_ram() {
    if !bios_existe() {
        eprintln!("BIOS nao encontrada — teste ignorado");
        return;
    }

    let bios = carregar_bios().expect("BIOS invalida");
    let ram = Ram::new();
    let mut bus = Bus::new(ram, bios);
    let mut cpu = Cpu::new();

    for _ in 0..5_000_000 {
        cpu.step(&mut bus);
    }

    let a_addr = parse_ahandler_addr(&bus);
    assert!(
        a_addr.is_some(),
        "A-handler deve ser parseavel de RAM[0xA0..0xA7]"
    );
    let a_addr = a_addr.unwrap();
    eprintln!("A-handler target addr = 0x{:08X}", a_addr);
    assert!(
        a_addr > 0 && a_addr < 0x0020_0000,
        "A-handler target 0x{:08X} deve estar em RAM baixa",
        a_addr
    );
}

fn b_bfun_table_base(bus: &Bus) -> Option<u32> {
    let b_addr = parse_bhandler_addr(bus)?;
    let w_lui = bus.read32::<psx_core::bus::BusRead>(b_addr);
    let w_addiu = bus.read32::<psx_core::bus::BusRead>(b_addr + 4);
    let op_lui = w_lui >> 26;
    let op_addiu = w_addiu >> 26;
    if op_lui != 0b001111 || op_addiu != 0b001001 {
        return None;
    }
    let imm_lui = w_lui & 0xFFFF;
    let imm_addiu = w_addiu & 0xFFFF;
    let base = (imm_lui << 16).wrapping_add(imm_addiu);
    if base == 0 || base >= 0x0020_0000 {
        return None;
    }
    Some(base)
}

#[test]
fn dump_atable_e_bdispatcher() {
    if !bios_existe() {
        eprintln!("BIOS nao encontrada — teste ignorado");
        return;
    }

    let bios = carregar_bios().expect("BIOS invalida");
    let ram = Ram::new();
    let mut bus = Bus::new(ram, bios);
    let mut cpu = Cpu::new();

    for _ in 0..5_000_000 {
        cpu.step(&mut bus);
    }

    let b_table_base = b_bfun_table_base(&bus);
    if let Some(base) = b_table_base {
        eprintln!("B-table base = 0x{:08X}", base);
        eprintln!("=== B-table entries ===");
        for i in 0..0x30u32 {
            let addr = base + i * 4;
            let val = bus.read32::<psx_core::bus::BusRead>(addr);
            if val != 0 {
                let name = match i {
                    0x0A => "WaitEvent",
                    0x0B => "TestEvent",
                    0x07 => "DeliverEvent",
                    0x08 => "OpenEvent",
                    0x09 => "CloseEvent",
                    0x0C => "EnableEvent",
                    0x0D => "DisableEvent",
                    0x20 => "UnDeliverEvent",
                    _ => "",
                };
                eprintln!(
                    "  B-table[0x{:02X} @ 0x{:08X}] = 0x{:08X} {}",
                    i, addr, val, name
                );
            }
        }
    } else {
        eprintln!("B-table base nao encontrada");
    }

    eprintln!("=== Table of Tables ===");
    let toc_entries = [(0x100u32, "ExCB"), (0x120, "EvCB"), (0x140, "FCB")];
    for &(addr, name) in &toc_entries {
        let ptr = bus.read32::<psx_core::bus::BusRead>(addr);
        let sz = bus.read32::<psx_core::bus::BusRead>(addr + 4);
        eprintln!(
            "  {} @ 0x{:04X}: ptr=0x{:08X} sz=0x{:X}",
            name, addr, ptr, sz
        );
    }
}
