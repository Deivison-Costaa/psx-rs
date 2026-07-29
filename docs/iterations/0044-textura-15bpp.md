# 0044 — textura-15bpp

- **Data:** 2026-07-29
- **Item do roadmap:** 2.5a
- **Objetivo:** Implementar GP0(E1h) Texpage, atributo Texpage em polígonos texturizados e amostragem de textura 15bpp com regra de transparência.

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| psx-spx | GP0(E1h) Draw Mode setting (L492-520) | docs/reference/03-gpu.md |
| psx-spx | Texpage Attribute (L471-479) | docs/reference/03-gpu.md |
| psx-spx | Clut Attribute (L481-491) | docs/reference/03-gpu.md |
| psx-spx | Texture Origin and X/Y-Flip (L417-425) | docs/reference/03-gpu.md |
| psx-spx | Texture Bitmaps 16bit — regra de transparência (L1295-1297) | docs/reference/03-gpu.md |
| psx-spx | UV/CLUT nos polígonos texturizados (L269-290) | docs/reference/03-gpu.md |
| psx-spx | GPU Texture Caching — fora de escopo (L1349) | docs/reference/03-gpu.md |

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | indexação | UV da palavra recebida ia para `uvs[vertex_count]` | O vertex_count já foi incrementado quando o vertex word foi processado; a UV corresponde ao vertex_count - 1 | Teste A4: UVs trocados produziam texel errado nos pixels do interior |
| 2 | geometria | Triângulo de 3 pixels de largura com 2 unidades de UV mapeava 1:1 | Com 3 pixels e 2 unidades de UV, cada pixel avança 0.67 UV → inteiro = 0 nos dois primeiros pixels | Teste A5: pixel (11,10) amostrava u=0 em vez de u=1, batendo no texel 0x0000 em vez do 0x7FFF |
| 3 | geometria | Page Y base=256 significa que texel (0,0) está em VRAM Y=0 com offset no cálculo | Page Y base é somado à coordenada V do texel: VRAM[(256+v)*1024 + (page_x+u)], portanto texel(0,0) vai para Y=256 | Teste A6: esperava 0xAAAA em VRAM Y=0 mas amostra correta era 0xBBBB em VRAM Y=256 |

## Bateria de mutação

Placar da bateria: 6/6 mutantes mortos, 2/2 controles verdes, 0 equivalente - ./docs/mutantes/0044-textura-15bpp.mut

| ID | Tipo | Mutação | Teste que pegou |
|---|---|---|---|
| m1 | Texpage attribute sobrescreve bits 9-10 | mask de 0x1FF para 0x7FF em apply_texpage_if_second | A3 (dither permanece 1) |
| m2 | Texel 0x0000 desenhado | Remove `if texel == 0 { continue; }` | A5 (fundo sobrevive) |
| m3 | Bit 11 do E1h → GPUSTAT.11 | << 15 → << 11 no E1h handler | A1 (GPUSTAT.15 não setado) |
| m4 | Colors=3 não tratado como 15bpp | `tex_colors == 2 \|\| tex_colors == 3` → `tex_colors == 2` | A2 (colors=3 funciona como 15bpp) |
| m5 | Page Y base lido do bit 15 | `stat >> 4` → `stat >> 15` em sample_texel | A6 (texel correto com page_y=256) |
| m6 | UV armazenado em vertex_count | `vertex_count.saturating_sub(1)` → `vertex_count` | A4 (texel exato nos 3 pixels) |
| K1 | Renomeia tex_colors → mode | Refatoração cosmética em sample_texel | Nenhum (controle verde) |
| K2 | Inverte ordem page_x/page_y | Reordenação de declarações independentes | Nenhum (controle verde) |

## Placar antes → depois

Antes: 339 testes (10 meta-testes + 8 bus_bios + 2 bios_flag + 1 version + 12 bus_scheduler + 9 bus_scratchpad_isc + 8 cpu_fetch_decode + 26 cpu_alu + 14 cpu_shifts + 19 cpu_load_delay + 24 cpu_branches + 7 cpu_jumps + 20 cpu_mult_div + 29 cpu_unaligned_load_store + 10 cpu_cop0_regs + 14 cpu_exception_mechanism + 1 cpu_exception_estado_previo + 9 cpu_tty_hook + 11 cpu_printf_hook + 11 cpu_opcode_reservado + 16 gpu_status_gp0_gp1 + 9 ci_scoreboard + 9 cli_runner + 21 gpu_vram_transfers + 20 gpu_triangulos_flat_gouraud + 21 gpu_linhas_retangulos + 1 spec_citations + 2 mutation_manifest + 2 mutation_anchors + 5 mutation_battery).

Depois: 345 testes (+6 gpu_textura_15bpp).

## Decisões e notas

1. **Textura para modos 4bpp/8bpp desabilitada.** Se o polígono é texturizado mas o texpage mode não é 15bpp (2 ou 3), o renderizador desenha como polígono não-texturizado (cor plana/gouraud). Isso preserva o comportamento dos testes existentes que consomem UV sem textura (item 10.9). Os modos 4bpp e 8bpp serão implementados no item 2.5b.

2. **Semi-transparência não implementada.** Texel com bit15=1 (0x8000..0xFFFF) é desenhado como opaco neste item. A semi-transparência (B/2+F/2, B+F, etc.) será implementada no item 2.6. Esta é uma simplificação consciente documentada aqui.

3. **Bit 11 do E1h ignorado para endereçamento.** Esta GPU tem 1 MB de VRAM (v0 GPU). O bit 11 de E1h é guardado no GPUSTAT.15 mas NÃO é usado no cálculo do endereço da textura (`docs/reference/03-gpu.md` L513-517). Apenas bit 4 controla page_y (0 ou 256).

4. **Clut do primeiro vértice é consumido e ignorado.** O atributo Clut (bits 16-31 do UV do primeiro vértice) é recebido mas não processado — será implementado no item 2.5b.

5. **Texture window, X/Y-Flip e dithering fora de escopo.** Seus bits são guardados no GPUSTAT via E1h mas seus efeitos não são implementados (itens 2.5c e 2.6).

## Revisão cruzada (orquestrador)

<!-- Preenchido pelo Claude na revisão do PR -->
