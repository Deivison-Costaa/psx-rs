# 0008d — fix: quoting em duas camadas derrubou o smoke test de novo

- **Data:** 2026-07-27
- **Item do roadmap:** 0.8d (2ª correção do 0.8, achada pelo smoke test 0008b/2)
- **Objetivo:** prompt chega ao opencode como UM argumento, e "ok" só com execução real.

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que era | Como foi pego |
|---|---|---|---|---|
| 1 | ambiente-host | shim .cmd preserva aspas do ArgumentList | a camada cmd degradou as aspas; o `--version` citado DENTRO do prompt virou flag: o CLI imprimiu "1.18.3" e saiu com 0 | JSON de saída com 7 bytes |
| 2 | ambiente-host | `\"` escapa aspas em string PowerShell (hábito bash) | PowerShell escapa com `` `" `` — o argumento quebrou e derramou fragmentos do prompt nos parâmetros seguintes (linha corrompida no metricas.csv) | campo `modelo` do CSV com texto do prompt |
| 3 | processo | exit 0 = sucesso | o CLI pode sair com 0 sem executar nada | duração 365 ms para uma "iteração" |

## O que foi feito

- oc-iter.ps1 chama o `opencode.exe` real (sem camadas de shim npm/cmd).
- Guarda anti-falso-ok: saída JSON < 1000 bytes ⇒ `falha:sem-execucao`.
- Linha corrompida do metricas.csv corrigida para `0008b,falha:sem-execucao`.
- Lição operacional para o orquestrador (registrada em orquestracao.md): prompts com
  aspas/flags passam por string single-quoted do PowerShell (escape `''`), nunca `\"`.

## Bateria de mutação

N/A (a verificação é o smoke test re-executado).

## Revisão cruzada (orquestrador)

Autor: orquestrador.
