# 0200 — rect-texturizado

- **Data:** 2026-08-04
- **Item:** Achado 10.11 (textura e texpage de retângulos: UV consumido e ignorado)
- **Objetivo:** desenhar retângulos/sprites texturizados — o HUD do Crash (frutas, vidas,
  Aku Aku) e de todo jogo 2D passa por GP0(64h-7Fh).

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| psx-spx | § GPU Render Rectangle Commands (L373-437) | docs/reference/03-gpu.md |
| psx-spx | § Mask Bit Setting, "upper bit ... equal to bit15 of the texture color" (L585) | docs/reference/03-gpu.md |

## O que entrou

- `render_rect_textured`: varredura UV incrementada por pixel com wrap de 256, clip pela
  drawing area, CLUT vindo do UV word do comando, texel 0x0000 totalmente transparente,
  semi-transparência gateada pelo bit STP do texel. Texpage vem do E1, como a spec manda
  para rects. X/Y-flip (E1 bits 12/13) continua fora — achado 0193.7.
- Os três pontos do dispatch que consumiam o comando e descartavam (`vram_state=Idle` sem
  desenhar) agora chamam o rasterizador.
- **Blend passou a escrever o bit 15 do texel no VRAM** (03-gpu.md L585) — o caminho dos
  polígonos descartava. Os testes t1-t4 de `gpu_semi_transparencia.rs` fixavam o
  comportamento errado e foram atualizados com citação (padrão 10.115: melhoria legítima
  reprova teste antigo).

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | endereçamento | y=600 era coordenada válida de teste | VRAM tem 512 linhas | reli o teste antes de rodar |
| 2 | nenhum (do código) — a implementação passou os 7 testes na primeira rodada | | | |

## Bateria de mutação

Placar da bateria: **6/6 mutantes mortos, 2/2 controles verdes, 0 equivalente.**

A primeira rodada deu 5/6: o m5 (tamanho variável vira 8×8) sobreviveu porque os texels
fora do 3×2 do teste eram 0 (transparentes) e o desenho extra não pintava nada — o teste
foi endurecido (textura 8×8 cheia) e a bateria reexecutada inteira. Manifestos antigos
0042 e 0047 arquivados: o bloco de clip duplicado tirou a unicidade das âncoras m5/m11 da
0042, e o fix do bit 15 reescreveu a linha ancorada pela m5 da 0047.

## Placar antes → depois

- Workspace: 1235 → 1242 (7 testes novos em `gpu_rect_textured.rs`).
- Scoreboard (régua do item): `rectangles` 11.560px → **7.265px** (o resto é modulação
  10.13 e flip 0193.7); `texture-overflow` 32.768px → **vram-ok 0px** (o wrap de 256
  acertou); `vram-to-vram-overlap` 14.474px → 7.557px (bônus do bit 15 no blend).
  Duas suítes pixel-perfeito agora (clipping, texture-overflow).

## Revisão cruzada (orquestrador)

n/a — o orquestrador é o autor (exceção registrada em `docs/orquestracao.md`, 2026-08-03).

## Decisões e notas

- Modulação de cor (bit 24 = 0) segue não implementada — é o 10.13, próximo da escada;
  por ora rect modulado desenha como raw, igual ao caminho dos polígonos.
- `sample_texel` clampa U/V em 0..255 em vez de dar wrap; o rect contorna aplicando
  `& 0xFF` antes de amostrar. O clamp interno segue como está (tocaria polígonos — 10.14).
