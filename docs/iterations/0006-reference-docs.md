# 0006 — specs do psx-spx fatiadas

- **Data:** 2026-07-27
- **Item do roadmap:** 0.6
- **Objetivo:** base auditável para a R1 (nunca implementar de memória) sem estourar contexto.

## O que foi feito

`scripts/fetch-reference-docs.ps1`: baixa 15 capítulos do `psx-spx/psx-spx.github.io` no
commit pinado `035c765`, prepende índice de seções (`L<n>: título`, offsets relativos à marca
`CORPO:`) e gera `docs/reference/README.md` com procedência. ~790 KB de spec commitados; o
agente paga só o índice + a seção do item (R8).

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | API-Rust* | headings do psx-spx em `#`–`###` | corpo usa `####`/`#####` (níveis 1–3 são só o título da página) | 1ª rodada indexou "1 seção" em 6 capítulos |

*categoria aproximada — erro de formato de fonte externa, não de Rust; registrado também em orquestracao.md se virar padrão.

## Bateria de mutação

N/A (script de infraestrutura; validação = re-rodar é idempotente e o diff é vazio).

## Revisão cruzada (orquestrador)

Autor: orquestrador (infra).
