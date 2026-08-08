# 0229 — media semi-transparente e o oraculo gpu/quad

- **Data:** 2026-08-07
- **Item do roadmap:** frente `gpu/texture-flip` e `gpu/quad` (oraculos de hardware)
- **Objetivo:** o modo 0 de semi-transparencia tem de ser `(B+F)/2`, e nao `B/2 + F/2`.

## Spec consultada

| Fonte | Secao | Arquivo local |
|---|---|---|
| psx-spx | Semi-transparency, as 4 formulas | `docs/reference/03-gpu.md` L1591 |
| psx-spx | GP0(E1h).5-6 = modo de semi-transparencia | `docs/reference/03-gpu.md` L1006 |

`L1591` diz `* 0.5 x B + 0.5 x F ;aka B/2+F/2`. A frase e ambigua sobre ONDE arredondar,
e nos tinhamos lido como "some as metades". O gabarito de hardware desempata.

## texture-flip: ja estava resolvido, nao refizemos

Remedido no HEAD antes de qualquer mudanca, como o handoff mandou. O numero do handoff
(304.909) foi medido antes dos PRs #243/#244.

| medicao | pixels divergentes |
|---|---|
| `9e2edce^1` (antes do #243) | 197.632 (37,70%) |
| `main` = 2b1090e (depois do #243/#244) | **0** |

O PR #243 (X/Y-flip do GP0(E1h).12/13) fechou o oraculo inteiro. Nada a fazer aqui.

**Armadilha de ferramenta encontrada no caminho:** o `diffvram` do ps1-tests reporta
`524288 pixels` (100%, ou seja "tudo diferente") sempre que o `vram.png` de gabarito e
PNG colortype 6 (RGBA) e o nosso e colortype 2 (RGB) — e o caso do `texture-flip` e do
`clut-cache`. Ele nao esta comparando cor: esta desistindo. Onde o gabarito e paletado
(colortype 3) ele bate exatamente com a nossa contagem. **Nenhum numero de `diffvram`
para esses dois oraculos jamais valeu.** As medicoes deste doc usam contagem propria de
pixel RGB, com controle positivo (gabarito do texture-flip contra o nosso dump do quad =
342.959 divergentes, como esperado).

## Como o defeito do quad foi achado

As duas imagens (gabarito e nossa) sao indistinguiveis a olho. O que separa e 1 LSB de
5 bits. Histograma dos pares (referencia, nosso) sobre a area de display:

```
23080  ref(120,120,248) our(120,120,240)
15904  ref(248,120,248) our(240,120,240)
 9592  ref(248,120,120) our(240,120,120)
 8744  ref(120,248,120) our(120,240,120)
  256  ref(120,184,248) our(120,176,240)   (x7 swatches)
```

Em 5 bits: `31 -> 30` e `23 -> 22`, enquanto `15` e `7` batem. Rastreando o GP0 do EXE
(trace temporario em `Gpu::write32`) o teste se revela: quatro `GP0(02h)` enchem a VRAM
de branco (`0xFFFFFF` -> 31,31,31), o `GP0(E1h)` com `0x020A` seleciona **modo 0**, e
tudo depois e `GP0(2Ah)` — quad monocromatico semi-transparente.

Quad de cor `0xFF` (F=31) sobre fundo branco (B=31):
- hardware: `(31+31)/2 = 31`
- nosso: `(31>>1)+(31>>1) = 15+15 = 30`

Os dois lados so divergem quando **B e F sao ambos impares** — por isso 15 e 7 batiam e
por isso o defeito sobreviveu aos testes existentes de `gpu_semi_transparencia.rs`, que
usam 16, 8 e 24 (todos pares).

Os swatches de cor `0xA0` (F=20) fixam o arredondamento: o gabarito le 25 para B=31
(`(31+20)/2 = 25,5`) e 17 para B=15 (`(15+20)/2 = 17,5`). **Trunca, nao arredonda.**

## Oraculo independente: DuckStation

`gpu_sw_rasterizer.inl:185` faz a media dos tres campos de 5 bits empacotados de uma vez:
soma as duas halfwords, subtrai o bit de carry de cada campo (mascara com bit 0, 5 e 10) e
desloca 1 para a direita. E `(B+F)>>1` por canal, com truncamento — a mesma conclusao que
o gabarito de hardware, chegada por outro caminho. (Descrito, nao copiado: a licenca dele
e CC BY-NC-ND.)

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | saturacao | `B/2+F/2` podia ser implementado somando as metades | `docs/reference/03-gpu.md` L1591 da a formula mas nao o ponto de arredondamento; o hardware soma e SO ENTAO divide | gabarito `gpu/quad/vram.png`, 59.352 px divergentes |
| 2 | processo | que a branch do worktree era o HEAD que eu tinha medido | — | a suite de oraculos "regrediu" 3 casos de uma vez; era `iter/fix-jogos-loop` (stale) e nao `main` |

O erro 2 vale registro: o worktree comecou em `main` (2b1090e) em HEAD destacado, e um
`git checkout iter/fix-jogos-loop` me jogou num commit **anterior** aos PRs #243/#244 sem
aviso. A medicao seguinte mostrou texture-flip voltando a 197.632 e triangle pulando de
12.775 para 118.335. **Foi a bateria completa de oraculos que pegou** — se eu tivesse
medido so o `quad`, teria concluido que o fix funcionou e commitado sobre a base errada.

## Bateria de mutacao

Placar da bateria: 5/5 mutantes mortos, 2/2 controles verdes, 0 equivalente — docs/mutantes/0229-media-semi-transparente.mut

O `m2` (arredondar para cima em vez de truncar) e o que justifica as asercoes A3 e A4:
sem elas ele sobreviveria, porque nenhum outro caso do teste tem soma impar.

## Placar antes -> depois

Todos os 11 oraculos de GPU com gabarito, mesma metrica, mesma base (`main` 2b1090e):

| oraculo | antes | depois |
|---|---|---|
| clipping | 0 | 0 |
| clut-cache | 921 | 921 |
| lines | 470 | **362** |
| **quad** | **59.352** | **240** |
| rectangles | 4.142 | **2.954** |
| texture-flip | 0 | 0 |
| texture-overflow | 0 | 0 |
| transparency | 447.488 | 447.488 |
| triangle | 12.775 | 12.775 |
| uv-interpolation | 3.393 | 3.393 |
| vram-to-vram-overlap | 7.557 | 7.557 |

Tres melhoram, oito ficam iguais, **nenhum piora**.

Workspace: 1430 -> 1434 testes.

## Regressao em jogo

Binarios de hash conferido (`6CCBF7D7...` base, `64D0D4BB...` com o fix), 900 M passos,
6 framebuffers por jogo:

| jogo | resultado |
|---|---|
| Crash Bandicoot | 6 quadros byte-identicos; ultimo le `CRASH BANDICOOT` com `START / LOAD GAME / PASSWORD / OPTIONS` |
| Tekken 3 | 6 quadros byte-identicos; FMV de abertura coerente |
| Resident Evil 2 | 6 quadros byte-identicos |
| Silent Hill | 6 quadros byte-identicos |

Zero diferenca. Nesses trechos os jogos nao caem no caso B e F ambos impares em modo 0.

## O que sobra no quad (240 px, nao consertado)

Os 240 pixels restantes sao de **costura**: ficam nas bordas entre quads vizinhos
(bbox 48,33-318,239) e tem a forma "faltou uma camada semi-transparente aqui" —
`ref(120,120,120)` contra `our(120,120,248)`, por exemplo. E a regra de preenchimento de
borda decidindo se o pixel da diagonal pertence a um triangulo, ao outro, ou aos dois.
O DuckStation carrega um bit de "top-left" por vertice para isso
(`gpu_sw_rasterizer.inl`, por volta das linhas 1441 a 1456). E outra micro-funcionalidade — R4, fica para a
proxima.

## Decisoes e notas

- A mudanca e de tres linhas em `Gpu::write_pixel`. O teste novo
  (`gpu_media_semi_transparente.rs`) cobre os dois casos que quebram (A1, A2) **e** os dois
  que fixam o truncamento (A3, A4) — sem A3/A4 um "fix" que arredondasse passaria igual.
- Modos 1, 2 e 3 nao foram tocados: nao ha medicao neste gabarito que os questione.
