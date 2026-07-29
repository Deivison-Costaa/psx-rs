# 0048 — dithering

- **Data:** 2026-07-29
- **Item do roadmap:** 2.6b
- **Objetivo:** implementar dithering 24→15 bit com matriz 4x4 indexada por (y&3, x&3).

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| psx-spx | § 24bit RGB to 15bit RGB Dithering (L1558-1572) | docs/reference/03-gpu.md |
| psx-spx | § GP0(E1h) — Draw Mode setting (L492-509) | docs/reference/03-gpu.md |

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | saturação-gte | esperado 0x7BDF para (31,31,31) em T6, mas 31|31<<5|31<<10 = 0x7FFF | O valor 0x7BDF em binário dá R=31, G=29, B=30 — erro de digitação no esperado | test t6 falhou com left: 32767 (0x7FFF) right: 31711 (0x7BDF) |
| 2 | API-Rust | assinaturas de render_single_line e draw_line_segment cresceram para 8 argumentos | Clippy pede #[allow(clippy::too_many_arguments)] se >7 | clippy com -D warnings |
| 3 | API-Rust | controle K2 era identidade (DE == PARA) | O meta-teste mutation_anchors reprova DE e PARA idênticos ignorando whitespace | mutation_anchors rejeitou "registro 'K2', edicao 1: @@DE e @@PARA identicos" |

## Bateria de mutação

Placar da bateria: 7/7 mutantes mortos, 2/2 controles verdes, 0 equivalente — docs/mutantes/0048-dithering.mut

| Mutante | Rótulo | Morreu | Teste que matou |
|---|---|---|---|
| m1 | matriz linha 0 vira [-4,-4,-4,-4] | sim | T1(1,0) offset 0 esperado, obteve -4 |
| m2 | dither aplicado a retângulo via color24_to_16 | sim | T5 retangulo nao aplica dither |
| m3 | offset sempre +1 ignora matriz | sim | T1(0,0) offset -4 esperado, obteve +1 |
| m4 | if dither_enabled vira if false | sim | T1(3,0) sem dither, esperado 0x0021 obteve 0x0000 |
| m5 | linha nao aplica dither (let dither = false) | sim | T4(3,0) sem dither, esperado 0x0021 obteve 0x0000 |
| m6 | matriz linha 2 trocada pela linha 0 | sim | T1(1,2) offset +1 esperado, obteve 0 |
| m7 | saturação removida (sem clamp) | sim | T7(0,0) R=1-4=-3 sem clamp vira 253, >>3=31 |
| K1 | inverte ordem r e g em color24_to_16_dithered | — | controle cosmético |
| K2 | inverte ordem das entradas do sorted antes do sort | — | controle cosmético |

## Placar antes → depois

Workspace: **362** → **369** testes (362 existentes + 7 novos).

## Revisão cruzada (orquestrador)

Revisão do PR anterior (#61 — iter 0047): sem achados
- Teste que não mede: conferido — todos os 7 mutantes morreram; 2 controles sobreviveram
- Parâmetro não consumido: conferido — semi_transparent lido para polys, lines e rects; write_pixel consome todos os bits
- Regra de borda trocada: não se aplica ao blend
- Campo de bit lido errado: conferido — bit 1 (0x02) do cmd é semi-transparent para os 3 primitivos; mode de blend lido de GPUSTAT bits 5-6 via E1h
- Panic/laço: conferido — sem unwrap/expect/unsafe fora de teste; branch _ => inalcançável no match mode (2 bits)
- Citação de spec: pwsh scripts/confere-citacoes.ps1 verde
- Escopo transbordado: conferido — implementação cobre todos os primitivos conforme spec

## Decisões e notas

1. Dithering só se aplica a polígonos com gouraud shading (modulation não implementado — item 10.13). A spec diz "gouraud shading or modulation"; como modulação é dívida futura, o caminho de dither no `render_triangle` só dispara quando `gouraud && !textured`.
2. Para linhas, o dither é aplicado sempre que GPUSTAT.9=1, independente de gouraud. A conversão 24→15 bit foi movida para dentro de `render_single_line`, que agora recebe cores 24-bit (u32) e aplica `color24_to_16_dithered` por pixel.
3. Retângulos nunca aplicam dither — `render_rect` continua usando `color24_to_16` sem posição.
4. A implementação adiciona `render_triangle_dithered` como caminho separado para não alterar o comportamento de interpolação 5-bit do caminho normal. A interpolação em 8-bit antes da quantização é necessária para que o dither funcione corretamente (o offset é aplicado a canais de 8 bits).
