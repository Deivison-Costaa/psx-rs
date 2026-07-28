# 0033 — cpu-opcode-reservado

- **Data:** 2026-07-28
- **Item do roadmap:** 1.14
- **Objetivo:** Opcode nao implementado gera excecao (RI 0Ah / CpU 0Bh) em vez de panic.

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| psx-spx | Opcodes reservados → RI excode=0Ah | `02-cpu.md` L230 |
| psx-spx | cop0r0..r2/r4/r10/r32..r63 → RI 0Ah | `02-cpu.md` L874 |
| psx-spx | TLBR/TLBWI/TLBWR/TLBP → RI 0Ah | `02-cpu.md` L878 |
| psx-spx | LWC0/SWC0 → CpU excode=0Bh | `02-cpu.md` L883-884 |

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | endereçamento | Faixa `0x30..=0x3B` toda gera CpU | `0x34..=0x37` nao sao coprocessor load/store — sao N/A e devem gerar RI (0Ah) | Teste `varios_primarios_reservados_nao_panicam` quebrou em `0x34` |
| 2 | flags | Mutante sobrevivente: reduzir faixa SWCn para `0x38..=0x39` nao foi pego por nenhum teste | `0x3A` (SWC2) e `0x3B` (SWC3) devem gerar CpU, nao RI | Bateria de mutacao — adicionado `todos_primarios_cpu_geram_cpu` |

## Bateria de mutação

Placar: **7/7 mutantes pegos, 2/2 controles verdes.**

| # | Mutação | Teste que pegou |
|---|---|---|
| 1 | Trocar `0x0A` → `0x0B` no default do execute | `opcode_primario_inexistente_gera_ri`, `opcode_reservado_em_delay_slot_seta_bd_e_epc_no_branch`, `varios_primarios_reservados_nao_panicam` |
| 2 | Trocar `0x0B` → `0x0A` nos LWCn/SWCn (range `0x30..=0x3B`) | `lwc0_gera_cpu`, `swc0_gera_cpu`, `swc0_em_delay_slot_gera_cpu_com_bd` |
| 3 | Trocar `0x0B` → `0x0A` nos COP1/COP2/COP3 (`0x11..=0x13`) | `cop1_gera_cpu`, `cop3_gera_cpu` |
| 4 | Remover `raise_exception` de TLB no `cop0_op` | `tlb_op_gera_ri` |
| 5 | Trocar `0x0A` → `0x0C` no catch-all do `special()` | `secondary_inexistente_gera_ri` |
| 6 | Remover `raise_exception` no catch-all do execute (silent no-op) | `opcode_primario_inexistente_gera_ri`, `opcode_reservado_em_delay_slot_seta_bd_e_epc_no_branch`, `varios_primarios_reservados_nao_panicam` |
| 7 | Reduzir faixa SWCn para `0x38..=0x39` (sobrevivente na 1a tentativa) | `todos_primarios_cpu_geram_cpu` (adicionado apos o mutante sobreviver) |
| C1 | Reordenar bracos do match (inverter `0x11..=0x13` com `0x30..=0x3B`) | Nenhum quebrou (controle verde) |
| C2 | Adicionar blank line no match | Nenhum quebrou (controle verde) |

## Placar antes → depois

247 → **258** testes (11 novos em `cpu_opcode_reservado`; 3 renomeados em `cpu_alu`, `cpu_fetch_decode`, `cpu_shifts`).

Scoreboard: nao executado (item 1.14 anterior ao runner 1.11+).

## Revisão cruzada (orquestrador)

Uma rodada, um achado, e ele foi de acréscimo. A melhor rodada da sessão.

### Verificado executando, em `7083845`

- `cargo fmt --check` e `cargo clippy -D warnings` limpos; `cargo test --all` = **258**.
- A5 confere: `grep -rn "unimplemented!\|panic!(\|unwrap()\|expect(" crates/psx-core/src/`
  volta vazio.
- **O andaime nao vazou:** `git diff main...HEAD -- crates/psx-core/src/bus.rs` esta vazio. O
  handoff da 0032 mandou usar um stub temporario de GPUSTAT no teste de aceitacao e avisou
  para nao commitar; era um risco assumido de proposito, e nao se materializou.
- **Bateria conferida por reaplicacao.** Peguei o mutante 7 — o que o proprio doc admite ter
  sobrevivido na primeira tentativa — e reapliquei:
  ```
  test todos_primarios_cpu_geram_cpu ... FAILED
  test result: FAILED. 10 passed; 1 failed
  ```
  Pega de verdade. Primeira bateria da sessao que passa nessa conferencia sem correcao (as
  anteriores: 0027 M6/C3 e 0029 M8, todas creditadas a testes que nao pegavam nada).

### J1 — o A4 nao tinha sido executado, e o resultado real e melhor que o esperado

A nota 5 original dizia "o comportamento esperado e que ... termine normalmente sem panic".
Comportamento esperado nao e medicao. Rodei:

```
$ psx-cli --bios bios/SCPH1001.BIN --exe tests/exes/ps1-tests/cpu/cop/cop.exe
cpu/cop
pass - testCop0Disabled
pass - testCop0Enabled
Runner: 50000000 passos, TTY: 205 bytes   [exit=0]
```

**Este e o primeiro veredito real de hardware que o projeto produz.** Dois testes de
coprocessador do ps1-tests passando — e no formato `pass - <nome>`, que e exatamente o que o
item 1.13 (veredito real no scoreboard) vai precisar parsear. O 1.13 deixou de depender de
palpite sobre o formato de saida.

Medindo a vizinhanca, uma ressalva util: `io-access-bitwidth` despeja `VSync: timeout` em
serie. Os 3 582 bytes que ele produziu na medicao da 0032 nao sao teste rodando, sao espera
por vblank — timers/IRQ, itens do M2. Nao contar aquele byte-count como progresso.

### Correcoes minhas nesta branch

- Nota 5 reescrita com a saida real; nota 6 nova registrando `CAUSE.CE` como limitacao
  conhecida (`02-cpu.md` L681) em vez de ampliar o item.
- **O placar do STATUS ficou em 247 quando o real e 258** — o passo 8 do protocolo pede a
  atualizacao e ela nao veio. Corrigido, com o detalhamento por arquivo recontado
  (). Terceira vez na sessao que o numero do STATUS diverge do
  medido (0029, 0031, esta): conferir contando, nunca lendo.

## Decisões e notas

1. **COP2 (0x12), LWC2 (0x32), SWC2 (0x3A) agora levantam CpU (0Bh).** Antes deste item, esses opcodes caiam no `unimplemented!()` e derrubavam o processo. Agora viram excecao silenciosa. O item M3 (GTE) precisara desfazer esse comportamento — registrado aqui para visibilidade.

2. **CFC0 (co=0x02) e CTC0 (co=0x06) continuam ignorados silenciosamente** no `cop0_op()`. A spec menciona "no cfcs on PSX" mas nao especifica o comportamento de excecao para esses opcodes. Manter como no-op ate que uma suite de teste (ps1-tests) os exerca.

3. **Tres testes existentes foram renomeados** de `*_panics` para `*_gera_ri`: em `cpu_alu.rs`, `cpu_fetch_decode.rs` e `cpu_shifts.rs`. A logica mudou de `catch_unwind` para verificacao de `cop0[13]` e `pc`.

4. **A5 confirmado:** `grep -rn "unimplemented!\|panic!\|unwrap()\|expect(" crates/psx-core/src/` retorna vazio (fora de `#[cfg(test)]`).

5. **A4 (cop.exe) — executado pelo orquestrador na revisao**, com o stub temporario de GPUSTAT
   do doc da 0032 aplicado a mao (nao commitado). O resultado esta na secao de revisao cruzada:
   nao so nao entra em panico, como produz o primeiro veredito real de hardware do projeto.

6. **Limitacao conhecida: `CAUSE.CE` nao e preenchido.** `raise_exception(0x0B, None)` levanta
   Coprocessor Unusable sem escrever o numero do coprocessador em `CAUSE.28-29`, que a spec
   define (`02-cpu.md` L681). Nenhuma suite cobra isso hoje; ampliar o item para cobrir seria
   violar R4. Fica registrado para quando o M3 (GTE) precisar.
