# 0020 — COP0: banco de registradores + MTC0/MFC0/RFE (sem exceções)

- **Data:** 2026-07-27
- **Item do roadmap:** 1.8a
- **Objetivo:** Implementar banco de registradores COP0 (SR, CAUSE, EPC, BadVaddr, PRID + garbage r16–r31) e os opcodes MFC0, MTC0 e RFE, sem mecanismo de exceção.

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| psx-spx | Coprocessor Opcode/Parameter Encoding | docs/reference/02-cpu.md L127 |
| psx-spx | Coprocessor Instructions (COP0..COP3) | docs/reference/02-cpu.md L422 |
| psx-spx | Caution - Load Delay / Store Delay | docs/reference/02-cpu.md L438, L446 |
| psx-spx | COP0 - Register Summary | docs/reference/02-cpu.md L568 |
| psx-spx | cop0r13 - CAUSE | docs/reference/02-cpu.md L590 |
| psx-spx | cop0r12 - SR | docs/reference/02-cpu.md L624 |
| psx-spx | cop0r14 - EPC | docs/reference/02-cpu.md L670 |
| psx-spx | cop0cmd=10h - RFE opcode | docs/reference/02-cpu.md L712 |
| psx-spx | cop0r8 - BadVaddr | docs/reference/02-cpu.md L730 |
| psx-spx | cop0r15 - PRID | docs/reference/02-cpu.md L775 |

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | nenhum | Nenhum erro de primeira tentativa. O handoff do STATUS estava excepcionalmente detalhado — trazia golden values derivados por duas rotas independentes, armadilhas nomeadas e regra nova de bateria. A implementação foi direta. | — | — |

## Bateria de mutação

Placar: **6/6 mutantes pegos, 2/2 controles verdes.**

| # | Tipo | Mutação | Teste que pegou |
|---|---|---|---|
| 1 | erro | RFE: shift único de 2 bits zerando bits 4-5 (`(sr & !0x3F) \| ((sr >> 2) & 0xF)`) | `sr_rfe_move_campos_ie_ku_corretamente` |
| 2 | erro | CAUSE: registrador comum (`cop0[13] = val` sem máscara) | `cause_mascara_escrita_apenas_bits_8_e_9` |
| 3 | erro | MFC0 sem load delay (`set_reg` direto em vez de `Some((rt, val))`) | `mfc0_tem_load_delay_de_um_opcode` |
| 4 | erro | MTC0 ignorado (no-op, simulando store delay) | `mtc0_nao_tem_store_delay` |
| 5 | erro | PRID inicializado com 0 em vez de 2 | `prid_retorna_0x00000002` |
| 6 | erro | RFE: ordem das cópias invertida (bit4-5→bit0-1, bit2-3→bit2-3) | `sr_rfe_move_campos_ie_ku_corretamente` |
| C1 | controle | Renomear variáveis locais `iec_kuc`/`iep_kup` → `lo`/`hi` | Nenhum (verde) |
| C2 | controle | Reordenar definições de `cop0_read` e `cop0_write` no fonte | Nenhum (verde) |

## Placar antes → depois

Workspace: **188** testes (178 anteriores + 10 de cop0_regs). Meta-testes: 10.

## Revisão cruzada (orquestrador)

<!-- Preenchido pelo Claude na revisão do PR -->

## Decisões e notas

1. **EPC e BadVaddr são graváveis — comportamento ASSUMIDO.** A spec marca ambos como (R), mas
   o handoff do STATUS orienta implementar como gravável e registrar. Ponto de resolução: Amidog
   `psxtest_cpu` (item 1.11). Se o hardware rejeitar escrita, `cop0_write` ganha `if reg == 8 ||
   reg == 14 { return; }`.

2. **Registradores N/A (r0-r2, r4, r10, r32-r63) não disparam exceção nesta iteração.**
   Leitura retorna 0, escrita é ignorada. O comportamento correto (Reserved Instruction
   Exception, excode=0Ah) depende do mecanismo de exceção que entra no item 1.8b.

3. **Registradores garbage (r16-r31) retornam 0.** A spec diz "garbage" sem garantia de valor;
   0 é tão válido quanto qualquer outro. O teste `registrador_garbage_nao_dispara_excecao`
   verifica apenas que a leitura não causa exceção (assert `== 0` — se o valor mudar no futuro
   por implementação de cache ou timing, o teste aceita qualquer valor).

4. **TAR (r6) é marcado (R) na spec mas implementado como R/W.** Mesmo critério de EPC/BadVaddr:
   comportamento assumido, sem evidência contrária. Resolução no item 1.11.

5. **Teste `mtc0_com_r0_escreve_zero` corrigido durante a implementação.** A versão original
   setava `cpu.regs[0] = 0x1234_5678` e esperava que MTC0 lesse 0 — mas `self.regs[0]` retorna
   o valor do array, não força zero. O teste foi reescrito para não adulterar R0 e verificar
   que o valor escrito é 0 (R0 é zero desde `Cpu::new`).
