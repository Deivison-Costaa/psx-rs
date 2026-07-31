<!-- Custo, tokens e duração NÃO entram aqui: são medidos por scripts/oc-iter.ps1 em
     docs/metricas.csv, casados por head_antes/head_depois. -->

# 0118 — timer-portas-16bits

- **Data:** 2026-07-30
- **Item do roadmap:** 4.4m
- **Objetivo:** descobrir por que o shell da BIOS não lê o disco, e destravar o que a medição
  apontasse.

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| psx-spx | § Timer 0..2 Current Counter Value | docs/reference/05-timers.md |
| psx-spx | § Timer 0..2 Counter Mode | docs/reference/05-timers.md |
| psx-spx | § Timer 0..2 Counter Target Value | docs/reference/05-timers.md |
| psx-spx | § Peripheral I/O Ports | docs/reference/14-io-map.md |

Do § Current Counter Value: *"0-15 Current Counter value (incrementing); 16-31 Garbage"* — são
registradores de 16 bits, e `lhu` é o acesso natural a eles. Do mesmo parágrafo veio o valor
esperado do teste do modo: o contador *"gets forcefully reset to 0000h on any write to the
Counter Mode register"*.

## Como o item foi encontrado (medição antes de código)

O critério de aceitação do 4.4m era duplo de propósito: mostrar o shell lendo o disco **ou provar
que ele nem tenta**. Provou-se o segundo, e a investigação seguiu de onde ela parou.

**Passo 1 — o shell não pede nada ao disco.** Harness `cdshell` (decodifica escrita de comando na
porta 1 com os parâmetros empilhados na porta 2), 400 M passos com o disco do Crash:

```
  passo 86989710  pc=0x800583B0     Test (0x19)  params=[20]
  passo 87464254  pc=0x80057554  GetStat (0x01)  params=[]
  ultimo acesso ao CD no passo 87464782 (de 400000000)
  passos com HINTSTS==INT1: 0
```

Dois comandos em toda a corrida, nenhum `GetID`, nenhum `Setloc`, nenhum `ReadN`, e 312 M passos
sem tocar no drive. Não é sistema de arquivos, não é leitura falhando: o shell **nunca pede**.

**Passo 2 — onde ele está.** Histograma de PC depois de 120 M passos: dois blocos com contagens
idênticas (16 470 588 cada), `0x80059ED8..0x80059F0C` e `0x8003D404..0x8003D414`. Despejando as
palavras e decodificando à mão:

```
  0x8003D404: jal 0x80059ED8       0x80059ED8: andi $a0,$a0,0xFFFF
  0x8003D408: move $a0,$s1         0x80059EDC: slti $at,$a0,3      ; indice < 3 ?
  0x8003D40C: slt  $at,$v0,$s0     0x80059EF0: lui  $t6,0x8008
  0x8003D410: bne  $at,$zero,-4    0x80059EF4: lw   $t6,0x80079CB4 ; ponteiro da tabela
                                   0x80059EF8: sll  $t7,$a0,4      ; indice * 16
                                   0x80059F00: lhu  $v0,0($t8)     ; <-- meia palavra
```

`do { v0 = tabela[indice].halfword0; } while (v0 < s0)`. Capturando os registradores no laço:
`$s1 & 0xFFFF = 2`, `$s0 = 5808`, **`$v0 = 0` sempre**, e o ponteiro em `0x80079CB4` = `1F801100h`.
A tabela é o bloco de registradores dos timers, o índice é o timer 2, e o `lhu` do contador
devolvia zero — enquanto o mesmo contador lido por `lw` marcava `0xBB70`.

**Passo 3 — a largura do acesso.** Contando acessos aos timers por opcode:

```
  LEITURA lhu 0x1F801120  x831700     <-- contador do timer 2
  LEITURA lw  0x1F801110  x518
  ESCRITA sh  0x1F801124  x7 ; sh 0x1F801128 x4 ; sh 0x1F801120 x2 ; ...
```

O kernel lê o contador quase um milhão de vezes por `lhu` e arma modo/alvo por `sh`. No `bus.rs`,
`region_read_byte` e `region_write_byte` tinham a faixa `0x1F801064..=0x1F801FFF` como
braço-sumidouro — e `0x1F801100..0x1F80112F` cai dentro dela. Leitura de 8/16 bits devolvia zero,
escrita de 8/16 bits era descartada; só `read32`/`write32` chegavam ao módulo. Os timers estavam
certos desde o M3 e eram **inalcançáveis pela largura de acesso que o kernel usa** — a invariante
25, agora pela segunda vez (a primeira foi o SIO0, na 0115).

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | endereçamento | Que o shell não lia o disco por algo do CD-ROM — o handoff mandava instrumentar `Setloc`/`ReadN` e conferir o `INT1`. | § Current Counter Value: os timers são registradores de 16 bits, e o `lhu` é o acesso natural. O problema estava a três subsistemas de distância do CD-ROM. | O primeiro despejo do harness: **dois** comandos em 400 M passos e `INT1` zero. Instrumentar o CD-ROM teria confirmado "não chega nada" sem dizer por quê; o histograma de PC é que apontou o laço. |
| 2 | API-Rust | Que bastava reusar `Timers::read32` no caminho de byte. | — | Ler o registrador de **modo** limpa os bits 11/12 como efeito colateral (`timers.rs:57-61`). Como `read16` chama `region_read_byte` duas vezes, a segunda chamada leria os flags já apagados — e eles moram no byte alto. Resolvido com `peek32`, não-destrutivo, que é o mesmo padrão que o braço da GPU já usava (`gpu.peek32`). |
| 3 | teste | Escrevi `assert_ne!(read16(MODE) & (1 << 10), 0)` num teste que só tinha escrito no registrador de **alvo**. | — | Vermelho depois do fix: 7 de 8 passaram e esse falhou. O bit 10 só é ligado por escrita no modo; a premissa do teste é que estava errada, não a implementação. Trocado por uma afirmação sobre os bits 0-9 escritos e relidos. |

## Bateria de mutação

Placar da bateria: 6/6 mutantes mortos, 2/2 controles verdes, 0 equivalente — docs/mutantes/0118-timer-portas-16bits.mut

| # | Mutação | Teste que pegou |
|---|---|---|
| m1 | leitura de byte devolve zero (o defeito original) | `lhu_do_contador_devolve_o_valor_e_nao_zero` |
| m2 | leitura de byte ignora o `offset` | `lbu_do_contador_devolve_o_byte_certo_nos_dois_lados` |
| m3 | escrita de byte não chega ao módulo | `sh_no_alvo_grava_as_duas_metades` |
| m4 | escrita de byte ignora o `offset` | `sh_no_contador_grava_as_duas_metades` |
| m5 | escrita de byte apaga os outros bytes em vez de mesclar | `sh_no_alvo_grava_as_duas_metades` |
| m6 | `peek32` do registrador de modo devolve zero | `lhu_do_alvo_e_do_modo_tambem_saem_do_registrador` |
| c1 | deslocamento com fatores trocados (cosmético) | verde |
| c2 | tipo do valor atual anotado (cosmético) | verde |

## Placar antes → depois

Workspace: 782 → **790** testes (8 novos em `timer_portas_16bits.rs`), 0 falhas.

Efeito medido no boot real (SCPH1001 + disco, 400 M passos), com o `cdshell`:

| | antes | depois |
|---|---|---|
| `$v0` lido no laço de `0x8003D40C` | `0` para sempre | `65, 92, 119, 146, 173, 200 …` até passar de 5808 |
| contador do timer 2 ao fim | `0xBB70` só por `lw` | `0xE613`, visível também por `lhu` |
| laço quente | `0x8003D404..14` (16,4 M voltas) | saiu; agora `0x8003D6FC..0x8003D710` |
| comandos ao CD-ROM | 2 | 2 |

**Critério de aceitação: cumprido na metade que a medição definiu.** O item pedia mostrar o shell
lendo o disco *ou* provar que ele não tenta — provou-se que não tenta, a causa imediata (timer
invisível) foi corrigida com teste e bateria, e o boot avançou para o próximo bloqueio. **O shell
continua sem pedir dados ao disco**; o que mudou de lugar está no handoff.

## Revisão cruzada (orquestrador)

Sem achados que barrem o merge.

- **Os dois braços novos vêm antes do sumidouro.** Braços de `match` são ordenados; `0x1F801100..=
  0x1F80112F` precisa vir antes de `0x1F801064..=0x1F801FFF`, senão nada muda. Conferido nos dois
  sentidos (leitura e escrita).
- **`byte_index` é mascarado com `& 3`.** `(phys & 3) + offset` pode dar 4 num acesso desalinhado,
  e `val >> 32` estoura em debug. O braço da GPU logo acima tem o mesmo cálculo **sem** a máscara —
  não mexi nele (R4), mas fica anotado como 10.51.
- **Escrita de byte é read-modify-write com `peek32`.** Usar `read32` aqui apagaria os bits 11/12
  do modo a cada `sb`. Duas escritas de byte viram duas chamadas a `write32`; para o registrador de
  modo isso significa dois resets do contador para zero, que é idempotente.
- **Buraco conhecido, deixado aberto (R4).** Um `lhu`/`lbu` no registrador de **modo** não limpa os
  bits 11/12, porque o caminho de byte usa `peek32`. O acesso de 32 bits continua limpando, como
  antes. Nenhum dos dois aparece no caminho medido do boot. Anotado como 10.52.
- **Gates do projeto:** `purity`, `file_size`, `comment_density`, `roadmap_size`, `status_size`,
  `spec_citations`, `mutation_manifest`, `mutation_anchors` e `mutation_battery` verdes.

## Decisões e notas

- **Terceira vez que a largura do acesso é o defeito.** 0115 (SIO0), 0118 (timers) e, por tabela,
  o braço da GPU que faz o mesmo cálculo. O padrão não é "o dispositivo está errado", é "o
  barramento não entrega naquele tamanho" — e o sintoma nunca aponta para o barramento.
- **O harness fez três perguntas, não uma.** Comando enviado ao CD (nenhum), onde o PC está (laço),
  o que o laço lê (meia palavra do timer). Cada resposta redefiniu a pergunta seguinte; se eu
  tivesse instrumentado só o CD-ROM, como o handoff pedia, teria confirmado o sintoma e parado.
- **Próximo degrau, já medido.** O laço agora é `0x8003D6FC`: `lw $t7,[0x80083C58]` /
  `slti $at,$t7,2` / `beq` — espera uma variável do kernel cair abaixo de 2. É o estado do driver
  de CD-ROM depois do `GetStat` do passo 87 464 254, que nunca é concluído. Item 4.4n: instrumentar
  quem escreve em `0x80083C58` e o caminho do handler de IRQ2.
