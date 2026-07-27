# 0012 — cpu-alu

- **Data:** 2026-07-27
- **Item do roadmap:** 1.3
- **Objetivo:** Implementar as instruções aritméticas, lógicas e de comparação da ALU do R3000A, tanto no formato SPECIAL (registrador-registrador) quanto ALU-imediato.

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| psx-spx | § CPU ALU Opcodes: arithmetic, comparison, logical | `docs/reference/02-cpu.md` |
| psx-spx | § Opcode/Parameter Encoding: SPECIAL, alu-imm | `docs/reference/02-cpu.md` |
| psx-spx | § Primary opcode field / Secondary opcode field | `docs/reference/02-cpu.md` |

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | API-Rust | `encode_alu_imm` helper com rs/rt trocados no shift (`rs << 16`, `rt << 21`) | bits 25..21=rs, 20..16=rt | Teste falhou: addiu, addi, andi, xori, slti com resultado 0 |
| 2 | flags | SLTIU com imm=0x8000: esperava que 0 < 0xFFFF_8000 unsigned desse 0 (achei que imm negativo não coubesse na comparação unsigned) | SLTIU compara rs com imm **sign-extended** como unsigned: 0 < 0xFFFF_8000 → true (1) | Teste específico `sltiu_rs_menor_imm_unsigned` falhou |

## Bateria de mutação

Placar: **6/6** mutantes pegos, **2/2** controles verdes.

| Mutação | Teste que pegou | Status |
|---|---|---|
| SUBU: `wrapping_add` em vez de `wrapping_sub` | `subu_basico`, `subu_wraparound` | pego |
| NOR: `!(rs & rt)` em vez de `!(rs \| rt)` | `nor_basico`, `nor_nao_e_or` | pego |
| SLT: unsigned `<` em vez de signed `(i32)` | `slt_rs_menor_rt_signed`, `slt_rs_nao_menor_rt_signed` | pego |
| SLTU: signed em vez de unsigned | `sltu_rs_maior_rt_unsigned`, `sltu_rs_menor_rt_unsigned` | pego |
| `sign_extend_imm` zero-extende | `addiu_sign_extends_imm`, `addi_sign_extends_imm`, `slti_rs_menor_imm_signed`, `sltiu_rs_menor_imm_unsigned` | pego |
| ANDI: sign-extend em vez de zero-extend | `andi_zero_extends` | pego |
| Controle: renomear `val` → `resultado` em ADDU | (passa) | verde |
| Controle: reordenar `xori` antes de `andi` | (passa) | verde |

## Placar antes → depois

- Workspace: **41** testes → **67** testes (41 + 26 novos)
- Meta-testes: todos verdes

## Revisão cruzada (orquestrador)

<!-- Preenchido pelo Claude na revisão do PR -->

## Decisões e notas

1. ADD/SUB (com overflow trap) implementados como ADDU/SUBU por enquanto, conforme instrução do handoff: o BIOS usa ADDU.
2. `sign_extend_imm` extraída como função auxiliar, reusada por addiu, addi, slti, sltiu e sw.
3. SPECIAL primário (0x00) decodifica pelo secondary opcode. Para já, a implementação cobre os 8 opcodes do item (ADDU, SUBU, AND, OR, XOR, NOR, SLT, SLTU); os demais dão `unimplemented!`.
