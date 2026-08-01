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

struct Evcb {
    class: u32,
    status: u32,
    spec: u32,
    mode: u32,
}

fn ler_evcb(bus: &Bus, evcb_base: u32, indice: u32) -> Evcb {
    let base = evcb_base + indice * 0x1C;
    Evcb {
        class: bus.read32::<BusRead>(base),
        status: bus.read32::<BusRead>(base + 4),
        spec: bus.read32::<BusRead>(base + 8),
        mode: bus.read32::<BusRead>(base + 0xC),
    }
}

fn ler_tabela_evcb(bus: &Bus) -> Option<(u32, u32)> {
    let evcb_ptr_raw = bus.read32::<BusRead>(0x0000_0120);
    let evcb_size = bus.read32::<BusRead>(0x0000_0124);
    let evcb_ptr = evcb_ptr_raw & 0x001F_FFFF;

    if evcb_ptr == 0 || evcb_size == 0 {
        return None;
    }

    Some((evcb_ptr, evcb_size))
}

fn descritor_para_indice(descritor: u32) -> u32 {
    descritor.wrapping_sub(0xF100_0000)
}

#[test]
fn evcb_descritor_mapeia_para_spec_correto() {
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
    let max_steps: usize = 90_000_000;

    for _step in 1..=max_steps {
        cpu.step(&mut bus);

        if let Some((evcb_ptr, evcb_size)) = ler_tabela_evcb(&bus) {
            let num_blocks = evcb_size / 0x1C;
            let mut indice20h: Option<u32> = None;
            let mut indice8000h: Option<u32> = None;

            for i in 0..num_blocks {
                let ev = ler_evcb(&bus, evcb_ptr, i);
                if ev.class == 0xF000_0003 && ev.status != 0 {
                    if ev.spec == 0x20 && indice20h.is_none() {
                        indice20h = Some(i);
                    }
                    if ev.spec == 0x8000 && indice8000h.is_none() {
                        indice8000h = Some(i);
                    }
                }
            }

            if let (Some(idx20), Some(idx8000)) = (indice20h, indice8000h) {

                let descritor_idx20 = 0xF100_0000u32.wrapping_add(idx20);
                let descritor_idx8000 = 0xF100_0000u32.wrapping_add(idx8000);

                eprintln!(
                    "EvCB[{}] spec=20h  → descritor esperado: 0x{:08X} (F1000000h + {})",
                    idx20, descritor_idx20, idx20
                );
                eprintln!(
                    "EvCB[{}] spec=8000h → descritor esperado: 0x{:08X} (F1000000h + {})",
                    idx8000, descritor_idx8000, idx8000
                );

                let a0_medido_20h = 0xF100_0001u32;
                let a0_medido_8000h = 0xF100_0004u32;

                assert_eq!(
                    descritor_para_indice(a0_medido_20h),
                    idx20,
                    "Descritor F1000001 (a0 medido) deve mapear para EvCB que contem spec=20h. \
                     Indice esperado={}, indice encontrado={}",
                    idx20,
                    descritor_para_indice(a0_medido_20h)
                );

                assert_eq!(
                    descritor_para_indice(a0_medido_8000h),
                    idx8000,
                    "Descritor F1000004 (a0 medido) deve mapear para EvCB que contem spec=8000h. \
                     Indice esperado={}, indice encontrado={}",
                    idx8000,
                    descritor_para_indice(a0_medido_8000h)
                );

                assert_eq!(
                    idx20, 1,
                    "EvCB[1] deve conter spec=20h (command completed) — \
                     BIOS abre 5 eventos CD-ROM em ordem: 10h, 20h, 40h, 80h, 8000h"
                );

                assert_eq!(
                    idx8000, 4,
                    "EvCB[4] deve conter spec=8000h (error happened)"
                );

                return;
            }
        }
    }

    if let Some((evcb_ptr, evcb_size)) = ler_tabela_evcb(&bus) {
        let num_blocks = evcb_size / 0x1C;
        eprintln!(
            "EvCB dump apos {} M passos ({} blocos):",
            max_steps / 1_000_000,
            num_blocks
        );
        for i in 0..num_blocks {
            let ev = ler_evcb(&bus, evcb_ptr, i);
            if ev.class == 0xF000_0003 {
                eprintln!(
                    "  EvCB[{}] class=0x{:08X} status=0x{:04X} spec=0x{:04X} mode=0x{:04X}",
                    i, ev.class, ev.status, ev.spec, ev.mode
                );
            }
        }
    }

    panic!(
        "EvCBs spec=20h e/ou spec=8000h nao encontrados ate {} M passos. \
         O kernel nao montou a tabela de eventos esperada.",
        max_steps / 1_000_000
    );
}

#[test]
fn descritor_decode_index_correto() {
    assert_eq!(descritor_para_indice(0xF100_0000), 0);
    assert_eq!(descritor_para_indice(0xF100_0001), 1);
    assert_eq!(descritor_para_indice(0xF100_0004), 4);
    assert_eq!(descritor_para_indice(0xF100_0005), 5);
}
