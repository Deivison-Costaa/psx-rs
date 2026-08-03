use std::path::PathBuf;
use std::process::Command;

fn workspace_bios() -> PathBuf {
    let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    d.push("..");
    d.push("..");
    d.push("bios");
    d.push("SCPH1001.BIN");
    d
}

/// `0x80000100` e a tabela de entrypoints das cadeias de excecao (ExCB), escrita pelo kernel
/// durante o boot — endereco que a BIOS mexe com certeza e cedo.
const ALVO: &str = "0x80000100";

fn roda(args: &[&str]) -> String {
    let bin = env!("CARGO_BIN_EXE_psx-cli");
    let out = Command::new(bin)
        .args(args)
        .output()
        .expect("executar psx-cli");
    assert!(
        out.status.success(),
        "psx-cli deve sair com 0; stderr={:?}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stderr).into_owned()
}

#[test]
fn watch_mem_atribui_a_escrita_ao_pc_que_a_fez() {
    let bios = workspace_bios();
    if !bios.exists() {
        eprintln!("SKIP: BIOS nao encontrada em '{}'", bios.display());
        return;
    }
    let bios = bios.to_string_lossy().into_owned();

    let saida = roda(&[
        "--bios",
        &bios,
        "--max-steps",
        "3000000",
        "--watch-mem",
        ALVO,
        "--dump-mem",
        ALVO,
        "0x4",
    ]);

    let linhas: Vec<&str> = saida
        .lines()
        .filter(|l| l.starts_with("watch 80000100:"))
        .collect();
    assert!(
        !linhas.is_empty(),
        "o kernel escreve em {ALVO} durante o boot; sem nenhuma linha `watch` o instrumento \
         nao esta observando nada. saida={saida:?}"
    );

    // Toda linha tem de nomear um PC e mostrar a troca de valor.
    for l in &linhas {
        assert!(
            l.contains("pc=0x") && l.contains("de=0x") && l.contains("para=0x"),
            "linha de watch sem pc/de/para: {l:?}"
        );
    }

    // O ultimo valor observado tem de ser o mesmo que o `--dump-mem` reporta no fim: se
    // divergirem, o instrumento perdeu uma escrita e nao serve para atribuir defeito.
    let ultimo = linhas
        .last()
        .and_then(|l| l.split("para=0x").nth(1))
        .map(|s| s.trim().to_string())
        .expect("ultima linha de watch com para=");
    let dumpado = saida
        .lines()
        .skip_while(|l| !l.starts_with("dump 80000100:"))
        .nth(1)
        .and_then(|l| l.split_whitespace().nth(1))
        .map(|s| s.to_string())
        .expect("dump de 0x80000100");
    assert_eq!(
        ultimo.to_uppercase(),
        dumpado.to_uppercase(),
        "o ultimo valor observado pelo watch tem de bater com o dump final"
    );
}

#[test]
fn watch_mem_recusa_endereco_invalido() {
    let bin = env!("CARGO_BIN_EXE_psx-cli");
    let out = Command::new(bin)
        .args(["--watch-mem", "naoehex"])
        .output()
        .expect("executar psx-cli");
    assert!(
        !out.status.success(),
        "endereco invalido tem de reprovar em vez de observar o endereco errado em silencio"
    );
    assert!(
        String::from_utf8_lossy(&out.stderr).contains("--watch-mem"),
        "a mensagem de erro tem de nomear a flag"
    );
}
