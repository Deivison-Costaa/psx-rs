use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn bins_dir() -> PathBuf {
    let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    d.push("tests");
    d.push("bins");
    d
}

fn write_bios(path: &PathBuf) {
    let mut data = vec![0u8; 0x80000];
    data[0x0000] = 0x3C;
    data[0x0001] = 0x1F;
    fs::write(path, &data).expect("escrever BIOS sintética");
}

#[test]
fn bios_flag_boota_bios_sintetica() {
    let bin = env!("CARGO_BIN_EXE_psx-cli");
    let bios_path = bins_dir().join("bios_test.bin");
    let _ = fs::create_dir_all(bins_dir());
    write_bios(&bios_path);

    let output = Command::new(bin)
        .arg("--bios")
        .arg(&bios_path)
        .output()
        .expect("falhou ao executar psx-cli --bios <path>");
    assert!(output.status.success(), "exit code deve ser 0");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("Runner:"),
        "--bios sozinho deve bootar; stderr={:?}",
        stderr
    );
    let _ = fs::remove_file(&bios_path);
}

#[test]
fn bios_flag_file_not_found() {
    let bin = env!("CARGO_BIN_EXE_psx-cli");
    let fake = bins_dir().join("nao_existe.bin");
    let output = Command::new(bin)
        .arg("--bios")
        .arg(&fake)
        .output()
        .expect("falhou ao executar psx-cli --bios <path inexistente>");
    assert!(!output.status.success(), "exit code deve ser != 0");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!stderr.is_empty(), "stderr deve conter mensagem de erro");
}
