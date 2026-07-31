use psx_core::bus::{Bios, Bus, BusRead, Ram};
use psx_core::cpu::Cpu;
use std::path::PathBuf;
use std::process::Command;

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

fn disc_path() -> Option<PathBuf> {
    {
        let candidate = "roms/extraido/Crash Bandicoot (USA).cue";
        let mut d = workspace_root();
        d.push(candidate);
        if d.exists() {
            return Some(d);
        }
    }
    let mut alt = workspace_root();
    alt.push("..");
    alt.push("roms");
    alt.push("extraido");
    alt.push("Crash Bandicoot (USA).cue");
    if alt.exists() {
        return Some(alt);
    }
    None
}

fn boot_and_read_bfunc_addresses() -> (u32, u32) {
    let bios_data = std::fs::read(bios_path()).expect("ler BIOS");
    let bios = Bios::from_bytes(bios_data).expect("BIOS valida");
    let ram = Ram::new();
    let mut bus = Bus::new(ram, bios);

    let disc = disc_path().expect("disco nao encontrado");
    let cue_text = std::fs::read_to_string(&disc).expect("ler CUE");
    let layout = psx_core::cdrom_bin_cue::parse_cue(&cue_text);
    let bin_dir = disc.parent().unwrap();
    let bin_data = std::fs::read(bin_dir.join(&layout.bin_path)).expect("ler BIN");

    bus.inject_disc(layout, bin_data);
    bus.cdrom_mut().insert_disc();

    let mut cpu = Cpu::new();
    for _ in 0..3_000_000 {
        cpu.step(&mut bus);
    }

    let w_b0 = bus.read32::<BusRead>(0x0000_00B0);
    let w_b4 = bus.read32::<BusRead>(0x0000_00B4);
    let imm_lui = w_b0 & 0xFFFF;
    let imm_addiu = w_b4 & 0xFFFF;
    let b_dispatch = ((imm_lui as u32) << 16).wrapping_add(imm_addiu as u32);

    let w_lui_disp = bus.read32::<BusRead>(b_dispatch);
    let w_addiu_disp = bus.read32::<BusRead>(b_dispatch + 4);
    let imm_lui_disp = w_lui_disp & 0xFFFF;
    let imm_addiu_disp = w_addiu_disp & 0xFFFF;
    let b_table = ((imm_lui_disp as u32) << 16).wrapping_add(imm_addiu_disp as u32);

    let wait_event_addr = bus.read32::<BusRead>(b_table + 0x0A * 4);
    let test_event_addr = bus.read32::<BusRead>(b_table + 0x0B * 4);

    assert!(wait_event_addr > 0, "WaitEvent addr nao pode ser zero");
    assert!(test_event_addr > 0, "TestEvent addr nao pode ser zero");

    (wait_event_addr, test_event_addr)
}

fn parse_step(part: &str) -> Option<u64> {
    part.strip_prefix("step=")?.parse::<u64>().ok()
}

#[test]
fn trace_wait_e_test_event_diagnostico() {
    if !bios_path().exists() {
        eprintln!("SKIP: BIOS nao encontrada");
        return;
    }
    if disc_path().is_none() {
        eprintln!("SKIP: disco nao encontrado");
        return;
    }
    let disc = disc_path().unwrap();

    let (wait_addr, test_addr) = boot_and_read_bfunc_addresses();
    eprintln!(
        "Enderecos: WaitEvent=0x{:08X} TestEvent=0x{:08X}",
        wait_addr, test_addr
    );

    let bin = env!("CARGO_BIN_EXE_psx-cli");
    let max_steps: usize = 150_000_000;

    let trace_addrs = format!("0x{:X},0x{:X}", wait_addr, test_addr);

    let output = Command::new(bin)
        .arg("--bios")
        .arg(bios_path().to_str().unwrap())
        .arg("--disc")
        .arg(disc.to_str().unwrap())
        .arg("--max-steps")
        .arg(max_steps.to_string())
        .arg("--trace-pcs")
        .arg(&trace_addrs)
        .output()
        .expect("executar psx-cli --bios --disc --max-steps --trace-pcs");

    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);

    let mut count_wait_event = 0u32;
    let mut count_test_event = 0u32;
    let mut last_wait_step: u64 = 0;
    let mut last_test_step: u64 = 0;

    for line in stderr.lines() {
        if !line.starts_with("trace pc=") {
            continue;
        }

        let is_wait = line.contains(&format!("pc=0x{:08X}", wait_addr))
            || line.contains(&format!("pc=0x{:08x}", wait_addr))
            || line.contains(&format!("pc=0x{:X}", wait_addr));
        let is_test = line.contains(&format!("pc=0x{:08X}", test_addr))
            || line.contains(&format!("pc=0x{:08x}", test_addr))
            || line.contains(&format!("pc=0x{:X}", test_addr));

        if !is_wait && !is_test {
            continue;
        }

        let mut step: Option<u64> = None;
        for part in line.split_whitespace() {
            if let Some(s) = parse_step(part) {
                step = Some(s);
                break;
            }
        }

        if is_wait {
            count_wait_event += 1;
            if let Some(s) = step {
                last_wait_step = s;
            }
        }
        if is_test {
            count_test_event += 1;
            if let Some(s) = step {
                last_test_step = s;
            }
        }
    }

    eprintln!("Diagnostico ({} passos):", max_steps);
    eprintln!(
        "  B(0Ah) WaitEvent = {} (ultimo: step {})",
        count_wait_event, last_wait_step
    );
    eprintln!(
        "  B(0Bh) TestEvent = {} (ultimo: step {})",
        count_test_event, last_test_step
    );
    eprintln!(
        "  TTY (ultimos 200 chars): {}",
        &stdout[stdout.len().saturating_sub(200)..]
    );

    let tty = stdout;
    assert!(
        tty.contains("PS-X Realtime Kernel"),
        "TTY deve conter 'PS-X Realtime Kernel'; TTY ({} bytes): {:?}",
        tty.len(),
        &tty[tty.len().saturating_sub(200)..]
    );

    assert!(
        count_test_event > 0,
        "B(0Bh) TestEvent deve ser chamado pelo menos uma vez em {} passos; \
         zero significa que o endereco esta errado ou o trace nao alcancou",
        max_steps
    );

    if count_wait_event == 0 {
        eprintln!(
            "DIAGNOSTICO: WaitEvent (B(0Ah)) NUNCA foi chamado em {} passos. \
             TestEvent foi chamado {} vezes, a ULTIMA no passo {}. \
             Os eventos de CD-ROM (EvCB[0] spec=10h e EvCB[5] spec=200h, \
             ambos status=4000h ready na iter 0126) permanecem nao consumidos. \
             HIPOTESE NAO MEDIDA: 'TestEvent retorna 0 (busy) para eventos \
             ready' — o discriminante em \
             evcb_status_checkpoints_discriminante (psx-core) decide entre \
             corrida (ready DEPOIS do ultimo TestEvent) e defeito (ready ANTES).",
            max_steps, count_test_event, last_test_step
        );
    } else {
        eprintln!(
            "DIAGNOSTICO: WaitEvent foi chamado {} vezes em {} passos.",
            count_wait_event, max_steps
        );
    }
}
