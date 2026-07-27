# 0001 — bootstrap do repositório

- **Data:** 2026-07-27
- **Item do roadmap:** 0.1
- **Objetivo:** repo git + GitHub público `Deivison-Costaa/psx-rs`, política de merge e template de PR.

## O que foi feito

`git init -b main`; `.gitignore` (BIOS/`.env`/target/exes/logs fora); `.gitattributes` com LF
(host Windows, CI Linux); `gh repo create --public`; settings via API: **só merge commit**
(squash e rebase desabilitados — decisão do usuário: commits test→feat→docs visíveis na main),
branch apagada após merge. Proteção de branch adiada para a iteração 0004: exigir checks que
ainda não existem bloquearia os PRs 0002–0003.

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que era | Como foi pego |
|---|---|---|---|---|
| — | nenhum | | | |

## Notas

Iteração executada pelo orquestrador (Claude) direto na main — o repo ainda não tinha como
receber PR. Da 0002 em diante, tudo via PR.
