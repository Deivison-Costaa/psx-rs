use psx_core::bus::{Bios, Bus, BusRead, Ram};
use psx_core::cpu::Cpu;
use std::path::PathBuf;

fn workspace_root() -> PathBuf {
    let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    d.push("..");
    d.push("..");
    d
}

fn bios_path() -> PathBuf {
    let mut d = workspace_root();
    d.push("bios");
    d.push("SCPH1001.BIN");
    d
}

fn disc_cue_path() -> PathBuf {
    let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    d.push("..");
    d.push("..");
    d.push("..");
    d.push("roms");
    d.push("extraido");
    d.push("Crash Bandicoot (USA).cue");
    d
}

fn carregar_bios_e_disco() -> (Bus, Cpu) {
    let bios_data = std::fs::read(bios_path()).expect("ler BIOS");
    let bios = Bios::from_bytes(bios_data).expect("BIOS invalida");
    let ram = Ram::new();
    let mut bus = Bus::new(ram, bios);
    let cue_path = disc_cue_path();
    let cue = std::fs::read_to_string(&cue_path).expect("ler CUE");
    let layout = psx_core::cdrom_bin_cue::parse_cue(&cue);
    let dir = cue_path.parent().unwrap();
    let bin = std::fs::read(dir.join(&layout.bin_path)).expect("ler BIN");
    bus.inject_disc(layout, bin);
    bus.cdrom_mut().insert_disc();
    let cpu = Cpu::new();
    (bus, cpu)
}

fn encontrar_primeiro_pc(alvo: u32, max_passos: usize) -> Option<(usize, Bus, u32)> {
    if !bios_path().exists() || !disc_cue_path().exists() {
        return None;
    }
    let (mut bus, mut cpu) = carregar_bios_e_disco();
    for passo in 1..=max_passos {
        let prev_pc = cpu.pc;
        cpu.step(&mut bus);
        if prev_pc == alvo {
            return Some((passo, bus, cpu.regs[3]));
        }
    }
    None
}

#[test]
fn slot_v1_18_constante_bfc06fdc_no_primeiro_trampolim() {
    if !bios_path().exists() || !disc_cue_path().exists() {
        eprintln!("SKIP: BIOS ou disco nao encontrados");
        return;
    }
    let (passo, bus, v1) = encontrar_primeiro_pc(0x2DAC, 500_000)
        .expect("0x2DAC deve ser alcancado em 500k passos com disco");

    let slot_addr = v1.wrapping_add(0x18);
    let slot_val = bus.read32::<BusRead>(slot_addr);
    eprintln!(
        "PC=0x2DAC passo={} v1=0x{:08X} slot_addr=0x{:08X} slot_val=0x{:08X}",
        passo, v1, slot_addr, slot_val
    );
    assert_eq!(
        slot_val, 0xBFC06FDC,
        "O slot em $v1+0x18 (0x{:08X}) ja deve conter BFC06FDC \
         no primeiro encontro do trampolim (passo {}); encontrado 0x{:08X}",
        slot_addr, passo, slot_val
    );
    assert_ne!(v1, 0, "v1 nao pode ser zero no ponto do trampolim");
}

#[test]
fn slot_v1_18_nao_muda_ate_jogo_bootar() {
    if !bios_path().exists() || !disc_cue_path().exists() {
        eprintln!("SKIP: BIOS ou disco nao encontrados");
        return;
    }
    let (passo_base, bus_base, v1_base) = encontrar_primeiro_pc(0x2DAC, 500_000)
        .expect("0x2DAC deve ser alcancado em 500k passos com disco");

    let slot_addr = v1_base.wrapping_add(0x18);
    let val_base = bus_base.read32::<BusRead>(slot_addr);
    assert_eq!(val_base, 0xBFC06FDC, "valor base do slot deve ser BFC06FDC");

    let pontos = [1_000_000usize, 15_000_000];
    for ponto in &pontos {
        if *ponto <= passo_base {
            continue;
        }
        let (mut bus, mut cpu) = carregar_bios_e_disco();
        let mut slot_capturado = None;
        for passo in 1..=*ponto {
            let prev_pc = cpu.pc;
            cpu.step(&mut bus);
            if prev_pc == 0x2DAC {
                let v1 = cpu.regs[3];
                let val = bus.read32::<BusRead>(v1.wrapping_add(0x18));
                slot_capturado = Some((passo, val));
            }
        }
        if let Some((passo, val)) = slot_capturado {
            eprintln!("Ponto passo={}: slot_val=0x{:08X}", passo, val);
            assert_eq!(
                val, 0xBFC06FDC,
                "No passo {} o slot em $v1+0x18 ainda deve ser BFC06FDC; encontrado 0x{:08X}",
                passo, val
            );
        } else {
            eprintln!("Ponto {}: 0x2DAC nao alcancado ate la", ponto);
        }
    }
}
