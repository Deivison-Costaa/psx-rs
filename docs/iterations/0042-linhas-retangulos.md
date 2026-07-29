# 0042 — linhas-e-retangulos

- **Data:** 2026-07-28; continuacao 2026-07-29
- **Item do roadmap:** 2.4
- **Objetivo:** Implementar renderizacao de linhas (flat/gouraud, single/polyline com terminador) e retangulos (nao-texturizados nos quatro tamanhos, texturizado consome UV). Continuacao: corrige quatro defeitos encontrados por leitura da spec (D1-D4).

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
| 4 | polyline-colors | Assumi que `colors[i]` estava preenchido para todos os vertices em flat polyline | So `colors[0]` e preenchido pelo comando; segmentos i>0 liam lixo (0) | Teste A5 falhou: segmentos alem do primeiro nao eram pintados |
| 5 | R6/panic | Polyline com vertices em array fixo de 16 — o 17o vertice causa index out of bounds em gpu.rs:484 | Nao ha limite maximo de vertices na spec (L352-355); wire-frame polygons sao caso de uso comum (L368-369) | Revisao adversarial: prova com 17 vertices derruba o emulador |
| 6 | hardware + teste cego | Retangulo texturizado chama render_rect com a cor do comando, pintando pixels solidos | Spec L402: "color is ignored when textured" | Revisao adversarial: teste A10 usava cor=0, indistinguivel de VRAM limpa. Prova com cor!=0 mostra pixel pintado onde deveria estar limpo |
| 7 | hardware | Checagem de distancia maxima (dx>1023 ou dy>511) so existia em render_polyline, nao nos caminhos de linha simples | Spec L447-451: "The maximum distance between two vertices is 1023 horizontally, and 511 vertically" — vale para toda linha, nao so polyline | Revisao adversarial: linha simples de Y=-100 a Y=+300 (dy=600) pinta pixel onde nao deveria |
| 8 | robustez | render_rect usava w_actual/h_actual CRUS (ate 0xFFFF) como limites de laco; o continue interno recorta pixel mas nao reduz iteracoes | Spec L405-411: Width/Height max 1023x511. Nao ha formula de mascaramento para valores acima, mas o laco com 0xFFFF_FFFF itera ~4.3 bilhoes de vezes | Revisao adversarial: correcao segura limita o laco a intersecao com a area de desenho em vez de contar com continue |

## Bateria de mutação

Placar da bateria: 11/11 mutantes mortos, 2/2 controles verdes, 0 equivalente - docs/mutantes/0042-linhas-retangulos.mut

A primeira rodada fechou com 7/7; os quatro mutantes acrescentados na continuacao (D1-D4) levaram a 11/11.

| Mutante | Descricao | Teste assassino |
|---|---|---|
| m1 | Regra de borda da linha de inclusiva para exclusiva (break por step >= steps) | A1, A2, A2c |
| m2 | Retangulo de tamanho fixo consome Width+Height (else if size != 3) | A7, A8 |
| m3 | Troca R e B em color24_to_16 | A3 |
| m4 | Flat polyline usa pending_color em vez de color0 (cor zero) | A5 |
| m5 | Enum de tamanho 1x1 -> 8x8 | A7 |
| m6 | t_max = steps+1 em vez de steps (interpolacao errada) | A3 |
| m7 | Terminador ignora bits 12-15 (mascara 0xF0000000) | polyline_flat_vertice_com_bits_28_a_31_iguais_a_5_nao_e_terminador |
| m8 | D1 — polyline incremental nao desenha segmentos flat (if false && has_prev) | D1, A5 |
| m9 | D2 — retangulo texturizado desenha com cor (if false em vez de if textured) | D2, A10 |
| m10 | D3 — linha simples flat sem checagem de distancia maxima | D3, D3b |
| m11 | D4 — render_rect com bounds fixos 16x16 em vez da intersecao com area | D4, A9b |

## Placar antes → depois

Antes: 321 testes (10 meta + 8 bus_bios + 2 bios_flag + 1 version + 12 bus_scheduler + 9 bus_scratchpad_isc + 8 cpu_fetch_decode + 26 cpu_alu + 14 cpu_shifts + 19 cpu_load_delay + 24 cpu_branches + 7 cpu_jumps + 20 cpu_mult_div + 29 cpu_unaligned_load_store + 10 cpu_cop0_regs + 14 cpu_exception_mechanism + 1 cpu_exception_estado_previo + 9 cpu_tty_hook + 11 cpu_printf_hook + 11 cpu_opcode_reservado + 16 gpu_status_gp0_gp1 + 9 ci_scoreboard + 9 cli_runner + 21 gpu_vram_transfers + 20 gpu_triangulos_flat_gouraud + 2 mutation_manifest + 2 mutation_anchors + 5 mutation_battery)

Depois (iteracao inicial): 338 testes (+17: 16 gpu_linhas_retangulos + 1 extra na mutation_battery)

Depois (continuacao): 345 testes (+7: 3 D3 movidos para gpu_linhas_retangulos + 4 continuacao em gpu_linhas_retangulos_continuacao)

Scoreboard: 5 com veredito (1p/4f), 45 só com saída, 0 sem saída, 1 não avaliados, de 51 arquivos

## Decisões e notas

- Vertex para linha usa Bresenham clássico com `err = dx + dy` (soma, não diferença), que funciona em todos os octantes sem branch de inclinação
- Linha INCLUI a coordenada inferior-direita (L361-362), ao contrário de polígono que EXCLUI (L323). Esta é a diferença mais importante do item e foi capturada pelo mutante m1
- Polígono e linha usam regras OPOSTAS de borda: o rasterizador de linha NÃO pode reusar `xr.min(area_x2 + 1)` do triângulo
- Retângulo size=1/2/3 NÃO envia Width+Height; ler uma palavra a mais dessincroniza o FIFO (defeito G3/H2 de iterações anteriores)
- O estado `LineRender` original armazenava arrays fixos de 16 vertices/cores. A continuacao substituiu por desenho incremental: cada segmento de polyline e desenhado assim que o par (anterior, atual) fica completo, guardando apenas o ultimo vertice. Remove o teto de 16 vertices.
- Retangulo texturizado: AwaitUV e AwaitDims consomem UV/dims sem chamar render_rect. A cor do comando e ignorada (spec L402).
- Checagem de distancia maxima (dx>1023, dy>511) aplicada em TODOS os caminhos de linha: render_single_line simples (flat e gouraud) + draw_line_segment (polyline).
- render_rect com size=0 (variavel) agora limita os lacos a intersecao com a area de desenho, em vez de iterar sobre w/h crus com continue. Spec NAO tem formula de mascaramento para valores acima do teto 1023x511 (L405-411), diferente de Fill (L642-645) e Copy (L666-669). A escolha foi limitar o laco — preserva comportamento para tamanhos validos e termina para lixo.
- Texturizado: UV consumido e ignorado; textura real e Texpage no item 2.5
- Semi-transparencia e dithering fora de escopo (item 2.6)
