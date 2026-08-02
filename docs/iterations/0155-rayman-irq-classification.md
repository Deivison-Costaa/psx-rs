<!-- Custo, tokens e duração ficam em docs/metricas.csv, medidos pelo runner. -->

# 0155 — rayman-irq-classification

- **Data:** 2026-08-02
- **Item do roadmap:** 10.81
- **Objetivo:** classificar no momento da vetorização os 458 intervalos sem ack observado em `0x4A1C`.

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| psx-spx | § 1F801074h I_MASK - Interrupt mask register (R/W) (L27-L39) | docs/reference/11-interrupts.md |
| psx-spx | § Interrupt Request / Execution (L45-L50) | docs/reference/11-interrupts.md |
| psx-spx | § B(19h) - HookEntryInt(addr) (L1476-L1483) | docs/reference/13-kernel-bios.md |

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | endereçamento | O ack poderia ser localizado pelo endereço destino `0x4A1C`. | A regra de ack é uma escrita de zero no bit correspondente de `I_STAT`, não uma escrita na memória do handler; docs/reference/11-interrupts.md § Interrupt Acknowledge (L52-L55). | A primeira sonda encontrou a cópia de instalação em `A0004A1C` e zero acks; a amostra anterior mostra que o PC `0x4A1C` escreve em `0x1F801070`, então o filtro foi corrigido para o PC. |
| 2 | controle | Parar no primeiro spin seria a mesma janela dos 1029 hooks. | A spec descreve quando o hook roda, mas não define a janela desta medição; docs/reference/13-kernel-bios.md § B(19h) - HookEntryInt(addr) (L1476-L1483). | O arreio curto produziu 13 execuções do PC do jogo; a janela longa, limitada ao 1029º hook do jogo, fechou 570 acks e 459 sem ack. |
| 3 | flags | Os 459 sem ack seriam diretamente os 458 do item. | `I_STAT.bit0` é VBlank, enquanto bit 2 é CDROM e bit 3 é DMA; docs/reference/11-interrupts.md § 1F801074h I_MASK - Interrupt mask register (R/W) (L27-L39). | O primeiro intervalo sem ack tinha `I_STAT=0x0001`; retirado o intervalo inicial da amostra de 1029 hooks, restam exatamente os 458 pedidos. |

## Bateria de mutação

Bateria de mutação: não se aplica — a iteração altera apenas um teste de integração e documentação; nenhum arquivo em `crates/*/src/` foi modificado, portanto não há produção para mutar.

## Placar antes → depois

Workspace: **889 → 890** testes; `rayman_irq_classification.rs` executa `lui`, `ori`, `lw` e `sw` reais no vetor e comprova o resultado por um sentinela.

## Revisão cruzada (orquestrador)

**A classificação está correta e fecha o item sem deixar resíduo.** Conferi a soma: 173 CDROM +
285 DMA = 458, exatamente os intervalos em aberto. Nenhum bit 0 entre eles. Rodei o portão por
conta própria: 890 testes, verde.

**Um "não há mistério" medido vale tanto quanto um defeito achado**, e é o caso aqui: os 458 não
são VBlank perdido, são interrupções de CDROM e DMA, para as quais o handler de VBlank em
`0x4A1C` não teria mesmo por que acertar o bit 0. A hipótese barata que o handoff levantou se
confirmou, e a rodada teve o cuidado de separar o intervalo inicial (que era VBlank) dos 458 —
459 sem ack menos o inicial fecha exatamente o balanço do 10.80.

**O teste permanente é o melhor da série.** Ele não congela contagem de ROM em constante:
levanta os bits 2 e 3, deixa a CPU **vetorizar de verdade** (verificando que o PC chegou a
`0x80000080`), e então executa `lui`/`ori`/`lw`/`sw` reais que copiam `I_STAT` para um sentinela
iniciado em `0xA5A5A5A5`. Exercita a nossa entrega de IRQ, a vetorização, o `SR` e a leitura de
`I_STAT`, tudo verificado por efeito.

**A ordem obrigatória do handoff funcionou.** Pela primeira vez a rodada respeitou o
sequenciamento: mediu apenas o item (1) e deixou (2) e (3) por fazer, em vez de acumular as três
medições. Ainda assim morreu pela parede de TPM (`Requested 200766`) no passo 73, com o teste
commitado e o PR não aberto — o orquestrador acrescentou a métrica (`falha:exit-143`,
US$ 0,2903) e fechou o ciclo, como nas 0153 e 0154. Pedir ordem de trabalho funciona; pedir
limite de passos não funcionou.

**O que ficou por medir virou item, não promessa.** As duas medições não iniciadas foram
registradas: **10.82** (período em ciclos entre subidas de IRQ0 comparado ao frame NTSC) e
**10.83** (o destino dos ~89 IRQ0 que não produzem entrada de hook). A 10.82 é a mais valiosa
das duas e ninguém olhou para ela ainda: se a nossa taxa de VBlank estiver errada, é defeito de
produção com conserto claro.

Três linhas do ROADMAP (10.14, 10.49, 10.50) foram comprimidas para caber sob o teto de 7 KB.
Encurtadas, nunca apagadas — o contexto mora nos docs de iteração.

## Decisões e notas

A sonda executou o BIOS SCPH1001 com `Rayman (USA) DADOS.cue` e registrou `I_STAT` imediatamente após cada entrada da CPU no vetor `0x80000080`, antes do hook. A primeira entrada no spin ocorreu no passo **166.378.016**. O filtro do hook foi `0x801B8E60`; a janela parou no 1029º hook do jogo, com 1029 intervalos vetorização→hook, 570 contendo escrita observada pelo PC `0x00004A1C` e 459 sem essa escrita.

O primeiro dos 459 intervalos sem ack tinha `I_STAT=0x0001`, isto é, VBlank. Ele é o intervalo inicial fora dos 1028 usados pelo balanço do item 10.80. Nos **458** restantes, a classificação no instante da vetorização foi:

| Bit pendente em `I_STAT` | Causa documentada | Intervalos |
|---|---|---:|
| 2 (`0x0004`) | CDROM | 173 |
| 3 (`0x0008`) | DMA | 285 |
| outros, incluindo bit 0 | nenhuma ocorrência | 0 |

O resultado fecha 10.81 como diagnóstico: não há mistério de VBlank nos 458. Eles são interrupções CDROM ou DMA, portanto não se espera que o handler de VBlank em `0x4A1C` faça o ack do bit 0. A spec também explica por que uma IRQ processada pode não chegar ao hook: `HookEntryInt` só chama o hook após o `ExceptionHandler` terminar, e `ReturnFromException` pode pulá-lo; docs/reference/13-kernel-bios.md § B(19h) - HookEntryInt(addr) (L1476-L1483).

O teste permanente não transforma a contagem da ROM em constante: ele levanta bits 2 e 3, habilita a interrupção, deixa a CPU vetorizar e executa os opcodes reais que leem `I_STAT` e gravam o sentinela. Não houve alteração em `crates/psx-core/src/`, nem item novo de produção. As medições (2) sobre IRQ0 que chega ao hook e (3) sobre o período em ciclos não foram iniciadas, conforme a ordem obrigatória de fechar o PR após (1).

`logs/metrics-pending.csv` continha uma linha de 0154 cujo par `(ts,iter)` já existia em `docs/metricas.csv`; ela foi removida sem duplicar a métrica.
