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
| E3 | handoff (orquestrador) | Que o teste de aceitação A4 do handoff estava correto | `%x %X\n` com um único argumento não pode produzir `deadbeef DEADBEEF` — cada `%` consome um argumento | O handoff mandava `setup_printf(..., "%x %X\n", ..., &[0xDEAD_BEEF])` esperando dois valores distintos. A implementação seguiu a spec corretamente; quem corrigiu foi o trabalhador, antes de commitar o teste A4 |
| E4 | implementação | Que aplicar o teto de 1 MiB no `%s` bastava e o laço principal da varredura de formato não precisava de teto | Ambos os laços percorrem ponteiros fornecidos pelo guest e precisam de proteção contra laço infinito | Revisão adversarial (H2) — `do_printf` sem teto no laço principal trava com ponteiro para região sem byte zero |
| E5 | teste-nao-mede | Que preencher uma faixa pequena de RAM (0x100..0x1000) com 'A' bastava para exercitar o teto de 1 MiB | A RAM zerada logo depois da faixa `0x1000` funciona como terminador NUL natural; a varredura para em `0x1000` e nunca chega perto de 1 MiB — com ou sem o teto, o teste passa | Orquestrador aplicou o mutante M8 (remover `if i >= 1_048_576 { break; }` do laço principal) e viu o teste passar em 0,00 s. É a **terceira ocorrência do mesmo padrão** no projeto: 0027 M6 (teste `cop0_dc ic_nao_altera_isc_fora_do_range` não testava o bit DCIC), 0027 C3 (`scratchpad_lw_da_regiao_de_controle_retorna_zero` também não media o que dizia medir), 0029 M8 (este). A recorrência é o dado, não o erro isolado |

## Bateria de mutação

Placar: **8/8 mutantes pegos, 3/3 controles verdes.**

| # | Mutação | Teste que pegou |
|---|---|---|
| M1 | `%d`/`%i` chama `emit_unsigned` em vez de `emit_signed` | A2a (`printf_d_negativo`) |
| M2 | `%u` chama `emit_signed` em vez de `emit_unsigned` | A2b (`printf_u_unsigned_decimal`) |
| M3 | `%X` usa `emit_hex(..., false)` (lower case em vez de upper) | A4 (`printf_x_hexadecimal`) |
| M4 | Especificador desconhecido engolido (`_ => {}`) | A5 (`printf_especificador_desconhecido_sai_literal`) |
| M5 | Offset de pilha errado: `sp+0x14` em vez de `sp+0x10` | A7 (`printf_argumento_da_pilha`) |
| M6 | `%` no fim da string engolido (sem emitir `%` antes do break) | A10 (`printf_percent_no_final_da_string`) |
| M7 | `(0x3F, 0xA0)` chama puts em vez de printf | A1 (`printf_d_signed_decimal`) |
| M8 | Teto de 1 MiB removido do laço principal (`if i >= 1_048_576` deletado) | A11 (`printf_fmt_sem_terminador_teto_1mib_evita_laco_infinito`) — **corrigido na rodada curta:** o teste original preenchia só 0x100..0x1000 com 'A'; a RAM zerada em 0x1000+ servia de terminador natural e o mutante sobrevivia. Reescrevemos o teste enchendo a RAM inteira (2 MiB) com 'A', afirmando `tty.len() == 1_048_576`. Aplicado M8 novamente, o teste falha (`2096897 != 1048576`) — mutante pego |
| C1 | Renomear variavel `spec` → `letter` | (todos) |
| C2 | Reordenar branches do match (`%d` antes de `%c`) | (todos) |
| C3 | `wrapping_add(0)` neutro no offset de pilha | (todos) |

## Placar antes → depois

- **Antes:** 230 testes, scoreboard 49/51 (`amidog/cpu sem-saida`, `amidog/gte sem-saida`)
- **Depois:** 241 testes (10 novos + 1 correção em `cpu_printf_hook.rs`), scoreboard **50/51 produziram saída** (`amidog/cpu tty`, `amidog/gte tty`)
- **Atenção:** `tty`/`sem-saida` NÃO é veredito de teste — significa apenas que o EXE emitiu bytes no TTY. Os 50 EXEs que produziram saída imprimem o banner da biblioteca do ps1-tests (`ResetGraph:itb=%08x,...`) e param; nenhum executou o próprio teste. O veredito real (`pass`/`fail`) é trabalho do 1.12.

## Revisão cruzada (orquestrador)

<!-- Preenchido pelo Claude na revisão do PR -->

## Decisões e notas

- **Expansão de 09h/0Ah NÃO implementada.** A spec diz que printf "expands char 09h and 0Ah accordingly" sem especificar para quê exatamente, e não temos hardware para conferir. Sem evidência, emitir os bytes literais é a decisão conservadora correta (R1). Se descoberto depois, implementar a expansão no próprio `do_printf` ou no `putchar` (A(3Ch)/B(3Dh)).

- **Fora do escopo (emitidos como literal):** `%o`/`%O` (octal), `%n` (writeback), `%p` (hex force32bit), `%D`/`%U`/`%O` (force32bit), prefixos `+`, ` `, `#`, `0`, `-`, larguras `NNN`/`.NNN`/`*`/`.*`, e os modificadores `h`/`l`/`L`. Para qualquer especificador nao suportado, a sequencia `%X` e emitida literalmente (ex.: `%o` → `%o`), conforme o handoff.

- **A chamada `printf` usa `(0x3F, 0xA0)` — B(3Fh) = `(0x3F, 0xB0)` continua sendo `puts`.** O match ja estava correto desde a iter 0025; esta iteracao so adicionou o braco `(0x3F, 0xA0)`. Teste A8 confirma que B0h com R9=3Fh dispara puts.

- **`%X` hex maiusculo usa `format!("{:X}", val)` do Rust std.** A formatacao de numeros usa `format!` (aloca), consistente com o resto do crate que ja usa `std::Vec`.

- **Limite de 1 MiB na varredura de `%s`** — mesmo teto usado pelo `puts`, evita laco infinito com ponteiro invalido.

- **Mesmo teto de 1 MiB no laço principal de varredura da string de formato** — aplicado na rodada de correção (H2). O laço `loop { ... }` que lê bytes da string de formato agora para em `i >= 1_048_576`. Ponteiro para região sem byte zero não trava mais. Teste A11 preenche a RAM inteira (2 MiB) com `b'A'`, planta `jal`/`nop` em 0x0-0x7 e aponta A0=0x100; afirma `tty.len() == 1_048_576`. Mutante (remover teto): teste falha com `2096897 != 1048576` — a varredura avança até encontrar o primeiro zero byte nos mirrors de RAM (byte zero do `jal` em 0x200001). **Corrigido na rodada curta** (E5): o teste original (preencher só 0x100..0x1000) permitia que a RAM zerada em 0x1000+ agisse como terminador natural; o mutante M8 sobreviveu por 2 rodadas até ser detectado pelo orquestrador.

- **`%08x` (largura + zero-padding) está FORA do escopo** por decisão do handoff e sai literal, o que está certo. Mas é o caso comum: a primeira linha de todo EXE do ps1-tests é `ResetGraph:itb=%08x,ehk=%08x`. Saída real de `bisect/branch/branch.exe`:

  ```
  ResetGraph:itb=%08x,ehk=%08x
  ResetGraph:SR=0
  ResetGraph:Interrupt hooks enabled.
  ResetGraph:About to init interrupts.
  ResetGraph:Interrupts enabled!
  ```

  Implementar largura e zero-padding no 1.12 ou em item futuro eliminaria esse ruído de 50 EXEs de uma vez.
