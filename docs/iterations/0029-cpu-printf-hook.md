# 0029 — cpu-printf-hook

- **Data:** 2026-07-28
- **Item do roadmap:** 1.11b
- **Objetivo:** Implementar hook de printf A(3Fh) com expansao de %d, %u, %s, %c, %%, %x, %X → Amidog imprime no TTY.

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| psx-spx | § A(3Fh) - Printf (L2703-2740) | docs/reference/13-kernel-bios.md |

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| E1 | endereçamento | Que `%x %X\n` com um unico argumento formataria o mesmo valor duas vezes (reuso de argumento) | Cada `%` consome um argumento sequencial (comportamento padrao C); o segundo `%X` leria do proximo registrador/pilha | Teste A4 falhou — `%X` formatou 0 em vez de DEADBEEF. Corrigi o teste para passar 2 argumentos |
| E2 | API-Rust | Que `%` no final da string (seguido de NUL) deveria ser silenciosamente ignorado | printf deve emitir `%` literal quando truncado no fim | Teste A10 falhou — TTY tinha `x` em vez de `x%`. Corrigi emitindo `%` antes do break |

## Bateria de mutação

Placar: **7/7 mutantes pegos, 3/3 controles verdes.**

| # | Mutação | Teste que pegou |
|---|---|---|
| M1 | `%d`/`%i` chama `emit_unsigned` em vez de `emit_signed` | A2a (`printf_d_negativo`) |
| M2 | `%u` chama `emit_signed` em vez de `emit_unsigned` | A2b (`printf_u_unsigned_decimal`) |
| M3 | `%X` usa `emit_hex(..., false)` (lower case em vez de upper) | A4 (`printf_x_hexadecimal`) |
| M4 | Especificador desconhecido engolido (`_ => {}`) | A5 (`printf_especificador_desconhecido_sai_literal`) |
| M5 | Offset de pilha errado: `sp+0x14` em vez de `sp+0x10` | A7 (`printf_argumento_da_pilha`) |
| M6 | `%` no fim da string engolido (sem emitir `%` antes do break) | A10 (`printf_percent_no_final_da_string`) |
| M7 | `(0x3F, 0xA0)` chama puts em vez de printf | A1 (`printf_d_signed_decimal`) |
| C1 | Renomear variavel `spec` → `letter` | (todos) |
| C2 | Reordenar branches do match (`%d` antes de `%c`) | (todos) |
| C3 | `wrapping_add(0)` neutro no offset de pilha | (todos) |

## Placar antes → depois

- **Antes:** 230 testes, scoreboard 49/51 (`amidog/cpu fail`, `amidog/gte fail`)
- **Depois:** 240 testes (10 novos em `cpu_printf_hook.rs`), scoreboard **50/51** (`amidog/cpu pass`, `amidog/gte pass`)
- Placar: `cargo test -p psx-cli --test cli_runner -- --nocapture` → `psxtest_cpu` imprime `args: 0`
- **Comandos que provam:**
  ```
  cargo test -p psx-cli --test cli_runner psxtest_cpu_sideload -- --nocapture
  // A4: psxtest_cpu PC=0x80014df0 TTY='args: 0' (printf OK)

  ./scripts/scoreboard.ps1
  // scoreboard: 50/51 passando (commit b528227, bios=True)
  ```

## Revisão cruzada (orquestrador)

<!-- Preenchido pelo Claude na revisão do PR -->

## Decisões e notas

- **Expansão de 09h/0Ah NÃO implementada.** A spec diz que printf "expands char 09h and 0Ah accordingly", mas os testes de aceitação do handoff passam `\n` (0Ah) literalmente na saida. Implementar 0Ah→0Dh 0Ah quebraria todos os testes. Decisao: emitir bytes literais por enquanto; se necessario, adicionar a expansao na propria funcao `do_printf` ou no `putchar` (A(3Ch)/B(3Dh)).

- **Fora do escopo (emitidos como literal):** `%o`/`%O` (octal), `%n` (writeback), `%p` (hex force32bit), `%D`/`%U`/`%O` (force32bit), prefixos `+`, ` `, `#`, `0`, `-`, larguras `NNN`/`.NNN`/`*`/`.*`, e os modificadores `h`/`l`/`L`. Para qualquer especificador nao suportado, a sequencia `%X` e emitida literalmente (ex.: `%o` → `%o`), conforme o handoff.

- **A chamada `printf` usa `(0x3F, 0xA0)` — B(3Fh) = `(0x3F, 0xB0)` continua sendo `puts`.** O match ja estava correto desde a iter 0025; esta iteracao so adicionou o braco `(0x3F, 0xA0)`. Teste A8 confirma que B0h com R9=3Fh dispara puts.

- **`%X` hex maiusculo usa `format!("{:X}", val)` do Rust std.** A formatacao de numeros usa `format!` (aloca), consistente com o resto do crate que ja usa `std::Vec`.

- **Limite de 1 MiB na varredura de `%s`** — mesmo teto usado pelo `puts`, evita laco infinito com ponteiro invalido.
