<!-- Custo, tokens e duração NÃO entram aqui: são medidos por scripts/oc-iter.ps1 em
     docs/metricas.csv, casados por head_antes/head_depois. -->

# 0186 — sio-ordem-dos-switches

> Metade de uma iteração só: a outra metade, o teto da lista encadeada do DMA2, está em
> [`0186-dma-lista-encadeada-longa.md`](0186-dma-lista-encadeada-longa.md), que também carrega
> o resultado geral e as notas de método.

- **Data:** 2026-08-03
- **Item do roadmap:** 4.4ae
- **Objetivo:** os dois bytes de switches do pad digital saem na ordem que a spec manda.

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| psx-spx | § Controller Transfer, bytes swlo/swhi (L546-549) | docs/reference/10-controllers-memcards.md |
| psx-spx | § Standard Controllers, Halfword 1 (L613-633) | docs/reference/10-controllers-memcards.md |

## O que estava quebrado

Com o desenho consertado (ver o doc irmão), o Crash chegou ao título completo e ficou preso no
`PRESS START`. A hipótese fácil era "o controle não chega ao jogo"; medi antes de acreditar
nela, e ela era falsa. Instrumentando `send_byte` apareceram **26 leituras com o botão
apertado** dentro das janelas de press. O botão chegava — na posição errada.

§ Controller Transfer manda `swlo` (Digital Switches **bit 0-7**) no primeiro byte de dados e
`swhi` (bit 8-15) no segundo. A implementação mandava o alto primeiro. Como § Standard
Controllers põe Start no **bit 3**, ele chegava ao jogo na posição do **R1** — e nenhum menu
respondia a Start em jogo nenhum.

O teste que deveria ter pego isso pinava a ordem invertida, com um comentário que não fecha:
`"buttons low: Cross(bit14) no low, Start(bit3)=0"` — bit 14 não está no byte baixo. É o sintoma
clássico de prova escrita a partir da implementação em vez da spec: passou verde por 94
iterações sem medir nada. O manifesto de mutação da 0092 tinha o mesmo vício, com um mutante
chamado "troca high/low dos botoes (shift errado)" cujo `@@PARA` era justamente o código certo.

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | endereçamento | Se o jogo não responde a Start, o botão não está chegando ao SIO | — | Instrumentei `send_byte`: 26 leituras com botão apertado durante as janelas de press |

## Bateria de mutação

Placar da bateria: 7/7 mutantes mortos, 2/2 controles verdes, 0 equivalente — docs/mutantes/0186-sio-ordem-dos-switches.mut

| Mutante | Pego por |
|---|---|
| m1 reinverte os dois bytes | `start_sozinho_sai_no_primeiro_byte_de_switches` |
| m2 swlo sempre "nenhum botão" | `botoes_pressionados_aparecem_na_resposta_42h` |
| m3 swhi sempre "nenhum botão" | `botoes_pressionados_aparecem_na_resposta_42h` |
| m4 swlo perde o bit 7 (Left) | `botoes_soltos_retornam_ff` |
| m5 ID low vira 0x53 (analog stick) | `pad_digital_responde_5a41_ffff_ao_comando_42h` |
| m6 ID high vira 0xA5 | `pad_digital_responde_5a41_ffff_ao_comando_42h` |
| m7 porta vazia responde 0x00 em vez de HiZ | `sem_pad_digital_rx_fifo_retorna_ff` |

## Efeito colateral

**0092-input-desktop** guardava a ordem invertida nas âncoras de três mutantes. Corrigidas e a
bateria reexecutada: 5/5, 2/2.

## Placar antes → depois

- Este manifesto acrescenta 1 teste (`start_sozinho_sai_no_primeiro_byte_de_switches`) e corrige
  um existente. Placar do workspace no doc irmão.
- Crash Bandicoot: `PRESS START` sem resposta → START entra no jogo, e Start dentro do nível
  pausa (`PAUSED / PUSH SELECT TO CONTINUE`), como no console.

## Revisão cruzada (orquestrador)

<!-- Preenchido pelo Claude na revisão do PR: achados no formato de docs/prompts/review.md, ou "sem achados". -->

## Decisões e notas

O defeito é de 2019 do ponto de vista do jogo — qualquer título que leia o pad pelo caminho
padrão estava recebendo os botões trocados. Vale reconferir o Rayman com entrada agora que a
ordem está certa: o que antes se atribuía a "o jogo não chega a ler o controle" pode ter sido,
em parte, isto.
