mod support;

use std::fs;

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

#[test]
fn wait_checks_consulta_check_runs_antes_de_mergestatestatus() {
    let script = oc_loop_content();
    let body = wait_checks_body(&script);

    let check_runs_pos = body
        .find("check-runs")
        .expect("Wait-Checks deve consultar check-runs");
    let merge_state_pos = body
        .find("mergeStateStatus")
        .expect("Wait-Checks deve consultar mergeStateStatus");

    assert!(
        check_runs_pos < merge_state_pos,
        "Wait-Checks deve consultar check-runs ANTES de mergeStateStatus. \
         Logo apos um push, mergeStateStatus responde com o estado do commit ANTERIOR. \
         Ordem atual: mergeStateStatus aparece antes (pos {merge_state_pos} < {check_runs_pos})."
    );
}

#[test]
fn gh_pr_merge_verifica_estado_merged_apos_o_merge() {
    let script = oc_loop_content();

    let merge_pos = script
        .find("gh pr merge")
        .expect("oc-loop.ps1 deve conter gh pr merge");
    let apos_merge = &script[merge_pos..];

    assert!(
        apos_merge.contains("MERGED"),
        "Apos gh pr merge, o script deve verificar que o PR esta MERGED (ex.: \
         gh pr view --json state --jq .state). Sem esta verificacao, falha de \
         merge passa por sucesso e o log imprime 'mergeado' incondicionalmente."
    );
}
