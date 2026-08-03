<!-- Custo, tokens e duracao ficam em docs/metricas.csv, medidos pelo runner. -->

# 0176 — timers-gpu-gte-oraculo

- **Data:** 2026-08-03
- **Item do roadmap:** 5.6 (GTE), mais 10.104/10.105 (novos, registrados nesta rodada)
- **Objetivo:** lote E do oráculo de TTY — fechar divergência por divergência em
  `gpu/bandwidth`, `gte/test-all`, `timer-dump` e `timers`.
- **Fonte:** trabalhador (dispatch direto do orquestrador, fora do `oc-iter.ps1`).

**R4 dobrado a pedido do usuário para esta tarefa.** A regra diz uma micro-funcionalidade por
iteração; aqui as quatro suítes do lote fecham numa rodada porque o custo não é o código, é a
espera de suíte e CI — decisão do usuário, registrada aqui para o histórico não mentir.

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| psx-spx | § 16bit Vectors (R/W) (L270-272) | docs/reference/07-gte.md |
| psx-spx | § GTE Data Register Summary (cop2r0-31) (L143) | docs/reference/07-gte.md |
| psx-spx | § Screen XYZ Coordinate FIFOs (L244-261) | docs/reference/07-gte.md |
| psx-spx | § Interpolation Factor (L286-290) | docs/reference/07-gte.md |
| psx-spx | § cop2r28 - IRGB - Color conversion Input (R/W) (L304-315) | docs/reference/07-gte.md |
| psx-spx | § cop2r29 - ORGB - Color conversion Output (R) (L317-329) | docs/reference/07-gte.md |
| psx-spx | § cop2r30-31 - LZCS/LZCR (L331-334) | docs/reference/07-gte.md |
| psx-spx | § Screen Offset and Distance (Input, R/W?) (L227-231) | docs/reference/07-gte.md |
| psx-spx | § COP2 0180001h - RTPS (L481-511) | docs/reference/07-gte.md |
| psx-spx | § cop2r63 - FLAG (L349-370) | docs/reference/07-gte.md |
| psx-spx | § GP1(08h) - Display mode (L887) | docs/reference/03-gpu.md |
| psx-spx | § Memory/Rendering Timings (L1107) | docs/reference/03-gpu.md |

Fonte adicional (fora de `docs/reference/`, R1 permite gabarito como oráculo quando a spec
local é omissa): código-fonte do `gte/test-all` e `timers` em
`github.com/JaCzekanski/ps1-tests` (`gte/test-all/tests.c`, `timers/main.c`) — usado para
entender a mecânica dos testes (poison values, ordem de escrita, medição por busy-loop), não
para inferir comportamento de hardware.

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a medição/spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | saturação-gte | Que `gte/test-all` em 997/999 fosse causado por opcodes faltando (5.4b/c/d). | Os primeiros 50 testes do binário não executam opcode nenhum (`opcode: 0xffffffff`, "No opcode") — só escrevem e leem os 64 registradores. O programa aborta os testes de opcode assim que o **primeiro** teste de registro falha. | Baixei `tests.c` do ps1-tests e li o `runTests()`: `if (tests[i].opcode != sentinela && testFailedCount > 0) break;`. Simulei os 50 testes em Python contra as regras que eu ia implementar antes de tocar Rust — bateu 0/50 mismatches. |
| 2 | saturação-gte | Que `write_data`/`read_data` devessem ser passthrough puro, como já eram. | § 16bit Vectors (L270-272) de docs/reference/07-gte.md: VZ0-2 e IR1-3 sign-extendem no MFC2. | Comparando `input[]` vs `output[]` de dois testes de registro completos, register a register, contra hipóteses (passthrough/zext16/sext16/especial). |
| 3 | saturação-gte | Que a FLAG.22 de IR3 no RTPS usasse a mesma faixa do valor armazenado (como IR1/IR2). | § COP2 0180001h - RTPS (L507-511) de docs/reference/07-gte.md: "the IR3 saturation flag (FLAG.22) is always checked as if lm=0, while the stored IR3 is clamped from MAC3 per the actual lm bit." | O teste 51 do oráculo (primeiro RTPS) divergia em FLAG mesmo com IR3 armazenado correto. Reli a seção de saturação com atenção à nota, que eu tinha pulado na primeira leitura. |
| 4 | teste | Que meu teste `gte_rtps_ir3_flag_lm.rs` com `sf=1` já cobrisse a separação flag/valor. | Com `sf=1` o próprio RTPS desloca 12 bits, então "MAC3 armazenado" e "MAC3 SAR 12" são sempre o mesmo número — o mutante `m7` (reverte a correção) sobreviveu à bateria. | `scripts/mutantes.ps1` reportou `m7 SOBREVIVEU`; adicionei um caso com `sf=0` onde as duas contas realmente divergem. |
| 5 | timing | Que "System Clock" e "Dot clock" nos testes de `timers` fossem afetados pela mesma causa (resolução da GPU não propagada). | Corrigido `update_gpu_timing`, o placar do oráculo `timers` não mudou (143/144 antes e depois) — inclusive "System Clock", que não depende de GPU (razão 1:1). | Medi antes/depois do fix com o binário real; a hipótese não sobreviveu à medição, então parei de forçá-la e registrei como achado separado, não corrigido. |

## As correções

**`gte` — formatos por registrador de dados.** `read_data`/`write_data` ganharam os formatos
documentados: VZ0-2/IR0-3 sign-extendem (16 bits) na leitura; OTZ/SZ0-3 mascaram para U16;
escrever SXYP (r15) empurra a FIFO SXY0←SXY1←SXY2←novo valor, e a leitura de SXYP espelha
SXY2 ao vivo (não um snapshot); escrever IRGB (r28) decompõe 5:5:5 em IR1-3 (×80h por canal);
ler IRGB/ORGB (r28/r29) recompõe de IR1-3 ao vivo, saturando negativo→0 e >1Fh→1Fh sem afetar
FLAG; LZCR (r31) conta bits à esquerda de LZCS (r30) ao vivo; H (cop2r58) ganhou o bug de
sign-extend documentado, que faltava na lista de registradores de controle de 16 bits.

**`gte` — RTPS/RTPT.** FLAG.22 de IR3 agora é decidida sempre pela faixa de lm=0
(`saturate_ir3_rtps`), separada do valor armazenado (faixa do lm real). SX2/SY2/IR0 trocaram
divisão truncada (`/`) por deslocamento aritmético (`>>`) — truncar para zero e arredondar para
baixo divergem para MAC0 negativo, e o gabarito confirma arredondamento para baixo. Ganharam os
flags de overflow que faltavam: MAC0 fora de 32 bits com sinal (FLAG.15/16) e o acumulador
interno de MAC1-3 fora de 44 bits (FLAG.25-30) — este último também se aplica ao MVMVA, que usa
o mesmo formato de acumulador.

**`timers` — resolução da GPU nunca chegava aos timers.** `Timers::update_gpu_timing` existe
desde `timers_dotclock_hblank.rs` mas nenhum caminho de execução real o chamava; `bus.tick_timers`
agora propaga `gpu.cycles_per_pix()`/`video_cycles_per_scanline()` a cada tick. Verificado por
teste dedicado (resolução 320px ⇒ razão 11/56, não o default de 256px ⇒ 11/70). **Efeito
colateral esperado, mas não medido no oráculo `timers`**: ver Placar antes → depois.

## O que não foi corrigido (e por quê)

- **`gpu/bandwidth`**: mede o tempo de operações de desenho da GPU (transferências
  VRAM↔CPU, fills, polígonos texturizados/semitransparentes) em hblanks. A seção
  Memory/Rendering Timings (L1107) de `docs/reference/03-gpu.md` é explicitamente omissa
  ("The exact timing differences are unknown"); reconstruir os ~14 valores a partir só do
  gabarito seria ajuste de curva, não hardware. Registrado como **10.104**.
- **`gte/test-all`, teste 72/1150**: depois de 71 opcodes RTPS corretos, o teste 72
  (`sf=0, lm=0, tx=3, vx=3, mx=0`) diverge em IR0/SXY2/SZ3/MAC0/FLAG — um defeito novo, ainda
  não isolado à causa raiz (possivelmente ligado à combinação sf=0 com overflow do divisor).
  Como o próprio `gte/test-all` aborta no primeiro teste que falha, os ~1078 testes restantes
  (incluindo os opcodes que faltam: DPCS/INTPL/NCDS/CDP/NCDT/NCCS/CC/NCS/NCT/DCPL/DPCT, já
  cobertos por 5.4b/5.4c/5.4d) continuam intocados. Registrado em STATUS.md (Bloqueios).
- **`timer-dump`**: o próprio cabeçalho do `psx.log` diz "This test requires you to have
  modified PSX motherboard with CPU PIN 160 (TCLK0) disconnected from GPU and connected to
  CPU PIN 70 (RTS)". O gabarito foi capturado clocando Timer0 manualmente via um pino
  fisicamente religado — um emulador de hardware **não-modificado** não tem como reproduzir
  essa sequência sem inventar uma ponte SIO→timer que nenhum jogo/BIOS real usa. Sem correção.
- **`timers`, "System Clock" e HBLANK real**: mesmo o clock de sistema (razão 1:1 com o CPU,
  sem depender de GPU) diverge ~13-70x do gabarito, e a correção de `update_gpu_timing` não
  moveu esse número — a causa raiz não foi encontrada. Separadamente, `gpu.set_hblank_active`
  não tem nenhum chamador em `src/` (só em teste): HBLANK nunca é gerado por um evento real do
  `scheduler`, ao contrário do VBLANK (R2). Ambos registrados como **10.105**.

## Bateria de mutação

Placar da bateria: **8/8 mutantes mortos, 2/2 controles verdes, 0 equivalente.**

Na primeira execução `m7` sobreviveu (ver Erros de primeira tentativa #4) e `c1` não compilava
(`i64 <` é lido como início de generic args nessa posição — corrigido com parênteses). Depois
de fechar a lacuna do teste e do manifesto, os 10 registros bateram.

Os registros declaram `teste:` individualmente porque a rodada tem três assassinos
(`gte_registros_dados`, `gte_rtps_ir3_flag_lm`, `timers_gpu_timing_wiring`) — contorna o item
10.71.

## Placar antes → depois

Workspace: **953 → 969** testes.

**Oráculo de TTY (`K/M` = K linhas divergentes de M):**

| Suíte | antes | depois | nota |
|---|---|---|---|
| `gpu/bandwidth` | 15/17 | 15/17 | sem correção (spec omissa, 10.104) |
| `gte/test-all` | 1048/1050 | **17/19** | causa raiz corrigida; segue no teste 72/1150 |
| `timer-dump` | 205/317 | 205/317 | sem correção (hardware modificado) |
| `timers` | 143/144 | 143/144 | fix real aplicado, não moveu esta suíte (10.105) |

Detalhe do `gte/test-all` que o K/M de linhas não mostra: dos **1150** testes internos do
binário, os **50** testes de registro (que antes nunca rodavam de verdade — o programa abortava
no primeiro) agora passam **50/50**, e o primeiro teste de opcode (RTPS) avança até o teste
**72** antes de esbarrar num defeito novo — eram **~2** testes "passando" por coincidência antes
desta rodada.

## Revisão cruzada (orquestrador)

Pendente — PR aberto para revisão do orquestrador.

## Decisões e notas

- **R4 dobrado por decisão do usuário**: as quatro suítes do lote fecham numa única rodada.
  O motivo declarado é que o custo desta tarefa é dominado pela espera de suíte/CI, não pelo
  volume de código — mantendo a narrativa de erro/decisão no lugar de forçar quatro PRs
  artificialmente pequenos.
- Três manifestos de mutação antigos (`0084-gte-registers`, `0086-gte-rtps-rtpt`,
  `0089-gte-mvmva`) tiveram âncoras envelhecidas por esta reescrita de `gte.rs` e foram
  marcados `arquivada:` — o código que testavam mudou de forma real (formatos por registrador,
  overflow de 43 bits), não regrediu.
- Quatro testes pré-existentes em `gte_registers.rs` usavam os registradores 3/5 (VZ1/VZ2,
  agora sign-extend) e 31 (LZCR, agora computado) como registradores "genéricos" para
  round-trip de MTC2/CFC2/LWC2/SWC2. Foram trocados para r0 (VXY0, passthrough real) ou
  reescritos para validar o formato correto — não é regressão, é a suposição de "registrador
  qualquer serve" que deixou de valer.
- `docs/mapa.md` não precisou de atualização: nenhum arquivo-fonte passou de 800 linhas
  (`gte.rs` foi de ~528 para ~600 linhas, coeso, sem necessidade de fatiar).
