mod support;

use std::fs;

const CAMINHO_CHECK_RUNS: &str = "commits/$sha/check-runs\"";

fn oc_loop_content() -> String {
    let path = support::repo_root().join("scripts/oc-loop.ps1");
    fs::read_to_string(&path).expect("scripts/oc-loop.ps1 deve existir")
}

fn wait_checks_body(script: &str) -> &str {
    let start = script
        .find("function Wait-Checks")
        .expect("Wait-Checks deve existir");
    let rest = &script[start..];
    let end = rest
        .find("\nforeach")
        .expect("foreach deve aparecer apos Wait-Checks");
    &rest[..end]
}

fn apos_gh_pr_merge(script: &str) -> &str {
    let pos = script
        .find("gh pr merge")
        .expect("oc-loop.ps1 deve conter gh pr merge");
    &script[pos..]
}

#[test]
fn wait_checks_consulta_check_runs_antes_de_mergestatestatus() {
    let script = oc_loop_content();
    let body = wait_checks_body(&script);

    let check_runs_pos = body.find(CAMINHO_CHECK_RUNS).unwrap_or_else(|| {
        panic!(
            "Wait-Checks deve consultar exatamente {CAMINHO_CHECK_RUNS} do headRefOid atual. \
             Caminho ausente ou renomeado."
        )
    });
    let merge_state_pos = body
        .find("mergeStateStatus")
        .expect("Wait-Checks deve consultar mergeStateStatus");

    assert!(
        check_runs_pos < merge_state_pos,
        "Wait-Checks deve consultar check-runs ANTES de mergeStateStatus. \
         Logo apos um push, mergeStateStatus responde com o estado do commit ANTERIOR, \
         e foi assim que os PRs #106 e #107 foram anunciados como mergeados estando abertos. \
         Ordem atual: mergeStateStatus em {merge_state_pos}, check-runs em {check_runs_pos}."
    );
}

#[test]
fn wait_checks_espera_enquanto_algum_check_esta_pendente() {
    let script = oc_loop_content();
    let body = wait_checks_body(&script);

    let pendente_pos = body
        .find("PENDENTE")
        .expect("Wait-Checks deve marcar check sem conclusao como PENDENTE");
    let merge_state_pos = body
        .find("mergeStateStatus")
        .expect("Wait-Checks deve consultar mergeStateStatus");

    assert!(
        pendente_pos < merge_state_pos,
        "a guarda de PENDENTE deve vir ANTES de mergeStateStatus (posicoes {pendente_pos} e {merge_state_pos})"
    );
    let entre = &body[pendente_pos..merge_state_pos];
    assert!(
        entre.contains("continue"),
        "com algum check PENDENTE, Wait-Checks tem de CONTINUAR esperando: \
         falta um `continue` entre a guarda de PENDENTE e a consulta a mergeStateStatus. \
         Sem ele o loop segue com os checks ainda rodando e o merge e recusado \
         com 'required status checks are expected'."
    );
}

#[test]
fn gh_pr_merge_verifica_estado_merged_apos_o_merge() {
    let script = oc_loop_content();
    let apos = apos_gh_pr_merge(&script);

    assert!(
        apos.contains("gh pr view"),
        "apos gh pr merge o script tem de RELER o PR com `gh pr view`; sem releitura \
         nao ha como saber se o merge aconteceu"
    );
    assert!(
        apos.contains("--json state"),
        "a releitura apos o merge tem de pedir `--json state`: e o campo que diz se mergeou"
    );
    assert!(
        apos.contains("MERGED"),
        "apos gh pr merge, o script deve comparar o estado com MERGED. Sem esta comparacao, \
         falha de merge passa por sucesso e o log imprime 'mergeado' incondicionalmente."
    );
}
