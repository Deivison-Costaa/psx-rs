<!-- Custo, tokens e duração NÃO entram aqui: são medidos por scripts/oc-iter.ps1 em
     docs/metricas.csv, casados por head_antes/head_depois. -->

# 0038 — vram-transfers

- **Data:** 2026-07-28
- **Item do roadmap:** 2.2
- **Objetivo:** VRAM 1MB (1024×512×16bit) + comandos de transferência: fill GP0(02h), CPU→VRAM GP0(A0h), VRAM→CPU GP0(C0h).

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| psx-spx | § Quick Rectangle Fill (L217-232) | docs/reference/03-gpu.md |
| psx-spx | § GPU Memory Transfer Commands (L603-702) | docs/reference/03-gpu.md |
| psx-spx | § VRAM Overview / VRAM Addressing (L234-252) | docs/reference/03-gpu.md |
| psx-spx | § GP0(02h) FillVram (L1156-1180) | docs/reference/03-gpu.md |
| psx-spx | § Masking and Rounding for FILL (L640-662) | docs/reference/03-gpu.md |
| psx-spx | § Masking for COPY Commands (L664-685) | docs/reference/03-gpu.md |
| psx-spx | § Wrapping (L697-702) | docs/reference/03-gpu.md |

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | endereçamento | Fill: Xpos=0x17 mascarado para 0x10 → pixel em 0x10 NÃO é preenchido. | Xpos & 0x3F0 alinha para 0x10, mas o fill cobre 0x10..0x1F com Xsiz=0x10, então 0x10 É preenchido (e 0x17 também). | Teste `fill_xpos_mascarado_para_multiplo_de_0x10` falhou na primeira escrita — asserção invertida. |
| 2 | endereçamento | CPU→VRAM: halfword baixa do word 0x5566_0000 é 0x5566. | Little-endian: val & 0xFFFF = 0x0000. A halfword baixa é os 16 bits inferiores. | Teste `cpu_para_vram_qtd_impar_halfwords` falhou — `gpu_vram_u16(2,0)` retornou 0 em vez de 0x5566. |
| 3 | flags | CPU→VRAM Xsiz contado em halfwords: primeira word de dados contém halfword=0, que é contada como "não preenchida". | Teste usava `(i as u32) \| ((i as u32 + 1) << 16)` com i=0, gerando halfword baixa = 0. | Teste `cpu_para_vram_xsiz_contado_em_halfwords` contou 7/8 — o valor 0 não difere do background. |
| 4 | flags | VRAM→CPU com 3 halfwords: halfwords [0]=0xAAAA, [1]=0x0000, [2]=0xBBBB. Esperava w1>>16 == 0xBBBB. | w1 = (buf[0], buf[1]) = (0xAAAA, 0x0000), não (0xAAAA, 0xBBBB). w1 = 0x0000_AAAA. | Teste `vram_para_cpu_qtd_impar_halfwords` falhou — asserções baseadas em ordenação errada dos halfwords na word. |
| 5 | flags | GPUREAD usa `active_cmd == 3` para decidir se retorna dados. | `advance_vram_to_cpu` chama `reset_command()` que zera `active_cmd` ANTES do primeiro GPUREAD. Buffer carregado mas condição falsa. | 4/5 testes VRAM→CPU falharam — GPUREAD retornava 0. Corrigido para verificar `readout_buf` em vez de `active_cmd`. |

## Bateria de mutação

**Placar: 7/7 mutantes pegos, 2/2 controles verdes.**

| Mutação | Teste que pegou |
|---|---|
| M1: Remover máscara Xpos fill (`& 0x3F0`) | `fill_xpos_mascarado_para_multiplo_de_0x10` |
| M2: Remover arredondamento Xsiz fill (`+0xF & !0xF`) | `fill_xsiz_arredondado_para_cima` |
| M3: Remover `wrapping_sub(1) & 0x3FF + 1` (Xsiz=0→max) em CPU→VRAM | `cpu_para_vram_xsiz_ysiz_zero_vira_max` |
| M4: GPUREAD sempre retorna 0 (readout quebrado) | `vram_para_cpu_le_dados_do_gpuread`, `vram_para_cpu_le_halfword_esperada`, `vram_para_cpu_duas_halfwords_por_word`, `vram_para_cpu_qtd_impar_halfwords` |
| M5: Pular escrita da halfword alta em `receive_data` | `cpu_para_vram_duas_halfwords_por_word` |
| M6: Trocar R por B na conversão de cor do fill | `fill_color_24bit_para_15bit_mascara_zero` |
| M7: Não carregar readout buffer em VRAM→CPU | `vram_para_cpu_le_dados_do_gpuread` |

Controles: reordenar match arms em `write_gp0` (21/21 verde), renomear `xy_index` → `vram_pos` (294/294 verde).

## Placar antes → depois

Workspace: 274 → 295 testes (+21 `gpu_vram_transfers`).

## Revisão cruzada (orquestrador)

<!-- Preenchido pelo Claude na revisão do PR: achados no formato de docs/prompts/review.md, ou "sem achados". -->

## Decisões e notas

1. **VRAM como `Box<[u16]>`**: 1 MB (524.288 elementos) é grande demais para stack. Heap-alocado via `vec![0u16; 524288].into_boxed_slice()`.
2. **GPUREAD com `Cell<usize>`**: A leitura de GP0 (VRAM→CPU) precisa avançar índice interno, mas `read32` usa `&self` para compatibilidade com o bus. `Cell` resolve sem `RefCell`.
3. **Máscara Ypos fill é redundante com `% 512`**: `ypos_raw & 0x1FF` ≡ `ypos_raw % 512`. Mutação não-capturável; o comportamento final é idêntico. O teste `fill_ypos_mascarado_para_9_bits` verifica o comportamento (y=0x201 → y=1), não a implementação da máscara.
4. **Guarda `xsiz==0 \|\| ysiz==0` no fill é redundante**: A fórmula de arredondamento já produz xsiz=0 para xsiz_raw=0, e o loop `0..0` não executa. Mantida por fidelidade à spec.
5. **Reset GP1(00h) preserva VRAM**: O reset limpa `stat`, `dma_direction`, estado de comando e readout, mas não zera o array VRAM. Compatível com hardware real.
