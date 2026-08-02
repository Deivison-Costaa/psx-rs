# 0148 — vsync-timeout

- **Data:** 2026-08-01
- **Item do roadmap:** 10.73
- **Objetivo:** Diagnosticar por que o Rayman fica preso em `VSync: timeout`, determinando qual das tres hipoteses do handoff e verdadeira.

## Spec consultada

| Fonte | Secao | Arquivo local |
|---|---|---|
| psx-spx | Interrupts | docs/reference/11-interrupts.md |
| psx-spx | GPU Vertical Display range (GP1 07h) | docs/reference/03-gpu.md |
| psx-spx | Timer 1 sync modes | docs/reference/05-timers.md |

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | timing | Assumi que o VSync timeout do Rayman seria detectavel em ~150M passos como no Crash | O Rayman so alcanca o loop de VSync apos desenhar a tela da Ubi Soft (~400M passos segundo o handoff) | A medicao mostrou o primeiro timeout entre 150M-200M passos (~449M total_cycles), consistente com a estimativa |
| 2 | dispatch | Assumi que entradas no handler (irq_handler_entries > 0) provam que o handler do jogo foi chamado | A entrada em 0x80000080 e a BIOS; o dispatch para o handler do jogo depende da cadeia ExCB/EvCB, que pode estar vazia | O contador do jogo em 0x801DF2CC = 0 prova que o handler do jogo nunca executou, apesar de 1470 entradas no handler |
| 3 | timer | Assumi que o Timer 1 poderia estar configurado para sincronia de VBlank (bit0=1 no mode) | O jogo configura Timer 1 em Free Run (bit0=0), nao para VBlank | A leitura do mode register (0x1D48) confirmou bit0=0 em todas as janelas |

## Bateria de mutacao

Diagnostico puro — nenhum codigo de producao foi alterado. Mutacao nao se aplica.
**Linha formal de dispensa:** iteracao exclusivamente de diagnostico; 0 mutantes gerados, 0/0 mortos, 0/0 controles verdes, 0 equivalente.

## Placar antes → depois

882 testes antes, 883 depois (+1: `diagnostico_vsync_timeout_rayman`).

## Diagnostico

### Metodo

Teste de integracao `crates/psx-core/tests/vsync_timeout_diag.rs`:
1. Boot com BIOS SCPH1001 + `Rayman (USA) DADOS.cue`
2. Executa ate 500M passos, parando ao detectar `"VSync: timeout"` na TTY
3. Mede: `irq.raise_count(0)`, `cpu.irq_handler_entries`, `irq.read_mask()`, Timer1 mode, contador em `0x801DF2CC`

### Resultados (release, ~73 s, 200M passos ate o timeout)

| Metrica | Valor |
|---|---|
| IRQ0 levantamentos | 660 |
| Entradas no handler (0x80000080) | 1470 |
| I_MASK final | 0x000D (bits 0,2,3) |
| Timer1 mode | 0x00001D48 (sync=DESABILITADO, free run) |
| Contador do jogo @ 0x801DF2CC | **0x00000000** |
| TTY contem "VSync: timeout" | sim |

### Julgamento das hipoteses

| Hip. | Descricao | Veredito | Evidencia |
|---|---|---|---|
| (a) | Contador de VBlank por handler via IRQ0 | **CONFIRMADA** | counter=0 com 660 IRQ0 e 1470 entradas: handler nunca alcancou o incremento |
| (b) | Root counter (Timer1) de VBlank | **REFUTADA** | Timer1 mode bit0=0 (sync disabled) em todo o boot |
| (c) | I_MASK bit0 desabilitado | **REFUTADA** | I_MASK=0x000D tem bit0=1 no momento do timeout |

### Mecanismo do defeito

O jogo chama `VSync()` da LIBGPU, que compara o contador global `0x801DF2CC` contra
um valor salvo e faz spin com timeout de 0xFFFF iteracoes (`bne $v0, $v1, loop` em
`0x801B958C`). O contador permanece em **zero** — nunca foi incrementado.

O handler vinculado a IRQ0 **e levantado** (660x), a CPU **vetoriza** para `0x80000080`
(1470x), e `I_MASK` tem o bit 0 habilitado. Portanto o defeito nao esta na sinalizacao
do VBlank nem na entrega da IRQ ate o vetor `0x80000080`.

O defeito esta **depois** do vetor: a cadeia de dispatch da BIOS (A/B-handler, ExCB/EvCB)
nao alcanca o handler do jogo que incrementa `0x801DF2CC`. A cadeia ExCB tem 1 entrada
com class=0x00006DA8 (nao e um class valido de evento), e a cadeia EvCB nao contem
nenhuma entrada com class F0000001 (VBlank callback). O jogo nunca registrou um
callback de VBlank via `OpenEvent`/`EnableEvent`, ou o registro falhou.

### Proximo passo

Determinar COMO o jogo instala seu handler de VBlank: se via `VSyncCallback()` (que
usa F0000001), `SetRCnt(RCntCNT1,...)` ou substituicao direta do vetor `0x80000080`.
A proxima iteracao deve rastrear a instalacao do handler durante a inicializacao do jogo.

## Decisoes e notas

- O contador `0x801DF2CC` foi identificado por analise do codigo no ponto de spin
  (`lui $2, 0x801D; lw $2, 0xF2CC($2)` em `0x801B95AC-B0`).
- O timer1 esta em free run porque o jogo nao o usa para VBlank — usa o mecanismo
  de callback de IRQ0, que falha no dispatch.
- O teste e de integracao pesada (~73 s em release, ~200M passos), mas necessario para
  medir o estado real do jogo no ponto do timeout. Sem BIOS/disco, faz skip limpo.
