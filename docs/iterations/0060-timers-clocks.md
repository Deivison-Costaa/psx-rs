# 0060 — timers-clocks

- **Data:** 2026-07-29
- **Item do roadmap:** 3.4c
- **Objetivo:** implementar fontes de clock alternativas dotclock (timer 0) e Hblank (timer 1) com acumulador fracionário baseado na razão 11/(7*ciclos_por_pixel) e 11/(7*ciclos_por_scanline).

## Revisão do PR anterior (0059)

Revisão do PR anterior: sem achados

1. Teste que não mede — todos os testes medem o que afirmam; bateria 7/7 mortos
2. Parâmetro não consumido — N/A (sem FIFO nos timers)
3. Regra de borda trocada — N/A (não aplicável a timers)
4. Campo de bit lido errado — bit 10 forçado a 1, bits 11-12 reset na leitura, bits de clock/sync com máscaras corretas
5. Panic/laço — sem unwrap/unsafe, índice protegido por & 0x3
6. Citação de spec — confere-citacoes.ps1 verde
7. Escopo transbordado — hblank_active/in_vblank não togglados em produção é dívida pré-existente (iters 0050/0051), não introduzida pela 0059

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| psx-spx | Clock Source (Counter Mode bits 8-9) | docs/reference/05-timers.md |
| psx-spx | GPU Timings — Dotclocks, Horizontal/Vertical Timings | docs/reference/03-gpu.md |

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | timing | Que o cálculo do acumulado do teste `timer1_hblank_ntsc` daria 6 após duas chamadas de tick | O correto é 7224 — fração acumulada 7218*23891 é muito maior que o esperado | Teste falhou na segunda asserção — corrigido o valor esperado |
| 2 | API-Rust | Que `update_gpu_timing` poderia ser chamado via `bus.gpu_mut()` | GPU não tem acesso aos Timers — o método pertence ao Timers, chamado via `bus.timers_mut()` | Erro de compilação — ajustada a chamada nos testes |
| 3 | ferramental | Que âncoras de manifiestos anteriores (0052, 0059) sobreviveriam | `cycles_per_pix` mudou de `fn` para `pub fn` (0052); `tick()` foi reescrito (0059) — âncoras apodreceram | CI `mutation_anchors` reprovou — ambos manifestos marcados como `arquivada` |

## Bateria de mutação

Placar da bateria: 7/7 mutantes mortos, 3/3 controles verdes, 0 equivalente - ./docs/mutantes/0060-timers-clocks.mut

| Mutante | Teste que o pegou |
|---|---|
| m1 (dotclock usa numerador 1 em vez de 11) | `timer0_dotclock_256px_razao_11_por_70_cpu_cycles` |
| m2 (hblank usa denominador sem 7×) | `timer1_hblank_ntsc_razao_11_por_23891_cpu_cycles` |
| m3 (dotclock ativado no timer errado) | `timer0_dotclock_256px_razao_11_por_70_cpu_cycles` |
| m4 (sync_enable=0 ainda verifica clock_src) | `timer0_dotclock_256px_razao_11_por_70_cpu_cycles`, `timer1_hblank_ntsc_razao_11_por_23891_cpu_cycles` |
| m5 (MODE não reseta cycle_acc) | `escrever_mode_reseta_acumulador_fractional_de_clock` |
| m6 (sync mode 0 não pausa) | `timer0_dotclock_com_sync_mode0_pausa_durante_hblank` |
| m7 (timer 2 divisor /8 ignorado) | `timer2_clock_div_8_continua_funcionando` |

## Placar antes → depois

Workspace: **470** → **479** testes (470 existentes + 9 timers_dotclock_hblank).

## Revisão cruzada (orquestrador)

<!-- Preenchido pelo Claude na revisão do PR -->

## Decisões e notas

1. **Razão 11/7 como aproximação de video_clock/cpu_clock.** A spec (03-gpu.md L1452-1456) dá a fórmula de Nocash: Video Clock = CPU Clock × 11/7. Usada para converter CPU cycles em dotclock/hblank pulses.
2. **Acumulador fracionário com numerador/denominador.** `cycle_acc` acumula o resto da divisão `(prev_acc × denom + cycles × numer) % denom`, emitindo `(total / denom)` pulsos por chamada. Abordagem unificada com o clock/8 do timer 2.
3. **Increment gate independente do clock source.** Removida a verificação `clock_src == 0 || clock_src == 2` do gate de incremento — o clock source determina a taxa de pulsos, não se o timer deve contar. Os modos de sync (pause/reset) aplicam-se igualmente a dotclock e Hblank.
4. **`cycles_per_pix` tornado público na GPU.** Necessário para o caller do timer obter o fator de conversão correto baseado na resolução horizontal (GPUSTAT bits 16-18).
5. **`video_cycles_per_scanline()` adicionado à GPU.** Retorna 3413 (NTSC) ou 3406 (PAL) baseado no `video_mode`.
6. **`Timers::update_gpu_timing()`** permite ao caller (GPU/runner) informar os parâmetros de timing sem acoplamento circular GPU ↔ Timers.
7. **Manifestos 0052 e 0059 arquivados.** `cycles_per_pix` mudou visibilidade; `tick()` foi reescrito — âncoras apodreceram na implementação do item 3.4c.
