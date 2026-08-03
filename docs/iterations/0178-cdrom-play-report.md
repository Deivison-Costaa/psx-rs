<!-- Custo, tokens e duracao ficam em docs/metricas.csv, medidos pelo runner. -->

# 0178 — cdrom-play-report

- **Data:** 2026-08-03
- **Item do roadmap:** 10.94 (avançado, não fechado) e 10.110.
- **Objetivo:** achar por que o Rayman gira em `[0x801CEEBC]`, medindo em vez de inferir.
- **Fonte:** orquestrador.

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| psx-spx | § Play - Command 03h (,track) (L1201-1211) | docs/reference/06-cdrom.md |
| psx-spx | § Setmode bits used for Play command (L1238-1245) | docs/reference/06-cdrom.md |
| psx-spx | § Report --> INT1(stat,track,index,...) (L1246-1265) | docs/reference/06-cdrom.md |
| psx-spx | § AutoPause --> INT4(stat) (L1267-1287) | docs/reference/06-cdrom.md |
| psx-spx | § Error Codes (L1020-1022) | docs/reference/06-cdrom.md |

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a medição diz | Como foi pego |
|---|---|---|---|---|
| 1 | diagnóstico | Que o item 10.94 (o jogo espera `[0x801CEEBC] != 0`) fosse o problema a atacar. | Não é assunto de spec. | `0x801CEEBC` é o **último** elo. Quem o escreve é uma máquina de estados que nunca rodava, e ela depende de um contador que nunca andava. Perseguir o endereço que aparece no laço teria continuado a não dar em nada — foi preciso subir a cadeia inteira. |
| 2 | hardware | Que o Rayman falasse com o CD-ROM pela BIOS, como os itens 10.79-10.87 pressupunham. | Os ponteiros em `[0x801CF1C4..0x801CF1D0]` valem `0x1F801800`..`0x1F801803`. | O jogo tem **driver de CD próprio** e sonda o registrador de interrupção direto no hardware, com índice 1. Toda a investigação de handler de BIOS das rodadas anteriores olhava para o lado errado desta parte. |

## A cadeia, medida elo por elo

Cada passo abaixo foi medido com `--trace-pcs`, `--dump-mem` e desmontagem da RAM, não inferido:

1. `0x8019FA1C` chama `0x80131DB8` — que é só `lh $v0, 0xEEBC($v0)` — e repete enquanto der zero.
2. Quem escreve `0x801CEEBC` são cinco sítios em `0x80130308..0x80131A84`, todos dentro de um
   callback despachado por modo em `0x801300AC`. Rastreados: **zero execuções**.
3. O callback é chamado por `jalr` a partir de `0x801A91FC`, com o ponteiro em `[0x801F99F8]`.
   Ele é instalado no passo **264.091.023**.
4. Quem dirige isso é o driver de CD do jogo: escreve `1` em `0x1F801800` (índice 1) e lê o
   registrador de interrupção em `0x1F801803`. Histograma do que a nossa porta devolveu:
   `0xE0` 534 vezes, `0xE1` 164, `0xE3` 10, `0xE2` 3.
5. Logo depois de instalar o callback o jogo emite `Setmode(0Eh)`, `Setloc(02h)` e **`Play(03h)`**
   no passo 264.144.524. A **última** interrupção de CD que ele vê é no passo 264.171.241 — 27 mil
   passos depois do Play. Os 356 milhões de passos seguintes não têm nenhuma.
6. `Play` **não estava implementado**: caía no braço `_` de `send_command`, que devolve
   `stat_byte()` e INT3 e não faz mais nada. Sem playback não há relatório periódico.
7. Sem relatório, o contador `[0x801F7CA8]` — que a rotina em `0x80130150` monta decodificando
   **BCD** de minuto/segundo/quadro, isto é, o MSF do relatório — fica em zero. Os dois `slt`
   que abrem a máquina de estados comparam limiares contra ele e nunca passam.

E a spec fecha o argumento sozinha, em § AutoPause (L1286) de docs/reference/06-cdrom.md:
**"AutoPause is used by Rayman and Tactics Ogre."**

## A mudança

`Play` (03h) implementado conforme a spec: INT5(stat,80h) sem disco (§ Error Codes (L1020) põe
`02h..09h` na faixa), INT3(stat) com disco e `stat.bit7` de Play ligado, e — só quando o bit 2 do
Setmode está ligado — INT1 periódico de oito bytes
`stat,track,index,mm/amm,ss+80h/ass,sect/asect,peaklo,peakhi`.

O relatório sai a cada dez quadros, alternando tempo absoluto (asect 00h/20h/40h/60h) e tempo
dentro da trilha (10h/30h/50h/70h, marcado pelo bit 7 do segundo). A trilha e o seu início vêm
do TOC quando há layout de disco; sem layout vale a trilha 1 começando em 00:02:00, que é
convenção MSF/LBA e não palpite. `peaklo`/`peakhi` saem zerados: a spec diz que o pico é
zerado a cada leitura e que 9 de cada 10 quadros se perdem, então não há valor a inventar.

## Bateria de mutação

Placar da bateria: **7/7 mutantes mortos, 2/2 controles verdes, 0 equivalente.**

## Placar antes → depois

Workspace: **958 → 964** testes.

O número que interessa é o contador que o jogo lê, em 700 M passos:

| | antes | depois |
|---|---|---|
| `[0x801F7CA8]` (posição, BCD decodificado) | `0x00000000` | **`0x00586F0A`** |
| `0x801300AC` (callback de streaming) | 0 execuções | **38.011 execuções** |
| `[0x801CEEBC]` (o que o laço espera) | 0 | 0 |

O contador que estava parado desde o começo do projeto anda. A máquina de estados acorda e
escreve. O jogo **ainda não sai do laço**: ela agora para um passo adiante, num teste de
`[0x801F51D8]`, que é o bit 7 de um byte da tabela do jogo em `0x801C4478`.

## Revisão cruzada (orquestrador)

Rodada de autoria do orquestrador; autorrevisão registrada como limite. O que sustenta o
resultado não é opinião: sete mutantes mortos, e um contador que era zero em todas as medições
anteriores do projeto passou a avançar.

## Decisões e notas

**O próximo elo é o INT4 de AutoPause, e não é palpite.** Os modos 4 e 5 do despacho em
`0x801300AC` são exatamente os dois que escrevem `1` em `0x801CEEBC` (sítios `0x80130314` e
`0x801302FC`), e a tabela de saltos do driver em `0x8012C738` liga INT4 a `0x801A956C` e INT5 a
`0x801A95AC` — os dois únicos sítios que gravam o modo.
§ AutoPause (L1267-1275) de docs/reference/06-cdrom.md diz que com `Setmode.bit1` o INT4 sai no
fim da **trilha**. Registrado como **10.110**.

Não implementei o AutoPause nesta rodada por um motivo concreto: o `.cue` em uso
(`Rayman (USA) DADOS.cue`) só tem trilha de dados, então não existe fim-de-trilha de áudio para
detectar. O `.cue` multi-trilha existe mas trava antes, em `boot file : cdrom:PSX.EXE;1` — item
separado. Implementar "fim de trilha" sem trilha seria inventar o comportamento que o teste
seguinte deveria medir.
