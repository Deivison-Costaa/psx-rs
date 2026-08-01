<!-- Custo, tokens e duração NÃO entram aqui: são medidos por scripts/oc-iter.ps1 em
     docs/metricas.csv, casados por head_antes/head_depois. -->

# 0130 — shell-vram-tela

- **Data:** 2026-08-01
- **Item do roadmap:** 4.4y
- **Objetivo:** Capturar a VRAM durante o boot com disco e responder: o que o shell desenhou
  e a tela evolui? Comparar com a referência do DuckStation.

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| psx-spx | § Quick Rectangle Fill (L217) | docs/reference/03-gpu.md |
| psx-spx | § VRAM Overview / VRAM Addressing (L234) | docs/reference/03-gpu.md |

Golden values do teste: GP0(02h) converte a cor 24 bits para 15 bits descartando os 3 LSB
de cada canal e zera o bit15 (docs/reference/03-gpu.md L225-L228); VRAM de 1 MB = 512 linhas
de 2048 bytes (docs/reference/03-gpu.md L242).

## Implementação

Flag `--dump-vram <arquivo>` no `psx-cli`: ao fim do run grava a VRAM inteira
(1024×512 halfwords little-endian, 1 048 576 bytes), linha a linha via `Gpu::vram_pixel`.
Nada muda no psx-core.

## Medição

```
.\target\release\psx-cli.exe --bios bios/SCPH1001.BIN --disc "..\roms\extraido\Crash Bandicoot (USA).cue" --max-steps 120000000 --dump-vram 0130-vram-120M.bin
```
(idem com 200 M → `0130-vram-200M.bin`; conversão para PNG por script descartável no
scratchpad, RGB555→RGB888)

- **A tela SCE está desenhada e correta**: losango dourado/laranja, "SONY",
  "COMPUTER ENTERTAINMENT", "TM", fundo cinza — visualmente igual à captura canônica
  `psx-estado/referencias/tela-de-boot-duckstation.png` (mesmo enquadramento, mesmas cores,
  sem "®", como a referência). Framebuffer em (0,0) ~640×480; texturas do logo estacionadas
  em x≥640.
- **A tela NÃO evolui: 0 pixels diferentes entre os dumps de 120 M e 200 M steps** (comparação
  halfword a halfword do 1 MiB). O shell desenhou a tela SCE uma vez e congelou nela — em
  hardware real ela dura ~3 s e 200 M steps ≈ 14 s emulados.
- Combinado com a 0129 (silêncio total do CD-ROM e só VBlank/IRQ0 dos ~92 M aos 200 M): o
  shell espera algo que nunca chega para sair da tela SCE. Candidatos para o próximo item:
  SPU (jingle de boot que nunca "termina"), contador de frames/timer, ou handshake de
  controle no loop de VBlank.

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | nenhum | — | — | teste e implementação passaram de primeira; a rodada não teve erro de emulação (o item é instrumentação de leitura) |

Nota de processo: a âncora do m4 da bateria 0127 (`i += 2;`, ocorrencias: 4) envelheceu com o
parse da flag nova (5ª ocorrência) e o portão `mutation_anchors` reprovou o workspace inteiro —
resolvido com `arquivada:` no manifesto 0127, placar histórico preservado no `.resultado`.

## Bateria de mutação

Placar da bateria: 5/5 mutantes mortos, 2/2 controles verdes, 0 equivalente — docs/mutantes/0130-shell-vram-tela.mut

Bateria MANUAL (invariante 29) — alvo `crates/psx-cli/src/main.rs`, assassino
`cargo test -p psx-cli --test shell_vram_tela` (aplicado → rodado → revertido, um a um):

| id | mutação | resultado |
|---|---|---|
| m1 | `to_le_bytes` → `to_be_bytes` | MORREU (0.07s) |
| m2 | dump de 256 linhas em vez de 512 | MORREU (0.07s) |
| m3 | dump de 512 colunas em vez de 1024 | MORREU (0.07s) |
| m4 | `vram_pixel(y, x)` (transposição) | MORREU (0.07s) |
| m5 | bit15 forçado (`| 0x8000`) | MORREU (0.09s) |
| c1 | comentário antes de `write_vram_dump` | SOBREVIVEU (esperado) |
| c2 | comentário antes do `fs::write` | SOBREVIVEU (esperado) |

O stub `dump_vram_grava_fill_convertido_para_15bpp` em `crates/psx-core/tests/shell_vram_tela.rs`
existe só para o portão `bateria_nomes_de_teste_existem` (mesmo padrão da 0128/0129).

## Placar antes → depois

840 → 842 testes no workspace (o teste da iteração no psx-cli + o stub do portão no psx-core).

## Revisão cruzada (orquestrador)

Sem achados. Verificações feitas: (a) o teste não depende de a BIOS ficar quieta — com
`--exe` o load salta direto para o código sintético e nos 64 steps só ele roda; (b) o
caminho `--disc` (sem `--exe`) foi exercitado pela própria medição (dois dumps de 1 048 576
bytes); (c) o dump de 200 M foi conferido pela igualdade byte a byte com o de 120 M, cujo
PNG foi inspecionado visualmente contra a referência.

## Decisões e notas

- A comparação com a referência é visual (mesma tela, mesmas cores) — a comparação
  pixel-perfeita contra o DuckStation não é possível porque a captura é um screenshot
  escalado da área de display, não um dump de VRAM.
- As costuras de gouraud no losango (candidato 10.14) não são distinguíveis nesta resolução —
  seguem em aberto.
- A dupla 120 M/200 M cumpre a invariante 30: a medida "a tela não muda" tem janela além do
  horizonte (80 M steps de margem depois do último evento conhecido).
