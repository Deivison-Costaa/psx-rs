<!-- Custo, tokens e duração NÃO entram aqui: são medidos por scripts/oc-iter.ps1 em
     docs/metricas.csv, casados por head_antes/head_depois. -->

# 0202 — poligono-modulacao

- **Data:** 2026-08-05
- **Item do roadmap:** 10.13 (achado legado, aberto desde a 0110)
- **Objetivo:** consertar o achado 10.13 — GP0(24h) e variantes (bit24=0) tem que modular o
  texel pela cor interpolada do vertice, não desenhar o texel cru.

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| psx-spx | § GPU Render Polygon Commands, bit 24 "raw texture / modulation" (L264) | docs/reference/03-gpu.md |
| psx-spx | § Modulation (also known as Texture Blending) (L1604) | docs/reference/03-gpu.md |

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a medição mostrou | Como foi pego |
|---|---|---|---|---|
| 1 | escopo | Que consertar 10.13 zeraria boa parte do resto de `rectangles` (7.265px), como o handoff do STATUS sugeria | `rectangles.exe` roda comandos de RETÂNGULO (GP0 60h-7Fh), não polígono — o bit24 dessa faixa é um código de tamanho distinto (10.13 cita especificamente a seção de polígono, L264). `render_rect_textured` nunca foi tocado nesta iteração; o placar do scoreboard ficou em 7.265px, sem mudança | Rodei `scripts/scoreboard.ps1` antes e depois do fix: `rectangles`, `triangle`, `quad`, `uv-interpolation` todos idênticos px a px — nenhum oráculo local exercita modulação de um jeito que o diff capture (a maioria já diverge o VRAM inteiro por outros defeitos maiores) |
| 2 | fixture de teste | Que só precisava adicionar a modulação sem tocar nos testes existentes | 5 arquivos de teste (`gpu_quad_texturizado`, `gpu_textura_15bpp`, `gpu_texturas_4bpp_8bpp`, `gpu_texture_disable`, `gpu_texture_window`) usavam GP0(24h) — bit24=0, modulado — com cor do vertice 0x000000 (preta) só para testar amostragem de texel (4bpp/8bpp/15bpp/CLUT/janela). Com a modulação real, cor preta zera o resultado e os 21 usos quebrariam | Rodei a suite antes de migrar; falhas em massa apontando pixel preto onde o teste esperava o texel. Migrado para GP0(25h) (raw), que preserva a intenção original (testar fetch, não cor) |
| 3 | manifesto de mutação | Que `ocorrencias: 3` com âncora `.min(31)` (fragmento de linha) seria aceito, já que a expressão aparece 3 vezes no arquivo | `mutation_anchors` reprovou: "encontrada 0 vez(es)" — o casamento é por **linha inteira** (`docs/mutantes/README.md`), não substring. Reescrito como 3 linhas completas num único mutante (10.71 já documentava essa regra; eu não tinha lido antes de escrever) | `cargo test --test mutation_anchors` |

## Bateria de mutação

Placar da bateria: 5/5 mutantes mortos, 2/2 controles verdes, 0 equivalente —
`docs/mutantes/0202-poligono-modulacao.mut`.

- m1 (escala `/8` em vez de `/16`): morto.
- m2 (bit `raw_texture` invertido): morto — `bit_raw_ignora_a_cor_do_vertice_mesmo_preta` passa
  a ver preto em vez do texel.
- m3 (bit 15/STP perdido na modulação): morto — `modulacao_preserva_o_bit_15_stp_do_texel`.
- m4 (canal R modula por si mesmo, não pela cor): morto —
  `modulacao_escurece_pelo_canal_da_cor_do_vertice`.
- m5 (clamp em 15 em vez de 31): morto — `cor_neutra_808080_nao_muda_o_texel` (g=24 estoura 15).
- c1 (renomeia variáveis locais, com todos os usos): verde.
- c2 (reordena `cr`/`cg`/`cb`, cada um só depende de `color`): verde.

## Placar antes → depois

Workspace: **1243** → **1246** testes (4 novos em `gpu_poligono_modulacao.rs`).

Oráculo local (`scripts/scoreboard.ps1`): sem mudança em nenhuma suíte — ver erro de primeira
tentativa #1. Evidência real veio de fora do oráculo: dump de VRAM do Crash Bandicoot (USA) em
900 M passos (`--press start@330000000 --press cross@700000000`), comparado byte a byte entre
o binário antes e depois do fix — **200.749 de 524.288 pixels (38%) mudaram**, confirmando que
o caminho de modulação é exercitado de verdade em jogo.

## Revisão cruzada (orquestrador)

Sem achados — esta iteração foi conduzida pelo próprio orquestrador (exceção vigente em
`docs/orquestracao.md`; ver STATUS.md).

## Decisões e notas

**1. Achado 10.13 fechado só para polígono.** A citação original é da seção de polígono de
`docs/reference/03-gpu.md`; retângulos texturizados (GP0 60h-7Fh) têm o mesmo bit24 mas `render_rect_textured`
continua sempre raw — se isso importa, é achado novo, não parte do 10.13. Não abri achado novo
para isso agora porque não tenho evidência de que afete jogo algum (os testes de rect já
cobrem raw+CLUT+STP+wrap da 0200, e nenhum jogo testado usa retângulo modulado de forma
visível até aqui).

**2. `modulate_texel` opera em 5 bits por canal, não 8.** O restante do rasterizador já
interpola cor em RGB555 (perda de precisão pré-existente, não desta iteração). A fórmula da
spec (`texel*cor/128` em 8 bits) foi escalada para `texel*cor/16` em 5 bits — os dois
coincidem exatamente no caso neutro (128→16) e são consistentes por escala nos demais, dentro
da precisão que o resto do código já usa.
