# 0015 — Branches/jumps + branch delay slot

- **Data:** 2026-07-27
- **Item do roadmap:** 1.5
- **Objetivo:** Implementar todos os jumps (J, JAL, JR, JALR) e branches (BEQ, BNE, BLEZ, BGTZ, BLTZ, BGEZ, BLTZAL, BGEZAL) com branch delay slot.

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| psx-spx | CPU Jump Opcodes / jumps and branches + JALR cautions | docs/reference/02-cpu.md L459-L487 |

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | API-Rust | Que o branch target se aplica no mesmo step do branch (testes com 1 step) | O delay slot executa no step SEGUINTE ao branch; o redirecionamento só acontece após o delay slot | Testes falharam com PC=4 em vez do target — corrigi para 2 steps por branch |
| 2 | nenhum | — | — | Nenhum erro de emulação na primeira implementação |
| 3 | nenhum | — | — | Todos os 29 testes passaram de primeira após correção do número de steps |

## Bateria de mutação

| # | Mutação | Teste que pegou | Resultado |
|---|---|---|---|
| 1 | Delay slot ignorado: branch redireciona imediatamente no mesmo step | Quase todos os 18 testes de branch tomado — PC pula 4 a mais que o esperado | Pego |
| 2 | J/JAL sem preservar 4 bits altos do PC | `j_preserva_4_bits_altos_do_pc` — PC = 4 em vez de 0x8000_0004 | Pego |
| 3 | BLEZ sem signed: `self.reg(rs) <= 0` em vez de `(self.reg(rs) as i32) <= 0` | `blez_tomado_negativo` — 0xFFFF_FFFF como u32 é > 0 | Pego |
| 4 | BLTZAL/BGEZAL link só quando tomado (em vez de SEMPRE) | `bltzal_nao_tomado_mas_linka`, `bgezal_nao_tomado_mas_linka`, `bgezal_com_rs_ra_compara_valor_antes_do_link` — $ra mantém valor anterior | Pego |
| 5 | JALR escreve rd antes de ler rs (perde target quando rs=rd) | `jalr_mesmo_reg_rs_rd` — PC = 8 em vez de 0x3000 | Pego |
| 6 | Branch offset sem sign-extend (imm sem sinal) | `bne_tomado` — offset 0xFFFC vira 0x0000FFFC em vez de -4 | Pego |

Controles:
| # | Controle | Resultado |
|---|---|---|
| 1 | Renomear variável local em `jr` (rs → reg_idx) | Verde |
| 2 | Reordenar funções `beq` e `bne` | Verde |

Placar: **6/6 mutantes pegos, 2/2 controles verdes**.

## Placar antes → depois

- Workspace: 98 → **127** testes (29 novos de branch/delay + 0 existentes quebrados)

## Revisão cruzada (orquestrador)

<!-- preenchido pelo Claude na revisão do PR -->

## Decisões e notas

- O branch delay slot foi implementado via um campo `branch_target: Option<u32>` na CPU.
  O `step()` lê a instrução, aplica `branch_target` se houver (redirecionando o PC),
  executa a instrução, e then comita load delays. Isso garante que a instrução no delay
  slot (lida no PC do step anterior) executa antes do redirecionamento.
- JAL/JALR/BLTZAL/BGEZAL salvam `self.pc + 4` como link address. Como `self.pc` já foi
  incrementado no início do step, isso resulta em PC_original + 8, que é o endereço de
  retorno após o delay slot (correto).
- Nota 3 do STATUS (load delay vs escrita no mesmo reg) permanece inalterada.
