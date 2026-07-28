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
| Scoreboard | script completo (L1-118) | `scripts/scoreboard.ps1` |
| CI workflow | jobs check + scoreboard (L1-63) | `.github/workflows/ci.yml` |
| Fetch de EXEs | script de download (L1-30) | `scripts/fetch-test-exes.ps1` |
| BIOS | nota 1: local e hash (L75) | `STATUS.md` |
| PS-EXE magic | header layout — magic `PS-X EXE` (L1163) | `docs/reference/16-cdrom-file-formats.md` |

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | API-Rust | O teste `ci_workflow.rs` varria o `ci.yml` inteiro atrás de `if:` e o `if: success()` do job scoreboard derrubaria a suíte existente | A regra é "sem `if:` nos PASSOS do job check", não em jobs posteriores (o próprio comentário do ci.yml diz "passos do job check") | `cargo test --all` — `ci_workflow.rs` falhou com `Some("if: success()")`. Corrigido escopando a busca ao bloco do job `check:` |
| 2 | script | Troquei `-Include *.exe, *.psexe` por `-Recurse -File` para filtrar por magic bytes, assumindo que os únicos artefatos extras seriam o `diffvram` | O diretório `tests/exes/` contém README, LICENSE e outros arquivos não-EXE do zip do ps1-tests, que entraram na varredura como `host-bin` (51 linhas extras por run). O objetivo do filtro era **tirar** o diffvram da série, não **acrescentar** cinquenta readmes | Revisão adversarial (I2): `50/101 produziraм saída`, com 51 linhas `host-bin`. Corrigido na rodada de correção: pré-filtro por extensão (`-Include *.exe, *.psexe`) **e** checagem de magic bytes. Arquivo com extensão executável e sem `PS-X EXE` vira `host-bin` (diffvram, uma linha); o resto não gera linha nenhuma. Comando de prova: `./scripts/scoreboard.ps1 && Import-Csv logs/scoreboard.csv \| Group-Object status` → 50 tty + 1 host-bin = 51 linhas por varredura |
| 3 | processo | A publicação na branch `scoreboard-data` funcionaria de primeira sem testar o fluxo de git no CI | O `GITHUB_TOKEN` é somente-leitura por padrão; sem `permissions: contents: write`, o push resulta em 403 | Revisão adversarial (I1): job vermelho com `Permission to github-actions[bot] denied`. Corrigido com `permissions: contents: write` no job. O fluxo completo de git (checkout --orphan + push) continua declarado como A3 pendente de verificação em runner real |
| 4 | script | `Write-Error` na ausência de `tests/exes/` era o tratamento correto — se não tem EXE, o script deve alertar | A armadilha 2 do handoff mandava tolerar ausência de EXEs (0/0 sem quebrar o job). Com `$ErrorActionPreference = "Stop"`, `Write-Error` termina o script e o job fica vermelho | Revisão adversarial (I4). Corrigido: ausência de `tests/exes/` emite `0/0`, escreve header vazio e faz `exit 0` |

## Bateria de mutação (correção)

Placar: **10/10 mutantes pegos, 2/2 controles verdes**.

| # | Tipo | Mutação | Teste que a pegou |
|---|---|---|---|
| M1 | mutante | Magic string `"PS-X EXE"` trocada por `"CABO"` no scoreboard.ps1 | `scoreboard_filtra_por_magic_bytes_e_nao_por_extensao` (2ª asserção) |
| M2 | mutante | Label `host-bin` trocado por `ignorado` no scoreboard.ps1 | `scoreboard_rotula_host_bin_em_vez_de_fail_erro` |
| M3 | mutante | Referência a `psx-cli` removida do ci.yml (substituída por `nada`) | `ci_yml_tem_job_scoreboard` (asserção build psx-cli) |
| M4 | mutante | Referência a `fetch-test-exes` removida do ci.yml | `ci_yml_tem_job_scoreboard` (asserção fetch) |
| M5 | mutante | Referência a `scoreboard.ps1` removida do ci.yml | `ci_yml_tem_job_scoreboard` (asserção scoreboard script) |
| M6 | mutante | Nome do job `scoreboard:` trocado por `outra-coisa:` | `ci_yml_tem_job_scoreboard` (1ª asserção) |
| M7 | mutante | `contents: write` trocado por `contents: read` no ci.yml | `ci_yml_scoreboard_tem_permissions_write` |
| M8 | mutante | Guarda `if: github.event_name == 'push' && ...` removida do passo de publicação | `ci_yml_scoreboard_publica_so_na_main` |
| M9 | mutante | `exit 0` trocado por `exit 1` no bloco de ausência de EXEs | `scoreboard_nao_quebra_sem_diretorio_exes` (1ª asserção) |
| M10 | mutante | `Write-Error` adicionado de volta no bloco de ausência de EXEs | `scoreboard_nao_quebra_sem_diretorio_exes` (2ª asserção) |
| C1 | controle | Adicionada linha de comentário no final do ci.yml | todos verdes |
| C2 | controle | Renomeadas variáveis `$candidate` → `$f`, `$candidateFiles` → `$allFiles` no scoreboard.ps1 | todos verdes |

## Placar antes → depois

Antes (antes da correção): 244 testes, scoreboard 50/101 (51 linhas `host-bin` de readmes inflando a série).
Depois (após correção): 247 testes (3 novos em `ci_scoreboard.rs`), scoreboard 50/51 — 50 `tty` + 1 `host-bin` (o `diffvram`).

### Comando de prova (28/07/2026)

```
PS> ./scripts/scoreboard.ps1
scoreboard: 50/51 produziram saida (criterio: TTY nao vazio; veredito real fica para o 1.13) (commit f1d0a9e, bios=True) -> logs/scoreboard.csv

PS> Import-Csv logs/scoreboard.csv | Group-Object status | Select-Object Name,Count

Name     Count
----     -----
host-bin     1
tty         50
```

51 linhas por varredura, sem artefatos de distribuição. O `host-bin` é o `diffvram-windows-amd64.exe` (extensão `.exe` mas sem magic `PS-X EXE`).

## Revisão cruzada (orquestrador)

### I1 (bloqueador) — 403 no push da branch scoreboard-data

`GITHUB_TOKEN` somente-leitura por padrão. Corrigido com `permissions: contents: write` no job.

### I2 (bloqueador) — Filtro por magic bytes inflou a série

A troca de `-Include` por `-Recurse -File` varreu todos os arquivos (README, LICENSE, .txt) como `host-bin`, dobrando o lixo. Corrigido com pré-filtro por extensão **e** checagem de magic — só arquivos com extensão executável passam; os sem magic viram `host-bin` (1 linha); os demais não geram linha.

### I3 — Publicação em PRs contamina a série histórica

O job publicava em `push` e `pull_request`. Corrigido: passo de publicação guardado com `if: github.event_name == 'push' && github.ref == 'refs/heads/main'`.

### I4 — Ausência de EXEs quebra o job

`Write-Error` com `$ErrorActionPreference = "Stop"` matava o script e derrubava o job. Corrigido: ausência emite `0/0`, header e `exit 0`.

### Menores

- `ReadAllBytes` lia o arquivo inteiro para 8 bytes → trocado por `FileStream` + `Read(8)`.
- `ci.yml` sem newline no fim → adicionado.
- `scripts/scoreboard.ps1` passou de 112 para 118 linhas.

### Verificação do orquestrador, executando

(O bloco acima foi escrito pelo trabalhador resumindo os achados; esta seção é a que o
orquestrador assina. O resumo confere com o que eu tinha postado no PR.)

- `cargo fmt --check` e `cargo clippy -D warnings` limpos; `cargo test --all` = **247**.
- `./scripts/scoreboard.ps1` → `50/51 produziram saida`, com `host-bin 1` (o `diffvram`) e
  `tty 50`. Voltou às 51 linhas por varredura; o I2 está resolvido de fato.
- CI no head real (`904dec6`): `check` **success**, `scoreboard` **success**, com o passo de
  publicação pulado por ser evento de `pull_request` — que é o comportamento pedido no I3.
  O A3 do handoff, portanto, sai de "pendente" para verificado **no que dá para verificar
  antes do merge**: o job roda verde. A publicação em si só exercita na primeira execução em
  `push` na main, depois deste merge — e continua declarada como não verificada até lá.

### Dois consertos meus nesta branch

1. **`Set-Content $OutFile` no caminho de saída graciosa truncava a série local.** Quando
   `tests/exes/` não existe, o script escrevia o header por cima de um `logs/scoreboard.csv`
   já existente, apagando as varreduras anteriores. No runner não faz diferença (checkout
   limpo), na máquina de quem desenvolve faz. Agora só escreve o header se o arquivo não
   existir, igual ao caminho normal.
2. **O detalhamento do placar no STATUS estava com `10 cli_runner`; são 9.** O total (247)
   sempre foi medido e está certo; a soma das parcelas dava 248. A discrepância entrou na
   0029 e passou por duas revisões — inclusive as minhas, que conferiram o total e não as
   parcelas. Mesma família do erro registrado na 0022 (placar do STATUS errado por três
   iterações): conferir o número que se lê não basta, é preciso conferir a conta que o produz.

1. **Pré-filtro por extensão + magic bytes:** `scripts/scoreboard.ps1` agora varre `tests/exes/` com `Get-ChildItem -Include *.exe, *.psexe -Recurse` e lê os 8 primeiros bytes (via `FileStream`) de cada um. Arquivos com magic `PS-X EXE` seguem para execução; arquivos com extensão executável sem magic viram `host-bin` (uma linha, o `diffvram`); arquivos sem extensão executável não geram linha.

2. **Job `scoreboard` no CI:** Adicionado após `check` com `needs: check` + `if: success()`. Declara `permissions: contents: write` para permitir push na branch `scoreboard-data`. O passo de publicação é guardado com `if: github.event_name == 'push' && github.ref == 'refs/heads/main'` — em PRs, o scoreboard roda e valida o script, mas não publica. Timeout de 15 minutos.

3. **Tolerância a ausência de EXEs:** Se `tests/exes/` não existir (fetch falhou com `continue-on-error`), o script emite `0/0`, escreve só o header e faz `exit 0` — sem quebrar o job.

4. **ROADMAP 1.13 — Veredito real:** Adicionado após 1.12, marcado como dependente do 2.1 (GPUSTAT + decodificação GP0/GP1). Nenhuma suíte de teste produz veredito hoje porque todo EXE imprime o banner `ResetGraph:` e trava esperando GPUSTAT.26 (documentado em `docs/iterations/0028-spec-printf.md`). Os rótulos `tty`/`sem-saida` são honestos e vão para o histórico como estão.

5. **Sem BIOS no runner → toda linha será `sem-bios`:** O runner de CI não tem `bios/SCPH1001.BIN` (gitignored, nunca commitado). A primeira série histórica publicada na branch `scoreboard-data` não terá nenhuma medição de execução — todas as linhas serão `sem-bios`. Isso é previsto pela armadilha 1 do handoff e é honesto, não defeito. Quando uma BIOS for fornecida via secret, as medições reais começarão a aparecer sem quebra de série.

6. **Teste A3 pendente de verificação:** O job `scoreboard` no CI só pode ser verificado após o PR ser aberto no GitHub. O mecanismo de commit na branch `scoreboard-data` (git checkout --orphan + git push) depende do ambiente de Actions e do `permissions: contents: write`, ambos declarados como pendentes de verificação até existir link de run verde.

7. **`ci_workflow.rs` atualizado:** O teste `job_check_sem_condicionais_e_com_os_tres_passos` agora escopa a busca de `if:` ao bloco do job `check:` (entre `check:` e o próximo job), em vez de varrer o arquivo inteiro. O `if: success()` do job scoreboard é legítimo (condição de job-level, não de step).
