use std::process::Command;

#[test]
fn version_flag_prints_name_and_version() {
    let bin = env!("CARGO_BIN_EXE_psx-cli");
    let output = Command::new(bin)
        .arg("--version")
        .output()
        .expect("falhou ao executar psx-cli --version");
    assert!(output.status.success(), "exit code deve ser 0");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.starts_with("psx-cli "),
        "deve começar com 'psx-cli '; got: {stdout:?}"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.is_empty(),
        "stderr deve estar vazio; got: {stderr:?}"
    );
}
