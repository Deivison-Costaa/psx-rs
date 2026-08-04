# 0192 — primeiro-frame-medido

- **Data:** 2026-08-03
- **Item do roadmap:** 4.5
- **Objetivo:** fechar o 4.5 pela unica coisa que o fecha — medicao. O item nasceu na 0137
  com um observavel preciso: "o Crash carrega 954 KB de WAD e nao desenha nenhum frame".

## Spec consultada

Nenhuma: esta iteracao nao implementa hardware, ela mede o observavel do item.

## O que o item pedia

A 0137 nomeou o 4.5 a partir de duas hipoteses de mecanismo — "rollback do init do LIBSN"
e "poll orfao do TMR2 com orcamento de 800h giros". As iteracoes 0141 a 0147 mediram as
duas e refutaram as duas:

- 0141: o handler do jogo **sobrevive** as duas chamadas de remocao de cadeia. Nao ha
  rollback; quem destroi a cadeia e outra coisa.
- 0147: o slot `$v1+0x18` **nunca muda** entre os dois boots — a premissa das 0142 e 0144
  esta refutada.

O mecanismo verdadeiro so apareceu depois, em outro lugar: o teto de 4096 nos da
`execute_linked_list` (0186) cortava a cadeia de DMA do Crash, que e maior, e o canal
ficava ocupado para sempre. Foi isso, e nao a cadeia de excecao do kernel, que impedia o
desenho.

## Medição

Comando (1,2 G passos, ~1 min de parede):

```
psx-cli --bios bios/SCPH1001.BIN --disc "Crash Bandicoot (USA).cue" \
        --max-steps 1200000000 --pad --press start@330000000 --press cross@700000000 \
        --memcard crash.mcd --dump-audio crash.pcm --dump-vram-every 150000000 crash
```

Linha do tempo de VRAM (1024x512x16bpp cru, pixels nao-zero e quantos mudaram desde o
dump anterior):

| Dump | passo | pixels nao-zero | mudaram |
|---|---|---|---|
| 1 | 150 M | 38.395 | — |
| 2 | 300 M | 287.955 | 298.862 |
| 3 | 450 M | 424.832 | 399.934 |
| 4 | 600 M | 424.819 | 19.220 |
| 5 | 750 M | 427.931 | 370.467 |
| 6 | 900 M | 426.666 | 217.995 |
| 7 | 1050 M | 426.736 | 8.222 |
| 8 | 1200 M | 426.631 | 6.354 |

**Nenhum intervalo tem zero pixels mudados.** O observavel do 4.5 — "nao desenha nenhum
frame" — nao existe mais.

Audio na mesma corrida: 3.033.611 quadros estereo (68,8 s a 44,1 kHz), 5.711.383 de
6.067.222 amostras diferentes de zero (94,1%), pico 30.513 de 32.767. O Crash **soa**.

Contraprova no Rayman (mesma corrida, sem `--press`): oito dumps, todos os intervalos com
pixel mudando (2.889 no menor, 311.989 no maior) e 3.370.562 quadros de audio com
5.252.254 amostras nao-zero. Nenhum dos dois jogos regrediu com o SPU ligado.

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a medição mostrou | Como foi pego |
|---|---|---|---|---|
| 1 | premissa herdada | Que o 4.5 exigia implementar alguma coisa nova de CDROM ou de kernel | O item e um observavel, e o observavel ja tinha caido junto com a correcao da lista encadeada da 0186 — em outro subsistema, por outro caminho | Antes de abrir o codigo, rodei o comando do handoff com `--dump-vram-every`. Oito dumps, oito intervalos com pixel mudando |
| 2 | nenhum | Que a contagem de quadros de audio bateria com `passos / 768` | `--max-steps` conta INSTRUCOES, nao ciclos; o R3000A gasta ~2 ciclos por instrucao aqui, entao 1,2 G passos dao ~2,4 G ciclos e ~3,1 M quadros | 3.033.611 quadros medidos contra 1,56 M esperados — a razao de 1,94 e a media de ciclos por instrucao, nao um defeito do SPU |

## Bateria de mutação

Bateria de mutação: não se aplica — esta iteração não altera código de produção, só mede
o observável do item e acrescenta a flag `--dump-audio` ao runner (coberta pelos testes
de CLI existentes e pela bateria 0189, que mede o anel que ela drena).

## Placar antes → depois

Nenhum teste novo. O item 4.5 sai da escada para `docs/ROADMAP-fechado.md`.

## Revisão cruzada (orquestrador)

## Decisões e notas

- **O 4.5 fecha como "observavel extinto", nao como "hipotese confirmada".** As duas
  hipoteses que o nomearam (rollback do LIBSN, poll orfao do TMR2) foram refutadas por
  medicao nas 0141 e 0147 e continuam refutadas; o congelamento tinha outra causa. Fica
  registrado assim para nao induzir quem ler o historico.
- **`--dump-audio <arquivo.raw>`** grava PCM cru estereo de 16 bits a 44,1 kHz. Reproduzir
  com `ffplay -f s16le -ar 44100 -ch_layout stereo arquivo.raw`.
- O runner drena o anel do SPU a cada 4096 passos; sem isso o anel bate no teto de 8192
  quadros e o resto do audio se perde.
