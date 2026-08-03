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

    let check_start = effective
        .iter()
        .position(|l| l.trim() == "check:")
        .expect("ci.yml deve ter job 'check:'");
    let check_end = effective[check_start..]
        .iter()
        .position(|l| {
            let t = l.trim();
            !t.is_empty()
                && t != "check:"
                && t != "name: check"
                && t != "runs-on: ubuntu-latest"
                && !t.starts_with("steps:")
                && t.ends_with(':')
                && !t.starts_with("-")
        })
        .map(|p| check_start + p)
        .unwrap_or(effective.len());
    let check_lines = &effective[check_start..check_end];

    let conditional = check_lines.iter().find(|l| l.starts_with("if:"));
    assert!(
        conditional.is_none(),
        "job check contém condicional ({:?}): um passo pulado não mede nada e ainda deixa o \
         job verde. O job check roda inteiro, sempre.",
        conditional
    );
    // `cargo test --all --doc` contem a substring `cargo test --all`, entao exigir a string
    // antiga passaria sozinha depois da troca para o nextest — verde oco. As duas linhas abaixo
    // sao exigidas explicitamente: a suite (nextest) e os doctests, que o nextest nao roda.
    for required in [
        "cargo fmt --all -- --check",
        "cargo clippy --all-targets -- -D warnings",
        "cargo nextest run --all-targets",
        "cargo test --all --doc",
    ] {
        assert!(
            effective.iter().any(|l| l.contains(required)),
            "ci.yml perdeu o passo obrigatório: {required}"
        );
    }
}

// Medido no PR #189: o passo de suite gasta 188 s compilando e linkando 145 binarios de teste e
// 14 s rodando — no runner os diagnosticos pesados saem em 5 ms porque BIOS e disco nao existem
// la. O custo e link, e debuginfo completo domina o link. Desligar no workflow, e nao no
// Cargo.toml, mantem o repositorio local depuravel.
#[test]
fn ci_nao_paga_debuginfo_para_linkar_os_binarios_de_teste() {
    let ci = fs::read_to_string(support::repo_root().join(".github/workflows/ci.yml"))
        .expect("ci.yml deve existir");
    let effective: Vec<&str> = ci
        .lines()
        .map(str::trim)
        .filter(|l| !l.starts_with('#'))
        .collect();

    for required in ["CARGO_PROFILE_DEV_DEBUG:", "CARGO_PROFILE_TEST_DEBUG:"] {
        let linha = effective
            .iter()
            .find(|l| l.starts_with(required))
            .unwrap_or_else(|| panic!("ci.yml deve definir {required} no `env:` do workflow"));
        assert!(
            linha.contains('0'),
            "{required} tem de valer 0; achei {linha:?}"
        );
    }
}
