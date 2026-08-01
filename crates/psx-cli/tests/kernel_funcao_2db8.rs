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

fn carregar_bios_e_bootar(passos: usize) -> (Bus, Cpu) {
    let bios_data = std::fs::read(bios_path()).expect("ler BIOS");
    let bios = Bios::from_bytes(bios_data).expect("BIOS invalida");
    let ram = Ram::new();
    let mut bus = Bus::new(ram, bios);
    let mut cpu = Cpu::new();
    for _ in 0..passos {
        cpu.step(&mut bus);
    }
    (bus, cpu)
}

fn parse_a_table_target(bus: &Bus, i: u32) -> Option<u32> {
    let base = 0x0000_0200u32;
    let w0 = bus.read32::<BusRead>(base + i * 8);
    let w1 = bus.read32::<BusRead>(base + i * 8 + 4);
    let op0 = w0 >> 26;
    let op1 = w1 >> 26;

    if op0 != 0b001111 {
        return None;
    }
    let imm_lui = w0 & 0xFFFF;
    let hi16 = imm_lui << 16;

    if op1 == 0b000010 {
        let j_target = (w1 & 0x3FF_FFFF) << 2;
        Some(hi16 | j_target)
    } else if op1 == 0b001001 {
        let imm_addiu = w1 & 0xFFFF;
        Some(hi16.wrapping_add(imm_addiu))
    } else {
        let funct = w1 & 0x3F;
        if funct == 0b001000 {
            Some(hi16.wrapping_add(w1 & 0xFFFF))
        } else {
            None
        }
    }
}

fn parse_b_table_base(bus: &Bus) -> Option<u32> {
    let w_lui = bus.read32::<BusRead>(0x0000_00B0);
    let w_addiu = bus.read32::<BusRead>(0x0000_00B4);
    if (w_lui >> 26) != 0b001111 || (w_addiu >> 26) != 0b001001 {
        return None;
    }
    let b_dispatch = ((w_lui & 0xFFFF) << 16).wrapping_add(w_addiu & 0xFFFF);
    let w_lui2 = bus.read32::<BusRead>(b_dispatch);
    let w_addiu2 = bus.read32::<BusRead>(b_dispatch + 4);
    if (w_lui2 >> 26) != 0b001111 || (w_addiu2 >> 26) != 0b001001 {
        return None;
    }
    let base = ((w_lui2 & 0xFFFF) << 16).wrapping_add(w_addiu2 & 0xFFFF);
    if base == 0 || base >= 0x0020_0000 {
        None
    } else {
        Some(base)
    }
}

fn parse_c_table_base(bus: &Bus) -> Option<u32> {
    let w_lui = bus.read32::<BusRead>(0x0000_00C0);
    let w_addiu = bus.read32::<BusRead>(0x0000_00C4);
    if (w_lui >> 26) != 0b001111 || (w_addiu >> 26) != 0b001001 {
        return None;
    }
    let c_dispatch = ((w_lui & 0xFFFF) << 16).wrapping_add(w_addiu & 0xFFFF);
    let w_lui2 = bus.read32::<BusRead>(c_dispatch);
    let w_addiu2 = bus.read32::<BusRead>(c_dispatch + 4);
    if (w_lui2 >> 26) != 0b001111 || (w_addiu2 >> 26) != 0b001001 {
        return None;
    }
    let base = ((w_lui2 & 0xFFFF) << 16).wrapping_add(w_addiu2 & 0xFFFF);
    if base == 0 || base >= 0x0020_0000 {
        None
    } else {
        Some(base)
    }
}

#[test]
fn tabelas_kernel_nao_contem_2db8() {
    if !bios_path().exists() {
        eprintln!("SKIP: BIOS nao encontrada");
        return;
    }
    let (bus, _cpu) = carregar_bios_e_bootar(5_000_000);

    let mut a_encontros = Vec::new();
    for i in 0..0xC0u32 {
        if let Some(addr) = parse_a_table_target(&bus, i) {
            let phys = addr & 0x1FFF_FFFF;
            if phys == 0x2DB8 {
                a_encontros.push(i);
            }
            eprintln!(
                " A[0x{:02X}] @ 0x{:05X} -> 0x{:08X}",
                i,
                0x0200 + i * 8,
                addr
            );
        }
    }
    assert!(
        a_encontros.is_empty(),
        "0x2DB8 NAO deve estar na A-table; encontrado em A({:02X}h): {:?}",
        a_encontros.first().unwrap_or(&0),
        a_encontros
    );
    eprintln!("A-table: 0x2DB8 NAO presente (OK)");

    if let Some(c_table) = parse_c_table_base(&bus) {
        let mut c_encontros = Vec::new();
        for i in 0..0x20u32 {
            let val = bus.read32::<BusRead>(c_table + i * 4);
            if val == 0x2DB8 || val == 0x0000_2DB8 {
                c_encontros.push(i);
            }
            if val != 0 {
                eprintln!(" C[0x{:02X}] = 0x{:08X}", i, val);
            }
        }
        assert!(
            c_encontros.is_empty(),
            "0x2DB8 NAO deve estar na C-table; encontrado em C({:02X}h)",
            c_encontros.first().unwrap_or(&0)
        );
        eprintln!("C-table: 0x2DB8 NAO presente (OK)");
    }

    if let Some(b_table) = parse_b_table_base(&bus) {
        let mut b_encontros = Vec::new();
        for i in 0..0x60u32 {
            let val = bus.read32::<BusRead>(b_table + i * 4);
            if val == 0x2DB8 || val == 0x0000_2DB8 {
                b_encontros.push(i);
            }
        }
        assert!(
            b_encontros.is_empty(),
            "0x2DB8 NAO deve estar na B-table; encontrado em B({:02X}h)",
            b_encontros.first().unwrap_or(&0)
        );
        eprintln!("B-table: 0x2DB8 NAO presente (OK)");
    }
}

#[test]
fn trace_pcs_inclui_ra_do_chamador() {
    if !bios_path().exists() {
        eprintln!("SKIP: BIOS nao encontrada");
        return;
    }

    let bin = env!("CARGO_BIN_EXE_psx-cli");

    let output = Command::new(bin)
        .arg("--bios")
        .arg(bios_path().to_str().unwrap())
        .arg("--max-steps")
        .arg("500000")
        .arg("--trace-pcs")
        .arg("0xA0")
        .output()
        .expect("executar psx-cli --bios --max-steps --trace-pcs 0xA0");

    let stderr = String::from_utf8_lossy(&output.stderr);

    let trace_lines: Vec<&str> = stderr.lines().filter(|l| l.starts_with("trace pc=")).collect();
    assert!(
        !trace_lines.is_empty(),
        "Esperava pelo menos uma linha de trace para 0xA0 em 500k passos"
    );

    eprintln!("Linhas de trace: {}", trace_lines.len());
    for line in &trace_lines {
        eprintln!("  {}", line);
    }

    let tem_ra = stderr.contains("ra($31)");
    assert!(
        tem_ra,
        "O trace --trace-pcs deve incluir ra($31); \
         sem ele o diagnostico do chamador e cego"
    );
}
