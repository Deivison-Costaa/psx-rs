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
| 5 | bios-stubs | **A4 nunca rodou porque o nome do arquivo estava errado** (`psxtest_cpu.psexe` em vez de `psxtest_cpu.exe`). O arquivo existia em `tests/exes/amidog/cpu/psxtest_cpu.exe` desde 27/07, mas o teste retornava cedo porque `exe_path.exists()` dava `false`. Isso escondeu o erro 6. | — | Revisão adversarial (F1): o orquestrador notou que o teste passava verde sem nunca executar uma linha do que se propunha a medir. Nome corrigido e teste renomeado para `psxtest_cpu_sideload_real`. |
| 6 | bios-stubs | **O EXE real chama a BIOS via `jal` mas não havia nada em A0h/B0h/C0h para onde retornar.** O teste A2 escrevia `jr $ra` à mão dentro do `.psexe` sintético, mascarando a ausência dos stubs no código de produção. A RAM estava zerada nesses endereços, a execução caía em nops por ~16k passos até reentrar no ponto de entrada do EXE, em loop infinito, sem produzir TTY. | A(00A0h)/B(00B0h)/C(00C0h) são chamadas via `jal` com número de função em R9 (`13-kernel-bios.md` L496-498, L685, L788). O código chamador espera retornar via `jr $ra`. | Revisão adversarial (F2): o orquestrador instrumentou o laço do runner e viu que após 51 passos a execução entrava em A0h, mas sem stub de retorno, caía até reentrar em 80010000h sem imprimir nada. Corrigido com `install_return_stubs()` em `psx-core::psexe` que escreve `jr $ra` + `nop` em A0h/B0h/C0h. A2 parou de escrever `jr $ra` à mão. |
| 7 | panic | **`unwrap()`/`expect()` em código de produção** (`main.rs` L68-69, `psexe.rs` L16). Reincidência: o achado F5 da iteração 0022 já tinha apontado o mesmo problema. | — (R6: zero `unwrap`/`expect` fora de teste) | Revisão adversarial (F3): corrigido com `match (bios_arg.take(), exe_arg.take())` e `read_u32` retornando `Result`. |
| 8 | panic | **`load_psexe` entrava em pânico com corpo truncado** (tamanho não múltiplo de 4 indexava fora do slice). A função devolve `Result` justamente para reportar arquivo inválido. | — | Revisão adversarial (F4): corrigido com guarda `load_size % 4 != 0` retornando `Err`. Adicionado teste `load_psexe_rejeita_corpo_nao_multiplo_de_4`. |
| 9 | dead-code | **Detecção de halt (`same_pc_count >= 2`) nunca disparava** com `JMP $` (PC alterna entre X e X+4) e não era coberta por teste — o teste tinha uma cópia própria de `run()` sem detecção. | `02-cpu.md` — `JMP $` executa `j target` + `nop` no delay slot, PC alterna (nunca repete consecutivo). | Revisão adversarial (F6): detecção removida (opção i); step-limit é a única parada. A1 ajustado para afirmar `steps == 20`. |
| 10 | processo | `cargo fmt --all` não foi rodado antes do push; CI vermelha. | — (passo 7 do protocolo) | Revisão adversarial (F5): fmt rodado e CI verde. |

## Bateria de mutação (segunda rodada)

Placar: **7/7 mutantes pegos, 3/3 controles verdes**.

| # | Tipo | Mutação | Teste que a pegou |
|---|---|---|---|
| M1 | mutante | BSS zero-fill removido (bloco `if bss_size > 0` deletado) | `zerofill_bss` (A3) |
| M2 | mutante | Swap PC (0x10) com GP (0x14) no header — PC vira 0 | `sideload_exe_minimo_jmp_self` (A1) |
| M3 | mutante | Body carregado com `from_be_bytes` (big-endian) em vez de LE | `print_ok_via_tty` (A2) e `sideload_exe_minimo_jmp_self` (A1) |
| M4 | mutante | header_size = 0x7FC (4 bytes antes do corpo real) | `sideload_exe_minimo_jmp_self` (A1) |
| M5 | mutante | Guarda `sp_fp_base != 0` removida — sempre sobrescreve SP/FP | `sp_fp_base_zero_nao_altera_registradores` |
| M6 | mutante | `install_return_stubs` removido (não chamado após `load_psexe`) | `psxtest_cpu_sideload_real` (A4) e `print_ok_via_tty` (A2) |
| M7 | mutante | Stub em A0h ausente (só B0h e C0h escritos) | `print_ok_via_tty` (A2) |
| C1 | controle | Renomear `_initial_pc` → `pc_init` | todos verdes |
| C2 | controle | Reordenar R4/R5 (R5=0 antes de R4=1) | todos verdes |
| C3 | controle | Reordenar stubs A0h/B0h/C0h (escrever C0h antes de A0h) | `print_ok_via_tty` (A2) e `psxtest_cpu_sideload_real` (A4) |

## Placar antes → depois

Workspace: 228 → **230** testes (3 novos: `psxtest_cpu_sideload_real`, `load_psexe_rejeita_corpo_nao_multiplo_de_4`, `argumento_desconhecido_rejeitado`; 1 removido: antigo `psxtest_cpu_nao_disponivel`; A1 e A2 modificados).

## Revisão cruzada (orquestrador)

7 achados (F1-F7) no PR #41:
- F1 (bloqueador): A4 nunca rodou — nome do arquivo `.psexe` em vez de `.exe`, arquivo existia mas teste retornava cedo.
- F2 (bloqueador): Sideload não funcionava com EXE real — stubs `jr $ra` só existiam no teste A2 escrito à mão.
- F3: `unwrap()`/`expect()` em produção (reincidência da iter 0022).
- F4: Pânico em corpo truncado dentro de função que devolve `Result`.
- F5: `cargo fmt --check` reprovava.
- F6: Detecção de halt era código morto, não coberta por teste.
- F7: Handoff do 1.12 raso (4 linhas), empurrava A4 para item seguinte.

Corrigidos nesta rodada: F1-F7 (ver erros 5-10 acima).

## Decisões e notas

1. **Critério de parada: step-limit único.** A detecção de self-loop foi removida. `RUNNER_MAX_STEPS = 50_000_000` é a única parada. Self-loop com `JMP $` alterna PC entre X e X+4, nunca repetindo consecutivo — a detecção por PC consecutivo igual era inútil. A parada por par de PCs repetidos ((X, X+4) se repete) seria correta para o hardware, mas o step-limit é suficiente para o sideload e não justifica a complexidade extra neste item.

2. **`install_return_stubs` em `psx-core::psexe`**: escreve `jr $ra` (0x03E00008) + `nop` (0x00000000) nos endereços físicos A0h, B0h, C0h. Chamado pelo runner (`main.rs`) após `load_psexe`. Os três endereços são pontos de entrada da BIOS documentados em `13-kernel-bios.md` L496-498 (A-Functions), L685 (B-Functions), L788 (C-Functions).

3. **Scoreboard adaptado**: `scripts/scoreboard.ps1` agora invoca `psx-cli --bios --exe` de verdade, detecta `sem-bios` se `bios/SCPH1001.BIN` não existir, e grava `pass`/`fail` conforme a saída de TTY. O critério é binário: TTY não vazio = `pass`.

4. **R4=1, R5=0 como parâmetros iniciais.** Mantidos conforme decisão da primeira tentativa (spec L1200-1202).

5. **`load_psexe` aceita o arquivo inteiro (`&[u8]`) com header de 800h + corpo.** O body é carregado via `bus.write32::<BusRead>()` que traduz KSEG0→físico automaticamente.
