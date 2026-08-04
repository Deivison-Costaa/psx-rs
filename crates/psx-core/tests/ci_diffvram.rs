mod support;

use std::fs;

fn scoreboard() -> String {
    fs::read_to_string(support::repo_root().join("scripts/scoreboard.ps1"))
        .expect("scripts/scoreboard.ps1 deve existir")
}

#[test]
fn scoreboard_procura_gabarito_vram_ao_lado_do_exe() {
    let script = scoreboard();
    assert!(
        script.contains("vram.png"),
        "scripts/scoreboard.ps1 nao procura o gabarito 'vram.png' ao lado do EXE. E ele que \
         carrega o retrato de hardware real do ps1-tests; sem procura-lo, 45/51 suites ficam \
         sem veredito grafico (achado 10.23)."
    );
    assert!(
        script.contains("--dump-vram"),
        "scripts/scoreboard.ps1 nao pede '--dump-vram' ao runner quando ha gabarito; sem o \
         retrato do emulador nao ha o que comparar."
    );
}

#[test]
fn scoreboard_compara_com_diffvram_e_registra_pixels() {
    let script = scoreboard();
    assert!(
        script.contains("diffvram"),
        "scripts/scoreboard.ps1 nao invoca a tool 'diffvram' \
         (tests/exes/ps1-tests/tools/diffvram/) para comparar o retrato com o gabarito."
    );
    assert!(
        script.contains("vram-ok") && script.contains("vram-diff"),
        "scripts/scoreboard.ps1 precisa dos status 'vram-ok' (identico ao hardware) e \
         'vram-diff' (K pixels divergentes) para o CSV ter veredito grafico."
    );
    assert!(
        script.contains("pixels"),
        "scripts/scoreboard.ps1 nao extrai a contagem de pixels da saida do diffvram \
         ('Images are different (N pixels)'); a coluna detalhe deve registrar 'Npx'."
    );
}

#[test]
fn veredito_textual_tem_prioridade_sobre_o_grafico() {
    let script = scoreboard();
    let pos_textual = script
        .find("veredito textual tem prioridade")
        .unwrap_or(usize::MAX);
    assert!(
        pos_textual != usize::MAX,
        "scripts/scoreboard.ps1 deve declarar (e cumprir) que o veredito textual \
         'pass -'/'fail -' tem prioridade sobre o grafico: gp0-e1 e mask-bit ja dao \
         veredito proprio e nao podem regredir para 'vram-*'."
    );
}

#[test]
fn psx_cli_converte_vram_crua_para_png() {
    let main = fs::read_to_string(support::repo_root().join("crates/psx-cli/src/main.rs"))
        .expect("crates/psx-cli/src/main.rs deve existir");
    assert!(
        main.contains("--vram-to-png"),
        "psx-cli precisa do modo '--vram-to-png <entrada.vram> <saida.png>': o diffvram so \
         compara PNG, e o dump do runner e cru (1024x512 halfwords BGR555 LE)."
    );
}
