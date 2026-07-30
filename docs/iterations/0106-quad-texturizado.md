# 0106 — quad-texturizado

- **Data:** 2026-07-30
- **Item do roadmap:** 2.2c
- **Objetivo:** o logo da BIOS desenhava a palavra "SONY" como duas barras vermelhas chapadas.
  Achar por que o quad texturizado não amostra a textura e consertar.

## Revisão do PR anterior

PR #121 (iter 0105), do próprio orquestrador: quatro checks verdes, `headRefOid` conferido, bateria
6/6 com 1 equivalente justificado.

## Spec consultada

`docs/reference/03-gpu.md`:

- **L246-252** — a VRAM é 512 linhas de 1024 halfwords, e *"The horizontal coordinates are
  addressing memory in 4bit/8bit/16bit/24bit/halfword units"*. A coordenada **horizontal** muda de
  unidade conforme a profundidade; a linha é escolhida separadamente.
- **L494-497** — base X da texpage em passos de 64 halfwords, base Y em passos de 256 linhas,
  bits 7-8 escolhem 4bit/8bit/15bit.

## A caça

O handoff da 0105 mandava contar cores distintas na região do defeito. Foi o que decidiu:

1. **Barras: 5 cores distintas** (vermelho chapado + fundo). **Losango: 37**, com gradiente suave.
   Gouraud funciona; a textura não está sendo amostrada.
2. **Histograma exato de GP0:** 360 comandos `2Ch` (quad texturizado, opaco, modulado).
3. **Despejo dos parâmetros de cada quad:** vértices `(200,69)..(440,93)`, UV `(0,0)..(239,47)`,
   `GPUSTAT=0x5006260D` → textura **4bpp**, texpage X=832, Y=0; CLUT em (256,480). Tudo correto —
   e a texpage X=832 é exatamente onde o sprite `SONY COMPUTER ENTERTAINMENT` está na VRAM.

Com os parâmetros certos e o resultado errado, sobrou o cálculo do endereço do texel:

```rust
let pixel_index = v_clamped * 256 + u_clamped;
let hw_x = page_x + (pixel_index / 4);   // <- v entra aqui
let hw_y = page_y + v_clamped;            // <- e a linha ja e escolhida aqui
```

**Causa raiz:** `v` era contado **duas vezes**. O deslocamento horizontal é `u/4` (4 texels por
halfword em 4bpp) e a linha vem de `page_y + v`. Somar `v*256` antes de dividir por 4 faz cada
linha andar **64 halfwords** para a direita, lendo lixo. O mesmo no 8bpp, com 128. O caminho de
15bpp nunca teve o termo e sempre acertou.

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que o comportamento real é | Como foi pego |
|---|---|---|---|---|
| 1 | teste | Que a suíte de texturas 4bpp/8bpp da iter 0045 cobrisse o modo | Ela só afirma pixels com **`v=0`**, a única linha em que a fórmula errada coincide com a certa. Seis testes verdes sobre um endereço quebrado | Ao escrever o teste novo com `v=1`, ele falhou de imediato — e os antigos continuaram verdes, o que é a definição de teste que não mede |
| 2 | teste | Que variar `v` bastasse para fixar o endereço | A bateria matou 4 de 6: `m3` (4bpp lendo 2 texels por halfword) e `m4` (8bpp lendo 4) **sobreviveram**, porque todas as minhas asserções de linha usavam `u=0` — e com `u=0` qualquer divisor dá o mesmo halfword | Bateria 4/6. Acrescentei asserções variando `u` **dentro** do halfword em `v=1`, com um segundo halfword de conteúdo diferente ao lado para que a leitura errada seja observável |
| 3 | processo | Que reancorar o manifesto 0100 uma vez resolvesse | Ele ancora numa linha de item **aberto** do M2; toda vez que eu fecho um item do M2 e abro o seguinte, a âncora envelhece de novo. Aconteceu duas vezes seguidas (0105 e 0106) | `mutation_anchors.rs` reprovou nas duas. Continua sendo reancorar e rerodar a bateria (5/5 as duas vezes), mas o atrito é estrutural — é o item 10.44 visto por outro ângulo |

## Bateria de mutação

Placar da bateria: 6/6 mutantes mortos, 2/2 controles verdes, 0 equivalente — docs/mutantes/0106-quad-texturizado.mut

| Registro | Rótulo | Testes que pegaram, conforme o `.resultado` |
|---|---|---|
| m1 | 4bpp volta a somar a linha no X | `textura_4bpp_le_a_linha_certa_para_v_maior_que_zero` |
| m2 | 8bpp volta a somar a linha no X | `textura_8bpp_le_a_linha_certa_para_v_maior_que_zero` |
| m3 | 4bpp empacota 2 texels por halfword | `textura_4bpp_le_a_linha_certa_para_v_maior_que_zero` |
| m4 | 8bpp empacota 4 texels por halfword | `textura_8bpp_le_a_linha_certa_para_v_maior_que_zero` |
| m5 | base X da texpage ignorada | `textura_4bpp_respeita_a_base_horizontal_da_texpage_em_v_maior_que_zero` |
| m6 | linha da textura ignora `v` | os três de 4bpp/8bpp |
| c1 | `* 64` escrito como `<< 6` | sobreviveu |
| c2 | leitura das máscaras da janela reordenada | sobreviveu |

## Placar antes → depois

Workspace: **731** → **735** testes (+4 em `gpu_quad_texturizado`).

Efeito visível: as duas barras vermelhas do logo da BIOS viram a palavra **SONY** legível. Foi a
primeira vez no projeto que uma correção de hardware produziu texto correto na tela.

## Revisão cruzada (orquestrador)

Iteração inteira do orquestrador.

## Decisões e notas

1. **O teste de 15bpp é controle, não cobertura.** Ele já passava antes da correção. Está ali para
   provar que o defeito era do endereço de 4/8bpp e **não** da interpolação de `v` — sem ele, a
   correção poderia ter sido no lugar errado com o mesmo resultado visual.
2. **A suíte antiga não foi apagada nem "corrigida".** Os seis testes da 0045 continuam verdes;
   o que faltava era um caso que eles não faziam. Teste fraco se soma, não se substitui.
3. **O logo ainda não está certo:** falta a metade de baixo de cada triângulo do losango. Medido no
   mesmo despejo — triângulos como `(195,240),(320,115),(320,365)` só aparecem de y=115 a y=240.
   Virou o item 2.2d, com o padrão nomeado no handoff.
4. **O que este item NÃO cobre:** modulação (item 10.13) e textura de retângulos (10.11) seguem
   sem medida contra este caso. A janela de textura (`E2h`) entra no cálculo antes do endereço e
   não foi tocada.
