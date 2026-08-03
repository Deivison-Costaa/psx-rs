use psx_core::bus::{Bios, Bus, BusRead, Ram};
use psx_core::cpu::Cpu;

const I_STAT: u32 = 0x1F80_1070;
const VBLANK_HANDLER: u32 = 0x0000_4A1C;
const GAME_HOOK: u32 = 0x801B_8E60;

fn optional_path<'a>(candidates: &'a [&'a str]) -> Option<&'a str> {
    candidates
        .iter()
        .copied()
        .find(|path| std::path::Path::new(path).exists())
}

fn b_table_base(bus: &Bus) -> Option<u32> {
    let lui = bus.read32::<BusRead>(0xB0);
    let addiu = bus.read32::<BusRead>(0xB4);
    if lui >> 26 != 0b001111 || addiu >> 26 != 0b001001 {
        return None;
    }
    let dispatch = ((lui & 0xFFFF) << 16).wrapping_add(addiu & 0xFFFF);
    let lui = bus.read32::<BusRead>(dispatch);
    let addiu = bus.read32::<BusRead>(dispatch + 4);
    if lui >> 26 != 0b001111 || addiu >> 26 != 0b001001 {
        return None;
    }
    Some(((lui & 0xFFFF) << 16).wrapping_add(addiu & 0xFFFF))
}

fn b_table_entry(bus: &Bus, index: u32) -> Option<u32> {
    let base = b_table_base(bus)?;
    let target = bus.read32::<BusRead>(base + index * 4);
    (target != 0).then_some(target)
}

fn sign_extend(instr: u32) -> u32 {
    (instr as i16 as i32) as u32
}

#[derive(Debug, PartialEq, Eq)]
struct IStatWrite {
    pc: u32,
    value: u32,
}

fn i_stat_store(cpu: &Cpu, bus: &Bus) -> Option<IStatWrite> {
    let instr = bus.read32::<BusRead>(cpu.pc);
    let op = instr >> 26;
    if !matches!(op, 0x28 | 0x29 | 0x2A | 0x2B | 0x2E) {
        return None;
    }
    let rs = ((instr >> 21) & 0x1F) as usize;
    let addr = cpu.regs[rs].wrapping_add(sign_extend(instr));
    if !(I_STAT..I_STAT + 4).contains(&(addr & 0x1FFF_FFFF)) {
        return None;
    }
    let rt = ((instr >> 16) & 0x1F) as usize;
    let value = if op == 0x29 {
        cpu.regs[rt] & 0xFFFF
    } else {
        cpu.regs[rt]
    };
    Some(IStatWrite { pc: cpu.pc, value })
}

#[derive(Debug)]
struct Activation {
    vector_step: usize,
    vector_stat: u32,
    writes: Vec<IStatWrite>,
    handler_step: Option<usize>,
    hook_step: usize,
    hook_stat: u32,
}

#[test]
fn ativacao_inicial_do_hook_preserva_vblank_antes_do_handler() {
    let bios_path = match optional_path(&["bios/SCPH1001.BIN", "../../bios/SCPH1001.BIN"]) {
        Some(p) => p,
        None => {
            eprintln!("BIOS nao encontrada — teste ignorado");
            return;
        }
    };
    let bios = Bios::from_bytes(std::fs::read(bios_path).expect("ler BIOS SCPH1001.BIN"))
        .expect("BIOS SCPH1001.BIN valida");
    let disc_path = match optional_path(&[
        "../roms/extraido/Rayman (USA) DADOS.cue",
        "../../roms/extraido/Rayman (USA) DADOS.cue",
        "../../../roms/extraido/Rayman (USA) DADOS.cue",
    ]) {
        Some(p) => p,
        None => {
            eprintln!("disco Rayman nao encontrado — teste ignorado");
            return;
        }
    };
    let cue = std::fs::read_to_string(disc_path).expect("ler CUE do Rayman");
    let layout = psx_core::cdrom_bin_cue::parse_cue(&cue);
    let bin_dir = std::path::Path::new(disc_path)
        .parent()
        .expect("CUE deve ter diretorio");
    let bin = std::fs::read(bin_dir.join(&layout.bin_path)).expect("ler BIN do Rayman");

    let mut bus = Bus::new(Ram::new(), bios);
    bus.inject_disc(layout, bin);
    bus.cdrom_mut().insert_disc();
    let mut cpu = Cpu::new();
    let mut b19_target = None;
    let mut hook_target = None;
    let mut active: Option<Activation> = None;
    let mut activations = Vec::new();

    for step in 1..=500_000_000usize {
        if step == 1 || step % 4096 == 0 {
            b19_target = b19_target.or_else(|| b_table_entry(&bus, 0x19));
        }
        let executed_pc = cpu.pc;
        if b19_target == Some(executed_pc) {
            let target = bus.read32::<BusRead>(cpu.regs[4]);
            if target == GAME_HOOK {
                hook_target = Some(target);
            }
        }

        if let Some(interval) = active.as_mut() {
            if executed_pc == VBLANK_HANDLER {
                interval.handler_step = Some(step);
            }
            if let Some(write) = i_stat_store(&cpu, &bus) {
                interval.writes.push(write);
            }
        }
        if hook_target == Some(executed_pc) {
            let mut interval = active.take().expect("hook deve ter vetorizacao anterior");
            interval.hook_step = step;
            interval.hook_stat = bus.read32::<BusRead>(I_STAT);
            activations.push(interval);
            if activations.len() == 4 {
                break;
            }
        }

        let entries_before = cpu.irq_handler_entries;
        cpu.step(&mut bus);
        if cpu.irq_handler_entries > entries_before {
            active = Some(Activation {
                vector_step: step,
                vector_stat: bus.read32::<BusRead>(I_STAT),
                writes: Vec::new(),
                handler_step: None,
                hook_step: 0,
                hook_stat: 0,
            });
        }
    }

    assert_eq!(
        activations.len(),
        4,
        "quatro ativacoes do hook devem ser observadas"
    );
    let inicial = &activations[0];
    // +15.801 na 0185: comandos de cor do GTE deixaram de ser no-op e o Rayman os emite.
    // Deslocamento uniforme, mesma sequencia de ativacoes — achado 10.115.
    assert_eq!(inicial.vector_step, 164_125_747);
    assert_eq!(inicial.hook_step, 164_126_577);
    assert_ne!(inicial.vector_stat & 1, 0);
    assert_ne!(inicial.hook_stat & 1, 0);
    assert_eq!(inicial.handler_step, None);
    assert_eq!(
        inicial.writes,
        vec![IStatWrite {
            pc: 0x0000_2710,
            value: 0xFFFF_FFFF,
        }]
    );

    let posterior = &activations[3];
    assert_ne!(posterior.vector_stat & 1, 0);
    assert_eq!(posterior.hook_stat & 1, 0);
    assert_eq!(posterior.handler_step, Some(164_171_976));
    assert!(posterior.handler_step.unwrap_or(usize::MAX) < posterior.hook_step);
    assert_eq!(
        posterior.writes,
        vec![
            IStatWrite {
                pc: 0x0000_45A8,
                value: 0xFFFF_FF7F,
            },
            IStatWrite {
                pc: 0x0000_45A8,
                value: 0xFFFF_FF7F,
            },
            IStatWrite {
                pc: VBLANK_HANDLER,
                value: 0xFFFF_FFFE,
            },
            IStatWrite {
                pc: 0x0000_2710,
                value: 0xFFFF_FFFF,
            },
        ]
    );
}
