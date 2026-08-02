<!-- Custo, tokens e duracao ficam em docs/metricas.csv, medidos pelo runner. -->

# 0160 — sio-ack-atrasado

- **Data:** 2026-08-02
- **Item do roadmap:** 10.86
- **Objetivo:** entregar o /ACK do SIO0 pelo scheduler, fora da janela que o driver do kernel ignora.
- **Fonte:** orquestrador.

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| psx-spx | § Address byte (01h) being sent (L381-L389) | docs/reference/10-controllers-memcards.md |
| psx-spx | § Emulation Note (L311-L315) | docs/reference/10-controllers-memcards.md |

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | timing | Que reconhecer o byte no mesmo ciclo da escrita em `JOY_TX_DATA` era só uma simplificação inofensiva. | § Emulation Note (L311-L315) de `docs/reference/10-controllers-memcards.md` é explícita: *"emulators can't trigger IRQ7 immediately within 0 cycles after sending the byte"*, porque o driver do kernel reconhece o IRQ7 **antigo** ~100 ciclos depois e só então espera o novo. | Lendo a seção antes de escrever o teste. O emulador fazia exatamente o que a spec proíbe. |
| 2 | teste-fraco | Que o meu próprio teste de cancelamento (`soltar_o_cs_cancela_o_ack_ainda_nao_entregue`) media o cancelamento. | Não é assunto de spec. | Ele passava porque, com `/CS` solto, `JOY_CTRL.12` também cai e o `STAT.9` não subiria de qualquer jeito — passava pelo motivo errado. Reforcei: solta e **reassere** `/CS` antes de avançar os ciclos. Sem isso o mutante `m5` sobreviveria. |
| 3 | teste-fraco | Que mudar o momento do /ACK não mexeria em testes antigos que já eram verdes. | Não é assunto de spec. | A **bateria do 0091 caiu para 5/6**: `ctrl_bit4_ack_limpa_stat_bit9` virou vácuo — afirmava `STAT.9 == 0` depois do ack, mas `STAT.9` nunca chegava a subir. Acrescentei a pré-condição explícita e a bateria voltou a 6/6. Foi o mutante que achou, não eu. |
| 4 | API-Rust | Que só o manifesto desta iteração precisaria de âncora nova. | Não é assunto de spec. | `mutation_anchors` reprovou `0091/m6` e `0159/m4` na primeira validação; atualizei as duas e reexecutei as quatro baterias que tocam `sio.rs`. |

## Bateria de mutação

Placar da bateria: 6/6 mutantes mortos, 3/3 controles verdes, 0 equivalente — docs/mutantes/0160-sio-ack-atrasado.mut

Nenhum morreu de erro de compilação (`grep -cE '^error\[E'` no log dá 0; há 9 `panicked` de
assertion), e o oráculo roda sem BIOS nem disco. `m1` devolve o /ACK ao ciclo zero, `m2` o joga
dentro dos 100 ciclos ignorados e `m3` além dos 100 µs de timeout — os três valores que a spec
delimita.

Reexecutadas por tocarem o mesmo fonte: `0091` (6/6, 2/2 — depois da correção do oráculo),
`0159` (6/6, 3/3) e `0092` (5/5, 2/2).

## Placar antes → depois

Workspace: **899 → 905** testes.

## Revisão cruzada (orquestrador)

Rodada de autoria do orquestrador; a autorrevisão é um limite real e fica registrado. O que é
verificável sem depender do meu julgamento:

- Os três testes que exercem o novo momento do /ACK falharam antes da correção e passam depois;
  os outros três já passavam e continuam (o commit `test` é anterior ao `fix`).
- Quatro baterias de mutação, com `.resultado` versionado, incluindo a que **regrediu** e foi
  consertada.
- O efeito no boot do Rayman foi medido e é **nulo**: o histograma de PC dos últimos 20 M passos
  de uma execução de 200 M é idêntico ao de 0159, incluindo as 19.400.685 ocorrências em
  `0x801B9500..0x801B95FF` e as mesmas 454.122 chamadas de `TestEvent`. A razão também está
  medida: o caminho do pad só começa em 164 M e `I_MASK.7` só é ligada em 164.754.404, enquanto o
  laço que trava começou em ~89 M.

## Decisões e notas

§ Address byte (01h) being sent (L384-L386) de `docs/reference/10-controllers-memcards.md`
dá os dois limites, e são eles que viraram valor de teste:

- o driver *ignora* pulsos nos primeiros 2-3 µs (**100 ciclos**) depois do último SCK;
- o driver *desiste* se o /ACK não chegar em **100 µs** (3386 ciclos a 33,8688 MHz).

O atraso escolhido é **338 ciclos** (10 µs): três vezes a janela ignorada e um décimo do timeout.
A spec fixa a janela, não o valor; o teste afirma os dois limites, não o 338 — por isso o
controle `c1`, que troca 338 por 300, é verde.

A entrega passou a ser um evento do scheduler (`SIO_ACK`), como exige R2: `send_byte` só
**pede** o pulso, o barramento agenda em `total_cycles + 338`, e o evento chama `deliver_ack`,
que acende `JOY_STAT.7`, `JOY_STAT.9` e o IRQ7. Soltar `/CS` ou resetar por `JOY_CTRL.6` cancela
um pulso ainda não entregue — um byte da transferência anterior não pode reconhecer a seguinte.

**O que continua bloqueando o Rayman** (medido em 0159 e reconfirmado aqui): o laço em
`0x801B9574` espera `[0x801CF2CC] >= 2` e o contador está em 1. O corpo do hook `0x801B8E60` foi
traçado nesta rodada como diagnóstico, e a divergência é de uma instrução só:

```
801B8E78  lhu  $v1, 0($v1)      ; I_STAT   — ativacao 0: 0x0001 | ativacao 3: 0x0000
801B8E90  lhu  $v0, 0($v0)      ; I_MASK   — 0x000D nas duas
801B8E98  and  $v1, $v1, $a0    ; $a0 = [0x801CF2E4] = 0x0009 (VBlank | DMA)
801B8E9C  and  $v0, $v0, $v1
801B8EA0  beq  $v0, $zero, 801B8F94   ; ativacao 3 sai por aqui, em 26 instrucoes
```

A ativação 0 executa 277 instruções e **alcança** `0x801B8C50` (o incremento); as ativações 1 e 2
(`I_STAT=0x08`, DMA) executam 444 e não incrementam; a ativação 3 sai em 26 instruções porque
`I_STAT` já está zerado. Ou seja: o hook do jogo só conta o VBlank que ainda estiver pendente
quando ele roda, e a partir da instalação do elemento de prioridade 2 do BIOS (0158) o VBlank é
reconhecido por `0x4A1C` **antes** do hook. A pergunta seguinte — se esse reconhecimento
antecipado é defeito nosso ou comportamento do BIOS que outra coisa deveria evitar — é o item
10.87 e o próximo handoff.

Sondas usadas e não commitadas; saídas em `logs/0160-*.log` (pasta ignorada pelo git).
