use std::path::PathBuf;
use std::process::Command;

fn workspace_bios() -> PathBuf {
    let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    d.push("..");
    d.push("..");
    d.push("bios");
    d.push("SCPH1001.BIN");
    d
}

#[test]
fn bios_alone_boota_e_tty_contem_kernel_id() {
    let bios_path = workspace_bios();

    if !bios_path.exists() {
        eprintln!(
            "SKIP: BIOS nao encontrada em '{}' — CI nao tem imagem de BIOS",
            bios_path.display()
        );
        return;
    }

    let bin = env!("CARGO_BIN_EXE_psx-cli");

    let output = Command::new(bin)
        .arg("--bios")
        .arg(&bios_path)
        .output()
        .expect("executar psx-cli --bios <BIOS>");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        output.status.success(),
        "--bios sozinho deve sair com exit code 0; stderr={:?}",
        stderr
    );

    assert!(
        stdout.contains("PS-X Realtime Kernel"),
        "TTY deve conter 'PS-X Realtime Kernel' apos boot da BIOS; TTY={:?}",
        stdout
    );
}
