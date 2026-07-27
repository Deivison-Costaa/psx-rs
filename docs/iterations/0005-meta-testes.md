# 0005 — meta-testes de processo

- **Data:** 2026-07-27
- **Item do roadmap:** 0.3
- **Objetivo:** as regras do processo reprovarem sozinhas, sem depender de memória do agente.

## O que foi feito

7 arquivos em `crates/psx-core/tests/` + `support/mod.rs` compartilhado:
`purity` (R3/R6: allowlist vazia + forbid(unsafe_code)), `comment_density` (R7: teto duro 10%,
arquivos ≥40 linhas), `file_size` (R8: ≤500 linhas por fonte), `status_size` (≤16 KB),
`roadmap_size` (≤10 KB), `ci_workflow` (sem `if:` efetivo no ci.yml + 3 passos presentes),
`metrics_freshness` (lag docs de iteração − linhas do CSV ≤ 1). Mensagens de erro pedagógicas
citando a medição do gb-rs que motivou cada regra.

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | API-Rust | `support/mod.rs` compartilhado compila limpo em todos os binários de teste | cada binário vê só o que usa: `-D warnings` reprova dead_code dos helpers não usados | clippy na primeira rodada |

## Bateria de mutação

**7/7 pegos, 1/1 controle verde.**

| Mutação | Pego por |
|---|---|
| STATUS.md com +17 KB | status_size |
| `if:` num passo do job check | ci_workflow |
| lib.rs sem forbid(unsafe_code) | purity |
| fonte com 501 linhas | file_size |
| arquivo com 33% de comentários | comment_density |
| linha removida do metricas.csv (lag 2) | metrics_freshness |
| 12 KB de prosa no ROADMAP | roadmap_size |
| Controle: suíte inteira após reverter | verde |

## Placar antes → depois

0 → 8 testes (7 binários; purity tem 2).

## Revisão cruzada (orquestrador)

Autor: orquestrador (infra).
