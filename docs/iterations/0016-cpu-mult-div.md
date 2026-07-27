# 0016 — MULT/MULTU/DIV/DIVU + HI/LO

- **Data:** 2026-07-27
- **Item do roadmap:** 1.6
- **Objetivo:** Implementar MULT, MULTU, DIV, DIVU, MFHI, MTHI, MFLO, MTLO com registradores hi/lo na CPU.

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| psx-spx | Multiply/divide (L329) | docs/reference/02-cpu.md |

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | API-Rust | `self.reg(rs) as i64` faz sign-extend corretamente | u32 → i64 zero-extende; precisa de `as i32 as i64` para sinalizar | `mult_negativo` falhou: hi=1999 em vez de 0xFFFF_FFFF |
| 2 | endereçamento | `encode_special(MFHI, 0, 0, 8)` escreve no registrador 8 | MFHI usa campo `rd` (bits 11..15), não `rs` (bits 21..25) | `mfhi_le_hi` e `mflo_le_lo` falharam — rd=0 (R0, ignorado) |
| 3 | nenhum | Expectativa de `mult_64bits_hi_lo` calculada errada | 0x1000_0001 * 0x0002_0000 = 0x2000_0002_0000, hi=0x2000, lo=0x20000 | Teste esperava hi=0x20000; corrigido |


## Bateria de mutação

Placar: **7/7 mutantes pegos, 2/2 controles verdes**.

| # | Mutação | Teste que pegou |
|---|---|---|
| 1 | MULT usa u64 (unsigned) em vez de i64 | `mult_negativo` |
| 2 | MULT: hi=0 sempre (ignora parte alta) | `mult_64bits_hi_lo`, `mult_negativo` |
| 3 | MULTU: hi=0 sempre | `multu_basico`, `multu_grande` |
| 4 | DIV sem tratar divisão por zero (panic) | `div_por_zero_rs_positivo`, `div_por_zero_rs_negativo` (panic) |
| 5 | DIV sem tratar overflow 0x80000000/-1 (panic) | `div_overflow_80000000_por_menos_1` (panic) |
| 6 | DIV: lo=0 para div por zero (sinal errado) | `div_por_zero_rs_positivo`, `div_por_zero_rs_negativo` |
| 7 | DIVU sem tratar divisão por zero (panic) | `divu_por_zero` (panic) |

Controles:
1. Renomear `a`/`b` para `x`/`y` em MULT — passou (18/18)
2. Reordenar MFHI/MTHI/MFLO/MTLO no match — passou (18/18)

## Placar antes → depois

147 testes no workspace (18 novos), todos passando.

## Revisão cruzada (orquestrador)

<!-- Preenchido pelo Claude na revisão do PR -->

## Decisões e notas

- Stalls (ciclos de MULT/DIV) registrados como dívida: serão observáveis quando o scheduler cobrar ciclos da CPU. A implementação atual é funcional mas não contabiliza latência.
