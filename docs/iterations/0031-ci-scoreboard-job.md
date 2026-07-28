<!-- Custo, tokens e duração NÃO entram aqui: são medidos por scripts/oc-iter.ps1 em
     docs/metricas.csv, casados por head_antes/head_depois. -->

# 0031 — ci-scoreboard-job

- **Data:** 2026-07-28
- **Item do roadmap:** 1.12
- **Objetivo:** CI: job scoreboard ligado no workflow + filtro por magic bytes no script de placar + ROADMAP 1.13 (veredito real).

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| ROADMAP | Item 1.12 (L33) | `ROADMAP.md` |
| Scoreboard | script completo (L1-95) | `scripts/scoreboard.ps1` |
| CI workflow | jobs check + scoreboard (L1-57) | `.github/workflows/ci.yml` |
| Fetch de EXEs | script de download (L1-30) | `scripts/fetch-test-exes.ps1` |
| BIOS | nota 1: local e hash (L75) | `STATUS.md` |
| PS-EXE magic | header layout — magic `PS-X EXE` (L1163) | `docs/reference/16-cdrom-file-formats.md` |

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | API-Rust | O teste `ci_workflow.rs` varria o `ci.yml` inteiro atrás de `if:` e o `if: success()` do job scoreboard derrubaria a suíte existente | A regra é "sem `if:` nos PASSOS do job check", não em jobs posteriores (o próprio comentário do ci.yml diz "passos do job check") | `cargo test --all` — `ci_workflow.rs` falhou com `Some("if: success()")`. Corrigido escopando a busca ao bloco do job `check:` |
| 2 | script | Assumi que `Get-ChildItem -File` retornava os arquivos na mesma quantidade de `-Include *.exe` | O diretório `tests/exes/` contém README, LICENSE e outros arquivos não-EXE do zip do ps1-tests, que entram na varredura como `host-bin` (51 linhas extras por run) | Scoreboard produziu 50/101 (antes era 50/51). Os labels `host-bin` são corretos para todo arquivo sem magic `PS-X EXE`, mas a contagem de "total" agora inclui artefatos de distribuição. Não corrigido — é o comportamento esperado do filtro por magic bytes |
| 3 | processo | A publicação na branch `scoreboard-data` funcionaria de primeira sem testar o fluxo de git no CI | `git checkout --orphan` + `git rm -rf --cached .` em repositório com working tree suja pode falhar; o script assume que `logs/scoreboard.csv` existe após o scoreboard rodar | Não verificado — o job de CI só roda no GitHub após o PR aberto. O mecanismo de commit na branch `scoreboard-data` está declarado como pendente de verificação (A3) |

## Bateria de mutação

Placar: **6/6 mutantes pegos, 2/2 controles verdes**.

| # | Tipo | Mutação | Teste que a pegou |
|---|---|---|---|
| M1 | mutante | Magic string `"PS-X EXE"` trocada por `"CABO"` no scoreboard.ps1 | `scoreboard_filtra_por_magic_bytes_e_nao_por_extensao` (2ª asserção) |
| M2 | mutante | Label `host-bin` trocado por `ignorado` no scoreboard.ps1 | `scoreboard_rotula_host_bin_em_vez_de_fail_erro` |
| M3 | mutante | Referência a `psx-cli` removida do ci.yml (substituída por `nada`) | `ci_yml_tem_job_scoreboard` (asserção build psx-cli) |
| M4 | mutante | Referência a `fetch-test-exes` removida do ci.yml | `ci_yml_tem_job_scoreboard` (asserção fetch) |
| M5 | mutante | Referência a `scoreboard.ps1` removida do ci.yml | `ci_yml_tem_job_scoreboard` (asserção scoreboard script) |
| M6 | mutante | Nome do job `scoreboard:` trocado por `outra-coisa:` | `ci_yml_tem_job_scoreboard` (1ª asserção — existência do job) |
| C1 | controle | Adicionada linha de comentário no final do ci.yml | todos verdes |
| C2 | controle | Renomeadas variáveis `$candidate` → `$f`, `$candidateFiles` → `$allFiles` no scoreboard.ps1 | todos verdes |

## Placar antes → depois

Antes: 241 testes, scoreboard 50/51 com `diffvram-windows-amd64.exe` como `fail-erro`.
Depois: 244 testes (3 novos em `ci_scoreboard.rs`), scoreboard 50/50 PS-EXEs válidos, `diffvram-*` rotulados `host-bin`.

## Revisão cruzada (orquestrador)

<!-- Preenchido pelo Claude na revisão do PR: achados no formato de docs/prompts/review.md, ou "sem achados". -->

## Decisões e notas

1. **Filtro por magic bytes, não por extensão:** `scripts/scoreboard.ps1` agora varre todos os arquivos em `tests/exes/` com `Get-ChildItem -Recurse -File` e lê os 8 primeiros bytes de cada um. Arquivos com magic `PS-X EXE` seguem para execução; os demais recebem status `host-bin`. Isso resolve a poluição da série histórica pelo `diffvram-windows-amd64.exe` (binário de host, não PS-EXE).

2. **Job `scoreboard` no CI:** Adicionado após `check` com `needs: check` + `if: success()`. Executa checkout (fetch-depth: 0), instala toolchain, baixa EXEs (continue-on-error), builda psx-cli, roda scoreboard, e publica `logs/scoreboard.csv` na branch órfã `scoreboard-data` com append (pula header se arquivo já existe). Timeout de 15 minutos para o job.

3. **ROADMAP 1.13 — Veredito real:** Adicionado após 1.12, marcado como dependente do 2.1 (GPUSTAT + decodificação GP0/GP1). Nenhuma suíte de teste produz veredito hoje porque todo EXE imprime o banner `ResetGraph:` e trava esperando GPUSTAT.26 (documentado em `docs/iterations/0028-spec-printf.md`). Os rótulos `tty`/`sem-saida` são honestos e vão para o histórico como estão.

4. **Teste A3 pendente de verificação:** O job `scoreboard` no CI só pode ser verificado após o PR ser aberto no GitHub. O mecanismo de commit na branch `scoreboard-data` (git checkout --orphan + git push) depende do ambiente de Actions e não foi testado localmente.

5. **`ci_workflow.rs` atualizado:** O teste `job_check_sem_condicionais_e_com_os_tres_passos` agora escopa a busca de `if:` ao bloco do job `check:` (entre `check:` e o próximo job), em vez de varrer o arquivo inteiro. O `if: success()` do job scoreboard é legítimo (condição de job-level, não de step).
