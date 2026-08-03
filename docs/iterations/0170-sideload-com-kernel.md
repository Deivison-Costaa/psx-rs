<!-- Custo, tokens e duração NÃO entram aqui: são medidos por scripts/oc-iter.ps1 em
     docs/metricas.csv, casados por head_antes/head_depois. -->

# 0170 — sideload-com-kernel

- **Data:** 2026-08-03
- **Item do roadmap:** 10.95
- **Objetivo:** `--bios` + `--exe` deixa a BIOS bootar de verdade até o kernel montar
  (A0h/B0h/C0h reais) e só então sobrepõe o PS-EXE, em vez de stubar os vetores de syscall.

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| psx-spx | § Cheat Devices (L3634) | docs/reference/13-kernel-bios.md |
| psx-spx | § BIOS RAM Map (L416) | docs/reference/13-kernel-bios.md |
| psx-spx | § Table of Tables (L442) | docs/reference/13-kernel-bios.md |

A citação-chave (§ Cheat Devices) descreve o Action Replay real: "uses the Pre-Boot handler to
set a COP0 hardware breakpoint at 80030000h and does then resume normal BIOS booting (which will
then initialize important things like A0h/B0h/C0h tables, and will then break when starting the
GUI code at 80030000h)". É exatamente o roteiro desta rodada. O RAM Map e a Table of Tables deram
os endereços usados para confirmar por VALOR que o kernel está montado (não só "não é zero").

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a medição diz | Como foi pego |
|---|---|---|---|---|
| 1 | ferramenta (mutantes) | Que um `@@DE`/`@@PARA` multi-linha pudesse cruzar uma linha em branco do fonte, como qualquer outro trecho de texto. | O parser do manifesto (`support/mutation_format.rs`) pula linha vazia incondicionalmente, mesmo dentro de `@@DE`/`@@PARA` — a linha em branco nunca entra no texto reconstruído, então a âncora não bate contra o arquivo real (que tem a linha em branco). | `mutation_anchors` reprovou m4 e m5 com "encontrada 0 vez(es)". Corrigido removendo a linha em branco entre os dois blocos `if let Err` em `main.rs` (não muda comportamento, `cargo fmt --check` continua verde) e recortando os `@@DE`/`@@PARA` para não precisarem dela. |
| 2 | ambiente/timing | Que a primeira rodada do `oraculo-tty.ps1` (16 de 21 suítes `sem-saída`, TTY vazio) fosse uma fronteira nova e real, atingida logo depois do `ResetGraph`. | Não é assunto de spec — é artefato de medição. | Essa primeira rodada foi disparada em paralelo com a bateria de mutação (`cargo test -p psx-cli --release` repetido 7x), disputando CPU. Uma segunda rodada limpa, sem nada mais rodando, deu **21/21 `difere` e 0 `sem-saída`** — todas as suítes que antes ficavam vazias agora produzem TTY maior que a rodada suja tinha capturado (ex.: `cpu/cop` foi de vazio para 52 linhas). `Get-Content` do stdout redirecionado por `Start-Process` sob disputa de CPU pode ler antes do flush terminar; a rodada "sem-saída" media o script, não o emulador. Corrigido rodando o oráculo sozinho, sem `cargo test` concorrente. |

## Bateria de mutação

Placar da bateria: **5/5 mutantes mortos, 2/2 controles verdes, 0 equivalente** —
`docs/mutantes/0170-sideload-com-kernel.resultado`.

Bateria MANUAL (invariante 29 — alvo em `crates/psx-cli/src/main.rs`, `mutantes.ps1` só
recompila `psx-core`), assassino `cargo test -p psx-cli --test exe_kernel_montado --release`
(aplicado → rodado → revertido, um a um; `scripts/mutantes.ps1 -Iter 0170` rodado e confirmado
que ele reconhece e pula o alvo fora do psx-core, como esperado):

| id | mutação | resultado |
|---|---|---|
| m1 | `KERNEL_ENTRYPOINT_PC` trocado (`0x80030000` → `0x80010000`) | MORREU (1.2s — estoura o teto de passos) |
| m2 | `BIOS_BOOT_TO_KERNEL_MAX_STEPS` pequeno demais (`20_000_000` → `2_000_000`) | MORREU (0.2s) |
| m3 | condição do laço invertida (`!=` → `==`) — boot nunca dá um passo | MORREU (0.1s — TTY vazio) |
| m4 | `install_return_stubs` volta a ser chamada depois do `load_psexe` | MORREU (0.2s — A0h = `jr $ra`) |
| m5 | ordem trocada — `load_psexe` roda antes do boot | MORREU (0.9s — CPU gira no próprio laço do EXE) |
| c1 | teto de passos do boot generoso (`20_000_000` → `40_000_000`) | sobreviveu (0.2s) |
| c2 | texto da mensagem de erro alterado, sem mudar comportamento | sobreviveu (0.2s) |

## Placar antes → depois

Workspace: **938 → 939** testes. `cargo fmt --check`, `cargo clippy -D warnings` e
`cargo test --all --no-fail-fast` verdes (1 falha esperada e já corrigida: o placar do
`status_handoff` acusando 938≠939 antes deste doc atualizar o `STATUS.md`).

Amidog CPU (`tests/exes/amidog/cpu/psxtest_cpu.exe`, 800 M passos): `Result: 00000101` **antes
e depois** — sem regressão, como exigido.

`tests/exes/amidog/gte/psxtest_gte.exe` (800 M passos): **antes e depois**, para em `Running
tests` — inalterado por esta rodada (não é o alvo do item 10.95, e não regrediu).

Oráculo TTY (`scripts/oraculo-tty.ps1 -MaxSteps 800000000 -TimeoutSec 180`), 21 suítes do
ps1-tests com gabarito — tabela da rodada limpa (sem `cargo test` concorrente; ver erro de
primeira tentativa #2 sobre a rodada suja descartada):

| suíte | antes (0168) | depois (0170) |
|---|---|---|
| cdrom/disc-swap | difere 11/11 | difere 24/24 |
| cdrom/getloc | difere 45/45 | difere 54/54 |
| cdrom/timing | difere 19/19 | difere 29/29 |
| cpu/access-time | difere 23/23 | difere 41/41 |
| cpu/code-in-io | difere 10/10 | difere 24/24 |
| cpu/cop | difere 19/19 | difere 52/52 |
| cpu/io-access-bitwidth | difere 67/67 | difere 85/85 |
| dma/chain-looping | difere 11/11 | difere 34/34 |
| dma/chopping | difere 132/132 | difere 147/147 |
| dma/dpcr | difere 15/15 | difere 18/18 |
| dma/otc-test | difere 15/15 | difere 48/48 |
| gpu/bandwidth | difere 16/16 | difere 46/46 |
| gpu/gp0-e1 | difere 12/12 | difere 38/38 |
| gpu/mask-bit | difere 7/7 | difere 28/28 |
| gte/test-all | difere 5/5 | difere 1935/1935 |
| mdec/4bit | difere 19/19 | difere 44/44 |
| mdec/8bit | difere 19/19 | difere 44/44 |
| mdec/step-by-step-log | difere 1665/1665 | difere **3178/3180** |
| spu/memory-transfer | difere 11/11 | difere 18/18 |
| timer-dump | difere 316/316 | difere 580/580 |
| timers | difere 128/129 | difere 172/172 |

Resumo: **0 → 0 idêntico** (esperado — o item não promete paridade de hardware), **21 → 21
difere**, **0 → 0 sem-saída/timeout**. "Antes" era 21 suítes travadas byte a byte idênticas em
56 bytes de `ResetGraph:SR=1001` (a assinatura exata do defeito de 10.95). "Depois" as 21
produzem TTY bem maior (K e M sobem em todas, sinal de execução real acontecendo depois do
`ResetGraph`) e nenhuma repete a assinatura antiga. `mdec/step-by-step-log` marca 3178/3180 —
**3178 linhas divergem**, não duas (ver a revisão cruzada: eu li o K/M ao contrário). As suítes continuam `difere` porque diferem de hardware real na GPU/MDEC/timers etc. —
material das próximas iterações, exatamente como o item previa.

## Revisão cruzada (orquestrador)

Rodada de trabalhador (`claude-sonnet-5`), revisada antes do merge. **Aprovada na correção,
corrigida na leitura.**

**O que está certo, e é grande.** As 21 suítes saíram da assinatura de 56 bytes do `ResetGraph`
e passaram a executar. O total de linhas de TTY comparadas vai de **2.566 para 6.641**, e a
`gte/test-all` salta de 5 para 1.935 linhas — ela roda a suíte inteira agora. O Amidog CPU
continua em `Result: 00000101`, sem regressão. A verificação do kernel é por VALOR
(`[0xA0]=0x3C080000`, `[0x100]=0xA000E004`, `[0x200]=0x00002958`), não por "não é zero", que era
o que a tarefa pedia.

**Erro de leitura, corrigido acima.** O doc afirmava que `mdec/step-by-step-log` em `3178/3180`
significava "só 2 linhas divergem". É o oposto: `Get-TtyVeredito` devolve `"$diferentes/$total"`,
então 3178 linhas divergem. Era a suíte mais distante do gabarito, apresentada como a mais
próxima. Esse é o tipo de falso progresso que já custou iterações a este projeto, e o número
estava a uma linha de código de distância.

**Por que K é quase igual a M em todas — e não é culpa do emulador.** Medi `cpu/cop` na mão:
das 56 linhas do nosso TTY, **23 pares são linhas adjacentes idênticas**. Removendo a duplicação
e o prefixo `% ` do gabarito, sobram **18 linhas de cada lado e 7 divergências reais**, todas de
exceção de coprocessador:

| teste | hardware | nós |
|---|---|---|
| `testCop0InvalidOpcode` | não lança | **lança** |
| `testSwc0Enabled` | não lança | **lança** |
| `testCop1Enabled` | não lança | **lança** |
| `testCop2Disabled` | **lança** | não lança |
| `testSwc2Disabled` | **lança** | não lança |
| `testCop3Enabled` | não lança | **lança** |
| `testSwc3Enabled` | não lança | **lança** |

Ou seja: o `difere 52/52` do CSV é 7 defeitos de verdade escondidos atrás de dois artefatos de
apresentação. Abri três itens: **10.97** (TTY duplicado — a interceptação de `A0h/3Fh` em
`do_printf` escreve o texto e a rotina real da BIOS escreve de novo, agora que o kernel existe),
**10.98** (o arreio precisa alinhar) e **10.99** (as 7 divergências de coprocessador).

**Conferido também:** o erro de primeira tentativa nº 2 está correto e bem diagnosticado — as 16
`sem-saída` da primeira rodada eram disputa de CPU com a bateria de mutação, e a rodada limpa que
eu mesmo acompanhei terminou `0 sem-saída`. A bateria manual está justificada (invariante 29, o
alvo é `psx-cli` e o `mutantes.ps1` só recompila `psx-core`) e nenhum mutante morreu por
compilação.

## Decisões e notas

`boot_bios_to_kernel()` (`crates/psx-cli/src/main.rs`) roda `cpu.step(bus)` num laço privado até
`cpu.pc == 0x8003_0000`, com teto de 20.000.000 de passos (a marca real medida foi 2.695.618 —
~7,4x de folga) e erro claro em vez de loop infinito se o kernel nunca montar. Confirmado por
leitura de memória com um PS-EXE sintético carregado em `0x80010000` (fora da área de kernel,
que vai até `0x00010000` — § BIOS RAM Map): `[0xA0]=[0xB0]=[0xC0]=0x3C080000` (dispatcher real
`lui $t0,hi`, não `jr $ra`), `[0x100]=0xA000E004` e `[0x104]=0x00000020` (ExCB da Table of
Tables) e `[0x200]=0x00002958` (primeira entrada da A-jump-table) — todos zero ou `jr $ra` sob o
código antigo. `install_return_stubs` continua existindo em `psx-core/src/psexe.rs` (ainda usada
diretamente pelos testes unitários de `psexe.rs`), só deixou de ser chamada neste caminho.

Nenhum teste de `crates/psx-cli/tests/` que já usava `--exe` quebrou: os que chamam
`load_psexe`/`install_return_stubs` diretamente (em `cli_runner.rs`) não passam pelo `main.rs` e
ficam intactos; os que sobem o binário de verdade com EXEs sintéticos minúsculos em `0x80000000`
(`deliverevent_diagnostico.rs`, `testevent_descritor.rs`, `shell_vram_tela.rs`,
`espera_tela_sce.rs`) continuam passando porque o boot ao kernel usa um orçamento de passos
**separado** do `--max-steps` do usuário — o contador que os testes verificam só começa a contar
depois do `load_psexe`, exatamente como antes.
