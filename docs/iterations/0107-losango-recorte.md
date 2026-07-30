# 0107 — losango-recorte

- **Data:** 2026-07-30
- **Item do roadmap:** 2.2d
- **Objetivo:** descobrir por que o losango do logo da BIOS aparece só pela metade, e consertar.
  **Resultado: não há defeito comprovado.** A iteração entrega a medição e a decisão de não mexer.

## Revisão do PR anterior

PR #122 (iter 0106), do próprio orquestrador: quatro checks verdes, `headRefOid` conferido, bateria
6/6 depois de dois mutantes sobreviventes corrigidos.

## Spec consultada

`docs/reference/03-gpu.md`:

- **L1004-1008** — `GP0(E3h)` canto superior-esquerdo da área de desenho, `GP0(E4h)` canto
  inferior-direito, `GP0(E5h)` offset somado a todo vértice.
- **L692-693** — o recorte pela área de desenho vale para o rasterizador (e **não** para as
  transferências de VRAM, que são absolutas).

## O que foi medido

O handoff da 0106 dizia que o rasterizador perdia a metade inferior de cada triângulo — "padrão
clássico de divisão em meia-superior e meia-inferior". **Falso.** O laço de scanline percorre o
triângulo inteiro; quem corta é a área de desenho, e ela corta certo.

Palavras cruas emitidas pela BIOS, medidas no ponto de decodificação do GP0:

| Comando | Palavra | Decodificação |
|---|---|---|
| `E3h` | `E3000400` | área começa em (0, 1) |
| `E4h` | `E403C27F` | área termina em (639, 240) |
| `E5h` | `E5000800` | offset (0, 1) |
| `E3h` | `E303C400` | buffer 2: área começa em (0, 241) |
| `E4h` | `E407827F` | buffer 2: área termina em (639, 480) |
| `E5h` | `E5078800` | buffer 2: offset (0, 241) |

O offset é **igual à origem da área** nos dois buffers — o padrão normal de double buffering, com
vértices em coordenadas relativas ao buffer. E os vértices do losango, ainda **crus**, vão de
`y=112` a `y=368`: **256 linhas dentro de uma área de 240**. Os 720 triângulos com dither são
cortados, todos, e nos dois buffers a fração visível é a mesma.

Ou seja: a própria BIOS desenha uma figura mais alta que a área que ela mesma programou, e nós
recortamos exatamente como a spec manda.

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que o comportamento real é | Como foi pego |
|---|---|---|---|---|
| 1 | **hipótese** | Que a metade faltante fosse defeito do rasterizador, dividido em meia-superior e meia-inferior | O laço vai de `y_start` a `y_end` cobrindo o triângulo todo. O corte vem da área de desenho, que é regra da spec | Li a função antes de escrever o teste. O handoff que eu mesmo tinha escrito estava errado — **duas iterações seguidas** em que atribuí um defeito visual ao componente errado (na 0105 foi o blit) |
| 2 | ferramenta | Que instrumentar `render_triangle` bastasse para ver os triângulos do losango | Eles nem passam por lá: `render_triangle` desvia no começo para `render_triangle_dithered` quando é gouraud com dither, e o losango é exatamente isso. Minha primeira medição mostrou 8 triângulos quando havia 720 | O número não batia com o despejo de polígonos. Instrumentei a outra função e apareceram os 720 |

## Bateria de mutação

Bateria de mutação: não se aplica — esta iteração não altera código de produção. O que ela entrega
é uma medição e a decisão de **não** mexer no recorte, que já é coberto pelos testes da 2.3 e da
2.7a. Mutar a área de desenho aqui só reprovaria testes de outro item.

## Placar antes → depois

Workspace: **735** → **735** testes. Nenhum código mudou.

## Revisão cruzada (orquestrador)

Iteração inteira do orquestrador.

## Decisões e notas

1. **Não mexer é a decisão, e ela é ativa.** Empurrar o offset ou afrouxar o recorte para o
   losango "ficar bonito" quebraria o recorte pela área de desenho, que tem teste próprio e é
   regra da spec. Sem uma **referência** (foto de console, suíte de hardware, ou o mesmo BIOS em
   outro emulador), qualquer mudança aqui é chute com aparência de conserto.
2. **O item 2.2d fica aberto, com o texto trocado** para dizer o que foi medido: 256 linhas contra
   240 de área. Quem pegar precisa trazer a referência junto.
3. **Invariante 20 registrada** para que a próxima pessoa não repita a caçada.
4. **Duas iterações seguidas com hipótese errada é um dado, não um acidente.** Na 0105 atribuí o
   defeito ao blit VRAM→VRAM (que nem é emitido); aqui, ao rasterizador (que está certo). As duas
   vezes o handoff foi escrito **antes** da medição. A correção de processo é a mesma que já está
   no handoff da 0105: medir se o componente sequer participa antes de acusá-lo.
