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
fn max_steps_limita_o_runner() {
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
        .arg("--max-steps")
        .arg("1000")
        .output()
        .expect("executar psx-cli --bios <BIOS> --max-steps 1000");

    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(output.status.success(), "exit code 0; stderr={stderr:?}");

    assert!(
        stderr.contains("passos"),
        "stderr deve conter contagem de passos; stderr={stderr:?}"
    );

    let steps_line = stderr
        .lines()
        .find(|l| l.contains("passos"))
        .expect("linha 'Runner: N passos' deve existir");
    let steps_str = steps_line
        .split_whitespace()
        .nth(1)
        .expect("stderr deve ter 'Runner: <N> passos'");
    let steps: usize = steps_str
        .parse()
        .expect("o numero de passos deve ser um inteiro");
    assert!(
        steps <= 1000,
        "--max-steps 1000 deve limitar a {steps} passos, nao mais"
    );
    assert!(
        steps > 0,
        "--max-steps 1000 deve executar pelo menos 1 passo"
    );
}
