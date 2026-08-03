use psx_core::bus::{Bios, Bus, BusRead, Ram};
use psx_core::cpu::Cpu;

fn optional_path<'a>(candidates: &'a [&'a str]) -> Option<&'a str> {
    candidates
        .iter()
        .copied()
        .find(|path| std::path::Path::new(path).exists())
}

fn table_base(bus: &Bus, dispatch: u32) -> Option<u32> {
    let lui = bus.read32::<BusRead>(dispatch);
    let addiu = bus.read32::<BusRead>(dispatch + 4);
    if lui >> 26 != 0b001111 || addiu >> 26 != 0b001001 {
        return None;
    }
    let table = ((lui & 0xFFFF) << 16).wrapping_add(addiu & 0xFFFF);
    let lui = bus.read32::<BusRead>(table);
    let addiu = bus.read32::<BusRead>(table + 4);
    if lui >> 26 != 0b001111 || addiu >> 26 != 0b001001 {
        return None;
    }
    Some(((lui & 0xFFFF) << 16).wrapping_add(addiu & 0xFFFF))
}

fn table_entry(bus: &Bus, dispatch: u32, index: u32) -> Option<u32> {
    let base = table_base(bus, dispatch)?;
    let target = bus.read32::<BusRead>(base + index * 4);
    (target != 0).then_some(target)
}

fn setup() -> Option<(Bus, Cpu)> {
    let bios_path = optional_path(&["bios/SCPH1001.BIN", "../../bios/SCPH1001.BIN"])?;
    let disc_path = optional_path(&[
        "../roms/extraido/Rayman (USA) DADOS.cue",
        "../../roms/extraido/Rayman (USA) DADOS.cue",
        "../../../roms/extraido/Rayman (USA) DADOS.cue",
    ])?;
    let bios = Bios::from_bytes(std::fs::read(bios_path).expect("ler BIOS")).expect("BIOS valida");
    let cue = std::fs::read_to_string(disc_path).expect("ler CUE");
    let layout = psx_core::cdrom_bin_cue::parse_cue(&cue);
    let directory = std::path::Path::new(disc_path)
        .parent()
        .expect("CUE deve ter diretorio");
    let bin = std::fs::read(directory.join(&layout.bin_path)).expect("ler BIN");
    let mut bus = Bus::new(Ram::new(), bios);
    bus.inject_disc(layout, bin);
    bus.cdrom_mut().insert_disc();
    Some((bus, Cpu::new()))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Slot {
    class: u32,
    status: u32,
    spec: u32,
}

fn slot(bus: &Bus, indice: u32) -> Slot {
    let base = bus.read32::<BusRead>(0x0000_0120) + indice * 0x1C;
    Slot {
        class: bus.read32::<BusRead>(base),
        status: bus.read32::<BusRead>(base + 4),
        spec: bus.read32::<BusRead>(base + 8),
    }
}

#[test]
fn sys_init_memory_troca_o_dono_dos_descritores_que_o_jogo_guardou() {
    let Some((mut bus, mut cpu)) = setup() else {
        eprintln!("BIOS ou disco Rayman nao encontrado — teste ignorado");
        return;
    };

    let mut sys_init_memory = None;
    let mut reinit_step = 0usize;
    let mut reinit_ra = 0u32;
    let mut antes = None;
    let mut depois = None;

    for step in 1..=165_000_000usize {
        if step == 1 || step % 4096 == 0 {
            sys_init_memory = sys_init_memory.or_else(|| table_entry(&bus, 0xC0, 0x08));
        }

        if step == 86_989_000 {
            antes = Some((slot(&bus, 1), slot(&bus, 4)));
        }
        if step == 165_000_000 {
            depois = Some((slot(&bus, 1), slot(&bus, 4)));
        }

        if sys_init_memory == Some(cpu.pc) && step > 1_000_000 {
            reinit_step = step;
            reinit_ra = cpu.regs[31];
        }

        cpu.step(&mut bus);
    }

    let (slot1_antes, slot4_antes) = antes.expect("snapshot do inicio da espera");
    assert_eq!(
        slot1_antes,
        Slot {
            class: 0xF000_0003,
            status: 0x0000_2000,
            spec: 0x0000_0020,
        },
        "no inicio da espera o descritor F1000001h e um evento de CDROM, nao de memory card"
    );
    assert_eq!(
        slot4_antes,
        Slot {
            class: 0xF000_0003,
            status: 0x0000_2000,
            spec: 0x0000_8000,
        },
        "e o F1000004h e o erro do mesmo CDROM"
    );

    // +15.801 na iteracao 0185: o Rayman emite comandos de cor do GTE, que deixaram de ser
    // no-op. Mesmo deslocamento uniforme de rayman_autoack.rs — achado 10.115.
    assert_eq!(reinit_step, 154_911_652);
    assert_eq!(
        reinit_ra, 0xBFC0_6F4C,
        "quem chama SysInitMemory de novo e a ROM do BIOS, nao o jogo"
    );

    let (slot1_depois, slot4_depois) = depois.expect("snapshot depois da reinicializacao");
    assert_eq!(
        slot1_depois,
        Slot {
            class: 0xF400_0001,
            status: 0x0000_2000,
            spec: 0x0000_0004,
        },
        "depois da reinicializacao o MESMO descritor aponta para um evento de memory card"
    );
    assert_eq!(
        slot4_depois,
        Slot {
            class: 0xF400_0001,
            status: 0x0000_2000,
            spec: 0x0000_2000,
        }
    );
}
