<!-- Custo, tokens e duração NÃO entram aqui: são medidos por scripts/oc-iter.ps1 em
     docs/metricas.csv, casados por head_antes/head_depois. -->

# 0173 — dma-oraculo

- **Data:** 2026-08-03
- **Item do roadmap:** 10.101 e 10.102 (novos); lote B do oráculo de TTY (`logs/orquestrador/lote-B.txt`)
- **Objetivo:** fechar o lote B (DMA) do oráculo de TTY — `dma/dpcr`, `dma/otc-test`,
  `dma/chain-looping`, `dma/chopping` — corrigindo o que for defeito real e registrando o que
  não for.
- **Fonte:** orquestrador (dispatch de lote); execução por Claude Sonnet 5.

**R4 dobrado a pedido do usuário.** A regra diz uma micro-funcionalidade por iteração; esta
rodada fecha as 4 suítes do lote B numa tacada só porque o custo não é o código, é a espera de
suíte e CI — decisão do usuário, registrada aqui para o histórico não mentir.

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| psx-spx | D#\_MADR (L48) | docs/reference/04-dma.md |
| psx-spx | D#\_BCR (L64) | docs/reference/04-dma.md |
| psx-spx | D#\_CHCR (L94) | docs/reference/04-dma.md |
| psx-spx | Linked List DMA (L211) | docs/reference/04-dma.md |

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a medição/spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | spec | Que `dma/dpcr` divergia por um bug de gate no próprio DPCR (prioridade/enable), como a pista do lote sugeria. | O gabarito espera `pass - writeToSPURAM` via DMA4; `try_execute_dma2/3/6` existem mas **canal 4 nunca é chamado** em `bus.rs`, e `spu.rs` é um stub de 1 linha (M7). Não é bug de DPCR: é peça inteira ausente. | Rodada isolada travou em 800M passos sem avançar uma linha após o cabeçalho; grep em `bus.rs` por `try_execute_dma` mostrou só os canais 2/3/6. |
| 2 | medição | Que `dma/otc-test` (15/17) fosse defeito nosso — 2 linhas de "pass" a mais que o gabarito pareciam sobra de teste inventado. | Removendo exatamente `testOtcBigTransfer` e `testOtcControlBitsAfterTransferWithChopping` da nossa saída, o restante bate **byte a byte** com o `psx.log`. O `.exe` local tem 2 subtestes que a captura de hardware gravada não tem — artefato de versão do binário do ps1-tests, não defeito de emulação. | Reimplementação manual do algoritmo de alinhamento de `Get-TtyVeredito` em Python sobre as duas saídas, linha a linha. |
| 3 | medição | Que o `K/M` de `dma/chain-looping` do lote (9/11) fosse o valor a bater. | Duas execuções isoladas e determinísticas (`psx-cli` direto, sem `oraculo-tty.ps1`) deram **4/11**, não 9/11, no mesmo commit em que o placar 9/11 foi gravado (`c7c0851`, sem mudança em `dma.rs` até aqui). | `git diff c7c0851 HEAD -- crates/psx-core/src/dma.rs` vazio; a única explicação plausível é a mesma corrida do `Start-Process` já documentada duas vezes no `STATUS.md` (0170: 16/21 sob disputa vs. 21/21 limpo). Reportado o valor medido, não o do CSV. |
| 4 | API-Rust | Que adicionar `execute_burst` fosse local e sem efeito colateral em outro arquivo. | `execute_burst` duplicou trechos de texto já usados como âncora por `docs/mutantes/0056-dma-otc.mut`, `0057-dma-gpu.mut` e `0117-dma-gpu-vram-para-ram.mut` — 3 registros passaram a casar 0 ou 2 vezes em vez de 1. | `cargo test --test mutation_anchors` (rodado antes de tocar em qualquer manifesto, como manda o passo 6.2) apontou as 5 âncoras quebradas com o commit atual. |

## As correções

**`dma/chain-looping` — cadeia sem end-marker deve ficar ocupada, não completar.**
`execute_linked_list` tinha um teto de segurança de 4096 nós (para o host nunca travar de
verdade), mas ao estourar esse teto sinalizava conclusão do mesmo jeito que um end-marker real.
A spec (`docs/reference/04-dma.md`, § Linked List DMA L211) só descreve o caminho feliz — "a
transferência para quando um end-marker é alcançado" — e é omissa sobre o que acontece quando
ele **nunca** é alcançado; o
gabarito (`psx.log`) foi o oráculo usado: `finished = false, irq = false` para cadeia
auto-referente e para ciclo de dois nós. A correção introduz `alcancou_fim: bool`, só `true`
quando o laço termina por end-marker real (ou por RAM fora dos limites, caminho inalterado); o
teto de segurança agora deixa o canal **ocupado para sempre** em vez de completar.

**`dma/chopping` — SyncMode=0 (Burst) do canal 2 não existia.** `try_execute_dma2` só tratava
`sync_mode` 1 (Block) e 2 (Linked List); o `match` caía no `_ => {}` para SyncMode=0, então
bit24 nunca era limpo e a suíte travava para sempre no primeiro caso `SyncMode: 0` — nunca
chegava a rodar as 131 linhas seguintes. `execute_burst` trata o `D2_BCR` como contagem simples
de palavras, conforme `docs/reference/04-dma.md` § D#\_BCR L64 ("BC Number of words"), e **não
atualiza o MADR**, conforme `docs/reference/04-dma.md` § D#\_MADR L48 ("In SyncMode=0, the
hardware doesn't update the MADR registers... unless Chopping is enabled") — não implementei o
caminho com chopping habilitado, então mantive o comportamento default (sem chopping) descrito
explicitamente na spec.

**`dma/dpcr` e `dma/chopping` (números) — não corrigidos, registrados.** Ver ROADMAP 10.101 e
10.102 abaixo.

## Bateria de mutação

Placar da bateria: **5/5 mutantes mortos, 2/2 controles verdes, 0 equivalente** —
`docs/mutantes/0173-dma-oraculo.mut`.

| Mutante | Alvo | Teste assassino |
|---|---|---|
| m1 | `alcancou_fim` inicializado `true` | `dma2_linked_list_auto_referente_nunca_completa`, `dma2_linked_list_ciclo_de_dois_nos_nunca_completa` |
| m2 | guarda `if alcancou_fim` vira `if true` | idem |
| m3 | `0 => self.execute_burst(...)` vira `0 => {}` (bug original) | `dma2_burst_sync_mode0_completa_em_vez_de_travar` |
| m4 | contagem do burst ignora o BCR (sempre 0) | idem |
| m5 | direção do burst invertida | idem |
| c1 | anotação de tipo cosmética em `alcancou_fim` | sobreviveu (esperado) |
| c2 | bloco cosmético em torno da chamada do burst | sobreviveu (esperado) |

Os registros declaram `teste:` individualmente (dois assassinos: `dma_chain_looping` e
`dma_burst_sync0`), contornando o item 10.71.

**Reparo de âncora envelhecida (não é bug desta rodada, é manutenção do formato).**
`execute_burst` duplicou texto já ancorado por três manifestos antigos (ver erro #4 acima).
Corrigi as âncoras de `docs/mutantes/0057-dma-gpu.mut` (m2 e m4, cuja região de código mudou de
forma — não dava para evitar por reformulação textual) e reexecutei a bateria: **6/6 mutantes
mortos, 2/2 controles verdes, 1 equivalente**, mesmo placar do doc original de 0057.
`0056-dma-otc.mut` e `0117-dma-gpu-vram-para-ram.mut` foram resolvidos sem tocar no manifesto:
bastou tornar o texto de `execute_burst` levemente diferente (`bc` em vez de `bcr`,
parênteses explícitos) do texto já ancorado alhures, então não precisaram de reexecução.

## Placar antes → depois

Workspace: **953 → 956** testes.

**Oráculo do lote B (`K/M` = K linhas divergentes de M; medição isolada por suíte, não
`oraculo-tty.ps1` — proibido pelo lote sob disputa de CPU):**

| Suíte | Antes | Depois | Classificação |
|---|---|---|---|
| `dma/dpcr` | 13/15 | 13/15 (sem mudança) | bloqueado — SPU/DMA4 ausentes (10.101) |
| `dma/otc-test` | 15/17 | 15/17 (sem mudança) | artefato de versão do `.exe`, não defeito |
| `dma/chain-looping` | 4/11* | 4/11* | defeito corrigido; `finished`/`irq` batem, ticks não (10.102) |
| `dma/chopping` | 131/132 | **130/132** | defeito corrigido (deixou de travar); ciclos não batem (10.102) |

\* O lote informava 9/11; medição direta e determinística (duas execuções idênticas) deu 4/11 no
mesmo commit — ver erro de primeira tentativa #3.

Nenhuma suíte do lote piorou. `dma/chain-looping` não muda de K porque a linha inteira
(`Work took N ticks[, finished=X, irq=Y]`) só conta como igual se **todos** os campos baterem;
o campo `finished`/`irq` passou a bater exatamente, mas `N` (ticks) continua errado por uma
causa alheia ao DMA (ver 10.102) — confirmado porque até o caso "sem DMA nenhum" do mesmo
gabarito diverge ~39x (628352 vs. 16040).

## Revisão cruzada (orquestrador)

Pendente — PR aberto para revisão do orquestrador.

## Decisões e notas

- **`dma/dpcr` não foi forçado.** O bloqueio real é a ausência total de SPU (M7: `spu.rs` tem 1
  linha) e do roteamento de DMA4 em `bus.rs`. Implementar um stub de SPU só para destravar este
  teste seria trabalho de outro lote (C: MDEC+SPU) rodando em paralelo agora — o risco de
  conflito de merge supera o ganho de uma suíte. Registrado como ROADMAP 10.101.
- **O custo de ciclo do chopping não foi modelado.** `dma/chopping` mede "CPU cycles"/"ticks"
  ao redor de transferências DMA via um timer de hardware; como toda transferência DMA neste
  projeto é síncrona e instantânea dentro da própria escrita do CHCR (não avança por
  timestamps do `scheduler`, ao contrário do que R2 pede para os demais componentes), qualquer
  medição baseada em timer ao redor de um DMA lê o overhead fixo do laço de poll do teste, não
  a duração real. Isso explica tanto o número fixo (65570) de `dma/chopping` quanto o de
  `dma/chain-looping` (628352) — é a mesma causa estrutural nas duas suítes, não dois bugs
  separados. Modelar isso corretamente é obra de scheduler, não de `dma.rs` isolado; registrado
  como ROADMAP 10.102 em vez de forçado nesta rodada.
- **`0057-dma-gpu.mut` foi reexecutado, não apenas editado.** O passo 6.2 pede rodar a
  validação de forma antes de commitar; como duas âncoras antigas mudaram de forma (não só de
  posição), reexecutei a bateria inteira do 0057 para confirmar que os mutantes ainda morrem
  com o texto novo, em vez de confiar que a edição preservava o comportamento.
