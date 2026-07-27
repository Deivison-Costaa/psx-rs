mod support;

use std::fs;

const MAX_BYTES: u64 = 16_000;

#[test]
fn status_dentro_do_teto() {
    let path = support::repo_root().join("STATUS.md");
    let size = fs::metadata(&path).expect("STATUS.md deve existir").len();
    assert!(
        size <= MAX_BYTES,
        "STATUS.md com {size} bytes (teto {MAX_BYTES}). Ele entra em contexto em TODA \
         iteração, então cada byte é pago vezes o número de turnos (no gb-rs chegou a 107 KB \
         e 96% era história no caminho quente). Estado atual fica; narrativa vai para \
         docs/iterations/ ou docs/orquestracao.md; índices apontam, não repetem."
    );
}
