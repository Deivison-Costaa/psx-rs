# 0027 — sideload de PS-EXE

- **Data:** 2026-07-28
- **Item do roadmap:** 1.11
- **Objetivo:** sideload de PS-EXE no psx-cli com suporte a TTY, BSS, halt via step-limit.

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| psx-spx | PSX.EXE header layout (L1162-1184) | docs/reference/16-cdrom-file-formats.md |
| psx-spx | A(43h) Exec (L1054-1063) | docs/reference/13-kernel-bios.md |
| psx-spx | Executable Memory Allocation (L1150-1157) | docs/reference/13-kernel-bios.md |

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | endereçamento | bss_addr=0x8000_0020 estava dentro do range do body (0x8000_0000..0x8000_07FF), então o zerofill do corpo carregava zeros sobre a BSS e a mutação M1 (skip BSS zero-fill) sobrevivia | BSS pode estar em qualquer endereço; body load e BSS zero-fill são operações independentes | M1 sobreviveu na primeira tentativa; bss_addr corrigido para 0x8000_1000 (fora do range do body) e M1 passou a ser pego por A3 |
| 2 | API-Rust | `build_ps_exe` com 8 argumentos disparava clippy `too-many-arguments` | — | clippy -D warnings reprovou; refatorado para `PsexeConfig` struct com 7 campos |
| 3 | borrow-checker | `if let (Some(bios_path), Some(exe_path)) = (bios_arg, exe_arg)` movia bios_arg, impedindo o segundo `if let Some(path) = bios_arg` no fluxo de bios-only | — | erro de compilação; trocado para `bios_arg.is_some() && exe_arg.is_some()` com `.take()` |
| 4 | endereçamento | `JMP $` no teste A2 saltava para `code_addr` (0x8000_0000) em vez do próprio endereço (0x8000_0020), fazendo o código re-executar em loop e produzir `OKOKOK...` no TTY | — | A2 falhou com TTY = 14 bytes; corrigido para `encode_j(0x02, code_addr + 8*4)` |
| 5 | arquivo | A4 procurava `psxtest_cpu.psexe` (extensão errada). O arquivo real é `psxtest_cpu.exe`, baixado por `scripts/fetch-test-exes.ps1` desde 27/07. Teste retornava cedo porque `exe_path.exists()` dava `false` — verde permanente, zero medições. | — | Revisão adversarial (F1): nome corrigido para `.exe`. **Mas a correção da 2ª rodada trocou o nome certo por um diretório errado**: `exe_dir()` usava `CARGO_MANIFEST_DIR + ..` = `<repo>/crates`, faltava um nível (`..` extra para `<repo>/`). A4 continuou retornando cedo. Pegueiro na revisão da 3ª rodada (G1): corrigido com dois `..`. |
| 6 | bios-stubs | O EXE real chama a BIOS via `jal` mas não havia `jr $ra` em A0h/B0h/C0h — a RAM estava zerada nesses endereços. O teste A2 escrevia `jr $ra` à mão dentro do `.psexe` sintético, mascarando a ausência dos stubs no código de produção. | A(00A0h)/B(00B0h)/C(00C0h) são chamadas via `jal` com número de função em R9 (`13-kernel-bios.md` L496-498, L685, L788). O código chamador espera retornar via `jr $ra`. | Revisão adversarial (F2): corrigido com `install_return_stubs()` em `psx-core::psexe`, chamado pelo runner. A2 parou de escrever `jr $ra` à mão. |
| 7 | panic | **`unwrap()`/`expect()` em código de produção** (`main.rs` L68-69, `psexe.rs` L16). Reincidência: o achado F5 da iteração 0022 já tinha apontado o mesmo problema. | — (R6: zero `unwrap`/`expect` fora de teste) | Revisão adversarial (F3): corrigido com `match (bios_arg.take(), exe_arg.take())` e `read_u32` retornando `Result`. |
| 8 | panic | **`load_psexe` entrava em pânico com corpo truncado** (tamanho não múltiplo de 4 indexava fora do slice). A função devolve `Result` justamente para reportar arquivo inválido. | — | Revisão adversarial (F4): corrigido com guarda `load_size % 4 != 0` retornando `Err`. Adicionado teste `load_psexe_rejeita_corpo_nao_multiplo_de_4`. |
| 9 | dead-code | **Detecção de halt (`same_pc_count >= 2`) nunca disparava** com `JMP $` (PC alterna entre X e X+4) e não era coberta por teste — o teste tinha uma cópia própria de `run()` sem detecção. | `02-cpu.md` — `JMP $` executa `j target` + `nop` no delay slot, PC alterna (nunca repete consecutivo). | Revisão adversarial (F6): detecção removida (opção i); step-limit é a única parada. A1 ajustado para afirmar `steps == 20`. |
| 10 | processo | `cargo fmt --all` não foi rodado antes do push; CI vermelha. | — (passo 7 do protocolo) | Revisão adversarial (F5): fmt rodado e CI verde. |
| 11 | arquivo | **O caminho de `exe_dir()` perdeu um nível na 2ª rodada**: `CARGO_MANIFEST_DIR + .. = <repo>/crates`, e `tests/exes/` não existe em `crates/`. O A4 continuou retornando cedo, e a bateria de mutação da 2ª rodada creditou M6 e C3 a um teste que não executava. | — | Revisão adversarial da 3ª rodada (G1): adicionado `..` extra em `exe_dir()`. Removido o teste `psxtest_cpu_nao_esta_disponivel_mas_ignorado` (passava com ou sem arquivo, media zero). |
| 12 | hardware | **O `psxtest_cpu` chama `printf` A(3Fh)**, documentado em `13-kernel-bios.md` L2703-2740. O hook do 1.10 só implementa `putchar` (A(3Ch)) e `puts` (A(3Eh)). Sem `printf`, o EXE roda até o step-limit com TTY de zero bytes. | `13-kernel-bios.md` L2703-2740 — printf recebe string em A0 + argumentos, expande `%`, usa `putchar` internamente. | Revisão adversarial (G2): A4 reescrito para afirmar o que é verdade hoje (sideload funciona, PC dentro de KSEG0). `printf` registrado como item 1.11b no ROADMAP. |
| 13 | processo | **Bateria de mutação da 2ª rodada afirmou M6 e C3 pegues por A4**, impossível porque A4 retornava cedo. Erro de processo: creditar resultado que não foi executado. | — | Revisão adversarial (G4): bateria refeita do zero na 3ª rodada, cada mutante executado e visto vermelho antes de reverter. |
| 14 | script | **Scoreboard dava 51/51 `fail-erro`** porque `Start-Process -ArgumentList` não cita caminhos com espaço; o CLI recebia `com` como argumento solto e abortava. `$cliBin` sem `.exe`, `$built`/`$builtBin` variáveis mortas. | — | Revisão adversarial (G3): argumentos citados com `` `" `` + `$argString` único no `Start-Process`. `$cliBin` resolve para `.exe`. Removidos `$built` e `$builtBin`. |

## Bateria de mutação (terceira rodada — refeita do zero)

Placar: **7/7 mutantes pegos, 3/3 controles verdes**.

Cada mutante foi aplicado, o teste visto vermelho, e a mutação revertida antes do próximo.

| # | Tipo | Mutação | Teste que a pegou |
|---|---|---|---|
| M1 | mutante | BSS zero-fill removido (bloco `if bss_size > 0` deletado) | `zerofill_bss` (A3) |
| M2 | mutante | Swap PC (0x10) com GP (0x14) no header — PC vira 0 | `sideload_exe_minimo_jmp_self` (A1) |
| M3 | mutante | Body carregado com `from_be_bytes` (big-endian) em vez de LE | `sideload_exe_minimo_jmp_self` (A1) |
| M4 | mutante | header_size = 0x7FC (4 bytes antes do corpo real) | `sideload_exe_minimo_jmp_self` (A1) |
| M5 | mutante | Guarda `sp_fp_base != 0` removida — sempre sobrescreve SP/FP | `sp_fp_base_zero_nao_altera_registradores` |
| M6 | mutante | `install_return_stubs` corpo vazio (no-op) | `print_ok_via_tty` (A2) |
| M7 | mutante | Stub escreve `jr $zero` em vez de `jr $ra` — jump para R0=0 | `print_ok_via_tty` (A2) |
| C1 | controle | Renomear `load_size` → `body_len` | todos verdes |
| C2 | controle | Reordenar R4/R5 (R4=1 antes de R5=0) | todos verdes |
| C3 | controle | Reordenar stubs A0h/B0h/C0h (C0h antes de A0h) | todos verdes |

**Nota sobre M6 da 2ª rodada:** o mutante original removia a *chamada* a `install_return_stubs` de `main.rs`, o que não é capturável por testes de unidade. Trocado por tornar o corpo da função um no-op, que A2 captura (sem stubs, só 'O' é impresso antes do CPU se perder).

## Placar antes → depois

Workspace: 231 → **230** testes (1 removido: `psxtest_cpu_nao_esta_disponivel_mas_ignorado`; `psxtest_cpu_sideload_real` renomeado para `psxtest_cpu_sideload_executa_sem_panico`).

Contagem verificada (soma das linhas `test result` de `cargo test --all`): psx-cli 12 + psx-core 208 + meta 10 = **230**.

## Scoreboard (3ª rodada)

Scoreboard executado e verificado: `./scripts/scoreboard.ps1` → 0/51 passando, Amidog `amidog/cpu` com status `fail` (TTY 0 bytes — printf A(3Fh) pendente, ROADMAP 1.11b).

Últimas 5 linhas de `logs/scoreboard.csv`:
```
2026-07-28T10:54:51-03:00,0e6b678,ps1-tests/spu/test,test.exe,fail,
2026-07-28T10:54:51-03:00,0e6b678,ps1-tests/spu/toolbox,toolbox.exe,fail,
2026-07-28T10:54:51-03:00,0e6b678,ps1-tests/timer-dump,timer-dump.exe,fail,
2026-07-28T10:54:51-03:00,0e6b678,ps1-tests/timers,timers.exe,fail,
2026-07-28T10:54:51-03:00,0e6b678,ps1-tests/tools/diffvram,diffvram-windows-amd64.exe,fail-erro,
```

O `diffvram-windows-amd64.exe` é um binário nativo Windows, não um PS-EXE (vem do ps1-tests). O CLI falha no magic check e sai com código ≠ 0 — `fail-erro` é esperado para esse arquivo. Nenhum `sem-runner` na saída; todos os EXEs reais têm status `pass`, `fail` ou o caso documentado `fail-erro`.

## Revisão cruzada (orquestrador)

### 1ª rodada (PR #41)

7 achados (F1-F7):
- F1 (bloqueador): A4 nunca rodou — nome do arquivo `.psexe` em vez de `.exe`, arquivo existia mas teste retornava cedo.
- F2 (bloqueador): Sideload não funcionava com EXE real — stubs `jr $ra` só existiam no teste A2 escrito à mão.
- F3: `unwrap()`/`expect()` em produção (reincidência da iter 0022).
- F4: Pânico em corpo truncado dentro de função que devolve `Result`.
- F5: `cargo fmt --check` reprovava.
- F6: Detecção de halt era código morto, não coberta por teste.
- F7: Handoff do 1.12 raso (4 linhas), empurrava A4 para item seguinte.

### 2ª rodada

Correções parciais: F3, F4, F5 e F6 resolvidos. F1 e F2 não: a correção do caminho perdeu um nível de diretório (G1) e o EXE real imprime zero bytes porque printf A(3Fh) não existe (G2). Bateria de mutação creditou resultados não executados (G4). Scoreboard com bug de aspas no Start-Process (G3).

### 3ª rodada (esta)

- G1-G6 resolvidos.
- A4 agora executa de verdade e afirma o que é verdade: PC em KSEG0 após execução, TTY 0 bytes (printf pendente).
- ROADMAP 1.11b adicionado para printf A(3Fh).
- Scoreboard funcional: `amidog/cpu` com status `fail` (não `sem-runner` nem `fail-erro`).
- Bateria de mutação refeita do zero (7/7, 3/3).

## Decisões e notas

1. **Critério de parada: step-limit único.** A detecção de self-loop foi removida. `RUNNER_MAX_STEPS = 50_000_000` é a única parada.

2. **`install_return_stubs` em `psx-core::psexe`**: escreve `jr $ra` + `nop` nos endereços físicos A0h, B0h, C0h. Chamado pelo runner após `load_psexe`.

3. **Scoreboard**: invoca `psx-cli --bios --exe` com aspas duplas escapadas (`` `" ``) para caminhos com espaço (Windows). Detecta `sem-bios`, `timeout`, `fail-erro`, e classifica `pass`/`fail` pelo TTY.

4. **R4=1, R5=0 como parâmetros iniciais.** Mantidos conforme spec L1200-1202.

5. **TTY do Amidog depende de printf A(3Fh).** O `psxtest_cpu` chama A(3Fh) printf (`13-kernel-bios.md` L2703-2740), que não existe no hook do 1.10 (cobre apenas putchar A(3Ch) e puts A(3Eh)). O A4 é honesto: verifica sideload + execução sem pânico, não afirma TTY não vazio. Item 1.11b no ROADMAP cobre o printf com expansão de `%`.
