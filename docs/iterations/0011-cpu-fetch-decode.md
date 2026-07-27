# 0011 — cpu-fetch-decode

- **Data:** 2026-07-27
- **Item do roadmap:** 1.2
- **Objetivo:** struct Cpu com regs (32×u32, R0=0, PC=0xBFC00000), step() que busca instrução via Bus::read32, decodifica pelo primary opcode e executa LUI, ORI e SW.

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| psx-spx | L19 CPU Registers, L74 CPU Opcode Encoding, L305 logical instructions, L219 Store instructions | docs/reference/02-cpu.md |

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | API-Rust | Passei `&Bus` para `step()` | SW precisa de `&mut Bus` para escrever na RAM | Compilador rejeitou — corrigido antes do primeiro verde |
| 2 | API-Rust | Teste escrevia SW em pc=4 e depois setava `cpu.regs[8]` achando que o LUI ainda valia | O SW usava `rs=8` que continha 0xAABB_CCDD gerando endereço 0xAABB_CDDD fora da RAM | Teste `sw_writes_to_ram_via_bus` falhou — corrigido trocando rs para R0 |
| 3 | flags | Mutação de ORI sign-extend sobreviveu com imm=0x5678 (bit 15 = 0) | Sign-extend e zero-extend são idênticos quando bit 15 = 0 | Bateria de mutação revelou — adicionado `ori_sign_extend_mutation_catcher` com imm=0xFFFF |

## Bateria de mutação

7/7 mutantes pegos, 2/2 controles verdes.

| # | Mutação | Teste que pegou |
|---|---|---|
| 1 | ORI sign-extend (`imm as i16 as u32`) | `ori_sign_extend_mutation_catcher` |
| 2 | LUI não zera baixos (`(imm << 16) \| imm`) | `lui_sets_upper_and_clears_lower` |
| 3 | SW addr errado (`wrapping_sub` em vez de `wrapping_add`) | `sw_writes_to_ram_via_bus` |
| 4 | R0 mutável (removeu guarda em set_reg) | `r0_is_always_zero` |
| 5 | PC não avança (removeu `wrapping_add(4)`) | `lui_sets_upper_and_clears_lower` |
| 6 | Opcode primário errado (`>> 24` em vez de `>> 26`) | `lui_sets_upper_and_clears_lower` |
| 7 | SW escreve addr em vez de val | `sw_writes_to_ram_via_bus` |
| 8 (controle) | Renomear variável local em `lui()` | Passou |
| 9 (controle) | Reordenar funções `lui`/`ori` | Passou |

## Placar antes → depois

Workspace: **33 → 37** testes (7 novos de cpu_fetch_decode).

## Revisão cruzada (orquestrador)

<!-- Preenchido pelo Claude na revisão do PR -->

## Decisões e notas

- Opcode encoding segue a tabela L74: primary opcode em bits 26..31.
- LUI: `rt = imm << 16` conforme L404.
- ORI: `rt = rs | imm` com zero-extension (imm é u32, bits 16..31 = 0) conforme L392.
- SW: `[rs + imm] = rt` conforme L303.
- R0 é forçado a 0 em `set_reg()`; leituras via `reg()` retornam o valor real (sempre 0 por construção).
- PC inicial 0xBFC00000 (reset vector, spec L736).
- Clippy exige `-D warnings`: precisou remover `as u32` desnecessário e usar hex em vez de binário nos testes.
