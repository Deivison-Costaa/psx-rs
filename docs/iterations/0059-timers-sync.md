# 0059 — timers-sync

- **Data:** 2026-07-29
- **Item do roadmap:** 3.4b
- **Objetivo:** implementar modos de sincronização Hblank/Vblank nos timers 0 e 1, expor sinais na GPU, e verificar timer 2 modos 1/2.

## Revisão do PR anterior (0058)

Achados 1 defeito.

1. **Campo de bit lido errado (Padrão 4):** Bit 10 do MODE é "(Set after Writing)" (05-timers.md L54) — deve ser forçado a 1 após escrita do MODE. O código preservava bit 10 de `prev & 0x7C00` sem forçar a 1. Corrigido em `write32` com `mode |= 1 << 10;`.
2. **Teste que não mede:** Teste `mode_gravavel_e_legivel` usava máscara `0x1FF` (bits 0-8) em vez de `0x3FF` (bits 0-9). Corrigido + assert de bit10=0x400.
3. Parâmetro não consumido: N/A (sem FIFO nos timers).
4. Regra de borda trocada: N/A.
5. Panic/laço: verificado — índice de array protegido pelo bus, `effective` não causa loop infinito em uso normal.
6. Citação de spec: `confere-citacoes.ps1` verde.
7. Escopo transbordado: não. Manifesto 0058 arquivado pois ancoras expiraram na reescrita do `tick()`.

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| psx-spx | Timer Counter Mode — sync modes (L32-L43) | docs/reference/05-timers.md |
| psx-spx | GPU Timers / Synchronization (L156-L158) | docs/reference/03-gpu.md |

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | flags | Que `sync_enable=0` no timer 2 com sync_mode=0 parava o contador | Spec: sync_mode só importa quando sync_enable=1. Com sync_enable=0, é Free Run independente do modo | Teste `tick_respeita_divisor_de_clock_do_timer2` falhou após reescrita do `increment` — corrigido com `2 => true` no branch `else` |
| 2 | API-Rust | Que poderia chamar `bus.timers_mut().tick(...)` com `bus.gpu().hblank_active()` na mesma expressão | Rust não permite borrow mutável e imutável simultâneo no mesmo escopo | Erro de compilação E0502 — resolvido com helper `tick_timer()` que extrai os sinais antes da chamada |

## Bateria de mutação

Placar da bateria: 7/7 mutantes mortos, 2/2 controles verdes, 0 equivalente - ./docs/mutantes/0059-timers-sync.mut

| Mutante | Teste que o pegou |
|---|---|
| m1 (modo 0 não pausa durante sync) | `timer0_sync_mode0_pausa_durante_hblank` |
| m2 (modo 2 não pausa fora do sync) | `timer0_sync_mode2_reseta_no_hblank_e_pausa_fora` |
| m3 (borda não reseta cnt modos 1/2) | `timer0_sync_mode1_reseta_no_hblank`, `timer1_sync_mode1_reseta_no_vblank` |
| m4 (modo 3 não detecta primeiro sync) | `timer0_sync_mode3_espera_primeiro_hblank_depois_free_run` |
| m5 (T2 sync_enable=1 não incrementa) | `timer2_modo_1_e_2_sao_free_run` |
| m6 (MODE não reseta prev_sync) | `escrever_mode_reseta_estado_de_sync` |
| m7 (modo 3 reseta a cada sync) | `timer0_sync_mode3_espera_primeiro_hblank_depois_free_run` (segunda borda) |

## Placar antes → depois

Workspace: **469** → **480** testes (459 existentes + 11 timers_sync).

## Revisão cruzada (orquestrador)

<!-- Preenchido pelo Claude na revisão do PR -->

## Decisões e notas

1. **Sinais hblank/vblank expostos na GPU.** `hblank_active: Cell<bool>` adicionado ao struct `Gpu` com getter/setter. `vblank_active()` delega para `in_vblank` existente. Reset no GP1(00h).
2. **`tick()` recebe sinais por parâmetro.** Nova assinatura: `tick(&mut self, base_addr, cycles, hblank_active, vblank_active)`. Evita acoplamento circular GPU ↔ Timers.
3. **Edge detection via `prev_sync_signal`.** Cada timer armazena o valor anterior do sinal de sync. Transição false→true (rising edge) dispara reset nos modos 1 e 2, e trigger no modo 3.
4. **Modo 3 com `mode3_triggered`.** Flag booleana que indica se o primeiro pulso de sync já ocorreu. Resetado ao escrever MODE.
5. **Timer 2 modos 1 e 2 confirmados Free Run.** Spec diz: sync modes 1/2 do timer 2 = Free Run (mesmo com sync_enable=1). Teste `timer2_modo_1_e_2_sao_free_run` verifica ambos.
6. **Método `gpu_mut()` adicionado ao Bus.** Para permitir que testes configurem hblank/vblank diretamente.
