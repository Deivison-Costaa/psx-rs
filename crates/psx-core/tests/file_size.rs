mod support;

use std::fs;

const MAX_LINES: usize = 500;

#[test]
fn fontes_ate_quinhentas_linhas() {
    let mut violations = Vec::new();
    for path in support::rust_source_files() {
        let lines = fs::read_to_string(&path)
            .expect("lendo fonte")
            .lines()
            .count();
        if lines > MAX_LINES {
            violations.push(format!("{} — {lines} linhas", support::relative(&path)));
        }
    }
    assert!(
        violations.is_empty(),
        "R8: arquivo fonte com mais de {MAX_LINES} linhas obriga o agente a pagar o arquivo \
         inteiro em contexto a cada iteração que o toca (no gb-rs, mcycle.rs chegou a 1.656 \
         linhas e testes eram 41% do contexto por turno). Fatie em submódulos e atualize \
         docs/mapa.md: {violations:#?}"
    );
}
