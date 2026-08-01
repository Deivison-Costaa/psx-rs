# 0127 — evento-consumo-shell

- **Data:** 2026-07-31
- **Item do roadmap:** 4.4v
- **Objetivo:** Diagnosticar por que dois eventos de CD-ROM (EvCB[0] spec=10h, EvCB[5] spec=200h) ficam ready (status=4000h) e o shell não os consome, travando o boot em `SetGraphDebug`.

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| psx-spx | § B(0Ah) — WaitEvent (L1625) | docs/reference/13-kernel-bios.md |
| psx-spx | § B(0Bh) — TestEvent (L1637) | docs/reference/13-kernel-bios.md |
| psx-spx | § BIOS RAM Map — Table of Tables (L440) | docs/reference/13-kernel-bios.md |
| psx-spx | § Event Control Blocks (EvCB) (L2889) | docs/reference/13-kernel-bios.md |

### Caminho canônico da referência

```
TestEvent(F10000xxh) → lê EvCB[xx].status
  status=2000h (busy) → retorna 0
  status=4000h (ready) → retorna 1 e reseta para 2000h
  status=1000h (disabled) → retorna 0
```

## Medições — discriminante

### Resultado final do discriminante (refinado)

O teste `evcb_status_checkpoints_discriminante` (psx-core) roda a emulação diretamente e cheque o status dos EvCBs a cada checkpoint:

| Checkpoint | EvCB[0] spec=10h | EvCB[5] spec=200h | TTY |
|---|---|---|---|
| 85 M | não alocado | não alocado | `ResetCallback: _96_remove` |
| 88 M | busy (2000h) | busy (2000h) | `System Controller ROM Version` |
| **89.9 M (último TestEvent)** | **READY (4000h)** | **READY (4000h)** | — |
| 90 M | ready | ready | — |
| 100 M (saturação) | ready | ready | `bad hankaku code` |

**Veredito: corrida no mesmo step.** Ambos os eventos viram `status=4000h` entre 88 M e 89.9 M passos. No exato step 89 906 602 — o ÚLTIMO TestEvent do shell — os EvCBs JÁ ESTÃO ready. Mas o `Cpu::step` executa a instrução (TestEvent lê EvCB) ANTES de despachar IRQs (handler → DeliverEvent → EvCB vira ready). Ou seja: TestEvent lê status=2000h (busy), retorna 0; DEPOIS, a IRQ2 dispara, DeliverEvent marca os EvCBs como ready (4000h); mas o shell já desistiu.

Observação adicional: EvCB[1] (spec=20h) já estava ready em 88 M — 1.9 M passos ANTES do último TestEvent. O shell não está esperando por spec=20h.

### Não é defeito no TestEvent

A hipótese "TestEvent retorna 0 (busy) para eventos ready" das rodadas anteriores era HIPÓTESE NÃO MEDIDA. O discriminante prova que os eventos viram ready NO MESMO STEP do último TestEvent, e a ordem intrainstrução (instrução → IRQ ao final do step) explica o retorno 0. Não é bug na BIOS, é timing.

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec/medição diz | Como foi pego |
|---|---|---|---|---|
| 1 | diagnóstico | Que "TestEvent retorna 0 para eventos ready" era fato medido (eprintln da rodada 1). | Era hipótese não verificada. O discriminante com checkpoints finos prova que a transição ready ocorre no mesmo step do último TestEvent — corrida, não defeito. | Checkpoint no step 89 906 602 mostrou EvCB já ready; ordem intrainstrução explica retorno 0. |
| 2 | diagnóstico | Que a janela de 85-90 M (checkpoints a cada 5 M) era suficiente para discriminar. | A primeira rodada com checkpoints 85/90/95/100 M mostrou "DEPOIS → corrida confirmada". Só a adição do checkpoint no step exato do último TestEvent (89 906 602) revelou que os eventos estavam ready NAQUELE step. | O refinamento dos checkpoints inverteu o veredito de "ready depois" para "ready no mesmo step". |
| 3 | processo | Que o manifesto de mutação com CRLF casaria os meta-testes. | Invariante 10.40: `mutantes.ps1` só casa âncora em LF. Os meta-testes lêem o manifesto com `.lines()` (que strip `\r`) e o alvo com `read_to_string` (que preserva `\r\n`), causando mismatch de `\n` vs `\r\n`. | `mutation_anchors` reportou "encontrada 0 vez(es)" para âncoras que existiam. Converti o manifesto para LF e os meta-testes passaram. |

## Bateria de mutação

Placar da bateria: 5/5 mutantes mortos, 2/2 controles verdes, 0 equivalente — docs/mutantes/0127-evento-consumo-shell.mut

**Bateria manual** (alvo `crates/psx-cli/src/main.rs`, teste `evento_consumo_shell` no crate `psx-cli`, invariante 29). Rodado com BIOS real + Crash Bandicoot (USA). Comando: `cargo test -p psx-cli --test evento_consumo_shell --release`.

| # | Mutação | Teste que pegou |
|---|---|---|
| m1 | `--trace-pcs` não insere endereços no HashSet (trace nunca dispara) | `trace_wait_e_test_event_diagnostico` — count_test_event=0 |
| m2 | `eprintln!` de trace perde o prefixo "trace pc=" (parser não encontra linhas) | `trace_wait_e_test_event_diagnostico` — count_test_event=0 |
| m3 | `--max-steps` zera o limite (passos=0, sem TTY) | `trace_wait_e_test_event_diagnostico` — tty vazio, assert "PS-X Realtime Kernel" falha |
| m4 | `i += 2;` → `i += 1;` nos 4 braços de parse (argumento seguinte tratado como flag, erro) | `trace_wait_e_test_event_diagnostico` — binário sai com erro, tty vazio |
| m5 | Bloco de trace do `run` removido (`if false`) | `trace_wait_e_test_event_diagnostico` — count_test_event=0 |
| c1 | Comentário cosmético antes de `fn main()` | verde |
| c2 | Renomeação consistente `max_steps` → `max_steps_parsed` | verde |

## Placar antes → depois

Workspace: 828 → **834** testes (+6: as 3 sondas B-handler/B-table da rodada 1 em
`cdrom_evento_kernel.rs`, o stub e o discriminante em `psx-core/tests/evento_consumo_shell.rs`,
e `trace_wait_e_test_event_diagnostico` em `psx-cli`), 0 falhas.

## Decisões e notas

- **Handoff para 4.4w (próximo item):** A corrida está confirmada. O shell desiste de polling (TestEvent) no step 89 906 602 e os eventos ficam ready no MESMO step (via IRQ2 pós-instrução). A correção pode ser: (a) fazer o shell esperar mais (aumentar o timeout do dispatch loop), (b) adiantar a entrega do evento (reduzir latência entre resposta do CD-ROM e IRQ2), ou (c) detectar que o scheduler precisa andar ANTES de verificar IRQs no mesmo step. A abordagem (c) é a menos invasiva e a mais alinhada com hardware: no PS1 real, a interrupção chega pelo pino físico e o CPU a detecta entre instruções, não no fim do step. Ver `docs/reference/11-interrupts.md` § Interrupt Request / Execution.

- **Invariante 30 aplicada:** O checkpoint de saturação (100 M confirmou que ambos os eventos permanecem ready, sem flip-flop). O primeiro veredito (com checkpoints 5 M) dizia "DEPOIS → corrida"; a adição do checkpoint no step exato do último TestEvent refinou para "NO MESMO STEP → corrida". Moral: número mágico de último evento é dado, não estimativa.

- **Manifesto em LF:** O manifesto foi escrito como LF (Unix) para casar com os meta-testes. Invariante 10.40 documenta o problema de CRLF em manifestos; a solução de longo prazo é o `mutantes.ps1` normalizar os arquivos antes de buscar âncoras.

## Revisão cruzada (orquestrador)

**O veredito "corrida no mesmo step" foi REFUTADO — pela terceira medicao consecutiva desta
sequencia, a mais fina derruba a anterior.**

1. **Amostragem esparsa lida como simultaneidade.** O discriminante so checava os EvCBs nos
   9 checkpoints; "ready no step 89 906 602" so provava "ready em algum ponto de
   (88,0 M .. 89,906602 M]" — janela de 1,9 M passos. Um checkpoint posicionado exatamente
   no step procurado devolve esse step por construcao.
2. **Deteccao continua (por step, janela 85–92 M) datou o flip:** spec `10h` ready no step
   **89 702 216**; spec `200h` no **89 702 837**. Ambos ~204 k passos ANTES do ultimo
   TestEvent (89 906 602) — o shell fez ~17 consultas COM os eventos ja ready e desistiu.
   Nao ha corrida. O teste `evcb_status_checkpoints_discriminante` foi atualizado para a
   deteccao continua e agora imprime o veredito correto ("ANTES → TestEvent devolveu
   errado"). Comando: `cargo test -p psx-core --test evento_consumo_shell --release --
   --nocapture`.
3. **A "correcao" proposta para o 4.4w era um no-op perigoso.** A invariante 31 original e o
   item de ROADMAP afirmavam que o `Cpu::step` verifica IRQ DEPOIS da instrucao; o codigo
   verifica ANTES do fetch (inicio de `Cpu::step` — IRQ do fim do step N vetoriza no inicio
   do N+1, como o pino assincrono do hardware). Alem de o diagnostico estar errado, o
   conserto prescrito ja era o comportamento vigente. Invariante 31 reescrita; ROADMAP e
   STATUS corrigidos para o 4.4w real: rastrear `$a0`/`$v0` do TestEvent.
4. **Fisica do impossivel como sanity check:** `DeliverEvent` e codigo de BIOS que roda
   milhares de instrucoes depois do vetor; "TestEvent le busy e no MESMO step o evento fica
   ready" nunca foi mecanicamente possivel neste emulador. Afirmacao que exige mecanismo
   impossivel e refutavel de graca, antes de qualquer medicao.
5. **Bateria reaplicada por amostragem (invariante 29):** m1 (trace nao insere no HashSet)
   FAILED em 4,5 s; m3 (max-steps zerado) FAILED em 0,34 s. Placar 5/5 do doc confere.
6. O PR desta iteracao foi aberto pelo orquestrador — a rodada 2 parou depois do push, sem
   `gh pr create`.

