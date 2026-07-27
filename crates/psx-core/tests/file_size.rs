mod support;

use std::fs;

const MAX_LINES: usize = 500;

#[test]
fn testes_ate_quinhentas_linhas() {
    let mut violations = Vec::new();
    for path in support::rust_test_files() {
        let lines = fs::read_to_string(&path)
            .expect("lendo teste")
            .lines()
            .count();
        if lines > MAX_LINES {
            violations.push(format!("{} — {lines} linhas", support::relative(&path)));
        }
    }
    assert!(
        violations.is_empty(),
        "R8: o teto de {MAX_LINES} linhas vale para ARQUIVO DE TESTE, não para fonte. O \
         agente lê o teste do item inteiro a cada iteração que o toca, e no gb-rs os testes \
         chegaram a 41% do contexto por turno. Um arquivo de teste por item do ROADMAP, não \
         um por subsistema: fatie e registre em docs/mapa.md. Fonte grande é decisão de \
         coesão, não de contagem, e não é reprovada aqui: {violations:#?}"
    );
}
