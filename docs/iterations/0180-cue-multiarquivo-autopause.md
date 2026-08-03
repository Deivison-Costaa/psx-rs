<!-- Custo, tokens e duracao ficam em docs/metricas.csv, medidos pelo runner. -->

# 0180 — cue-multiarquivo-autopause

- **Data:** 2026-08-03
- **Item do roadmap:** 10.110 e 10.111 (fechados); 10.94 avançado.
- **Objetivo:** destravar o laço em que o Rayman está preso desde a iteração 0167.
- **Fonte:** orquestrador.

**O laço quebrou.** `[0x801CEEBC]` passou de `0` para `1` e o jogo saiu de `0x8019FA1C`. Ele
para agora noutro lugar, mais adiante — ver "Placar".

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| psx-spx | § AutoPause --> INT4(stat) (L1267-1286) | docs/reference/06-cdrom.md |
| psx-spx | § Setmode - Command 0Eh,mode (L685-700) | docs/reference/06-cdrom.md |
| psx-spx | § Report --> INT1(stat,track,index,...) (L1246-1265) | docs/reference/06-cdrom.md |

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a medição diz | Como foi pego |
|---|---|---|---|---|
| 1 | hardware | Que concatenar os `.bin` das trilhas bastasse: `read_sector_from_disc` já indexa por LBA absoluto, então a imagem certa resolveria. | O `INDEX 01` de um `.cue` rasgado por trilha é **relativo ao próprio arquivo**. Toda trilha de áudio declara `00:02:00`. | Concatenei, o boot passou a funcionar (597 → 4311 bytes de TTY) e o Rayman **não andou um passo**. Consertei os dados e deixei o TOC errado: `trilha_em` achava que a trilha 2 começava em `00:02:00`, a posição `09:19:60` já estava depois dela, e a fronteira nunca era cruzada. |
| 2 | teste | Que o RED do `.cue` multi-arquivo valesse. | Não é assunto de spec. | Os testes de `cdrom_cue_multiarquivo` falharam por **erro de compilação** (campo `file` inexistente), não por asserção — o mesmo furo de R5 registrado na 0169. Só os três de autopause deram RED legítimo. O commit `test(...)` desta rodada, sozinho, não compila. |
| 3 | método | Que valesse implementar o AutoPause direto, já que a spec nomeia o Rayman. | O jogo manda `Setmode = 07h` — bits 0, 1 e 2. **Medido**, instrumentando o comando. | Instrumentei antes de implementar. Se o bit 1 estivesse desligado, o AutoPause seria trabalho correto e inútil, e eu não teria como saber pela spec. |

## As três mudanças

**10.111 — `.cue` com um arquivo por trilha.** `parse_cue` guardava um `bin_path` só, sobrescrito
por cada `FILE`: sobrava o **último**. No Rayman a trilha de dados passava a ser lida do arquivo
da trilha 6, e o boot morria em `boot file : cdrom:PSX.EXE;1`. Agora cada `TrackInfo` lembra o seu
arquivo, e o carregador concatena na ordem das trilhas — o que reconstrói exatamente a imagem
indexável por LBA que `read_sector_from_disc` já esperava.

**TOC absoluto.** `atribui_lbas_absolutos` soma o tamanho dos arquivos anteriores ao `INDEX 01`
de cada trilha. Fica em `psx-core` como função pura, recebendo os tamanhos de quem faz I/O (R3).
Num `.cue` de arquivo único o acumulado é zero e nada muda.

**10.110 — AutoPause.** § AutoPause (L1270) de docs/reference/06-cdrom.md:
*"Setmode.bit1=1: AutoPause=On --> Issue INT4(stat) and PAUSE at end of TRACK"*. Ao cruzar a
fronteira da trilha em que o Play começou, sai INT4 com `stat.bit7` desligado e o playback para.

## Bateria de mutação

Placar da bateria: **6/6 mutantes mortos, 2/2 controles verdes, 0 equivalente.**

A âncora `m6` do manifesto **0064** envelheceu (extraí `index01_em_quadros` de `bin_offset`).
Reparada e a bateria daquela iteração reexecutada: 7/7 mortos, 2/2 verdes, mesmo placar do doc
original.

## Placar antes → depois

Workspace: **1010 → 1019** testes.

| | antes | depois |
|---|---|---|
| `[0x801CEEBC]` — o que o laço espera desde a 0167 | `0` | **`1`** |
| PC dominante em 650-700 M | `0x8019FA1C` | **`0x80132BF0`** |
| TTY com o `.cue` multi-trilha | 597 bytes | **5304 bytes** |

O jogo agora carrega o executável de verdade e imprime `EXEC:PC0(801abce0) T_ADDR(80125000)`,
`Execute !` e `PS-X Control PAD Driver Ver 3.0`.

**Ele ainda não desenha nada.** A parada nova é `0x80132BF0`: `while (*(u8*)$s0 == 0) {}`, logo
depois que `0x80133F40` retorna. E os 296 `VSync: timeout` continuam — o item 10.90 não foi
tocado e provavelmente é o próximo obstáculo real.

## Revisão cruzada (orquestrador)

Rodada de autoria do orquestrador; autorrevisão registrada como limite. O que sustenta o
resultado é externo ao meu julgamento: um endereço que valia zero em toda medição do projeto
desde a 0167 passou a valer 1, e o histograma de PC mudou de laço.

## Decisões e notas

**O AutoPause só funciona com report ligado.** O avanço da posição é acionado pelo ack do
relatório INT1; sem o bit 2 do Setmode não há relógio de áudio e a fronteira nunca é avaliada.
Para o Rayman isso basta (ele manda `07h`), mas é limitação real — um jogo com autopause e sem
report não seria pausado. Um relógio de áudio livre no `scheduler` resolveria, e é maior que
esta rodada.

**Não implementei áudio.** Nada é decodificado nem reproduzido das trilhas CD-DA: o que existe é
posição e fronteira de trilha. O jogo destrava porque é disso que a máquina de estados dele
depende, não porque esteja ouvindo alguma coisa.
