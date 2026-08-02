# 0153 — rayman-vsync

- **Data:** 2026-08-02
- **Item do roadmap:** 10.79
- **Objetivo:** medir se o `lw` e o `sw` do contador convergem e localizar a perda do VBlank antes do hook.

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| psx-spx | § Opcode/Parameter Encoding (L202-L203), `docs/reference/02-cpu.md` | `docs/reference/02-cpu.md` |
| psx-spx | § Interrupt Acknowledge (L52-L66), `docs/reference/11-interrupts.md` | `docs/reference/11-interrupts.md` |
| psx-spx | § Priority Chains (L1484-L1502), `docs/reference/13-kernel-bios.md` | `docs/reference/13-kernel-bios.md` |

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | enderecamento | A sonda existente contava as 1029 entradas como o hook do jogo. | O alvo desta medicao e o hook instalado cujo primeiro word e `0x801B8E60`; o outro `B(19h)` instala `0x8005A1D8`. | A primeira execucao produziu `513/1029` VBlank pendentes e `I_MASK=0x0009`; a filtragem por `hook[0]` reproduziu `1/1029` e `0x000D/0x008D`. |
| 2 | enderecamento | O rótulo `0x801B95AC` era o PC do `lw`. | A instrucao de load usa `[rs+imm]` em § Opcode/Parameter Encoding (L202-L203), `docs/reference/02-cpu.md`; a sequencia observada tem `lui` em `0x801B95AC` e `lw` em `0x801B95B0`. | O probe capturou o opcode `0x8C42F2CC` no PC dinamico `0x801B95B0`, em vez de presumir o PC do handoff. |

## Bateria de mutação

Bateria de mutação: não se aplica — a iteração é diagnóstico puro e não altera nenhum arquivo em `crates/*/src`.

## Placar antes → depois

- Testes do workspace: **887 → 888**.
- `cargo test --all --no-fail-fast`: 888 testes passaram após atualizar o handoff.
- `cargo test -p psx-core --test rayman_vsync_address`: passou.

## Revisão cruzada (orquestrador)

**As três medições estão corretas e o teste permanente é o melhor das últimas três iterações.**
Ele não afirma constantes sobre si mesmas: monta um `Bus`/`Cpu` reais, planta o sentinela
`0xDEADBEEF` em `0x801DF2CC`, executa os opcodes de verdade (`lui $2,0x801D`, `lw $2,0xF2CC($2)`
e depois `sw`) pela CPU do emulador, e verifica por EFEITO que leitura e escrita caem em
`0x801CF2CC` deixando o sentinela intacto. Falharia se a nossa CPU tratasse o imediato como sem
sinal. Rodei o portão por conta própria: 888 testes, verde.

**Uma hipótese do handoff foi refutada por medição, e era minha.** Eu levantei a possibilidade
(b) de a CPU estar vetorizando sem causa real, apontando que 1470 vetorizações para 660 IRQ0 era
diferença demais. Medido: as 1470 tinham `I_STAT & I_MASK != 0`. Não há vetor espúrio — o nosso
caminho de exceção está íntegro, e a suspeita era infundada.

**Um erro meu de handoff também foi corrigido pela rodada.** Eu afirmei que o `lw` do spin
estava em `0x801B95AC`; ali está o `lui`, e o `lw` está em `0x801B95B0`. A rodada mediu o PC
dinâmico em vez de aceitar o meu rótulo, que é exatamente o comportamento certo.

**A rodada morreu pela parede de TPM,** com `Requested 200996` contra o teto de 200 000 — mil
tokens acima. Matei-a em vez de esperar os 25 minutos do detector de travamento, porque
`Request too large` não é limite transitório: a requisição é maior que o orçamento de um minuto
inteiro e nenhuma espera resolve. Ela já havia commitado o teste e escrito doc, STATUS e
ROADMAP; o orquestrador acrescentou a linha de métricas (`falha:exit-143`, US$ 0,2862, 75
passos) e fechou o ciclo. A métrica registra a morte, não o proveito: o trabalho aproveitado
está aqui.

**O quadro que se fecha, e o que sobra.** Com endereço confirmado, o timeout do `VSync()` é de
CONTAGEM: o contador em `0x801CF2CC` recebe **um** incremento e o spin fica esperando o segundo.
A causa provável agora tem nome e PC: `0x00004A1C` escreve `0xFFFFFFFE` em `I_STAT` — ack do bit
0, conforme `docs/reference/11-interrupts.md` § Interrupt Acknowledge (L52-L66) — em 570 dos
1028 intervalos entre a vetorização e a entrada do hook. Ou seja, o kernel consome o VBlank
antes de o hook do jogo poder vê-lo. Restam 458 entradas sem ack nessa janela, que a rodada
honestamente não explica.

A pergunta seguinte, registrada como **10.80**, é se esse ack deveria acontecer: em hardware
real o jogo funciona, então ou o handler do kernel não deveria consumir o VBlank nesta
configuração, ou o hook do jogo não depende de `I_STAT` e o nosso emulador lhe entrega estado
diferente do real.

## Decisões e notas

- **Endereço:** na execução do `lw`, `$2=0x801D0000`, o imediato assinado `0xF2CC` resulta em `0x801CF2CC`, e o valor lido foi `1`. O `sw` de `0x801B8C50` também gravou `1` em `0x801CF2CC`; `0x801DF2CC` permaneceu separado. A escrita e a leitura concordam, portanto o timeout é de contagem, não de endereço.
- **Hook:** com `hook[0]==0x801B8E60`, foram medidas 1029 entradas; somente a entrada 0, no passo 164112358, tinha `I_STAT & I_MASK & 1 != 0`. Todas tiveram `CAUSE.ExcCode=00h`.
- **Hipótese (a):** entre a vetorização mais recente e a entrada do hook, `0x00004A1C` escreveu `0xFFFFFFFE` e limpou o bit 0 em 570 intervalos. O escritor `0x8005A298` também limpou bit 0 em 512 ocorrências da execução ampliada, mas não apareceu nesses intervalos do hook. Os demais 458 hooks sem VBlank não tiveram um ack de bit 0 nesse intervalo; o bit já estava ausente ou outro IRQ era a causa. A hipótese explica parte das 1028 entradas, não todas.
- **Hipótese (b):** na janela que chega a `VSync: timeout`, houve 660 IRQ0 e 1470 vetorizacoes; as 1470/1470 tinham `I_STAT & I_MASK != 0` no momento capturado após a vetorização. Não houve vetor espúrio sem causa pendente.
- **Spec de ack:** `docs/reference/11-interrupts.md`, § Interrupt Acknowledge (L52-L66), exige escrever zero no bit correspondente.
- **Spec de cadeia:** `docs/reference/13-kernel-bios.md`, § Priority Chains (L1484-L1502), coloca `VblankIrq` antes dos handlers posteriores. A medição registra os PCs e não atribui identidade além do endereço observado.
- O teste permanente `crates/psx-core/tests/rayman_vsync_address.rs` executa `lw` e `sw` reais, verifica o endereço comum por seus efeitos e mantém `0x801DF2CC` como sentinela. Não foi adicionada correção em `crates/*/src`.
