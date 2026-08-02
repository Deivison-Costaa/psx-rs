<!-- Custo, tokens e duração ficam em docs/metricas.csv, medidos pelo runner. -->

# 0154 — rayman-vblank-ack

- **Data:** 2026-08-02
- **Item do roadmap:** 10.80
- **Objetivo:** identificar quem instala `0x00004A1C` e medir se o handler de VBlank consulta EvCB antes do ack de `I_STAT`.

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| psx-spx | § Priority Chains (L1484) | docs/reference/13-kernel-bios.md |
| psx-spx | § C(00h) - EnqueueTimerAndVblankIrqs(priority) (L1579) | docs/reference/13-kernel-bios.md |
| psx-spx | § B(07h) - DeliverEvent(class, spec) (L1642) | docs/reference/13-kernel-bios.md |
| psx-spx | § Default IRQ Handler Events (very unstable, don't use) (L1781) | docs/reference/13-kernel-bios.md |
| psx-spx | § 1F801070h I_STAT - Interrupt status register (R=Status, W=Acknowledge) (L21) | docs/reference/11-interrupts.md |
| psx-spx | § Interrupt Acknowledge (L52) | docs/reference/11-interrupts.md |

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | controle/endereçamento | `C(00h)` seria o instalador do código residente em `0x4A1C`, porque a função enfileira os handlers de Timer/VBlank. | A spec só documenta que `C(00h)` adiciona handlers à cadeia de prioridade 1; ela não atribui a essa função a cópia física do corpo do handler. | A sonda executou o BIOS real: a única escrita em `A0004A1C` ocorreu no passo 44.349, em `BFC00448`, antes das duas chamadas observadas a `C(00h)` nos passos 138.123 e 154.914.471. |
| 2 | flags/ordenação | O ack poderia depender de encontrar um EvCB correspondente a `F0000001h/1000h`. | `I_STAT` é reconhecido escrevendo zero no bit correspondente; o corpo do BIOS medido pode testar a entrega depois do ack. | A CPU executou os opcodes reais de `0x4A10` a `0x4A1C`: `sw $t9,0($t0)` com `$t9=FFFFFFFE` vem antes do `lw` de `0x74BC` e do `beq` que decide a chamada de entrega. |

## Bateria de mutação

Bateria de mutação: não se aplica — a iteração altera apenas teste de integração e documentação; nenhum arquivo em `crates/*/src/` foi modificado.

## Placar antes → depois

Workspace: 888 → 889 testes; o novo `rayman_vblank_ack.rs` executa os opcodes reais do handler e verifica a mudança observável em `I_STAT`.

## Revisão cruzada (orquestrador)

**As três medições estão corretas, e duas delas refutam o handoff do orquestrador.** Rodei o
portão por conta própria: 889 testes, verde.

**A rodada me corrigiu.** Eu apontei `C(00h) EnqueueTimerAndVblankIrqs` como suspeito de
instalar o corpo em `0x4A1C`, com base em `docs/reference/13-kernel-bios.md` § C(00h) -
EnqueueTimerAndVblankIrqs(priority) (L1579). A medição mostra que não: a escrita veio de
`BFC00448` no passo 44.349, **antes** das duas chamadas a `C(00h)` (passos 138.123 e
154.914.471). A spec diz que `C(00h)` enfileira handlers na cadeia, não que copie o corpo — eu
li plausibilidade onde não havia afirmação. A rodada mediu em vez de aceitar o meu rótulo, que é
o comportamento certo, e é a segunda vez seguida que isso acontece (na 0153 ela corrigiu o PC do
`lw` que eu dera errado).

**A segunda hipótese também caiu.** Eu perguntei se o ack seria condicionado a existir um EvCB
correspondente. Não é: o `sw $t9,0($t0)` com `$t9=0xFFFFFFFE` executa **antes** da leitura de
`0x74BC` que decide a entrega. O ack é incondicional.

**O teste permanente é sólido e segue o padrão bom.** Como o da 0153, ele não afirma constantes
sobre si mesmas: levanta os bits 0 e 1 de `I_STAT`, executa os opcodes reais do handler pela CPU
do emulador e verifica que o resultado é `0x2` — bit 0 limpo, **bit 1 preservado**. Falharia se
a nossa semântica de escrita em `I_STAT` gravasse o valor direto em vez de reconhecer.

**Uma ressalva ao teste.** O nome `ack_de_vblank_ocorre_antes_da_consulta_de_evento` afirma uma
ordem que o corpo não observa: o teste para no `sw` e nunca executa nem verifica a leitura de
`0x74BC`. A ordem foi medida pela sonda e está registrada nas notas, mas o teste permanente
sustenta só a metade do que o nome promete. Não estendi o teste porque isso exigiria os opcodes
seguintes do handler, que não foram medidos — inventá-los seria pior que a ressalva.

**A rodada morreu pela parede de TPM** (`Requested 202300`), no passo 88. Ela havia commitado o
teste e escrito doc, STATUS e ROADMAP; o orquestrador acrescentou a métrica (`falha:exit-143`,
US$ 0,3264) e fechou o ciclo. Vale notar que o handoff pedia explicitamente para parar por volta
do passo 55 e fechar o ciclo — a instrução textual não segurou, terceiro caso do mesmo padrão.
As métricas, por outro lado, foram tratadas corretamente desta vez: a rodada checou o par
`(ts, iter)` e não duplicou nada.

**O que este achado significa, e é desconfortável.** O BIOS real acka IRQ0 incondicionalmente
antes do hook, e o hook do Rayman depende de ver o bit em `I_STAT`. Como executamos o BIOS real
instrução a instrução, o mesmo aconteceria em hardware real — e ali o jogo funciona. Logo a
divergência está em outro lugar: ou na frequência/momento em que levantamos IRQ0, ou no fato de
o hook ser alcançado apenas 1 vez com VBlank pendente em 660 IRQ0 levantadas. É isso que o
**10.81** precisa medir.

## Decisões e notas

O código em `0x00004A1C` não foi instalado por `C(00h)`. No boot medido, `PC=0xBFC00448` executou `sw $a3,-4($a1)` (`0xACA7FFFC`) e escreveu `0xAD190000` em `A0004A1C` no passo 44.349. A primeira chamada real a `C(00h)` foi no passo 138.123, com `priority=1`; a segunda foi no passo 154.914.471. Não houve outra escrita observada em `0x4A1C` depois da cópia inicial.

O corpo residente começa, na amostra, com `lui $t0,0`, `lw $t0,0x725C($t0)`, `addiu $t9,$zero,-2` e `sw $t9,0($t0)` em `0x4A1C`. Na primeira entrada com `I_STAT=1`, `$t0=0x1F801070` e `$t9=0xFFFFFFFE`; a escrita preserva o bit 1 e limpa o bit 0. Só depois o fluxo lê `0x74BC` e, se o valor não for zero, segue para a rotina de entrega. Portanto o ack não é condicionado à existência de EvCB. A spec documenta o evento padrão `F0000001h,1000h` como IRQ0 VBlank, mas também o classifica como instável; não documenta esse corpo nem a ordem interna do BIOS.

O hook do Rayman confirmou a consequência. Na ativação 0, `0x801B8E78` lê `I_STAT=1` e `0x801B8E90` lê `I_MASK=0xD`; nas leituras seguintes dentro da mesma ativação, `I_STAT` já é zero. Na ativação 1, a primeira leitura de `I_STAT` vale `8`, não `1`. Além desses registradores, o hook lê os ponteiros `0x801CF2DC -> 0x1F801070`, `0x801CF2E0 -> 0x1F801074` e `0x801CF2E4 -> 9`; não apareceu outro indicador de VBlank que substituísse `I_STAT`.

O resultado resolve 10.80 como diagnóstico, sem correção de produção: o BIOS real acka IRQ0 incondicionalmente antes do hook, e o hook do jogo depende de observar o bit de `I_STAT`. A diferença entre esse comportamento e hardware real ainda não foi medida. Permanecem sem explicação os 458 intervalos, entre os 1028 intervalos vetorização→hook restantes, sem escrita observada pelo `PC=0x4A1C`; isso foi registrado no novo item 10.81, sem ampliar esta iteração.

A linha pendente de métricas de 0153 já existia em `docs/metricas.csv` com o mesmo par `(ts, iter)`; ela foi removida sem duplicar a entrada e nenhuma métrica de 0154 foi fabricada.
