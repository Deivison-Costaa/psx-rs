mod support;

use std::fs;

const MAX_BYTES: u64 = 10_000;

#[test]
fn roadmap_dentro_do_teto() {
    let path = support::repo_root().join("ROADMAP.md");
    let size = fs::metadata(&path).expect("ROADMAP.md deve existir").len();
    assert!(
        size <= MAX_BYTES,
        "ROADMAP.md com {size} bytes (teto {MAX_BYTES}). O roadmap é uma escada de \
         checkboxes de uma linha — no gb-rs ele virou 29 KB de prosa e um segundo corpo \
         narrativo. Justificativa e contexto de cada item moram em docs/iterations/NNNN-*.md."
    );
}
