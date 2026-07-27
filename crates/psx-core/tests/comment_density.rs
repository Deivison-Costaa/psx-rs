mod support;

use std::fs;

const HARD_LIMIT: f64 = 0.10;
const MIN_LINES: usize = 40;

#[test]
fn comentarios_ate_dez_por_cento_por_arquivo() {
    let mut violations = Vec::new();
    for path in support::rust_source_files() {
        let text = fs::read_to_string(&path).expect("lendo fonte");
        let (mut comment, mut code) = (0usize, 0usize);
        let mut in_block = false;
        for line in text.lines() {
            let t = line.trim();
            if t.is_empty() {
                continue;
            }
            if in_block {
                comment += 1;
                if t.contains("*/") {
                    in_block = false;
                }
            } else if t.starts_with("//") {
                comment += 1;
            } else if t.starts_with("/*") {
                comment += 1;
                in_block = !t.contains("*/");
            } else {
                code += 1;
            }
        }
        let total = comment + code;
        if total < MIN_LINES {
            continue;
        }
        let density = comment as f64 / total as f64;
        if density > HARD_LIMIT {
            violations.push(format!(
                "{} — {:.0}% ({comment}/{total})",
                support::relative(&path),
                density * 100.0
            ));
        }
    }
    assert!(
        violations.is_empty(),
        "R7: alvo de 5% de comentários por arquivo, teto duro de 10%. Justificativa de \
         decisão, divergência do ROADMAP e citação de spec moram em docs/iterations/ e \
         docs/orquestracao.md — não no código. Acima do teto: {violations:#?}"
    );
}
