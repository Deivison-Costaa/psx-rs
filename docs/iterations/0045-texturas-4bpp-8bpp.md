# 0045 — texturas-4bpp-8bpp

- **Data:** 2026-07-29
- **Item do roadmap:** 2.5b
- **Objetivo:** Implementar texturas 4bpp e 8bpp com lookup via CLUT (Color Lookup Table).

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| psx-spx | Clut Attribute (L481-491) | docs/reference/03-gpu.md |
| psx-spx | Texture Bitmaps 8bit e 4bit (L1299-1306) | docs/reference/03-gpu.md |
| psx-spx | Texture Palettes — CLUT (L1318-1331) | docs/reference/03-gpu.md |
| psx-spx | UV/CLUT nos polígonos texturizados (L269-290) | docs/reference/03-gpu.md |

## Revisão do PR anterior (iter 0044)

Revisão do PR anterior: sem achados

- Teste que mede: verificado — cada um dos 6 testes de textura 15bpp tem pelo menos um mutante que o mata
- Parâmetro não consumido → FIFO dessincronizado: contagem de palavras conferida para GP0(24h) texturizado (7 palavras)
- Regra de borda trocada: polígono usa exclusive right-bottom, drawing area usa inclusive top-left, linha usa inclusive — conferido
- Campo de bit lido errado: máscaras 0x1FF, bit 11→GPUSTAT.15, page_y do bit 4 — conferido
- Panic/laço ilimitado: saturating_sub, clamp, xl..xr com clipping — conferido
- Citação de spec: confere-citacoes.ps1 verde
- Escopo transbordado/dívida: simplificações declaradas como decisões 1-5 no doc 0044, itens 10.13 e 10.14 no ROADMAP

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | endereçamento | Escrevi CLUT no offset 0 da base (posição 48,50 mapeava CLUT[0] e CLUT[1]) | Cada entrada CLUT ocupa 1 halfword; CLUT[3] está em base+3, não base+0 | Teste B3: escrevi CLUT[3]=0x9ABC em (48,50) mas CLUT base=48 → entrada 3 está em (51,50) |
| 2 | endereçamento | CLUT e textura no mesmo endereço base (VRAM[0]) não conflitam | A escrita da textura sobrescrevia CLUT[0], quebrando o teste de transparência | Teste B4: CLUT[0]=0x0000 (transparente) virava 0x0001 (dado da textura), pixel não transparente desenhava |
| 3 | endereçamento | E1h define texpage e isso basta para textura 15bpp no teste misto | O atributo Texpage no vértice 1 sobrescreve GPUSTAT; se o UV não carrega o texpage correto, o modo volta para o default | Teste B6: vertex 1 UV com 0x00800000 zerava tex_colors, 15bpp virava 4bpp |
| 4 | regressão | Mudar tex_active de `== 2 \|\| == 3` para `<= 3` é inócuo para testes existentes | Dois testes de FIFO (`gpu_triangulos_flat_gouraud`) enviavam UV dummy (0x0000) e esperavam cor plana; com textura ativa em modo 4bpp, CLUT vazio → nada desenhado | Testes `polygon_texturizado_consome_palavras_de_uv` e `gouraud_texturizado_consome_9_palavras` passaram a falhar na suite completa |
| 5 | manifesto | Manifesto da iter 0044 usa âncoras que envelheceram com a reestruturação de sample_texel | — | CI: mutation_anchors reprovou m4, m5, K1, K2; manifesto arquivado |

## Bateria de mutação

Placar da bateria: 6/6 mutantes mortos, 2/2 controles verdes, 0 equivalente - ./docs/mutantes/0045-texturas-4bpp-8bpp.mut

| ID | Tipo | Mutação | Teste que pegou |
|---|---|---|---|
| m1 | 4bpp nibble shift errado (×8) | `(u_clamped % 4) * 4` → `(u_clamped % 4) * 8` | B2 (4bpp 4 pixels) |
| m2 | 8bpp sempre low byte | `if/else` → `hw & 0xFF` | B1 (8bpp two pixels) |
| m3 | CLUT X sem ×16 | `(attr & 0x3F) * 16` → `attr & 0x3F` | B3 (CLUT posição arbitrária) |
| m4 | CLUT Y shift errado | `attr >> 6` → `attr >> 4` | B3 (CLUT posição arbitrária) |
| m5 | tex_active só 15bpp | `tex_colors <= 3` → `tex_colors == 2 \|\| tex_colors == 3` | B1 e B2 (8bpp e 4bpp sem textura) |
| m6 | CLUT do vértice 1 em vez do 0 | `vertex_idx == 0` → `vertex_idx == 1` | B3 e B4 (CLUT na posição errada) |
| K1 | Renomeia nibble → idx | Refatoração cosmética em sample_texel | Nenhum (controle verde) |
| K2 | Inverte ordem clut_x/clut_y | Reordenação de declarações independentes | Nenhum (controle verde) |

## Placar antes → depois

Antes: 345 testes (10 meta-testes + 8 bus_bios + 2 bios_flag + 1 version + 12 bus_scheduler + 9 bus_scratchpad_isc + 8 cpu_fetch_decode + 26 cpu_alu + 14 cpu_shifts + 19 cpu_load_delay + 24 cpu_branches + 7 cpu_jumps + 20 cpu_mult_div + 29 cpu_unaligned_load_store + 10 cpu_cop0_regs + 14 cpu_exception_mechanism + 1 cpu_exception_estado_previo + 9 cpu_tty_hook + 11 cpu_printf_hook + 11 cpu_opcode_reservado + 16 gpu_status_gp0_gp1 + 9 ci_scoreboard + 9 cli_runner + 21 gpu_vram_transfers + 20 gpu_triangulos_flat_gouraud + 21 gpu_linhas_retangulos + 6 gpu_textura_15bpp + 1 spec_citations + 2 mutation_manifest + 2 mutation_anchors + 5 mutation_battery).

Depois: 351 testes (+6 gpu_texturas_4bpp_8bpp).

## Decisões e notas

1. **CLUT armazenado em campo separado do GPUSTAT.** O atributo CLUT (vértice 0, bits 16-31 do UV) NÃO vai para GPUSTAT — ele é guardado em `Gpu.clut_attribute`. A spec (`docs/reference/03-gpu.md` L286-287) diz que "the first word holds the Clut index" e o formato está em Clut Attribute (`docs/reference/03-gpu.md` L485-489), sem menção a GPUSTAT.

2. **Modos 0 e 1 agora ativam textura.** Antes da 0045, `tex_active` só aceitava modos 2 e 3 (15bpp). Agora `tex_colors <= 3` cobre todos os modos. Isso mudou o comportamento de polígonos texturizados com modo 4bpp/8bpp default (sem E1h prévio): antes caíam no caminho de cor plana/gouraud, agora tentam samplear textura com CLUT default (0,0). Os dois testes de FIFO em `gpu_triangulos_flat_gouraud` foram ajustados para refletir o novo comportamento.

3. **Transparência herdada do 15bpp.** Entrada CLUT 0x0000 resulta em pixel transparente (não desenhado), usando o mesmo `if texel == 0 { continue; }` do caminho 15bpp. A spec de CLUT (`docs/reference/03-gpu.md` L1324) confirma: "Color 0000h = Fully-transparent".

4. **CLUT resetado em GP1(00h).** O reset da GPU (`write_gp1` cmd 0x00) agora também zera `clut_attribute`, consistente com o reset completo do estado de renderização.

5. **Texture window, X/Y-Flip e dithering continuam fora de escopo** (itens 2.5c e 2.6).

## Revisão cruzada (orquestrador)

<!-- Preenchido pelo Claude na revisão do PR -->
