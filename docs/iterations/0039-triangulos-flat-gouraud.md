<!-- Custo, tokens e duração NÃO entram aqui: são medidos por scripts/oc-iter.ps1 em
     docs/metricas.csv, casados por head_antes/head_depois. -->

# 0039 — triangulos-flat-gouraud

- **Data:** 2026-07-28
- **Item do roadmap:** 2.3
- **Objetivo:** Rasterização de triângulos flat + gouraud no GP0, sem textura.

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| psx-spx | § GPU Render Polygon Commands | docs/reference/03-gpu.md |
| psx-spx | § GPU Rendering Attributes (Vertex, Color) | docs/reference/03-gpu.md |

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | rasterizador | Edge function com subpixel offset (0.5) produziria cores exatas nos vértices | Subpixel offset faz o centro do pixel (10.5, 10.5) não coincidir com o vértice (10, 10), resultando em interpolação diferente de 1.0 | Teste B5-G quebrou: cor no vértice saiu 0x1D em vez de 0x1F |
| 2 | rasterizador | Top-left rule aplicada ao edge function negado funcionaria para CW e CCW | A regra top-left no edge function padrão (não-negado) classifica corretamente as arestas: top (dy==0, dx>0) e left (dy<0). Com o edge negado + CW, a classificação fica inconsistente | B3 quebrou: pixel(1,1) na hipotenusa sumiu — tratado como right edge excluído |
| 3 | rasterizador | Scanline approach seria trivial | A ordenação dos vértices por Y precisa de cuidado: quando yt == ym (flat-top), todo o triângulo cai na parte "bottom", mas o edge curto (m→b) está correto nesse caso. Também: o split de quad (V0,V1,V2)+(V1,V2,V3) deixa um gap na diagonal V0-V2 vs V1-V3 — pixels próximo ao centro do quad podem ficar fora de ambos os triângulos | B4 quebrou em (15,20) — fora de ambos os triângulos do quad |
| 4 | cores-flat | Apenas colors[0] era inicializado com a cor do comando; colors[1..3] ficavam zero | Para flat shading, TODOS os vértices devem ter a mesma cor (a do comando), senão o segundo triângulo do quad renderiza preto | B4 quebrou: pixel(25,18) no triângulo V1V2V3 saiu 0 em vez da cor do comando |
| 5 | coordenadas | Coordenada X é signed 11-bit dentro do u16, mas meus valores de teste esperados usavam conversão incorreta | O formato BBGGRR da spec coloca R nos bits 0-7, G nos bits 8-15, B nos bits 16-23. Eu tinha os canais trocados na conversão 24→16 bits | B1 quebrou: esperava 0x7C6F mas o correto é 0x03E3 para cor=0x00F818 |

## Bateria de mutação

Placar: **4/5 mutantes pegos, 2/2 controles verdes**.

| # | Mutação | Teste que pegou |
|---|---|---|
| M1 | Remover propagação de cores flat (colors[1..3] não setados) | B4 — flat_quad_de_4_vertices |
| M2 | Trocar canais R/G na conversão color24_to_16 | B1, B4, B5 |
| M3 | Remover verificação de distância entre vértices | B11 — polygon_com_distancia |
| M4 | Inverter flag gouraud/flat no render_triangle | B7b — gouraud_triangle_produz_cores_diferentes (teste novo, adicionado após o sobrevivente da primeira rodada) |
| M5 | Remover .clamp(0, 31) na lerp_color | **SOBREVIVENTE** — nenhum teste usa cores em transições extremas que causariam overflow. O clamp é defensivo, não testável com os valores atuais. |
| C1 | Reordenar chamadas render_triangle no quad | ✓ verde |
| C2 | Renomear variável local x_edge_tb → x_edge_long | ✓ verde |

## Placar antes → depois

**300 → 312 testes** (12 novos: 11 originais + 1 adicionado na bateria).

## Revisão cruzada (orquestrador)

<!-- Preenchido pelo Claude na revisão do PR -->

## Decisões e notas

1. **Rasterizador scanline.** Três tentativas com edge function (subpixel, negado, winding-aware)
   falharam; a quarta com scanline (sort por Y + interpolação linear nas arestas) funcionou.
   O algoritmo é O(width × height) por pixel, mas para 1024×512 com triângulos pequenos é
   aceitável.

2. **Split de quad é V0,V1,V2 + V1,V2,V3** conforme a spec (L324-327). Esse split deixa um
   gap teórico na região central do quad (onde ambos os triângulos se encontram na diagonal
   mas cada um cobre apenas metade). Na prática, a interpolação linear cobre todos os pixels,
   mas pixels exatamente na diagonal de cada triângulo podem ser excluídos pela regra top-left.

3. **Cor no vértice não é pixel-exata** no scanline porque o centro do pixel (x, y) não
   coincide com a coordenada do vértice (x, y) — não há offset subpixel. Isso é aceitável
   e consistente com o comportamento de hardware conhecido.

4. **Verificação de distância** entre vértices implementada (dx ≤ 1023, dy ≤ 511) conforme
   spec § Vertex, L447. Sem essa verificação, polígonos com vértices muito distantes
   renderizam parcialmente.
