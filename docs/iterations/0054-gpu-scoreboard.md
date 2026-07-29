# 0054 — gpu-scoreboard

- **Data:** 2026-07-29
- **Item do roadmap:** 2.9
- **Objetivo:** confirmar que a suite GPU do ps1-tests esta integrada no scoreboard (fetch + execucao + coluna).

## Revisão do PR anterior

Revisão do PR anterior (0053): sem achados.
- 1. Teste que não mede: d1+d2+d3 + 5 mutantes todos mortos. OK.
- 2. Parâmetro não consumido → FIFO dessincronizado: sem comandos GP0 novos. OK.
- 3. Regra de borda trocada: sem rasterização. OK.
- 4. Campo de bit lido errado: bit 23 do GPUSTAT correto. OK.
- 5. Panic ou laço ilimitado: sem unwrap() fora de teste. OK.
- 6. Citação de spec: confere-citacoes.ps1 verde. OK.
- 7. Escopo transbordado ou dívida não declarada: 2.8b fechado, dívidas no doc. OK.

## Spec consultada

Nenhuma — item de infraestrutura (scripts de scoreboard), não de hardware.

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | nenhum | Que o item 2.9 exigia baixar GPU e integrar no scoreboard do zero | Os EXEs de GPU ja estavam no tests.zip do ps1-tests (baixado desde a iter 0007) e o scoreboard ja varre recursivamente tests/exes/ | verificacao do scoreboard.csv: 17 suites GPU listadas como tty, 2 com veredito (gp0-e1 7p/3f, mask-bit 3p/2f) |

## Bateria de mutação

Bateria de mutação: não se aplica — item de infraestrutura sem codigo Rust novo (apenas testes de propriedades estaticas dos scripts PowerShell fetch-test-exes.ps1 e scoreboard.ps1). Os scripts ja eram corretos desde iterações anteriores.

## Placar antes → depois

Workspace: **402** → **408** testes (402 existentes + 6 gpu_scoreboard).

## Revisão cruzada (orquestrador)

<!-- Preenchido pelo Claude na revisão do PR -->

## Decisões e notas

1. A GPU do ps1-tests ja estava integrada desde a iter 0007 (fetch-test-exes.ps1 baixa tests.zip que inclui gpu/). O scoreboard.ps1 varre recursivamente tests/exes/ e ja processava todos os EXEs de GPU. A maioria das suites GPU aparece como `tty` porque renderiza na VRAM em vez de emitir `pass -`/`fail -` no TTY.
2. Apenas duas suites GPU produzem veredito textual: gp0-e1 (7p/3f no commit atual) e mask-bit (3p/2f). As demais 15 suites executam sem crash mas nao tem saida parseavel.
3. O fetch-test-exes.ps1 nao precisa de fonte separada para GPU — o tests.zip do ps1-tests inclui todas as suites (cpu, gpu, gte, dma, timers, spu, mdec, cdrom, input).
4. O meta-teste mutation_anchors.rs rejeita alvos fora de crates/*/src/ — scripts PowerShell nao sao mutaveis pelo protocolo atual. Opt-out via linha de nao-aplicabilidade no doc da iteracao.
5. A suite GPU esta confirmada no pipeline: fetch → build → sideload → scoreboard → CSV.
