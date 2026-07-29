# 0047 — semi-transparencia

- **Data:** 2026-07-29
- **Item do roadmap:** 2.6a
- **Objetivo:** Implementar semi-transparência — 4 modos de blend (GPUSTAT bits 5-6), controlada por bit 25 do comando de renderização.

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| psx-spx | GP0(E1h) bits 5-6 (L496-497), Semi-transparency (L1582-1602), GP0(20h..7Fh) Render Command Bits (L1512-1523) | docs/reference/03-gpu.md |

## Revisão do PR anterior (iter 0046)

Revisão do PR anterior: sem achados

- Teste que não mede: bateria 6/6 mortos, cada teste tem asserção com golden value distinto que não se confunde com VRAM limpa
- Parâmetro não consumido → FIFO dessincronizado: E2h retorna VramState::Idle, comando de palavra única
- Regra de borda trocada: textura não depende de borda de rasterização — N/A
- Campo de bit lido errado: bits 0-4 mask X, 5-9 mask Y, 10-14 offset X, 15-19 offset Y conferidos contra spec; mutante m6 testa shift errado
- Panic/laço ilimitado: sample_texel clampa 0..255, VRAM com máscara de bounds
- Citação de spec: confere-citacoes.ps1 verde
- Escopo transbordado/dívida: apenas o que o item 2.5c pedia

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | bit de comando | Usei `(cmd & 0x20)` para detectar textura em retângulos | O bit 26 (texture mapping) é o bit 2 do cmd byte: `(cmd & 0x04)` — `0x20` é o bit 5, sempre 1 para retângulos (primitive type 3) | Compilou mas teria quebrado todos os retângulos não-texturizados; corrigi revendo a tabela de bits do spec `docs/reference/03-gpu.md` (L1512-1523) e o diff mostrou a regressão no `0x04` → `0x20` |
| 2 | formatação | Âncora de mutação em linha única funciona após cargo fmt | cargo fmt quebra chamadas longas em múltiplas linhas; âncoras DE/PARA precisam casar exatamente | mutation_anchors falhou em m8 e K2 do 0042 após fmt |
| 3 | identidade | `(0u32 << 5)` como placeholder explícito para o modo 0 nos testes | clippy `identity_op` reprova `0 << N`; o modo 0 é o default e não precisa do OR | clippy -D warnings |

## Bateria de mutação

Placar da bateria: 7/7 mutantes mortos, 2/2 controles verdes, 0 equivalente - ./docs/mutantes/0047-semi-transparencia.mut

| ID | Tipo | Mutação | Teste que pegou |
|---|---|---|---|
| m1 | Modo errado | modo 1 (B+F) trocado por modo 2 (B-F) | T2, T4 |
| m2 | Flag ignorada | `semi_transparent &&` removido → sempre blend | T5 |
| m3 | Modo errado | modo 0 (avg) trocado por modo 1 (aditivo) | T1 |
| m4 | Modo errado | modo 3 (B+F/4) trocado por modo 1 (B+F) | T4 |
| m5 | Blend removido | write_pixel sempre escreve pixel cru | T1, T2, T3, T4 |
| m6 | Máscara errada | bit 14 em vez de bit 15 (0x4000 vs 0x8000) | T6 |
| m7 | Bit errado | semi_transparent lido de bit 4 (0x10) em vez de bit 1 (0x02) do cmd | T5 |
| K1 | Cosmético | inverte ordem das extrações r_b e g_b (independentes) | Nenhum (controle verde) |
| K2 | Cosmético | inverte operandos em modo 0 (comutativo) | Nenhum (controle verde) |

## Placar antes → depois

Antes: 356 testes.

Depois: 362 testes (+6 gpu_semi_transparencia).

## Decisões e notas

1. **Item 2.6 foi dividido em 3 (R4).** Semi-transparência (2.6a), dithering (2.6b), mask bit (2.6c). Implementar os 3 de uma vez violaria R4 e exigiria um arquivo de teste com >500 linhas.

2. **Blend opera sobre canais de 5 bits (R5G5B5).** As fórmulas da spec usam 8 bits (0-255), mas o framebuffer armazena 5 bits por canal. A implementação opera diretamente nos 5 bits com deslocamento de bits (equivalente a dividir/multiplicar por 8).

3. **Modo 0 (B/2+F/2) é a média aritmética.** `(r_b >> 1) + (r_f >> 1)` — comutativo, resultado máximo 30 (cabe em 5 bits sem saturação).

4. **Modo 2 (B-F) usa i32 para evitar underflow.** Subtração de u16 pode dar negativo; a conversão para i32 antes da subtração e `.max(0)` garante o clamp.

5. **Untextured polygons têm bit15=0.** `color24_to_16` nunca seta bit 15, então polígonos não-texturizados nunca disparam semi-transparência, mesmo com bit 25=1. O teste usa polígonos texturizados 15bpp com texel tendo bit15=1.

6. **O campo `semi_transparent` foi adicionado a PolygonRender, LineRender e RectRender.** Isso exigiu atualizar âncoras de mutação antigas (0042 m8, K2) que referenciam `draw_line_segment` cuja assinatura ganhou um parâmetro.

7. **Semi-transparência NÃO afeta Fill-VRAM.** O `execute_fill` escreve diretamente na VRAM sem passar por `write_pixel`.

## Revisão cruzada (orquestrador)

<!-- Preenchido pelo Claude na revisão do PR -->
