mod support;

use std::fs;

#[test]
fn scoreboard_filtra_por_magic_bytes_e_nao_por_extensao() {
    let script = fs::read_to_string(support::repo_root().join("scripts/scoreboard.ps1"))
        .expect("scripts/scoreboard.ps1 deve existir");

    let nao_filtra_so_por_extensao = !script.contains("-Include *") || script.contains("PS-X EXE");
    assert!(
        nao_filtra_so_por_extensao,
        "scripts/scoreboard.ps1 usa -Include baseado em extensão (*.exe, *.psexe) sem \
         verificar magic bytes 'PS-X EXE'. O diretório tests/exes/ contém binários de host \
         (ex.: diffvram-windows-amd64.exe) que NÃO são PS-EXE e poluem a série histórica \
         com 'fail-erro'. O script deve ler os 8 primeiros bytes de cada arquivo e filtrar \
         pelo magic 'PS-X EXE' (conforme 16-cdrom-file-formats.md L1163)."
    );

    let verifica_magic = script.contains("PS-X EXE");
    assert!(
        verifica_magic,
        "scripts/scoreboard.ps1 não contém a string 'PS-X EXE' — não está filtrando por \
         magic bytes. O script deve ler os primeiros 8 bytes de cada arquivo e comparar \
         com o magic 'PS-X EXE' antes de tentar executá-lo como PS-EXE."
    );
}

#[test]
fn ci_yml_tem_job_scoreboard() {
    let ci = fs::read_to_string(support::repo_root().join(".github/workflows/ci.yml"))
        .expect("ci.yml deve existir");
    let effective: Vec<&str> = ci
        .lines()
        .map(str::trim)
        .filter(|l| !l.starts_with('#'))
        .collect();

    let tem_job = effective.windows(3).any(|w| {
        w[0].trim() == "scoreboard:"
            || (w[0].starts_with("scoreboard")
                && w[1].starts_with("name:")
                && w[2].starts_with("runs-on:"))
    });
    assert!(
        tem_job,
        "ci.yml não contém job 'scoreboard'. O item 1.12 requer um job scoreboard após o \
         job check, que rode scripts/scoreboard.ps1 e publique o histórico em \
         logs/scoreboard.csv na branch scoreboard-data."
    );

    let tem_scoreboard_script = effective.iter().any(|l| l.contains("scoreboard.ps1"));
    assert!(
        tem_scoreboard_script,
        "ci.yml não invoca scripts/scoreboard.ps1. O job scoreboard deve rodar o script \
         de placar completa após build do psx-cli."
    );

    let tem_fetch = effective.iter().any(|l| l.contains("fetch-test-exes"));
    assert!(
        tem_fetch,
        "ci.yml não invoca scripts/fetch-test-exes.ps1. O job scoreboard precisa baixar \
         EXEs de teste antes de rodar o placar."
    );

    let tem_build_cli = effective.iter().any(|l| l.contains("psx-cli"));
    assert!(
        tem_build_cli,
        "ci.yml não faz build do psx-cli. O job scoreboard precisa compilar o binário \
         antes de chamar o script."
    );
}

#[test]
fn scoreboard_rotula_host_bin_em_vez_de_fail_erro() {
    let script = fs::read_to_string(support::repo_root().join("scripts/scoreboard.ps1"))
        .expect("scripts/scoreboard.ps1 deve existir");

    let tem_host_bin = script.contains("host-bin");
    assert!(
        tem_host_bin,
        "scripts/scoreboard.ps1 não rotula binários de host como 'host-bin'. \
         Arquivos sem magic 'PS-X EXE' devem ser registrados com status 'host-bin' \
         (um rótulo honesto), não tentar execução e cair em 'fail-erro'."
    );
}

#[test]
fn scoreboard_nao_quebra_sem_diretorio_exes() {
    let script = fs::read_to_string(support::repo_root().join("scripts/scoreboard.ps1"))
        .expect("scripts/scoreboard.ps1 deve existir");

    let after_missing_dir: Vec<&str> = script
        .lines()
        .skip_while(|l| !l.contains("Test-Path $ExeRoot"))
        .take(8)
        .collect();
    let bloco = after_missing_dir.join("\n");

    let tem_exit_zero = bloco.contains("exit 0");
    assert!(
        tem_exit_zero,
        "scripts/scoreboard.ps1 não faz exit 0 quando tests/exes/ não existe. \
         A ausência de EXEs deve produzir 0/0 sem quebrar o job (armadilha 2 do handoff 1.12)."
    );

    let nao_tem_write_error = !bloco.contains("Write-Error");
    assert!(
        nao_tem_write_error,
        "scripts/scoreboard.ps1 usa Write-Error quando tests/exes/ não existe. \
         Com $ErrorActionPreference = 'Stop', Write-Error termina o script e o job \
         fica vermelho. A ausência de EXEs deve ser tolerada (saída 0, zero linhas)."
    );
}

#[test]
fn ci_yml_scoreboard_tem_permissions_write() {
    let ci = fs::read_to_string(support::repo_root().join(".github/workflows/ci.yml"))
        .expect("ci.yml deve existir");

    let scoreboard_start = ci
        .lines()
        .position(|l| l.trim() == "scoreboard:")
        .expect("ci.yml deve ter job 'scoreboard:'");
    let scoreboard_end = ci
        .lines()
        .skip(scoreboard_start + 1)
        .position(|l| {
            let t = l.trim();
            !t.is_empty()
                && t.ends_with(':')
                && !t.starts_with('-')
                && !t.starts_with("name:")
                && !t.starts_with("runs-on:")
                && !t.starts_with("needs:")
                && !t.starts_with("if:")
                && !t.starts_with("timeout-minutes:")
                && !t.starts_with("permissions:")
                && !t.starts_with("steps:")
                && !t.starts_with("contents:")
        })
        .map(|p| scoreboard_start + 1 + p)
        .unwrap_or_else(|| ci.lines().count());
    let scoreboard_block: Vec<&str> = ci
        .lines()
        .skip(scoreboard_start)
        .take(scoreboard_end - scoreboard_start)
        .collect();
    let bloco = scoreboard_block.join("\n");

    let tem_permissions = bloco.contains("permissions:") && bloco.contains("contents: write");
    assert!(
        tem_permissions,
        "ci.yml job scoreboard não declara 'permissions: contents: write'. \
         O GITHUB_TOKEN é somente-leitura por padrão, e o push para scoreboard-data \
         resulta em 403 sem essa permissão (I1 da revisão adversarial da iter 0031)."
    );
}

#[test]
fn ci_yml_scoreboard_publica_so_na_main() {
    let ci = fs::read_to_string(support::repo_root().join(".github/workflows/ci.yml"))
        .expect("ci.yml deve existir");

    let tem_guard =
        ci.contains("if: github.event_name == 'push' && github.ref == 'refs/heads/main'");
    assert!(
        tem_guard,
        "ci.yml job scoreboard publica na branch scoreboard-data sem a guarda \
         'if: github.event_name == 'push' && github.ref == 'refs/heads/main'. \
         Sem essa guarda, medições de PRs não revisados entram na série histórica \
         com o mesmo peso das da main (I3 da revisão adversarial da iter 0031)."
    );
}
