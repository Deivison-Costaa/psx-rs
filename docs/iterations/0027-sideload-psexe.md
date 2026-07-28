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

## Bateria de mutação

Placar: **5/5 mutantes pegos, 2/2 controles verdes**.

| # | Tipo | Mutação | Teste que a pegou |
|---|---|---|---|
| M1 | mutante | BSS zero-fill removido (bloco `if bss_size > 0` deletado) | `zerofill_bss` (A3) |
| M2 | mutante | Swap PC (0x10) com GP (0x14) no header — PC vira 0 | `sideload_exe_minimo_jmp_self` (A1) |
| M3 | mutante | Body carregado com `from_be_bytes` (big-endian) em vez de LE | `print_ok_via_tty` (A2) e `sideload_exe_minimo_jmp_self` (A1) |
| M4 | mutante | header_size = 0x7FC (4 bytes antes do corpo real) | `sideload_exe_minimo_jmp_self` (A1) |
| M5 | mutante | Guarda `sp_fp_base != 0` removida — sempre sobrescreve SP/FP | `sp_fp_base_zero_nao_altera_registradores` (novo teste D1) |
| C1 | controle | Renomear `_initial_pc` → `pc_init` | todos verdes |
| C2 | controle | Reordenar R4/R5 (R5=0 antes de R4=1) | todos verdes |

## Placar antes → depois

Workspace: 221 → **228** testes (7 novos: 5 de aceitação + 1 SP/FP + 1 scoreboard-skip).

## Revisão cruzada (orquestrador)

<!-- Preenchido pelo Claude na revisão do PR -->

## Decisões e notas

1. **Critério de parada: step-limit primário.** A detecção de self-loop (`same_pc_count >= 2`) é tentada como otimização, mas o step-limit (`RUNNER_MAX_STEPS = 50_000_000`) é a parada correta em todos os casos. O `JMP $` padrão alterna PC entre dois endereços (`j` + `nop` no delay slot), então a detecção simples por PC consecutivo igual NUNCA dispara com `JMP $` puro — o step-limit é o que sempre funciona. Esta observação confirma a armadilha 5 do handoff.

2. **psxtest_cpu.psexe não disponível no repo.** O teste A4 verifica existência do arquivo e retorna sem falhar se ausente. Os EXEs de teste são baixados por `scripts/fetch-test-exes.ps1` e são gitignored.

3. **R4=1, R5=0 como parâmetros iniciais.** A spec diz "usually R4=1 and R5=0" (L1200-1202). Adotamos esses valores fixos no sideload. Nenhum teste atual depende deles (o código sintético configura os próprios registradores), mas o psxtest_cpu real pode depender.

4. **`load_psexe` aceita o arquivo inteiro (`&[u8]`) com header de 800h + corpo.** O body é carregado via `bus.write32::<BusRead>()` que traduz KSEG0→físico automaticamente. Endereços virtuais do header (tipicamente `80010000h`) são tratados corretamente pelo Bus.

5. **TTY hook no runner**: o mesmo hook A0h/B0h implementado no item 1.10 é reutilizado. O runner carrega `jr $ra` no endereço do TTY para que a "função da BIOS" retorne ao código após o putchar.
