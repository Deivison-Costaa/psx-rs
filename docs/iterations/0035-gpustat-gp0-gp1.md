<!-- Custo, tokens e duração NÃO entram aqui: são medidos por scripts/oc-iter.ps1 em
     docs/metricas.csv, casados por head_antes/head_depois. -->

# 0035 — gpustat-gp0-gp1

- **Data:** 2026-07-28
- **Item do roadmap:** 2.1
- **Objetivo:** GPUSTAT como espelho do estado que GP1(03h/04h/08h) e GP0(E1h/E6h) escrevem, mais bits de pronto fixos.

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| psx-spx | GPU I/O Ports (L144-147) | docs/reference/03-gpu.md |
| psx-spx | GPUSTAT table (L1002-1033) | docs/reference/03-gpu.md |
| psx-spx | GP1(00h) Reset (L747-765) | docs/reference/03-gpu.md |
| psx-spx | GP1(01h) Reset Command Buffer (L767) | docs/reference/03-gpu.md |
| psx-spx | GP1(02h) Ack IRQ1 (L773-777) | docs/reference/03-gpu.md |
| psx-spx | GP1(03h) Display Enable (L779-785) | docs/reference/03-gpu.md |
| psx-spx | GP1(04h) DMA Direction (L789-796) | docs/reference/03-gpu.md |
| psx-spx | GP1(08h) Display Mode (L885-893) | docs/reference/03-gpu.md |
| psx-spx | GP0(E1h) Draw Mode (L492) | docs/reference/03-gpu.md |
| psx-spx | GP0(E6h) Mask Bit Setting (L578) | docs/reference/03-gpu.md |
| psx-spx | GP0(00h) NOP (L721, L734) | docs/reference/03-gpu.md |
| psx-spx | Ready Bits (L1041-1057) | docs/reference/03-gpu.md |

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | bit-mapping | GP0(E1h) bit 15 do parâmetro mapeia para GPUSTAT.15 | GP0(E1h) bit **11** do parâmetro mapeia para GPUSTAT.15 (L500: "Texture page Y Base 2 — GPUSTAT.15 ;GP0(E1h).11") | Teste A3 usava `0x87FF` que tem ambos bits 11 e 15 setados — passou com a implementação errada. Corrigido na comparação spec: a tabela GPUSTAT diz explicitamente `.11`, não `.15`. |
| 2 | bit-computation | `(self.stat >> 28) & (1 << 25)` computa bit 25 como espelho de bit 28 | Isso dá 0 sempre porque `(x >> 28)` retorna 0 ou 1, e `1 << 25` está 25 posições acima — o AND é sempre 0 | Teste `gpustat_bit_25_espelha_dma_direction` falhou no modo 2. Corrigido para `((self.stat >> 28) & 1) << 25`. |
| 3 | bit-mapping | GP1(08h) bit 7 → GPUSTAT bit 14 com `(param & 0xC0) << 8` | O shift de 8 posições coloca bit 7 em bit 15, não bit 14. O mapeamento correto é bit 7→14 (shift 7), bit 6→16 (shift 10), bits 0-5→17-22 (shift 17) | Primeira execução: Flip(bit14) ficou 0 quando deveria ser 1. Corrigido decompondo os shifts: `(param & 0x80) << 7`, `(param & 0x40) << 10`, `(param & 0x3F) << 17`. |

## Bateria de mutação

Placar: **7/7 mutantes pegos, 2/2 controles verdes.**

| Mutação | Teste que pegou |
|---|---|
| Golden value 0x14802000 → 0x14802001 | `reset_gpu_produz_golden_value` |
| GP1(03h) invertido (on→bit23=1, off→bit23=0) | `gp1_03h_alterna_bit_23_display` |
| GP0(E1h) vira nop | `gp0_e1h_draw_mode_escreve_gpustat_0_10_e_15` |
| GP0(E6h) vira nop | `gp0_e6h_mask_bit_escreve_gpustat_11_12` |
| Bit 25 sempre 0 | `gpustat_bit_25_espelha_dma_direction` |
| GP1(08h) bit 7 → bit 16 (shift errado) | `gp1_08h_display_mode_mapeia_para_gpustat` (Flip) |
| GP1(04h) ignora parâmetro, sempre 0 | `gp1_04h_dma_direction_escreve_bits_29_30` + `gpustat_bit_25_espelha_dma_direction` |

| Controle | Resultado |
|---|---|
| Renomear `param` → `p` em write_gp1 | 13/13 verdes |
| Trocar ordem dos arms 0xE1/0xE6 no match | 13/13 verdes |

## Placar antes → depois

Workspace: 258 → **271** testes (13 novos: `gpu_status_gp0_gp1`).

## Revisão cruzada (orquestrador)

<!-- Preenchido pelo Claude na revisão do PR. -->

## Decisões e notas

1. **Bit 27 (Ready to send VRAM→CPU) mantido em 0** — o golden value `0x14802000` tem bit 27=0, e a spec (L1049-1050) diz que ele é setado após GP0(C0h) e seus parâmetros. Como GP0(C0h) não está implementado, manter em 0 é correto para este item.
2. **Bit 13 (Interlace Field) mantido em 1** — para GPU v2 com interlace off, a spec diz "always 1" (L1068). Toggling de interlace será tratado no item 2.7 (timing).
3. **GPUREAD retorna 0 (stub)** — respostas a GP0(C0h) e GP1(10h) não estão implementadas. Leitura em `1F801810h` retorna 0 por enquanto.
4. **Comandos GP0 desconhecidos são ignorados silenciosamente** — sem contagem de palavras de parâmetro para comandos multi-palavra (armadilha 5). Apenas GP0(00h/04h-1Eh/E0h/E1h/E6h/E7h-EFh) têm handlers. Comandos de rendering (20h-7Fh), transferência (80h-DFh) e IRQ (1Fh) são aceitos e descartados sem consumir parâmetros. Registrado para o item 2.2.
5. **GP1(05h/06h/07h/09h/10h+) não implementados** — fora do escopo deste item (R4). Registrados para os itens 2.7 (display regs) e 2.9 (GP1(10h) para detecção de GPU).
