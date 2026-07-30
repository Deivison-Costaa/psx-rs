use std::fs;
use std::process::Command;

#[test]
fn disc_flag_cue_minimo_aceito_com_bios() {
    let tmp = std::env::temp_dir().join("psx-cli-test-disc");
    let _ = fs::create_dir_all(&tmp);

    let cue_path = tmp.join("disc.cue");
    let bin_path = tmp.join("disc.bin");

    let cue = "FILE \"disc.bin\" BINARY\n  TRACK 01 MODE1/2352\n    INDEX 01 00:00:00\n";
    fs::write(&cue_path, cue.as_bytes()).expect("escrever CUE");

    let mut bin_data = vec![0u8; 2352];
    bin_data[0] = b'P';
    bin_data[1] = b'S';
    fs::write(&bin_path, &bin_data).expect("escrever BIN");

    let bios_path = tmp.join("bios.bin");
    fs::write(&bios_path, vec![0u8; 0x80000]).expect("escrever BIOS dummy");

    let binary = env!("CARGO_BIN_EXE_psx-cli");

    let output = Command::new(binary)
        .arg("--bios")
        .arg(&bios_path)
        .arg("--disc")
        .arg(&cue_path)
        .output()
        .expect("executar psx-cli --bios --disc");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    eprintln!("stdout: {stdout}\nstderr: {stderr}");

    assert!(
        output.status.success(),
        "G1: --bios --disc deve retornar codigo 0"
    );
    assert!(
        stdout.contains("DISCO: 1 faixa(s)"),
        "G1: stdout deve conter 'DISCO: 1 faixa(s)'; stdout={stdout:?}"
    );

    let _ = fs::remove_dir_all(&tmp);
}

#[test]
fn disc_flag_sem_bios_erro() {
    let tmp = std::env::temp_dir().join("psx-cli-test-disc-sem-bios");
    let _ = fs::create_dir_all(&tmp);

    let cue_path = tmp.join("disc.cue");
    let bin_path = tmp.join("disc.bin");

    let cue = "FILE \"disc.bin\" BINARY\n  TRACK 01 MODE1/2352\n    INDEX 01 00:00:00\n";
    fs::write(&cue_path, cue.as_bytes()).expect("escrever CUE");
    fs::write(&bin_path, vec![0u8; 2352]).expect("escrever BIN");

    let binary = env!("CARGO_BIN_EXE_psx-cli");

    let output = Command::new(binary)
        .arg("--disc")
        .arg(&cue_path)
        .output()
        .expect("executar psx-cli --disc");

    assert!(
        !output.status.success(),
        "G1b: --disc sem --bios deve sair com codigo != 0"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stderr.is_empty(),
        "G1b: stderr deve conter mensagem de erro"
    );

    let _ = fs::remove_dir_all(&tmp);
}

#[test]
fn disc_flag_arquivo_cue_inexistente_erro() {
    let tmp = std::env::temp_dir().join("psx-cli-test-disc-cue-nao-existe");
    let _ = fs::create_dir_all(&tmp);

    let bios_path = tmp.join("bios.bin");
    fs::write(&bios_path, vec![0u8; 0x80000]).expect("escrever BIOS dummy");

    let cue_path = tmp.join("nao_existe.cue");

    let binary = env!("CARGO_BIN_EXE_psx-cli");

    let output = Command::new(binary)
        .arg("--bios")
        .arg(&bios_path)
        .arg("--disc")
        .arg(&cue_path)
        .output()
        .expect("executar psx-cli --bios --disc com CUE inexistente");

    assert!(
        !output.status.success(),
        "G1c: CUE inexistente deve sair com codigo != 0"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stderr.is_empty(),
        "G1c: stderr deve conter mensagem de erro; stderr={stderr:?}"
    );

    let _ = fs::remove_dir_all(&tmp);
}
