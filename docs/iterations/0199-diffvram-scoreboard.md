# 0199 — diffvram-scoreboard

- **Data:** 2026-08-04
- **Item:** Achado 10.23 (45/51 suítes do scoreboard sem veredito de VRAM)
- **Objetivo:** dar veredito gráfico ao scoreboard comparando o retrato de VRAM do
  emulador com os `vram.png` capturados em hardware real pelo ps1-tests.

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| psx-spx | § GPU / VRAM 15bpp (BGR555, 1024x512) | docs/reference/03-gpu.md |

O gabarito é externo à spec: `tests/exes/ps1-tests/*/vram.png` (hardware real, build-158)
comparado pela tool `diffvram` do próprio ps1-tests (exit 0 = idênticas; exit 2 +
`Images are different (N pixels)`).

## Como funciona

1. `psx-cli --vram-to-png <entrada.vram> <saida.png>`: raw de 1.048.576 bytes
   (1024×512 halfwords LE) → PNG RGB8 1024×512, expansão de canal `(c & 0x1F) << 3`
   (mesma convenção do capturador do ps1-tests — validada por `clipping` dar `vram-ok, 0px`: pixel-perfeito contra o hardware), bit de máscara
   ignorado no display.
2. `scoreboard.ps1`: quando existe `vram.png` ao lado do EXE, roda com `--dump-vram`,
   converte e compara. Status novos: `vram-ok` (0 px), `vram-diff` (K px, coluna detalhe),
   `vram-erro`. Veredito textual `pass -`/`fail -` continua com prioridade. Os PNGs de
   diferença ficam em `logs/diffvram/` para inspeção.

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | API-Rust | `png::Decoder::new(File)` compila | png 0.18 exige `BufRead` | erro de compilação no teste |

## Bateria de mutação

Alvo `crates/psx-cli` está fora do `mutantes.ps1` (invariante 29); bateria executada
A MÃO com o manifesto `0199-diffvram-scoreboard.mut`, resultado colado no
`.resultado` correspondente.

Placar da bateria: **5/5 mutantes mortos, 2/2 controles verdes, 0 equivalente.**

## Placar antes → depois

- Workspace: 1228 → 1235 (+3 vram_png, +4 ci_diffvram).
- Scoreboard: de 2 para **20 suítes com veredito** (6p/1f textual + 1 vram-ok/12 vram-diff).
  Textual novo medido nesta rodada: `code-in-io` 7p, `cop` 17p, `dpcr` 2p, `otc-test` 15p
  passam; `spu/memory-transfer` 7p/4f. Gráfico: `clipping` **0px** (pixel-perfeito);
  `lines` 518px; `rectangles` 11.560px (o retângulo texturizado do 10.11); `quad` 76.800px;
  `uv-interpolation` 35.272px; `texture-overflow` 32.768px; `vram-to-vram-overlap`
  14.474px; `triangle` 118.335px; `transparency` 454.912px; `clut-cache`, `texture-flip`,
  `mdec/4bit` e `mdec/8bit` 524.288px (a VRAM inteira — nada desenhado como o hardware).

## Revisão cruzada (orquestrador)

n/a — o orquestrador é o autor (exceção registrada em `docs/orquestracao.md`, 2026-08-03).

## Decisões e notas

- A convenção de expansão `<<3` (sem replicação dos bits baixos) foi confirmada
  empiricamente pelo `vram-ok, 0px` do `clipping`: qualquer canal na escala errada
  divergiria em todos os pixels não-pretos.
- `vram-erro` separa falha de ferramenta de divergência de emulação: `conversao` (o raw
  não converteu) vs `diffvram` (a tool não deu veredito parseável).
- O `--dump-vram` entra na MESMA execução da suíte (nada de rodar o EXE duas vezes).
