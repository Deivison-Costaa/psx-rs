use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn bins_dir() -> PathBuf {
    let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    d.push("tests");
    d.push("bins");
    d
}

fn workspace_bios() -> PathBuf {
    let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    d.push("..");
    d.push("..");
    d.push("bios");
    d.push("SCPH1001.BIN");
    d
}

fn j(opcode: u32, target_addr: u32) -> u32 {
    (opcode << 26) | ((target_addr >> 2) & 0x03FF_FFFF)
}

fn nop() -> u32 {
    0x0000_0000
}

fn build_ps_exe(code: &[u32], dest_addr: u32, initial_pc: u32) -> Vec<u8> {
    let header_size = 0x800;
    let body_words = code.len();
    let body_size = ((body_words * 4) + 0x7FF) & !0x7FF;

    let mut data = vec![0u8; header_size + body_size];
    data[0..8].copy_from_slice(b"PS-X EXE");

    data[0x10..0x14].copy_from_slice(&initial_pc.to_le_bytes());
    data[0x14..0x18].copy_from_slice(&0u32.to_le_bytes());
    data[0x18..0x1C].copy_from_slice(&dest_addr.to_le_bytes());
    data[0x1C..0x20].copy_from_slice(&(body_size as u32).to_le_bytes());
    data[0x30..0x34].copy_from_slice(&0x801F_FFF0u32.to_le_bytes());
    data[0x34..0x38].copy_from_slice(&0u32.to_le_bytes());

    for (i, &word) in code.iter().enumerate() {
        let pos = header_size + i * 4;
        data[pos..pos + 4].copy_from_slice(&word.to_le_bytes());
    }

    data
}

#[test]
fn sample_pcs_amostra_o_loop_na_janela() {
    let bios_path = workspace_bios();

    if !bios_path.exists() {
        eprintln!("SKIP: BIOS nao encontrada");
        return;
    }

    let _ = fs::create_dir_all(bins_dir());

    let code_addr: u32 = 0x8000_0000;
    let loop_addr: u32 = 0x8000_0000;
    let code = [j(0x02, loop_addr), nop()];
    let exe_data = build_ps_exe(&code, code_addr, code_addr);

    let exe_path = bins_dir().join("sample_pcs_loop.psexe");
    fs::write(&exe_path, &exe_data).expect("escrever EXE sintetico");

    let bin = env!("CARGO_BIN_EXE_psx-cli");

    let output = Command::new(bin)
        .arg("--bios")
        .arg(&bios_path)
        .arg("--exe")
        .arg(&exe_path)
        .arg("--max-steps")
        .arg("64")
        .arg("--sample-pcs")
        .arg("16:48:8")
        .output()
        .expect("executar psx-cli com --sample-pcs");

    assert!(
        output.status.success(),
        "psx-cli deve aceitar --sample-pcs e sair com exit code 0; exit={} stderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    let samples: Vec<&str> = stderr
        .lines()
        .filter(|l| l.starts_with("sample pc="))
        .collect();

    assert_eq!(
        samples.len(),
        5,
        "janela 16..48 com passo 8 amostra nos steps 16,24,32,40,48 (5 amostras); stderr:\n{}",
        stderr
    );

    for line in &samples {
        assert!(
            line.contains("pc=0x80000000") || line.contains("pc=0x80000004"),
            "toda amostra deve cair no loop de 2 instrucoes (0x80000000/0x80000004); linha: {}",
            line
        );
    }
}
