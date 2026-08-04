mod support;

use std::fs;

// Entraram na 0194 (item 9.2, save states). As duas sao puras: `serde` so gera codigo de
// (de)serializacao e `bincode` so troca esse codigo por bytes — nenhuma das duas abre
// arquivo, soquete ou relogio. Quem grava o `.state` e o frontend.
const ALLOWED_DEPENDENCIES: &[&str] = &["serde", "bincode"];

#[test]
fn psx_core_sem_dependencias_fora_da_allowlist() {
    let manifest = fs::read_to_string(support::repo_root().join("crates/psx-core/Cargo.toml"))
        .expect("Cargo.toml do psx-core");
    let mut in_deps = false;
    let mut violations = Vec::new();
    for line in manifest.lines() {
        let line = line.trim();
        if line.starts_with('[') {
            in_deps = line == "[dependencies]";
            continue;
        }
        if !in_deps || line.is_empty() || line.starts_with('#') {
            continue;
        }
        let name = line
            .split(['=', ' ', '.'])
            .next()
            .expect("nome da dependência")
            .trim();
        if !ALLOWED_DEPENDENCIES.contains(&name) {
            violations.push(name.to_string());
        }
    }
    assert!(
        violations.is_empty(),
        "R3: psx-core deve ser puro (sem I/O e sem dependências fora da allowlist), para \
         rodar headless na CI e portar para qualquer frontend. Fora da allowlist \
         {ALLOWED_DEPENDENCIES:?}: {violations:?}. Se a dependência é mesmo necessária e \
         pura (ex.: serde para save states, item 9.2), adicione-a à allowlist neste teste \
         no mesmo PR, com justificativa no doc da iteração."
    );
}

#[test]
fn psx_core_proibe_unsafe() {
    let lib = fs::read_to_string(support::repo_root().join("crates/psx-core/src/lib.rs"))
        .expect("lib.rs do psx-core");
    assert!(
        lib.lines().any(|l| l.trim() == "#![forbid(unsafe_code)]"),
        "R6: lib.rs do psx-core deve conter #![forbid(unsafe_code)]."
    );
}
