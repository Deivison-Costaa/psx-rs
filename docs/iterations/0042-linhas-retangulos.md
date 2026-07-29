# 0042 — linhas-e-retangulos

- **Data:** 2026-07-28
- **Item do roadmap:** 2.4
- **Objetivo:** Implementar renderização de linhas (flat/gouraud, single/polyline com terminador) e retângulos (não-texturizados nos quatro tamanhos, texturizado consome UV).

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| psx-spx | GPU Render Line Commands (L334-372) | docs/reference/03-gpu.md |
| psx-spx | GPU Render Rectangle Commands (L373-416) | docs/reference/03-gpu.md |
| psx-spx | Polygon notes (L323) | docs/reference/03-gpu.md |

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | byte-order | Assumi que 0x50_FF_0000 era vermelho em vez de azul no comando GP0 (bits 0-7=R, 8-15=G, 16-23=B) | xxBBGGRR: bits 0-7 = Red, bits 16-23 = Blue (L463) | Testes de cor falharam (A3, A4, A7) com pixel(0,0) azul em vez de vermelho |
| 2 | Bresenham-steep | Assumi que o pixel (0,1) estava no traçado da linha (0,0)-(3,5) | O algoritmo clássico põe (1,1) para essa inclinação — o pixel mais próximo da reta y=1.667x em x=1 é (1,1) | Teste A2b falhou; recalculei o Bresenham manualmente |
| 3 | vertex-encoding | Escrevi vértice vertical como 0x0000_0005 (X=5,Y=0) em vez de 0x0005_0000 (X=0,Y=5) | Formato YYYYXXXX: bits 16-31 = Y, bits 0-15 = X (L441-446) | Teste A2c falhou: linha "vertical" era horizontal |
| 4 | polyline-colors | Assumi que `colors[i]` estava preenchido para todos os vértices em flat polyline | Só `colors[0]` é preenchido pelo comando; segmentos i>0 liam lixo (0) | Teste A5 falhou: segmentos além do primeiro não eram pintados |

## Bateria de mutação

Placar da bateria: 7/7 mutantes mortos, 2/2 controles verdes, 0 equivalente - docs/mutantes/0042-linhas-retangulos.mut

| Mutante | Descrição | Teste assassino |
|---|---|---|
| m1 | Regra de borda da linha de inclusiva para exclusiva (break por step >= steps) | A1, A2, A2c |
| m2 | Retângulo de tamanho fixo consome Width+Height (else if size != 3) | A7, A8 |
| m3 | Troca R e B em color24_to_16 | A3 |
| m4 | Regressão do flat polyline (colors[i]=0 em vez de flat_c0) | A5 |
| m5 | Enum de tamanho 1x1 → 8x8 | A7 |
| m6 | t_max = steps+1 em vez de steps (interpolação errada) | A3 |
| m7 | Terminador ignora bits 12-15 (máscara 0xF0000000) | polyline_flat_vertice_com_bits_28_a_31_iguais_a_5_nao_e_terminador |

## Placar antes → depois

Antes: 321 testes (10 meta + 8 bus_bios + 2 bios_flag + 1 version + 12 bus_scheduler + 9 bus_scratchpad_isc + 8 cpu_fetch_decode + 26 cpu_alu + 14 cpu_shifts + 19 cpu_load_delay + 24 cpu_branches + 7 cpu_jumps + 20 cpu_mult_div + 29 cpu_unaligned_load_store + 10 cpu_cop0_regs + 14 cpu_exception_mechanism + 1 cpu_exception_estado_previo + 9 cpu_tty_hook + 11 cpu_printf_hook + 11 cpu_opcode_reservado + 16 gpu_status_gp0_gp1 + 9 ci_scoreboard + 9 cli_runner + 21 gpu_vram_transfers + 20 gpu_triangulos_flat_gouraud + 2 mutation_manifest + 2 mutation_anchors + 5 mutation_battery)

Depois: 338 testes (+17: 16 gpu_linhas_retangulos + 1 extra na mutation_battery)

Scoreboard: 5 com veredito (1p/4f), 45 só com saída, 0 sem saída, 1 não avaliados, de 51 arquivos

## Decisões e notas

- Vertex para linha usa Bresenham clássico com `err = dx + dy` (soma, não diferença), que funciona em todos os octantes sem branch de inclinação
- Linha INCLUI a coordenada inferior-direita (L361-362), ao contrário de polígono que EXCLUI (L323). Esta é a diferença mais importante do item e foi capturada pelo mutante m1
- Polígono e linha usam regras OPOSTAS de borda: o rasterizador de linha NÃO pode reusar `xr.min(area_x2 + 1)` do triângulo
- Retângulo size=1/2/3 NÃO envia Width+Height; ler uma palavra a mais dessincroniza o FIFO (defeito G3/H2 de iterações anteriores)
- O estado `LineRender` armazena até 16 vértices (15 segmentos de polyline) em arrays fixos para manter `Copy` no `Cell<VramState>`
- Texturizado: UV consumido e ignorado; textura real e Texpage no item 2.5
- Semi-transparência e dithering fora de escopo (item 2.6)
