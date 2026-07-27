mod support;

use std::fs;

#[test]
fn job_check_sem_condicionais_e_com_os_tres_passos() {
    let ci = fs::read_to_string(support::repo_root().join(".github/workflows/ci.yml"))
        .expect("ci.yml deve existir");
    let effective: Vec<&str> = ci
        .lines()
        .map(str::trim)
        .filter(|l| !l.starts_with('#'))
        .collect();
    let conditional = effective.iter().find(|l| l.starts_with("if:"));
    assert!(
        conditional.is_none(),
        "ci.yml contém condicional ({:?}): um passo pulado não mede nada e ainda deixa o \
         job verde. O job check roda inteiro, sempre.",
        conditional
    );
    for required in [
        "cargo fmt --all -- --check",
        "cargo clippy --all-targets -- -D warnings",
        "cargo test --all",
    ] {
        assert!(
            effective.iter().any(|l| l.contains(required)),
            "ci.yml perdeu o passo obrigatório: {required}"
        );
    }
}
