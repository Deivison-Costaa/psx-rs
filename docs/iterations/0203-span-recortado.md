<!-- Custo, tokens e duração NÃO entram aqui: são medidos por scripts/oc-iter.ps1 em
     docs/metricas.csv, casados por head_antes/head_depois. -->

# 0203c — span-recortado

- **Data:** 2026-08-05
- **Item do roadmap:** 10.14 (achado legado, iteração de origem desconhecida)
- **Objetivo:** cor gouraud e coordenadas de textura (U/V) de um triângulo têm que ser
  interpoladas pelo span original do vértice, não pelo span já recortado pela drawing area —
  recortar deve mudar só quais pixels ficam visíveis, não a cor/textura deles.

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| psx-spx | § Vertex (Parameter for Polygon, Line, Rectangle commands) — "the hardware renders only the portion that is inside of the drawing area" (L452) | docs/reference/03-gpu.md |

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a medição mostrou | Como foi pego |
|---|---|---|---|---|
| 1 | ferramenta | Que um `@@DE` com uma linha em branco no meio (duas declarações separadas por uma linha vazia) seria aceito, como os blocos multi-linha já usados em manifestos anteriores | `mutation_anchors` reprovou 3 registros com "encontrada 0 vez(es)" mesmo com o texto batendo byte-a-byte contra o fonte (conferido com Python); o parser do manifesto (`mutation_format.rs:175`) descarta QUALQUER linha vazia antes mesmo de checar se está dentro de um bloco `@@DE`/`@@PARA` em coleta — uma âncora com linha em branco no meio nunca é reconstruída corretamente. Reescrevi as âncoras pra não cruzar linha em branco (isolando cada `let` mutado na própria linha, sem o `\n` vazio de separação visual do código-fonte) |
| 2 | escopo do teste | Que testar só a interpolação de U (com V fixo em 0 nos três vértices) bastava pra cobrir "gouraud/UV sobre span recortado" | A bateria de mutação (m3, mutante isolado em `tex_v`) sobreviveu: com v0=v1=v2 iguais, `lerp_i32(vl, vr, ...)` sempre devolve o mesmo valor não importa o quê aconteça com o denominador — a fórmula `a + (b-a)*t/t_max` colapsa quando `a == b`. Precisei de um terceiro teste espelhando o de U mas variando V com U fixo |

## Bateria de mutação

Placar da bateria: 5/5 mutantes mortos, 2/2 controles verdes, 0 equivalente —
`docs/mutantes/0203-span-recortado.mut`.

- m1 (cor gouraud volta a interpolar pelo `xl`/`xr` pós-recorte): morto por
  `gouraud_sobre_span_recortado_da_a_mesma_cor_do_span_inteiro`.
- m2 (tex_u volta a interpolar pelo span recortado): morto por
  `uv_sobre_span_recortado_amostra_o_mesmo_texel_do_span_inteiro`.
- m3 (tex_v volta a interpolar pelo span recortado): morto por
  `v_sobre_span_recortado_amostra_o_mesmo_texel_do_span_inteiro` (só depois do erro #2 acima).
- m4 (`xl_span` captura o `xl` já recortado): morto pelos três testes.
- m5 (`dx_span` usa a largura já recortada): morto pelos três testes.
- c1 (trocar a ordem das duas declarações de span): verde.
- c2 (parênteses redundantes em `dx_span`): verde.

## Placar antes → depois

Workspace: **1248** → **1251** testes (3 novos em `gpu_span_recortado.rs`).

## Revisão cruzada (orquestrador)

Sem achados — esta iteração foi conduzida pelo próprio orquestrador (exceção vigente em
`docs/orquestracao.md`; ver STATUS.md).

## Decisões e notas

**1. `render_triangle_dithered` tem o mesmo padrão de bug e não foi corrigido aqui.** O
caminho dither+gouraud+não-texturizado (`cmd & 0x10 != 0`, dither ligado, sem textura) usa a
mesma estrutura de recorte (`xl`/`xr` recortados reusados como base da fração de
`lerp_color24`) — mas eu não escrevi um teste dedicado pra esse caminho nesta iteração, e R5
proíbe corrigir sem teste que falhe antes. Registrado como achado novo `0203.1` pra não ficar
perdido.

**2. Por que 75px de span e clip em X1=130 especificamente.** O triângulo (topo (100,0),
meio (0,50), base (200,100)) foi escolhido pra que, na scanline y=75, a aresta visível vá de
x=100 a x=175 (span de 75px, todos os denominadores das frações exatos, sem arredondamento
escondendo o bug) — recortando em X1=130 corta a metade esquerda mas deixa o pixel de amostra
x=150 visível nos dois renders, então a única forma de eles darem valores diferentes é o bug
de reinterpolação.
