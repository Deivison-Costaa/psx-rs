# 0051 — timing-vblank

- **Data:** 2026-07-29
- **Item do roadmap:** 2.7b
- **Objetivo:** Implementar timing NTSC/PAL (frame_cycles), estado de vblank e GPUSTAT bit 31 dinamico.

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| psx-spx | § GP1(08h) - Display mode (L885-905) | docs/reference/03-gpu.md |
| psx-spx | § GPUSTAT bits 16-22 (L1015-1022) | docs/reference/03-gpu.md |
| psx-spx | § GPUSTAT bit 31 (L1032-1036) | docs/reference/03-gpu.md |
| psx-spx | § Vertical Video Timings (L1414-1443) | docs/reference/03-gpu.md |
| psx-spx | § Vertical Timings (NoCash) (L1460-1467) | docs/reference/03-gpu.md |

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | API-Rust | O controle K2 (renomear variavel local) era cosmético | O renome parcial quebrou a compilacao porque `stat` era referenciado em outras linhas | mutantes.ps1: erro de compilacao do mutante K2 |
| 2 | timing | O item pedia vblank IRQ com disparo automatico por hardware | O IRQ1 (GPUSTAT.24) e setado via software (GP0(1Fh)), nao pelo vblank em si | leitura da spec GP0(1Fh) e GPUSTAT.24 durante o design |
| 3 | nenhum | — | — | — |

## Bateria de mutação

Placar da bateria: 6/6 mutantes mortos, 2/2 controles verdes, 0 equivalente — docs/mutantes/0051-timing-vblank.mut

| Mutante | Rótulo | Morreu | Teste que matou |
|---|---|---|---|
| m1 | frame_cycles usa PAL para NTSC | sim | t4 (esperava 566187, recebeu 680659) |
| m2 | frame_cycles usa NTSC para PAL | sim | t5 (esperava 680659, recebeu 566187) |
| m3 | enter_vblank nao seta in_vblank | sim | t7 (in_vblank continuou false) |
| m4 | exit_vblank nao limpa in_vblank | sim | t8 (in_vblank continuou true) |
| m5 | GP1(08h) bit3 invertido | sim | t2 (video_mode false quando bit3=1) |
| m6 | reset nao zera video_mode | sim | t9 (video_mode PAL apos reset) |
| K1 | inverte bracos do if em frame_cycles | — | controle cosmético |
| K2 | variavel _cosmetico nao usada em read32 | — | controle cosmético |

## Placar antes → depois

Workspace: **384** → **393** testes (384 existentes + 9 novos).

## Revisão cruzada (orquestrador)

<!-- Preenchido pelo Claude na revisão do PR -->

## Decisões e notas

1. O item original 2.7 do ROADMAP juntava display registers + timing + vblank IRQ. Por R4, foi dividido em 2.7a (0050) e 2.7b (0051).
2. `frame_cycles()` deriva da spec de refresh rate: NTSC 59.826 Hz e PAL 49.761 Hz com CPU clock de 33.8688 MHz, resultando em 566_187 e 680_659 ciclos de CPU por frame.
3. GPUSTAT bit 31 e 0 durante vblank (`docs/reference/03-gpu.md` L1035). Fora de vblank, reflete `odd_line` (para tracking de scanlines futuros). Por enquanto, `set_odd_line()` e exposto para testes e para o futuro tracking de scanline.
4. O IRQ1 (GPUSTAT.24) NAO e disparado automaticamente pelo vblank — e um comando de software via GP0(1Fh). O scheduler de eventos dispara o vblank como evento de timing; o IRQ propriamente dito sera no modulo de interrupcoes (M3).
5. `enter_vblank()` e `exit_vblank()` sao chamados pelo loop principal (ou scheduler) para marcar transicoes de vblank. O metodo `in_vblank()` expoe o estado atual.
6. O reset (GP1(00h)) zera `video_mode`, `in_vblank` e `odd_line` para false (NTSC, nao-vblank, linha par).
7. A ancora K2 do manifesto 0050 quebrou porque GP1(08h) ganhou a linha `self.video_mode.set(...)`. Foi atualizada nesta iteracao.
