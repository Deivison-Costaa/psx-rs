# BOOTSTRAP

Registro de como o projeto foi inicializado (2026-07-27) e como retomar a operação.

## O que o bootstrap fez (M0, iterações 0001–0009)

Repo público + merge-commit-only → workspace 3 crates → docs de gestão → CI + proteção de
branch → meta-testes → specs fatiadas → EXEs de teste → orquestração → 1ª iteração do
trabalhador (BIOS). Executor: orquestrador (Claude), cada item como PR. Detalhes:
`docs/orquestracao.md` e `docs/iterations/0001..0009`.

## Como operar depois do bootstrap

No terminal do Claude (orquestrador), com árvore limpa na `main`:

```
Supervisionado (1 iteração):  scripts/oc-iter.ps1
Loop (N iterações):           scripts/oc-loop.ps1 -N 3
```

O orquestrador revisa cada PR com `docs/prompts/review.md` antes do merge. Comece com N=1 —
um loop desatendido com protocolo ruim produz N PRs ruins em vez de 1. O custo do trabalhador
é appendado automaticamente em `docs/metricas.csv`; conferir a linha nova faz parte do merge.

## Pré-requisitos da máquina

- Rust ≥1.85, gh autenticado, PowerShell 7, opencode com provider `deepseek` autenticado.
- `bios/SCPH1001.BIN` local (gitignored) para os checks que exigem BIOS.
