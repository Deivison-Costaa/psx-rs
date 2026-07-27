# 0007 — EXEs de teste e scoreboard esqueleto

- **Data:** 2026-07-27
- **Item do roadmap:** 0.7
- **Objetivo:** as suítes de validação de hardware disponíveis desde antes do emulador existir.

## O que foi feito

`scripts/fetch-test-exes.ps1`: baixa para `tests/exes/` (gitignored) o release `build-158`
do JaCzekanski/ps1-tests (49 EXEs: gpu, timers, dma, mdec, spu, cdrom, gte...) e os zips
`psxtest_cpu`/`psxtest_gte` do Amidog, logando SHA-256. `scripts/scoreboard.ps1`: varre os 51
EXEs e acumula `logs/scoreboard.csv` (`ts,commit,suite,exe,status,ciclos`); sem runner ainda,
todos entram como `sem-runner` — 0/51. O runner real chega no 1.11 (sideload de PS-EXE);
publicação na branch `scoreboard-data` no 1.12.

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| — | nenhum | | | |

## Bateria de mutação

N/A (infra de download; validação: idempotência + contagem de EXEs logada).

## Revisão cruzada (orquestrador)

Autor: orquestrador (infra).
