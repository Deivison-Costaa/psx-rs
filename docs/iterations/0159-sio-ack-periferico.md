<!-- Custo, tokens e duracao ficam em docs/metricas.csv, medidos pelo runner. -->

# 0159 — sio-ack-periferico

- **Data:** 2026-08-02
- **Item do roadmap:** 10.84
- **Objetivo:** /ACK — e portanto IRQ7 — só do periférico endereçado e presente.
- **Fonte:** orquestrador (o trabalhador não foi despachado nesta rodada).

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| psx-spx | § Device addressing (L262-L278) | docs/reference/10-controllers-memcards.md |
| psx-spx | § DSR (/ACK) Controller and Memory Card - Byte Received Interrupt (L280-L287) | docs/reference/10-controllers-memcards.md |
| psx-spx | § Event Classes (L1656-L1698) | docs/reference/13-kernel-bios.md |

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | hipotese | Que o defeito de /ACK era o que travava o boot do Rayman: sem card, o BIOS esperaria para sempre. | A spec descreve o /ACK, não o desfecho do boot. | Medição: o caminho do boot é **idêntico** antes e depois (mesmas 454.122 chamadas de `TestEvent`, mesmos passos). A espera do card termina sozinha em 166.321.383 com `DeliverEvent(F4000001h,0100h)` = *card err busy*. O defeito é real e a correção é da spec, mas **não** era o bloqueio. |
| 2 | ferramenta | Que `scripts/mutantes.ps1 0159` aceitaria o número como parâmetro posicional. | Não é assunto de spec. | O script avisou `parametro posicional ignorado` e morreu com `use -Iter NNNN ou -Alterados`; a bateria só rodou na segunda tentativa. |
| 3 | API-Rust | Que reestruturar `send_byte` afetaria só esta iteração. | Não é assunto de spec. | `mutation_anchors` reprovou: as âncoras de `0091/m1`, `0091/m6` e `0092/c2` envelheceram na hora. Atualizei as três **e reexecutei as duas baterias antigas** (0091: 6/6 e 2/2; 0092: 5/5 e 2/2), como o próprio meta-teste exige. |
| 4 | hipotese | Que a espera do jogo era pelo memory card, porque `TestEvent` era o que mais aparecia no traço. | § Event Classes (L1656-L1698) de `docs/reference/13-kernel-bios.md` diz que `F1xxxxxx` é *descritor* de evento, não classe. | Os descritores `F1000001h`/`F1000004h` são os slots 1 e 4 do EvCB — `F4000001h,0004h` (card done) e `F4000001h,2000h` (card err eject). O histograma de PC mostrou depois que o laço final não é esse: são 19.400.685 de 20.000.000 passos em `0x801B95xx`. |

## Bateria de mutação

Placar da bateria: 6/6 mutantes mortos, 3/3 controles verdes, 0 equivalente — docs/mutantes/0159-sio-ack-periferico.mut

Os seis morreram por **assertion panic** no `sio_ack_dispositivo`, nenhum por erro de compilação
(`logs/mutantes-0159-sio-ack-periferico.txt`), e o oráculo não depende de BIOS nem de disco, então
mede o código e não o ambiente. `m1` e `m2` reintroduzem o defeito original por dois caminhos
diferentes; `m6` troca `01h` por `81h` na tabela de endereçamento da spec.

Reexecutadas por âncora envelhecida: `0091` (6/6 mutantes, 2/2 controles) e `0092` (5/5, 2/2).

## Placar antes → depois

Workspace: **893 → 899** testes. Seis testes novos em `sio_ack_dispositivo.rs`.

## Revisão cruzada (orquestrador)

Rodada de autoria do orquestrador; a revisão adversarial é do mesmo agente que escreveu, e isso é
um limite real desta rodada — fica registrado em vez de escondido. O que dá para afirmar com
medição independente do meu julgamento:

- Os seis testes falharam antes da correção (commit `test` separado) e passam depois.
- Dois testes antigos (`sio_portas_16bits`) **afirmavam o comportamento errado**: pediam
  `JOY_STAT.9` aceso depois de enviar `01h` numa porta sem periférico. O contrário está em
  § DSR (/ACK) (L280-L283) de `docs/reference/10-controllers-memcards.md`, com todas as
  letras: *"there will
  be no IRQ if the peripheral fails to send an /ACK, or if there's no peripheral connected at
  all"*. Conectei um controle no helper `bus()` daquele arquivo — os testes são sobre a largura
  dos registradores de 16 bits, não sobre presença de periférico.
- O efeito no boot do Rayman foi medido e é **nulo** até 200 M passos; está no erro 1 acima.

## Decisões e notas

O defeito: `send_byte` acendia `JOY_STAT.9` e pedia IRQ7 em toda transferência com
`JOY_CTRL.12` ligado, sem olhar se havia periférico.
§ Device addressing (L262-L269) de `docs/reference/10-controllers-memcards.md`
diz que o primeiro byte depois de `/CSn` é o endereço
do dispositivo e que *"the device that was addressed shall pull /ACK low to signal its presence"*;
a tabela da mesma seção fixa `01h` para o controle e `81h` para o memory card. Como o emulador não
tem memory card, endereço `81h` não tem quem puxe /ACK.

A correção guarda o primeiro byte da transferência em `address`, responde `0xFF` e não gera /ACK
quando o dispositivo endereçado não está presente, e zera o latch quando `/CS` é solto ou o reset
de `JOY_CTRL.6` acontece. O controle digital continua respondendo `41h`/`5Ah` e os botões — mas só
depois do endereço `01h`, o que antes não era verificado.

**Onde o boot do Rayman realmente para.** O histograma de PC dos últimos 20 M passos de uma
execução de 200 M dá 19.400.685 ocorrências em `0x801B9500..0x801B95FF`, 97% do tempo. O laço é:

```
801B9574  lw    $v0, 0x10($sp)      ; contador de timeout na pilha
801B957C  addiu $v0, $v0, -1
801B9580  sw    $v0, 0x10($sp)
801B958C  bne   $v0, $v1, 801B95AC
801B95B0  lw    $v0, 0x801CF2CC     ; contador do jogo — vale 1
801B95B8  slt   $v0, $v0, $a0       ; $a0 = 2
801B95BC  bne   $v0, $zero, 801B9574
```

O jogo espera `[0x801CF2CC] >= 2` e o contador está parado em **1**. É o mesmo endereço que
0154/0157 já tinham apontado, agora com o consumidor identificado: não é uma leitura solta, é a
condição de saída de um laço de espera com timeout. O incremento mora no hook do próprio jogo, e
0157 mediu que a ativação 0 (que viu `I_STAT.bit0=1`) incrementou e a ativação 3 (que viu o bit
já limpo) não incrementou. A pergunta seguinte é qual desvio dentro do corpo do hook
`0x801B8E60` separa as duas — é o handoff.

Sondas usadas e **não** commitadas: leitura do `ExCB`, traço de instruções do handler de VBlank,
inventário de EvCB e chamadas de evento, e o histograma de PC. Saídas em `logs/0159-*.log`
(pasta ignorada pelo git).
