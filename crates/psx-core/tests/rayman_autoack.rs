use psx_core::bus::{Bios, Bus, BusRead, Ram};
use psx_core::cpu::Cpu;

const GAME_HOOK: u32 = 0x801B_8E60;

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
struct Chamada {
    step: usize,
    arg: u32,
    ra: u32,
}

#[test]
fn start_pad_reabilita_o_auto_ack_que_o_jogo_tinha_desligado() {
    let Some((mut bus, mut cpu)) = setup() else {
        eprintln!("BIOS ou disco Rayman nao encontrado — teste ignorado");
        return;
    };

    let mut change_clear_pad = None;
    let mut start_pad = None;
    let mut hook_entry = None;
    let mut clear_pad: Vec<Chamada> = Vec::new();
    let mut start_pad_step = 0usize;
    let mut hook_install_step = 0usize;

    for step in 1..=170_000_000usize {
        if step == 1 || step % 4096 == 0 {
            change_clear_pad = change_clear_pad.or_else(|| table_entry(&bus, 0xB0, 0x5B));
            start_pad = start_pad.or_else(|| table_entry(&bus, 0xB0, 0x13));
            hook_entry = hook_entry.or_else(|| table_entry(&bus, 0xB0, 0x19));
        }

        let pc = cpu.pc;
        if change_clear_pad == Some(pc) {
            clear_pad.push(Chamada {
                step,
                arg: cpu.regs[4],
                ra: cpu.regs[31],
            });
        }
        if start_pad == Some(pc) {
            start_pad_step = step;
        }
        if hook_entry == Some(pc) && bus.read32::<BusRead>(cpu.regs[4]) == GAME_HOOK {
            hook_install_step = step;
        }

        cpu.step(&mut bus);
        if start_pad_step != 0 && clear_pad.iter().any(|c| c.step > start_pad_step) {
            break;
        }
    }

    let desliga = clear_pad
        .iter()
        .rev()
        .find(|c| c.arg == 0 && c.ra >= 0x8010_0000)
        .copied()
        .expect("o jogo deve desligar o auto-ack antes de instalar o proprio handler");
    assert_eq!(desliga.step, 164_110_587);
    assert_eq!(desliga.ra, 0x801B_8BC0);
    assert!(
        desliga.step < hook_install_step,
        "o jogo desliga o auto-ack antes de instalar o hook"
    );
    assert_eq!(hook_install_step, 164_111_334);

    assert_eq!(start_pad_step, 164_123_374);
    assert!(
        start_pad_step > desliga.step,
        "StartPAD2 vem depois do desligamento"
    );

    let religa = clear_pad
        .iter()
        .find(|c| c.step > start_pad_step)
        .copied()
        .expect("StartPAD2 deve reabilitar o auto-ack por dentro");
    assert_eq!(
        religa,
        Chamada {
            step: 164_123_851,
            arg: 1,
            ra: 0x0000_4BEC,
        },
        "a religada e do proprio kernel, nao do jogo: o retorno esta na RAM do kernel"
    );
}
