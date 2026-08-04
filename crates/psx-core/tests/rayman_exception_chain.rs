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

fn words(bus: &Bus, address: u32) -> [u32; 4] {
    [
        bus.read32::<BusRead>(address),
        bus.read32::<BusRead>(address + 4),
        bus.read32::<BusRead>(address + 8),
        bus.read32::<BusRead>(address + 12),
    ]
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Node {
    priority: u32,
    address: u32,
    next: u32,
    second: u32,
    first: u32,
}

struct BiosCall {
    index: u32,
    step: usize,
    priority: u32,
    structure: u32,
    before: [u32; 4],
}

struct Activation {
    vector_step: usize,
    vector_stat: u32,
    nodes: Vec<Node>,
    handlers: Vec<(usize, u32)>,
    hook_step: usize,
    hook_stat: u32,
}

fn node(bus: &Bus, priority: u32, address: u32) -> Node {
    let values = words(bus, address);
    Node {
        priority,
        address,
        next: values[0],
        second: values[1],
        first: values[2],
    }
}

fn known_nodes(bus: &Bus, structures: &[(u32, u32)]) -> Vec<Node> {
    let mut nodes = Vec::new();
    for &(priority, address) in structures {
        if address == 0 || nodes.iter().any(|item: &Node| item.address == address) {
            continue;
        }
        let mut current = address;
        for _ in 0..32 {
            if current == 0 || nodes.iter().any(|item| item.address == current) {
                break;
            }
            let item = node(bus, priority, current);
            current = item.next;
            nodes.push(item);
        }
    }
    nodes
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

#[test]
fn caminhada_da_cadeia_muda_antes_do_quarto_hook() {
    let Some((mut bus, mut cpu)) = setup() else {
        eprintln!("BIOS ou disco Rayman nao encontrado — teste ignorado");
        return;
    };

    let mut b19 = None;
    let mut c02 = None;
    let mut c03 = None;
    let mut hook = None;
    let mut hook_install_step = None;
    let mut structures = Vec::new();
    let mut calls = Vec::new();
    let mut active: Option<Activation> = None;
    let mut activations = Vec::new();

    for step in 1..=500_000_000usize {
        if step == 1 || step % 4096 == 0 {
            b19 = b19.or_else(|| table_entry(&bus, 0xB0, 0x19));
            c02 = c02.or_else(|| table_entry(&bus, 0xC0, 0x02));
            c03 = c03.or_else(|| table_entry(&bus, 0xC0, 0x03));
        }

        let executed_pc = cpu.pc;
        if b19 == Some(executed_pc) {
            let structure_address = cpu.regs[4];
            let target = bus.read32::<BusRead>(structure_address);
            if target == GAME_HOOK {
                hook = Some(target);
                hook_install_step = Some(step);
            }
        }
        if c02 == Some(executed_pc) || c03 == Some(executed_pc) {
            let index = if c02 == Some(executed_pc) { 0x02 } else { 0x03 };
            let priority = cpu.regs[4];
            let structure = cpu.regs[5];
            if index == 0x02 {
                structures.push((priority, structure));
            }
            calls.push(BiosCall {
                index,
                step,
                priority,
                structure,
                before: words(&bus, structure),
            });
        }

        if let Some(interval) = active.as_mut() {
            if interval
                .nodes
                .iter()
                .any(|item| item.first == executed_pc || item.second == executed_pc)
                || executed_pc == VBLANK_HANDLER
            {
                interval.handlers.push((step, executed_pc));
            }
        }

        if hook == Some(executed_pc) {
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
            let nodes = known_nodes(&bus, &structures);
            active = Some(Activation {
                vector_step: step,
                vector_stat: bus.read32::<BusRead>(I_STAT),
                nodes,
                handlers: Vec::new(),
                hook_step: 0,
                hook_stat: 0,
            });
        }
    }

    assert_eq!(
        activations.len(),
        4,
        "quatro ativacoes devem ser observadas"
    );

    let inicial = &activations[0];
    let posterior = &activations[3];
    // Os quatro passos abaixo andaram +15.801 na 0185 (comandos de cor do GTE deixaram de ser
    // no-op e o Rayman os emite). Deslocamento uniforme, mesma sequencia — achado 10.115.
    // Segundo deslocamento uniforme de +6.372 na 0187: com o SPU vivo o SPUSTAT passou a
    // espelhar o SPUCNT e as esperas do kernel terminam em vez de girar (achado 10.115).
    assert_eq!(inicial.vector_step, 164_132_119);
    assert_eq!(inicial.hook_step, 164_132_949);
    assert_eq!(inicial.vector_stat & 1, 1);
    assert_eq!(inicial.hook_stat & 1, 1);
    assert_eq!(posterior.vector_step, 164_174_473);
    assert_eq!(posterior.hook_step, 164_178_575);
    assert_eq!(posterior.vector_stat & 1, 1);
    assert_eq!(posterior.hook_stat & 1, 0);

    let inicial_handlers: Vec<u32> = inicial
        .handlers
        .iter()
        .map(|(_, address)| *address)
        .collect();
    assert_eq!(
        inicial_handlers,
        vec![0x1A00, 0x18BC, 0x19C8, 0x1858, 0x17F4, 0x1794, 0x2458],
        "a ativacao 0 deve registrar a ordem dos handlers visitados"
    );
    assert!(!inicial_handlers.contains(&VBLANK_HANDLER));

    let posterior_handlers: Vec<u32> = posterior
        .handlers
        .iter()
        .map(|(_, address)| *address)
        .collect();
    assert_eq!(
        posterior_handlers,
        vec![
            0x1A00,
            0x18BC,
            0x19C8,
            0x1858,
            0x17F4,
            0x1794,
            0x4A4C,
            0x49BC,
            VBLANK_HANDLER,
            0x2458,
        ],
        "a ativacao 3 deve visitar 0x4A1C depois do novo elemento da cadeia"
    );

    assert!(inicial.nodes.iter().all(|node| node.address != 0x74A8));
    assert_eq!(
        posterior.nodes.iter().find(|node| node.address == 0x74A8),
        Some(&Node {
            priority: 2,
            address: 0x74A8,
            next: 0,
            second: 0x49BC,
            first: 0x4A4C,
        }),
        "C(02h) deve deixar o elemento de prioridade 2 observável na cadeia"
    );

    let install = hook_install_step.expect("HookEntryInt do jogo deve ser observado");
    let near_hook: Vec<(u32, u32, u32)> = calls
        .iter()
        .filter(|call| call.step.abs_diff(install) <= 2_000)
        .map(|call| (call.index, call.priority, call.structure))
        .collect();
    assert_eq!(
        near_hook,
        vec![
            (0x03, 0, 0xA000_91D0),
            (0x03, 0, 0xA000_91E0),
            (0x03, 0, 0xA000_91D0),
            (0x03, 0, 0xA000_91E0),
        ],
        "perto do hook so ha C(03h) de prioridade 0, nao a cadeia de VBlank"
    );

    let enqueue = calls
        .iter()
        .find(|call| {
            call.index == 0x02
                && call.priority == 2
                && call.structure == 0x74A8
                && call.step > inicial.hook_step
                && call.step < posterior.vector_step
        })
        .expect("o novo elemento de prioridade 2 deve ser enfileirado entre as ativacoes");
    assert_eq!(enqueue.before, [0, 0x49BC, 0x4A4C, 0]);
}
