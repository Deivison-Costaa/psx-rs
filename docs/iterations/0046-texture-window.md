# 0046 — texture-window

- **Data:** 2026-07-29
- **Item do roadmap:** 2.5c
- **Objetivo:** Implementar texture window GP0(E2h) — mask e offset de U/V.

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| psx-spx | GP0(E2h) Texture Window setting (L521-550) | docs/reference/03-gpu.md |

## Revisão do PR anterior (iter 0045)

Revisão do PR anterior: sem achados

- Teste que mede: cada teste tem mutante que o mata (6/6)
- Parâmetro não consumido → FIFO dessincronizado: E2h é command do top3 0xE0, enfileira Idle — não consome FIFO
- Regra de borda trocada: textura não depende de borda de rasterização — N/A
- Campo de bit lido errado: máscaras 0x1F (5 bits), shifts 5, 10, 15 conferidos
- Panic/laço ilimitado: sample_texel clampa em 0-255, VRAM bounds com & 0x3FF/0x1FF
- Citação de spec: confere-citacoes.ps1 verde
- Escopo transbordado/dívida: apenas o que o item 2.5c pede

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | endereçamento | Fórmula do offset era `Offset * 8` sem AND com Mask | `Texcoord = (Texcoord AND (NOT (Mask * 8))) OR ((Offset AND Mask) * 8)` — o AND com Mask é parte da fórmula | Teste T4 com Mask=3, Offset=5: (5&3)*8=8, mas sem AND seria 40. Mutante m3 matou T4 |
| 2 | API-Rust | GP1(00h) reset se escreve com `write32(0, 0x0000_0000)` | GP1 está no offset 4, não 0 | Teste T3: `gpu.write32(0, ...)` escrevia em GP0 (NOP), não resetava a janela |
| 3 | endereçamento | Escrevi textura na linha 2 esperando que V janela=16 caísse na linha 16 (page_y=0) | Com page_y=0, V=16 mapeia para linha 16 da VRAM; textura na linha 2 não é visível | Teste T5: corrigi a posição da textura para (0, 16) |

## Bateria de mutação

Placar da bateria: 6/6 mutantes mortos, 2/2 controles verdes, 0 equivalente - ./docs/mutantes/0046-texture-window.mut

| ID | Tipo | Mutação | Teste que pegou |
|---|---|---|---|
| m1 | Mask X ignorada | `u_win = (u & !(mask*8)) \| ...` → `u as u32` | T1, T2, T4 |
| m2 | Offset X ignorado | `... \| ((off & mask) * 8)` removido | T1, T2, T4 |
| m3 | Offset sem AND Mask | `(off_x & mask_x) * 8` → `off_x * 8` | T4 (offset 5 > mask 3) |
| m4 | Mask * 4 em vez de * 8 | `mask_x * 8` → `mask_x * 4` | T1, T2 |
| m5 | Offset Y ignorado | `v_win = ...` → `v as u32` | T5 |
| m6 | Offset X bit shift errado | `(val >> 10)` → `(val >> 11)` na decodificação E2h | T1, T2, T4 |
| K1 | Renomeia u_win/v_win | Refatoração cosmética | Nenhum (controle verde) |
| K2 | Inverte ordem offset_x/y | Linhas independentes | Nenhum (controle verde) |

## Placar antes → depois

Antes: 351 testes.

Depois: 356 testes (+5 gpu_texture_window).

## Decisões e notas

1. **Texture window armazena 4 valores independentes.** `tex_window_mask_x`, `mask_y`, `offset_x`, `offset_y` — cada um 5 bits (0-31), armazenados como `u8`.

2. **Transformação aplicada antes do clamp em sample_texel.** A fórmula `(Texcoord AND (NOT (Mask * 8))) OR ((Offset AND Mask) * 8)` é aplicada a U e V cru (i32), e só depois o resultado é clampado para 0-255.

3. **E2h não afeta GPUSTAT.** Ao contrário de E1h, o E2h armazena em campos separados do Gpu e não modifica o registro de status.

4. **Reset em GP1(00h) zera os 4 campos.** A janela volta ao default (sem máscara, sem offset) no reset da GPU.

5. **A textura repete naturalmente.** Como a fórmula substitui bits da coordenada U/V, o wrap (repeat) é automático — nenhum código extra de módulo é necessário.

## Revisão cruzada (orquestrador)

<!-- Preenchido pelo Claude na revisão do PR -->
