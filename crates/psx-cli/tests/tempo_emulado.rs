use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const CICLOS_POR_SEGUNDO: u64 = 33_868_800;

fn bios() -> Option<PathBuf> {
    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../bios/SCPH1001.BIN");
    if p.exists() {
        Some(p)
    } else {
        eprintln!("SKIP: BIOS nao encontrada em '{}'", p.display());
        None
    }
}

fn psx_cli(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_psx-cli"))
        .args(args)
        .output()
        .expect("executar psx-cli")
}

fn com_bios(bios: &Path, args: &[&str]) -> Output {
    let mut todos = vec!["--bios", bios.to_str().expect("caminho utf-8")];
    todos.extend_from_slice(args);
    psx_cli(&todos)
}

fn stderr(o: &Output) -> String {
    String::from_utf8_lossy(&o.stderr).into_owned()
}

fn ciclos_emulados(o: &Output) -> u64 {
    stderr(o)
        .lines()
        .find_map(|l| l.strip_prefix("# ciclos emulados: "))
        .and_then(|n| n.trim().parse().ok())
        .expect("linha '# ciclos emulados: N'")
}

fn passos(o: &Output) -> u64 {
    stderr(o)
        .lines()
        .find_map(|l| l.strip_prefix("Runner: "))
        .and_then(|l| l.split_whitespace().next())
        .and_then(|n| n.parse().ok())
        .expect("linha 'Runner: N passos'")
}

fn pasta_temporaria(nome: &str) -> PathBuf {
    let d = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(nome);
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).expect("criar pasta temporaria");
    d
}

#[test]
fn max_time_para_no_ciclo_certo() {
    let Some(bios) = bios() else { return };
    let o = com_bios(&bios, &["--max-time", "0.01s"]);
    assert!(o.status.success(), "stderr={}", stderr(&o));
    let alvo = CICLOS_POR_SEGUNDO / 100;
    let ciclos = ciclos_emulados(&o);
    assert!(
        ciclos >= alvo && ciclos < alvo + 1_000,
        "--max-time 0.01s deve parar logo depois de {alvo} ciclos, parou em {ciclos}"
    );
}

#[test]
fn max_time_sem_sufixo_tambem_e_segundos() {
    let Some(bios) = bios() else { return };
    let o = com_bios(&bios, &["--max-time", "0.01"]);
    assert!(o.status.success(), "stderr={}", stderr(&o));
    assert!(ciclos_emulados(&o) >= CICLOS_POR_SEGUNDO / 100);
}

#[test]
fn max_steps_continua_valendo_junto_com_max_time() {
    let Some(bios) = bios() else { return };
    let o = com_bios(&bios, &["--max-steps", "1000", "--max-time", "10s"]);
    assert!(o.status.success(), "stderr={}", stderr(&o));
    assert_eq!(passos(&o), 1000, "o limite que vier primeiro vale");
}

#[test]
fn dump_vram_every_em_segundos_numera_pelo_tempo() {
    let Some(bios) = bios() else { return };
    let d = pasta_temporaria("dump_vram_segundos");
    let prefixo = d.join("t");
    let o = com_bios(
        &bios,
        &[
            "--max-time",
            "0.01s",
            "--dump-vram-every",
            "0.004s",
            prefixo.to_str().expect("utf-8"),
        ],
    );
    assert!(o.status.success(), "stderr={}", stderr(&o));
    assert!(d.join("t-1.vram").exists(), "dump aos 0,004 s");
    assert!(d.join("t-2.vram").exists(), "dump aos 0,008 s");
    assert!(!d.join("t-3.vram").exists(), "0,012 s passa do --max-time");
    assert!(
        !d.join("t-0.vram").exists(),
        "nada antes do primeiro periodo"
    );
}

#[test]
fn sample_pcs_em_segundos_amostra_por_ciclo() {
    let Some(bios) = bios() else { return };
    let o = com_bios(
        &bios,
        &[
            "--max-time",
            "0.005s",
            "--sample-pcs",
            "0.001s:0.0031s:0.001s",
        ],
    );
    assert!(o.status.success(), "stderr={}", stderr(&o));
    let amostras: Vec<u64> = stderr(&o)
        .lines()
        .filter_map(|l| l.split("cyc=").nth(1))
        .filter_map(|c| c.trim().parse().ok())
        .collect();
    assert_eq!(amostras.len(), 3, "amostras em 1, 2 e 3 ms: {amostras:?}");
    let ms = CICLOS_POR_SEGUNDO / 1000;
    for (k, c) in amostras.iter().enumerate() {
        let alvo = ms * (k as u64 + 1);
        assert!(
            *c >= alvo && *c < alvo + 200,
            "amostra {k} em {c}, esperado ~{alvo}"
        );
    }
}

#[test]
fn press_em_segundos_e_aceito() {
    let Some(bios) = bios() else { return };
    let o = com_bios(
        &bios,
        &[
            "--max-time",
            "0.01s",
            "--press",
            "start@0.002s:0.001s",
            "--press",
            "cross@0.004s",
            "--press",
            "circle@1000:500",
        ],
    );
    assert!(o.status.success(), "stderr={}", stderr(&o));
}

#[test]
fn press_com_inicio_em_segundos_e_duracao_em_passos_e_recusado() {
    let o = psx_cli(&["--press", "start@1s:100"]);
    assert!(!o.status.success());
    assert!(
        stderr(&o).contains("duracao em segundos"),
        "stderr={}",
        stderr(&o)
    );
}

#[test]
fn press_com_segundos_invalidos_e_recusado() {
    let o = psx_cli(&["--press", "start@xs"]);
    assert!(!o.status.success());
    assert!(stderr(&o).contains("segundos"), "stderr={}", stderr(&o));
}

#[test]
fn swap_disc_mistura_passos_e_segundos_e_recusado() {
    let o = psx_cli(&["--swap-disc", "disco.cue@30s:100"]);
    assert!(!o.status.success());
    assert!(stderr(&o).contains("mistura"), "stderr={}", stderr(&o));
}

#[test]
fn swap_disc_em_segundos_age_no_ciclo() {
    let Some(bios) = bios() else { return };
    let o = com_bios(
        &bios,
        &[
            "--max-time",
            "0.01s",
            "--open-lid",
            "0.002s",
            "--close-lid",
            "0.004s",
        ],
    );
    assert!(o.status.success(), "stderr={}", stderr(&o));
    let ciclos: Vec<u64> = stderr(&o)
        .lines()
        .filter(|l| l.starts_with("# porta:"))
        .filter_map(|l| l.split("cyc=").nth(1))
        .filter_map(|c| c.trim().parse().ok())
        .collect();
    let ms = CICLOS_POR_SEGUNDO / 1000;
    assert_eq!(ciclos.len(), 2, "abre e fecha: {ciclos:?}");
    assert!(ciclos[0] >= 2 * ms && ciclos[0] < 2 * ms + 200);
    assert!(ciclos[1] >= 4 * ms && ciclos[1] < 4 * ms + 200);
}

#[test]
fn dump_vram_every_zero_segundos_e_recusado() {
    let o = psx_cli(&["--dump-vram-every", "0s", "x"]);
    assert!(!o.status.success());
    assert!(
        stderr(&o).contains("maior que zero"),
        "stderr={}",
        stderr(&o)
    );
}
