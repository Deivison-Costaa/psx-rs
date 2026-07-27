# 0008 — orquestração opencode/DeepSeek

- **Data:** 2026-07-27
- **Item do roadmap:** 0.8
- **Objetivo:** o pipeline trabalhador↔orquestrador executável e medido.

## O que foi feito

- `.claude/skills/iterate/SKILL.md` — protocolo de 12 passos, **fonte única** (correção da
  falha nº 1 do gb-rs: protocolo duplicado e divergente; a bateria de mutação agora É um
  passo, não prosa). Diferença estrutural vs gb-rs: o trabalhador **não faz merge** — abre o
  PR e para; revisão adversarial e merge são do orquestrador.
- `scripts/oc-iter.ps1` — guardas (árvore limpa, main atualizada), `opencode serve` +
  `run --attach` (bug opencode#28407 no Windows), timeout, JSON em `logs/`, extração de
  custo/tokens/steps e **append automático em docs/metricas.csv** (correção da falha nº 3:
  métricas congeladas por dependência de memória humana).
- `scripts/oc-loop.ps1` — N iterações; sem `-AutoMerge` força N=1 (o encadeamento exige
  merge, e merge exige revisão); `-AutoMerge` documentado como modo "revisão a posteriori".

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| | | | | (smoke test 0008b valida na prática) |

## Bateria de mutação

N/A (scripts; validação real = smoke test 0008b de ponta a ponta, doc próprio).

## Revisão cruzada (orquestrador)

Autor: orquestrador (infra).

## Decisões

Parser de métricas do JSON do opencode marcado como "calibrar no smoke test" — heurística de
última ocorrência para tokens/custo até ver o formato real do 1.18.3.
