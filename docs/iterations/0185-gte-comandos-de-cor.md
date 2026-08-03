<!-- Custo, tokens e duração NÃO entram aqui: são medidos por scripts/oc-iter.ps1 em
     docs/metricas.csv, casados por head_antes/head_depois. -->

# 0185 — gte-comandos-de-cor

- **Data:** 2026-08-03
- **Item do roadmap:** 5.4b, 5.4c e 5.4d (os três fecham juntos: são um motor só)
- **Objetivo:** o Crash Bandicoot desenhar com cor por vértice em vez de silhueta branca.
- **Fonte:** orquestrador.

O Crash já chegava ao menu principal (placa de madeira texturizada, START / LOAD GAME /
PASSWORD / OPTIONS), mas o modelo do personagem saía como **silhueta branca chapada**. A causa
não foi inferida: o `execute_command` do GTE tinha `_ => {}` para todo opcode fora dos oito
implementados, então **instrumentei o dispatch para registrar cada opcode distinto** e rodei o
jogo. O Crash emite três que caíam no vazio: `0x13` NCDS, `0x3F` NCCT e `0x10` DPCS. São
exatamente as instruções que calculam a cor por vértice — sem elas o registrador de cor nunca é
escrito e o polígono sai com o que estivesse lá.

Os doze comandos da família (`NCS/NCT/NCCS/NCCT/NCDS/NCDT`, `CC/CDP`, `DCPL/DPCS/DPCT/INTPL`)
compartilham o mesmo motor: matriz de luz → matriz de cor → modulação pelo RGBC → interpolação
com o far color → FIFO de cor. Implementar só os três do Crash e deixar os outros nove no
`_ => {}` custaria a mesma bateria e o mesmo teste, então os doze entraram juntos.

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| psx-spx | § GTE Color Calculation Commands (L580-596) | docs/reference/07-gte.md |
| psx-spx | § Details on "MAC+(FC-MAC)\*IR0" (L596-600) | docs/reference/07-gte.md |
| psx-spx | § Notes (L607-609) | docs/reference/07-gte.md |
| psx-spx | § cop2r63 (cnt31) - FLAG (L336-366) | docs/reference/07-gte.md |
| psx-spx | § GTE Saturation (L329-335) | docs/reference/07-gte.md |

## Gabarito de hardware

`tests/exes/ps1-tests/gte-fuzz/gte_valid_0xc0ffee_50.log` (JaCzekanski/ps1-tests) despeja os 64
registradores antes e depois de **50 execuções por comando em console real**. São 600 casos para
esta família. A lição da 0184 — constante derivada da spec errou 16 de 64 bytes do MDEC — foi
aplicada aqui desde o começo: **nenhum valor esperado veio da nossa implementação nem de conta
feita à mão a partir da spec.** O teste é transcrição automática do log.

Antes de escrever uma linha de Rust, modelei a spec em Python e rodei contra os 600 casos. A
primeira versão acertou **0 de 50** no NCS, o comando mais simples da família. As quatro
correções abaixo levaram o modelo a 600/600, e só então o Rust foi escrito.

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec/hardware diz | Como foi pego |
|---|---|---|---|---|
| 1 | endereçamento | Que a translação da segunda multiplicação era o far color, porque no MVMVA a matriz de cor (`mx=2`) anda junto com `cv=2` = FC | A spec escreve `BK*1000h + LCM*IR`: a translação é a **cor de fundo** (cnt13-15), e o FC só aparece na interpolação | NCS 0/50. O MAC1 do gabarito era exatamente `RBK*1000h`, o que denunciou o registrador trocado |
| 2 | saturação-gte | Que o IR saturava direto do acumulador aritmético | `MAC1..3` são registradores de **32 bits**: o valor que chega na saturação já veio truncado, e o sinal muda | NCCS caso 2: MAC1 truncado é positivo (`0x16260000`), o inteiro exato de 44 bits é negativo — IR saía `-8000h` em vez de `+7FFFh` |
| 3 | saturação-gte | Que `(FC<<12) - MAC` era aritmética exata de 64 bits | O acumulador tem **44 bits com sinal e dá a volta**; o intermediário é estendido a partir do bit 43, deslocado, e só então truncado em 32 bits | DPCS 32/50. Nos 18 casos errados o gabarito exigia `ir=-8000h` onde a conta exata dava `+7FFFh` |
| 4 | flags | Que o overflow do MAC era decidido no resultado final da soma | O hardware checa **a cada parcela acumulada**, e a parcela intermediária dá a volta antes da seguinte | NCS caso 40: o gabarito liga o bit 29 (MAC2 positivo) **e** o 26 (MAC2 negativo) no mesmo comando. Somando tudo de uma vez nenhum dos dois liga |
| 5 | nenhum (formato) | Que o bloco `>` do log era o estado dos registradores antes do comando | É a **sequência de valores escritos** em r0..r63: escrever r15 (SXYP) empurra a FIFO de SXY e escrever r28 (IRGB) reescreve IR1..IR3 | O RTPS do gabarito devolvia em SXY0 o valor de r14, não o de r13 — só fecha se a FIFO já tiver sido empurrada uma vez pela própria escrita |

O erro #4 vale para **todo** o GTE, não só para esta família: `rtps`, `mvmva`, `op` e `sqr`
continuam checando overflow no total. Não foram tocados aqui (é o item 5.5, ainda aberto) — o
achado ficou registrado em `docs/achados.md`.

## Bateria de mutação

Placar da bateria: 14/14 mutantes mortos, 2/2 controles verdes, 0 equivalente —
`docs/mutantes/0185-gte-comandos-de-cor.mut`. A bateria da 0088 foi reexecutada com a âncora
renovada (o mutante m4 do `sf` passou a casar 6 vezes em vez de 4, porque os comandos de cor
também usam `let shift = sf * 12;`): 7/7 e 2/2.

**A primeira rodada foi 12/14, e os dois sobreviventes eram testes meus verdes sem medir nada.**

| Mutante | Por que sobreviveu | Caso de hardware que passou a matá-lo |
|---|---|---|
| m1 — acumulador não dá a volta | Nenhum dos 12 casos escolhidos tinha parcela passando de 2^43 | NCDS #40 e NCCT #34 (divergem em `FLAG`) |
| m7 — FIFO guarda `MACn` em vez de `MACn/16` | Nos 12 casos o canal saturava em `00h`/`FFh` de qualquer jeito, com ou sem o `/16` | DPCS #9 e NCCT #12 (divergem em `RGB2` e `FLAG`) |

Os quatro casos não foram escolhidos a gosto: rodei o modelo Python **com a mutação aplicada**
contra os 600 casos e peguei os que separam o correto do mutado. Isso é o inverso do fluxo
normal — a bateria disse qual buraco existia e o gabarito disse qual caso o tapa.

## Placar antes → depois

Workspace: **1039** → **1055** testes (16 novos em `gte_comandos_cor`).

**O Crash desenha em cor.** O menu principal saía com a placa de madeira texturizada correta e o
modelo do personagem como silhueta branca chapada; agora sai com o logo `CRASH` em amarelo e o
Crash com pelo laranja, olhos e focinho — o mesmo quadro, a mesma execução, só com os três
opcodes que faltavam.

**Ele ainda não entra no primeiro nível**, então o 4.4ae continua aberto. Com `--press start` o
cursor não sai do lugar, e o TTY do jogo mostra por quê: o runtime da Naughty Dog imprime
`GPU timeout:que=0,stat=5604267e,chcr=01000401,madr=00058498` em laço. O `chcr` é DMA2 em lista
encadeada (SyncMode=2) que fica com o bit de ocupado ligado e não completa. **Isso já aparecia
antes desta iteração** (uma vez em 400 M passos, nove em 900 M), então não é regressão da cor —
é o próximo obstáculo, registrado como 0185.2.

## Efeito colateral: cinco provas do Rayman andaram +15.801 passos

O Rayman é 2D, mas **também emite comandos de cor do GTE** — que até agora eram no-op. Assim que
passaram a fazer trabalho, o jogo gastou ciclos que antes não gastava e todo o boot deslizou
**+15.801 passos, uniformemente**. Reprovaram quatro arquivos (`rayman_autoack`,
`rayman_evcb_descritores`, `rayman_exception_chain`, `rayman_hook_activation`), nove constantes
no total.

Antes de repinar, confirmei que era isso e não regressão: restaurei o `gte.rs` da revisão
anterior, rodei `rayman_autoack` e ele passou; com o novo, falha. Em todas as nove, `arg`, `ra` e
os endereços ficaram **idênticos** — só o instante mudou, e o deslocamento é o mesmo número nas
nove. As constantes foram medidas uma a uma, não somadas de cabeça.

É o achado **10.115** cobrando o preço que ele mesmo previu: prova que fixa passo absoluto
reprova por melhoria legítima. O comentário no ponto de cada repinagem registra a causa.

## Revisão cruzada (orquestrador)

<!-- Preenchido na revisão do PR. -->

## Decisões e notas

- **Por que os doze e não só os três do Crash.** Os doze são o mesmo motor com três chaves
  (`Modulacao`, `FarColor`, `OrigemRgb`); o dispatch é uma linha por opcode. Cortar em três
  deixaria nove `_ => {}` que voltariam a ser diagnosticados do zero no próximo jogo, com o
  mesmo custo de bateria e de gabarito.
- **O instrumento antes da hipótese.** A silhueta branca tinha explicação plausível pronta
  ("textura faltando"), e ela estava errada — a placa de madeira ao lado do modelo é texturizada
  e desenha certo. Medir quais opcodes o jogo emite custou um `eprintln` e uma rodada, e trocou
  a hipótese por um fato antes de qualquer implementação.
- **O `_ => {}` é o defeito estrutural.** Opcode não implementado virava no-op silencioso: o
  jogo roda, desenha, e só a cor sai errada. Se o dispatch tivesse registrado o opcode
  desconhecido desde o início, a lacuna teria aparecido na primeira execução do primeiro jogo 3D.
