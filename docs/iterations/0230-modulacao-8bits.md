# 0230 — modulacao de textura em 8 bits

- **Data:** 2026-08-07 (codigo e bateria) / 2026-09-22 (doc, medicao e PR)
- **Item do roadmap:** frente `gpu/rectangles` (oraculo de hardware), ROADMAP 10.115
- **Objetivo:** a modulacao texel x cor do vertice tem de usar a cor com os 8 bits inteiros,
  como o GPU v2; cortar a cor para 5 bits antes do produto e o comportamento do GPU v0.

## Spec consultada

| Fonte | Secao | Arquivo local |
|---|---|---|
| psx-spx | § Modulation (also known as Texture Blending) | `docs/reference/03-gpu.md` |
| psx-spx | tabela de diferencas v0/v1/v2, linha "Shaded Textures" | `docs/reference/03-gpu.md` |
| psx-spx | § Shaded Textures, v0 corta 8:8:8 para 5:5:5 | `docs/reference/03-gpu.md` |

`§ Modulation` da a formula sobre canais de 8 bits: `finalChannel = texel * vertexColour / 128`.
`a tabela de versoes` separa as geracoes: v0 faz `((color/8)*texel)/2`, v2 faz `(color*texel)/16`.
Com o texel em 5 bits (canal do 15bpp) e a cor em 8 bits, `(t*8)*c/128 = t*c/16`: o
produto sai em 8 bits e so entao desce para os 5 bits do framebuffer.

## O defeito

`modulate_texel` recebia a cor ja convertida por `color24_to_16`, isto e, cortada para 5
bits, e fazia `t*c5/16`. E exatamente o GPU v0 descrito em § Shaded Textures ("crops 8:8:8 bit
gouraud shading color to 5:5:5 bit before multiplying"). Como o corte so descarta os 3
bits baixos da cor, o erro so pode **perder** brilho, nunca ganhar: `t=25`, `c=0x87`
(135) da `25*135/16 = 210 -> 26` no v2 e `25*16/16 = 25` no v0.

O conserto passa a cor de 24 bits inteira aos dois chamadores (poligono texturizado e
retangulo texturizado) e faz, por canal, `((t*c/16).min(255)) >> 3`.

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | saturacao | que a formula de § Modulation podia ser aplicada com os dois lados em 5 bits ("128 em 8 bits vira 16 em 5 bits", comentario antigo da funcao) | § Modulation fala em canais de 8 bits; a tabela de versoes e § Shaded Textures dizem que reduzir a cor a 5 bits ANTES e o bug do GPU v0 | oraculo de hardware `gpu/rectangles`: 2.954 pixels divergentes, todos 1 LSB mais escuros |
| 2 | processo | que as baterias antigas continuariam valendo depois de mudar a assinatura de `modulate_texel` | — | `mutation_anchors` reprovou 0200 e 0202: as ancoras citavam o corpo antigo em 5 bits. Reancoradas com rotulo e intencao preservados e rodadas de novo (commits `eaff128`, `82f9e24`, `8e7deea`) |
| 3 | processo | que a iteracao estava completa ao fim da sessao de 2026-08-07 | o protocolo exige doc da iteracao e placar no STATUS | a branch ficou 6 semanas sem doc; `mutation_reconciliation` e `status_handoff` reprovaram na retomada |

O erro 3 vale registro por si: a sessao anterior terminou com o codigo, o teste e as
tres baterias commitados e empurrados, mas sem o doc. Os meta-testes pegaram na
retomada — sem eles, o placar declarado da 0230 nunca teria sido reconciliado.

## Bateria de mutacao

Placar da bateria: 6/6 mutantes mortos, 2/2 controles verdes, 0 equivalente — docs/mutantes/0230-modulacao-8bits.mut

| mutante | o que faz | quem pega |
|---|---|---|
| m1 | volta a cortar a cor para 5 bits (GPU v0) | a1, a2, a3 |
| m2 | arredonda o produto em vez de truncar | a7 |
| m3 | arredonda a reducao de 8 para 5 bits | a5, a6, a7 |
| m4 | divisor 32 (ponto neutro vira FFh) | a1-a6 |
| m5 | satura em 127 | a1-a6 |
| m6 | troca o deslocamento do canal do texel pelo da cor | a1-a7 |

Controles: c1 (produto comutado) e c2 (`/16` escrito como `>> 4`) sobrevivem, como devem.
As baterias de 0200 e 0202, reancoradas, foram rodadas de novo sobre o novo corpo.

## Placar antes -> depois

Oraculos de VRAM (contagem propria RGB, VRAM inteira; mesma metrica da 0229), binario da
`main` (4a60dbc) contra o da branch:

| oraculo | antes | depois |
|---|---|---|
| clipping | 0 | 0 |
| clut-cache | 921 | 921 |
| lines | 362 | 362 |
| quad | 240 | 240 |
| **rectangles** | **2.954** | **0** |
| texture-flip | 0 | 0 |
| texture-overflow | 0 | 0 |
| transparency | 447.488 | 447.488 |
| triangle | 12.775 | 12.775 |
| uv-interpolation | 3.393 | 3.393 |
| vram-to-vram-overlap | 7.557 | 7.557 |
| mdec/4bit | 2 | 2 |
| mdec/8bit | 4 | 4 |

Um oraculo zera, nenhum piora. Workspace: 1434 -> 1441 testes.

## Regressao em jogo

Crash Bandicoot, 800 M passos, `start@330M` e `cross@700M`, 4 framebuffers por binario:
os tres primeiros quadros sao byte-identicos entre `main` e a branch; o quarto (mapa
`N. SANITY BEACH`) difere so no brilho das texturas moduladas, indistinguivel a olho.

## Revisao cruzada (orquestrador)

Sem achados na retomada: o diff de `gpu.rs` foi relido contra `03-gpu.md` (tabela de versoes e § Modulation), e o caso A1 recalculado a mao (25*135/16 = 210, que vira 26 em 5 bits).

## Decisoes e notas

- O `transparency` diverge em 447.488 pixels, mas **nenhum dentro da area de display**
  (0-319 x 0-239 bate inteira). O gabarito tem o resto da VRAM em branco uniforme
  (248,248,248) e o nosso nao. Nao e defeito de blending; foi registrado como frente
  propria (preenchimento da VRAM fora do display).
- Modulacao do retangulo e do poligono passam pela mesma funcao; nao ha caminho de
  modulacao separado para linhas (linhas nao sao texturizadas).
