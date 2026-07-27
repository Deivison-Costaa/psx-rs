# 0008c — fix: oc-iter não disparava no Windows

- **Data:** 2026-07-27
- **Item do roadmap:** 0.8c (correção do 0.8, achada pelo smoke test)
- **Objetivo:** oc-iter.ps1 executável de verdade no host Windows.

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que era | Como foi pego |
|---|---|---|---|---|
| 1 | API-Rust* | `Start-Process opencode` resolve o executável como no cmd | o shim do npm é `opencode.ps1`, e Start-Process só executa PE/cmd ("%1 não é um aplicativo Win32 válido") | smoke test 0008b falhou na linha 41 antes de chamar o modelo |

*categoria: ambiente-host (fora do enum do TEMPLATE — registrada em orquestracao.md se recorrer).

## O que foi feito

Resolução explícita de `opencode.cmd` via `Get-Command` (fallback ao Source genérico) nas
duas chamadas (`serve` e `run`). Validação: re-execução do smoke test 0008b.

## Bateria de mutação

N/A (fix de infra; o próprio smoke test é a verificação).

## Revisão cruzada (orquestrador)

Autor: orquestrador.
