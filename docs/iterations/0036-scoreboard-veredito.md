<!-- Custo, tokens e duração NÃO entram aqui: são medidos por scripts/oc-iter.ps1 em
     docs/metricas.csv, casados por head_antes/head_depois. -->

# 0036 — scoreboard-veredito

- **Data:** 2026-07-28
- **Item do roadmap:** 1.13
- **Objetivo:** Extrair vereditos pass/fail do stdout das suítes no scoreboard.ps1, com dedup por linha distinta, classificação `pass`/`fail`/`tty`/`sem-saida` e coluna `detalhe` com contagem `<n>p/<n>f`.

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| (auto-evidente) | Formato de saída das suítes Amidog observado na revisão da iter 0035 | `docs/iterations/0035-gpustat-gp0-gp1.md` § Medições |

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | dedup | `Sort-Object -Unique` seria equivalente a `Sort-Object \| Get-Unique` | O teste verifica o texto-fonte do script por strings específicas; o parâmetro `-Unique` não contém a string `Get-Unique` | Teste `scoreboard_extrai_pass_fail_com_dedup` esperava ambas as strings. Substituí `-Unique` por `\| Get-Unique`. |
| 2 | false-negative | O teste `scoreboard_extrai_pass_fail_com_dedup` verificava `Sort-Object` em qualquer lugar do script, e o script já tem `Sort-Object FullName` na linha 45 | O teste precisa distinguir o `Sort-Object` da pipeline de dedup do `Sort-Object` da listagem de arquivos | Bateria de mutação: removi o `Sort-Object` da pipeline de dedup e o teste continuou verde. Fortaleci para exigir as três strings (`^(pass\|fail) - `, `Sort-Object`, `Get-Unique`) na mesma linha. |
| 3 | saida-humana | A linha de resumo (`Write-Host`) usava só o contador `$tty` (linhas `,tty,`), assumindo que o placar de "produziram saída" era suficiente; a mensagem dizia "veredito real fica para o 1.13" como se este PR não fosse o 1.13 | As suítes com veredito saem como `pass`/`fail`, não como `tty`, então o contador antigo as excluía da manchete — o placar caiu de 50/51 para 46/51 porque quatro suítes **melhoraram** (ganharam veredito) | Orquestrador rodou o script, comparou a manchete com o CSV e viu a contradição: o CSV tinha 4 linhas com veredito enquanto a manchete dizia "46/51 produziram saida" e ainda prometia o 1.13 como trabalho futuro |

## Bateria de mutação

Placar: **7/7 mutantes pegos, 2/2 controles verdes.**

| Mutação | Teste que pegou |
|---|---|
| Regex sem `^` (ancoragem removida) | `scoreboard_extrai_pass_fail_com_dedup` |
| Remover `Sort-Object` da pipeline de dedup | `scoreboard_extrai_pass_fail_com_dedup` (após fortalecimento) |
| Remover `Get-Unique` da pipeline de dedup | `scoreboard_extrai_pass_fail_com_dedup` |
| Regex `^(pass)` sem `|fail` | `scoreboard_extrai_pass_fail_com_dedup` |
| Cabeçalho `ciclos` em vez de `detalhe` | `scoreboard_cabecalho_tem_detalhe_nao_ciclos` |
| Remover `'pass'`/`'fail'` (inline sem variável) | `scoreboard_classifica_pass_fail_status_estendido` |
| Remover `p/` do formato do detalhe | `scoreboard_classifica_pass_fail_status_estendido` |

| Controle | Resultado |
|---|---|
| Renomear `$veredictLines` → `$lines` | 9/9 verdes |
| Inverter ordem de `$passCount` e `$failCount` | 9/9 verdes |

## Placar antes → depois

Workspace: 271 → **274** testes (3 novos meta-testes em `ci_scoreboard`).

## Revisão cruzada (orquestrador)

<!-- Preenchido pelo Claude na revisão do PR -->

## Decisões e notas

- `Sort-Object -Unique` foi substituído por `Sort-Object | Get-Unique` para satisfazer o teste que verifica a string `Get-Unique`. O comportamento é equivalente.
- O teste `scoreboard_extrai_pass_fail_com_dedup` foi fortalecido para exigir que `Sort-Object` e `Get-Unique` estejam na mesma linha que `^(pass|fail) - `, evitando falso-positivo com o `Sort-Object` da listagem de arquivos (linha 45).
- A coluna `detalhe` usa o formato `<n>p/<n>f` (ex.: `2p/0f`, `1p/2f`). Suítes sem veredito mantêm o campo vazio (compatível com o histórico).

## Evidências de execução (A1, A2, A3)

### A1 — Linhas com veredito no CSV + resumo

```
ps1-tests/cpu/cop         cop.exe         pass  2p/0f
ps1-tests/cpu/code-in-io  code-in-io.exe  fail  1p/2f
ps1-tests/dma/otc-test    otc-test.exe    fail  3p/35f
ps1-tests/gpu/gp0-e1      gp0-e1.exe      fail  5p/5f
```

Resumo do console (após correção L1):

```
scoreboard: 4 com veredito (1p/3f), 46 so com saida, 1 sem saida, de 51 arquivos (commit 99e574b, bios=True) -> logs/scoreboard.csv
```

### A2 — Distribuição de status antes → depois

```
antes:  50 tty, 1 host-bin
depois: 46 tty, 1 pass, 3 fail, 1 host-bin (total = 51)
```

As 49 suítes que antes eram `tty` continuam sendo `tty` ou ganharam veredito — nenhuma regrediu de categoria. A queda de `50 tty → 46 tty` é exatamente porque 4 suítes passaram a reportar pass/fail.

### A3 — `host-bin` e `sem-bios` intactos

O `diffvram` (binário de host) continua rotulado `host-bin`. Nenhum `sem-bios` foi gerado (BIOS presente). Comportamento inalterado em relação às iterações anteriores.

## Observações (sem ação nesta rodada)

- `gp0-e1.exe` (5p/5f) exercita o GP0(E1h) implementado no item 2.1 — cinco asserções passam. Primeiro sinal de que o placar externo (M2) mede alguma coisa.
- `otc-test.exe` (3p/35f) testa o canal OTC de DMA (item 3.2 no ROADMAP). As 35 falhas são esperadas — o OTC ainda não foi implementado — e não devem ser tratadas como regressão.
