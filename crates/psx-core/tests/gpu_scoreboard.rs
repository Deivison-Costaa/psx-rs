mod support;

use std::fs;

#[test]
fn fetch_test_exes_inclui_fonte_ps1_tests_que_prove_gpu() {
    let script = fs::read_to_string(support::repo_root().join("scripts/fetch-test-exes.ps1"))
        .expect("scripts/fetch-test-exes.ps1 deve existir");

    let tem_ps1_tests = script.contains("ps1-tests");
    assert!(
        tem_ps1_tests,
        "scripts/fetch-test-exes.ps1 nao contem a fonte 'ps1-tests'. \
         O tests.zip do ps1-tests inclui as suites cpu, gpu, gte, dma, timers, etc. \
         Sem esta fonte, os EXEs de GPU nao sao baixados."
    );

    let tem_dir_tests_exes = script.contains("tests/exes/ps1-tests");
    assert!(
        tem_dir_tests_exes,
        "scripts/fetch-test-exes.ps1 nao referencia 'tests/exes/ps1-tests'. \
         O diretorio de destino dos EXEs de GPU deve ser tests/exes/ps1-tests/gpu/."
    );
}

#[test]
fn scoreboard_varre_recursivamente_para_incluir_gpu() {
    let script = fs::read_to_string(support::repo_root().join("scripts/scoreboard.ps1"))
        .expect("scripts/scoreboard.ps1 deve existir");

    let varre_recursivo = script.contains("-Recurse");
    assert!(
        varre_recursivo,
        "scripts/scoreboard.ps1 nao usa -Recurse no Get-ChildItem. \
         Sem varredura recursiva, os EXEs em tests/exes/ps1-tests/gpu/ nao seriam \
         encontrados. A GPU do ps1-tests depende de varredura recursiva para aparecer \
         no scoreboard."
    );

    let raiz_e_tests_exes = script.contains("tests/exes");
    assert!(
        raiz_e_tests_exes,
        "scripts/scoreboard.ps1 nao referencia 'tests/exes'. \
         A raiz da varredura deve ser tests/exes/ para alcancar todas as suites, \
         incluindo ps1-tests/gpu/."
    );
}

#[test]
fn fetch_test_exes_ja_inclui_gpu_sem_precisar_de_nova_fonte() {
    let script = fs::read_to_string(support::repo_root().join("scripts/fetch-test-exes.ps1"))
        .expect("scripts/fetch-test-exes.ps1 deve existir");

    let fontes = script.matches("Url = ").count();
    assert!(
        fontes >= 3,
        "scripts/fetch-test-exes.ps1 tem apenas {} fonte(s) — esperado pelo menos 3 \
         (ps1-tests, amidog-cpu, amidog-gte). A GPU do ps1-tests ja esta inclusa no \
         tests.zip do ps1-tests; nao precisa de fonte separada.",
        fontes,
    );
}

#[test]
fn scoreboard_aceita_status_tty_para_gpu_sem_veredito_textual() {
    let script = fs::read_to_string(support::repo_root().join("scripts/scoreboard.ps1"))
        .expect("scripts/scoreboard.ps1 deve existir");

    let tem_tty = script.contains("tty");
    assert!(
        tem_tty,
        "scripts/scoreboard.ps1 nao classifica suites com saida TTY mas sem veredito \
         como 'tty'. A maioria dos EXEs de GPU do ps1-tests renderiza na VRAM em vez de \
         emitir 'pass -'/'fail -' no TTY; o status 'tty' e o rotulo correto para esses \
         casos, indicando que o EXE executou sem crash."
    );

    let tem_status_pass = script.contains("status='pass'") || script.contains("'pass'");
    let tem_status_fail = script.contains("status='fail'") || script.contains("'fail'");
    assert!(
        tem_status_pass && tem_status_fail,
        "scripts/scoreboard.ps1 nao classifica status 'pass'/'fail'. \
         As suites gp0-e1 e mask-bit do ps1-tests GPU emitem vereditos textuais; \
         o scoreboard precisa extrai-los e classificar corretamente."
    );
}

#[test]
fn scoreboard_cabecalho_generico_suporta_suite_gpu() {
    let script = fs::read_to_string(support::repo_root().join("scripts/scoreboard.ps1"))
        .expect("scripts/scoreboard.ps1 deve existir");

    let tem_cabecalho = script.contains("ts,commit,suite,exe,status,detalhe");
    assert!(
        tem_cabecalho,
        "scripts/scoreboard.ps1 nao define o cabecalho 'ts,commit,suite,exe,status,detalhe'. \
         A coluna 'suite' e generica (ex.: ps1-tests/gpu/triangle) e suporta qualquer \
         suite, incluindo GPU, sem precisar de coluna dedicada."
    );
}

#[test]
fn scoreboard_primeiro_tty_vem_antes_de_primeiro_sem_saida() {
    let script = fs::read_to_string(support::repo_root().join("scripts/scoreboard.ps1"))
        .expect("scripts/scoreboard.ps1 deve existir");

    let first_tty = script.find("tty,\"").unwrap_or(usize::MAX);
    let first_sem_saida = script.find("sem-saida,\"").unwrap_or(usize::MAX);
    assert!(
        first_tty < first_sem_saida,
        "scripts/scoreboard.ps1: a primeira ocorrencia de 'tty,\"' (pos {}) deve vir \
         antes da primeira ocorrencia de 'sem-saida,\"' (pos {}). \
         O bloco principal de classificacao TTY (if ttyBytes -gt 0) deve usar status \
         'tty', nao 'sem-saida'. Se o status 'tty' for removido deste bloco, suites \
         GPU sem veredito textual (maioria do ps1-tests) seriam incorretamente \
         classificadas como 'sem-saida'.",
        first_tty,
        first_sem_saida,
    );
}
