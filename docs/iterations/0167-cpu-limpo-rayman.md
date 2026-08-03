<!-- Custo, tokens e duracao ficam em docs/metricas.csv, medidos pelo runner. -->

# 0167 — cpu-limpo-rayman

- **Data:** 2026-08-02
- **Item do roadmap:** 10.94 (diagnóstico)
- **Objetivo:** medir o Rayman com a CPU aprovada pelo Amidog e decidir se o jogo volta a ser a frente.
- **Fonte:** orquestrador.

## Spec consultada

Nenhuma. Esta rodada não implementa hardware: é medição comparativa entre dois commits.

## O experimento

Duas execuções do mesmo disco (`Rayman (USA) DADOS.cue`, track de dados, fora do repositório),
mesma BIOS, mesmo horizonte, mesma amostragem — variando só o binário:

| | commit | Amidog `psxtest_cpu` |
|---|---|---|
| antes | `0768725` (pré-0164) | `Result: 00000909`, 4.918 linhas `error @` |
| depois | `a21d500` (pós-0166) | `Result: 00000101`, **0** linhas `error @` |

O binário "antes" foi reconferido contra o próprio Amidog antes de valer como base, para não
comparar contra uma compilação que na verdade já tivesse as correções.

## Resultado

**As três correções de CPU não mudaram nada no Rayman.**

| Medida | antes (`0768725`) | depois (`a21d500`) |
|---|---|---|
| `VSync: timeout` em 200 M passos | 142 | **142** |
| `VSync: timeout` em 600 M passos | 522 | **522** |
| PC quente em 590-600 M | `0x8019FA1C`..`28` + `0x80131DB8`..`C4` | **idêntico** |
| Amostras no `DeliverEvent` do kernel (`0x00001B44`) | 21.428 | 21.429 |

O histograma bate amostra a amostra (diferença de 1 em 103 mil). Zerar 4.918 reprovações de um
teste de hardware não moveu um passo do jogo.

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a medição diz | Como foi pego |
|---|---|---|---|---|
| 1 | processo | Que `Rayman (USA).cue` fosse o disco usado nas rodadas anteriores. | Não é assunto de spec. | O boot parou em `boot file : cdrom:PSX.EXE;1` com 868 bytes de TTY, muito antes do `Execute !`. As rodadas 0149-0157 usam `Rayman (USA) DADOS.cue` (só a track de dados). Com o `.cue` certo: 8.900 bytes e `Execute !`. Se eu não tivesse conferido, teria registrado uma regressão inexistente. |
| 2 | processo | Que o `cargo build` de 1,3 s no commit antigo não tivesse recompilado de verdade. | Não é assunto de spec. | Rodei o Amidog com o binário "antes" e ele deu `Result: 00000909` / 4.918 erros. A base é legítima; a comparação vale. |

## Onde o jogo está parado agora

O laço quente decodificado a partir do dump (`--dump-mem`):

```
8019FA1C  jal   0x80131DB8
8019FA20  nop
8019FA24  beq   $v0, $zero, 0x8019FA1C
8019FA28  nop

80131DB8  lui   $v0, 0x801D
80131DBC  lh    $v0, -0x1144($v0)      ; carrega [0x801CEEBC], 16 bits com sinal
80131DC0  jr    $ra
80131DC4  nop
```

Ou seja: `do { } while ([0x801CEEBC] == 0)`. Logo antes, em `0x8019FA0C`..`0x8019FA18`, o jogo
registra o endereço `0x8019F848` por meio da rotina `0x81802E0`.

Duas coisas mudam de figura com isso:

1. **Não é mais o laço de `0x801B9574` esperando `[0x801CF2CC] >= 2`** (0159-0163). Aquele era o
   estado em ~166 M passos; em 590-600 M o jogo espera outra variável, `0x801CEEBC`. O item 10.85
   descreve uma parada anterior na linha do tempo, não a atual.
2. **O kernel está entregando eventos o tempo todo** — `DeliverEvent` (`0x00001B44`) é o PC mais
   amostrado de todos. Não é um caso de "nenhuma interrupção chega".

## Bateria de mutação

Bateria de mutação: não se aplica — rodada de diagnóstico comparativo entre dois commits, sem
uma linha de código de produção alterada; não há mutante a matar em `crates/*/src/`.

## Placar antes → depois

Workspace: **921** testes (rodada de diagnóstico, sem produção nova).
Amidog: inalterado nesta rodada (`Result: 00000101`, 0 linhas de erro).

## Revisão cruzada (orquestrador)

Rodada de autoria do orquestrador. O que sustenta o resultado não é julgamento meu: são dois
binários, um oráculo externo confirmando qual é qual, e um histograma que bate amostra a amostra.
O risco desta rodada era o oposto do usual — eu queria que a CPU limpa tivesse destravado o jogo,
e ela não destravou. O registro fica como está.

## Decisões e notas

**O que isto fecha:** a hipótese, escrita no STATUS desde a 0164, de que o Rayman estava sendo
depurado contra uma CPU defeituosa e que por isso o sintoma do jogo era "a altitude errada". A
CPU era mesmo defeituosa, as correções eram mesmo devidas — e não eram a causa do travamento.

**O que isto abre:** `VSync: timeout` continua sendo impresso 142 vezes até 200 M. A iteração
0104 já havia medido que essa mensagem não significa "evento não entregue": o contador de vblank
do kernel incrementa certo, e o que estoura é o **orçamento do laço de espera, contado em
iterações, que depende do custo em ciclos de cada instrução**. Com a CPU agora correta em
*função*, a suspeita natural passa a ser correção em *tempo* — e é exatamente o que os gabaritos
`tests/exes/ps1-tests/timers/psx.log` e `cpu/access-time/psx.log` medem, sem que ninguém nunca os
tenha comparado (item 10.23). A prioridade seguinte deixa de ser o jogo e passa a ser a régua.

**Novo item 10.94** para o laço de `0x801CEEBC`, para que a parada atual pare de ser confundida
com a de `0x801B9574` (10.85).
