<!-- Custo, tokens e duracao ficam em docs/metricas.csv, medidos pelo runner. -->

# 0168 — oraculo-tty

- **Data:** 2026-08-02
- **Item do roadmap:** 10.23 (parte 1 de 2 — TTY; VRAM fica para `task-10.23-vram.txt`)
- **Objetivo:** dar veredito automático (`identico`/`difere`/`sem-saida`/`sem-bios`) às 21
  suítes de `tests/exes/ps1-tests/**/` que já têm `psx.log` — TTY capturado em hardware real
  pelo projeto ps1-tests — ao lado do EXE, e nunca foram comparadas contra o psx-cli.

## Spec consultada

Nenhuma. Tarefa explícita (`logs/orquestrador/task-10.23-tty.txt`) instrui não ler specs de
hardware: não há hardware novo aqui, é ferramenta de medição.

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que aconteceu | Como foi pego |
|---|---|---|---|---|
| 1 | API-PowerShell | Que `[AllowNull()][string]$Gabarito` bastava para o parâmetro aceitar `$null` e distinguir "gabarito ausente" de "gabarito vazio". | O PowerShell converte `$null` em `""` ao vincular um argumento a um parâmetro tipado `[string]`, mesmo com `AllowNull()` — a conversão de tipo roda antes da validação. `Get-TtyVeredito -Real "qualquer coisa" -Gabarito $null` caiu no ramo de diff em vez do ramo `sem-gabarito`. | Teste sintético `gabarito_ausente_nao_e_confundido_com_diferenca` (que dependia exatamente dessa distinção) falhou com `difere` em vez de `sem-gabarito`. Removido o tipo `[string]` do parâmetro `$Gabarito` (fica sem tipo/`[object]` implícito) — `$null` passa a chegar como `$null` de verdade. Registrado também em `docs/orquestracao.md`. |

## Bateria de mutação

Bateria de mutação: não se aplica — nenhuma linha de `crates/*/src/` foi tocada nesta rodada;
o trabalho é script (`scripts/oraculo-tty.ps1`, `scripts/lib/tty-veredito.ps1`) + teste
(`crates/psx-core/tests/oraculo_tty.rs`), conforme o próprio handoff da tarefa 10.23 previu.

## Placar antes → depois

Workspace: **921 → 930** testes (9 novos em `oraculo_tty.rs`: 5 casos sintéticos de
classificação + 4 checagens estruturais do arreio).

`cargo fmt --all` → limpo. `cargo fmt --all -- --check` → limpo. `cargo clippy --all-targets
-- -D warnings` → limpo. `cargo test --all --no-fail-fast` → verde (após atualizar o placar do
STATUS.md, cobrado por `status_handoff.rs`).

### Placar do `scripts/oraculo-tty.ps1` — as 21 suítes com gabarito, medidas pela primeira vez

Execução real: `pwsh scripts/oraculo-tty.ps1 -MaxSteps 800000000 -TimeoutSec 180`, BIOS
`SCPH1001.BIN` presente, commit `ee5fc83`. Nenhuma suíte estourou o timeout de 180 s (cada uma
levou ~35-40 s para os 800 M passos, ~13 min no total). Resultado gravado em
`logs/oraculo-tty.csv` (gitignored).

**Resumo: 0 `identico`, 21 `difere`, 0 `sem-saida`, 0 `sem-bios`, de 21 suítes.**

| Suíte | Status | K/M linhas |
|---|---|---|
| `timers` | difere | 128/129 |
| `timer-dump` | difere | 316/316 |
| `gte/test-all` | difere | 5/5 |
| `gpu/bandwidth` | difere | 16/16 |
| `gpu/gp0-e1` | difere | 12/12 |
| `gpu/mask-bit` | difere | 7/7 |
| `mdec/4bit` | difere | 19/19 |
| `mdec/8bit` | difere | 19/19 |
| `mdec/step-by-step-log` | difere | 1665/1665 |
| `cdrom/getloc` | difere | 45/45 |
| `cdrom/disc-swap` | difere | 11/11 |
| `cdrom/timing` | difere | 19/19 |
| `cpu/access-time` | difere | 23/23 |
| `cpu/io-access-bitwidth` | difere | 67/67 |
| `cpu/code-in-io` | difere | 10/10 |
| `cpu/cop` | difere | 19/19 |
| `dma/chain-looping` | difere | 11/11 |
| `dma/chopping` | difere | 132/132 |
| `dma/dpcr` | difere | 15/15 |
| `dma/otc-test` | difere | 15/15 |
| `spu/memory-transfer` | difere | 11/11 |

Nenhuma das quatro candidatas apontadas no handoff como prováveis `identico` (`cpu/cop`,
`dma/dpcr`, `gpu/gp0-e1`, `mdec/4bit`) bateu — sinal para desconfiar do ambiente de boot antes
do emulador (ver "Decisões e notas").

## Revisão cruzada (orquestrador)

<!-- Preenchido pelo Claude na revisão do PR. -->

## Decisões e notas

**O achado real desta rodada não é "0/21 identico" isolado — é que as 21 são o MESMO
travamento.** Inspecionei o TTY capturado (não só o K/M) para confirmar que "difere" não
escondia 21 causas diferentes: `cdrom/disc-swap`, `cpu/cop`, `dma/dpcr`, `gpu/gp0-e1`,
`gte/test-all`, `dma/chopping`, `mdec/step-by-step-log`, `timer-dump` e `spu/memory-transfer`
(amostrados manualmente) emitem **exatamente os mesmos 56 bytes** — `ResetGraph:itb=<endereço
variável>,ehk=<endereço variável>` seguido de `ResetGraph:SR=1001` — e então o TTY para de
crescer pelo resto dos 800 M passos. `ResetGraph()` é rotina comum do PSn00bSDK chamada por
todas essas suítes antes do corpo do teste.

Motivo provável (não investigado a fundo — fora de escopo desta rodada de medição): o caminho
`--exe` (sideload direto, sem BIOS real) chama `psx_core::psexe::install_return_stubs`
(`crates/psx-cli/src/main.rs:336`, implementação em `crates/psx-core/src/psexe.rs:70`), que
grava `jr $ra` puro nas tabelas de syscall A0h/B0h/C0h. Qualquer chamada de BIOS que
`ResetGraph()` faça para instalar/armar sua interrupção "retorna com sucesso" sem fazer nada —
e a rotina fica esperando por uma interrupção que o stub nunca entrega. Isso explicaria por que
**nenhuma** das quatro candidatas a `identico` do handoff bateu: a causa não é uma correção de
hardware suíte a suíte, é uma lacuna única no ambiente de boot do sideload. Registrado como
**novo item 10.95**; não investiguei mais fundo nem tentei corrigir (fora de escopo — medir
primeiro).

**Isto é um caminho de código diferente do boot via CD investigado em 0158-0167.** O Rayman
(e qualquer disco) entra pela BIOS real (`(Some(bios_path), None, disc_path)` em `main.rs`),
que não passa por `install_return_stubs`. Os achados desta rodada sobre A0/B0/C0 stubados NÃO
se aplicam ao caminho do Rayman — são bugs de ambientes diferentes dentro do mesmo `psx-cli`.

**A anomalia `timers` 128/129 (não 129/129) não é progresso real.** O TTY do `timers` também
para em `ResetGraph:SR=1001`; a 1 linha que "bate" contra o gabarito é uma linha vazia inicial
coincidente nos dois lados (artefato de como `take_tty()`/o gabarito começam), não conteúdo do
teste em si.

**O que não foi feito e por quê:** (1) a parte de VRAM do item 10.23 (13 `vram.png`,
`diffvram`) fica para a próxima iteração (`task-10.23-vram.txt`, branch
`iter/0169-oraculo-vram`) — fora de escopo desta rodada por definição da própria tarefa. (2)
Nenhuma tentativa de corrigir o travamento em `ResetGraph` — a tarefa é explícita: medir agora,
corrigir depois, um item por vez (R4). (3) Não toquei `scripts/scoreboard.ps1`: o oráculo de
TTY é um script novo (`scripts/oraculo-tty.ps1` + `scripts/lib/tty-veredito.ps1`) para não
arriscar as 9 asserções de `ci_scoreboard.rs`/`gpu_scoreboard.rs` que já travam o formato do
scoreboard existente.
