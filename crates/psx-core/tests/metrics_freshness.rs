mod support;

use std::fs;

const MAX_LAG: usize = 1;

#[test]
fn metricas_acompanham_as_iteracoes() {
    let root = support::repo_root();
    let iterations = fs::read_dir(root.join("docs/iterations"))
        .expect("docs/iterations deve existir")
        .filter_map(|e| {
            let name = e.expect("entrada").file_name().into_string().expect("nome");
            (name.ends_with(".md") && name.chars().take(4).all(|c| c.is_ascii_digit()))
                .then_some(name)
        })
        .count();
    let rows = fs::read_to_string(root.join("docs/metricas.csv"))
        .expect("docs/metricas.csv deve existir")
        .lines()
        .skip(1)
        .filter(|l| !l.trim().is_empty())
        .count();
    assert!(
        iterations.saturating_sub(rows) <= MAX_LAG,
        "{iterations} docs de iteração mas só {rows} linhas em docs/metricas.csv (lag máximo \
         {MAX_LAG}). No gb-rs a série congelou por 26 PRs porque dependia de memória humana. \
         scripts/oc-iter.ps1 appenda a linha do trabalhador; iteração do orquestrador \
         appenda a própria linha no commit docs(iter)."
    );
}
