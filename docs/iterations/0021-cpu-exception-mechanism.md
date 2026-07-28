# 0021 — cpu-exception-mechanism

- **Data:** 2026-07-27
- **Item do roadmap:** 1.8b
- **Objetivo:** Mecanismo de exceção: overflow, syscall, break, AdEL/AdES, bit BD.

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| psx-spx | § arithmetic instructions | docs/reference/02-cpu.md |
| psx-spx | § exception opcodes | docs/reference/02-cpu.md |
| psx-spx | § Coprocessor Instructions | docs/reference/02-cpu.md |
| psx-spx | § COP0 - Exception Handling (CAUSE, SR, EPC, RFE, BadVaddr) | docs/reference/02-cpu.md |
| psx-spx | § Exception Vectors | docs/reference/02-cpu.md |
| psx-spx | § Exception Priority | docs/reference/02-cpu.md |
| psx-spx | § Illegal Opcodes | docs/reference/02-cpu.md |

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | flags | Assumi que `branch_target.is_some()` bastava para detectar delay slot. Mas branch-target só é setado quando o branch é TAKEN; branches condicionais não tomados também têm delay slot. | Todo branch/jump tem delay slot, tomado ou não. BD deve ser 1 para exceção em qualquer delay slot. | Adicionei `delay_slot_pending` e `branch_taken` como flags separados, setados por TODOS os branches. |
| 2 | enderecamento | Escrevi o vetor do break como `0x80000040` via match `if exc_code == 0x09`, mas esqueci que o teste B3 exige literalmente `0x80000040`. A lógica estava correta, mas eu quase usei `0x80000080` para tudo. | `Exception Vectors` tabela: "COP0 Break" → `80000040h`. | Pego pela revisão do código antes do commit; o teste B3 verifica explicitamente `PC = 0x80000040`. |
| 3 | API-Rust | Loads (`lb`, `lh`, `lw`, etc.) tomavam `&self`; precisei mudar para `&mut self` porque `raise_exception()` precisa de `&mut self`. | N/A (decisão de design). | Erro de compilação ao chamar `self.raise_exception()` dentro dos loads. |

## Bateria de mutação

Placar: **6/6 mutantes pegos, 2/2 controles verdes.**

| # | Tipo | Mutação | Teste que pegou |
|---|---|---|---|
| 1 | Mutante | ADD sem overflow check (`wrapping_add` em vez de `checked_add`) | `overflow_em_add_seta_cause_ovf_e_nao_escreve_rt` |
| 2 | Mutante | ADDI sem overflow check | `addi_overflow_seta_cause_ovf_e_nao_escreve_rt` |
| 3 | Mutante | break vai para vetor geral (0x80000080) em vez de 0x80000040 | `break_desvia_para_vetor_cop0_break_nao_para_vetor_geral` |
| 4 | Mutante | BD sempre = 0 (não seta bit 31 no CAUSE) | `excecao_em_delay_slot_seta_bd_e_epc_aponta_para_o_branch` |
| 5 | Mutante | EPC no delay slot = `instr_pc` em vez de `instr_pc - 4` | `excecao_em_delay_slot_seta_bd_e_epc_aponta_para_o_branch` |
| 6 | Mutante | LW sem verificação de alinhamento | `load_word_desalinhado_dispara_adel` |
| C1 | Controle | Reordenar match arms em `special()` (MFHI/MFLO ↔ MTHI/MTLO) | Todos passaram |
| C2 | Controle | Reordenar match arms em `cop0_op()` (MFC0 ↔ MTC0) | Todos passaram |

## Placar antes → depois

Workspace: **188 → 198** testes (+10).

## Revisão cruzada (orquestrador)

<!-- Preenchido pelo Claude na revisão do PR -->

## Decisões e notas

1. **Flags de delay slot separados de branch_target.** `delay_slot_pending` é setado por TODO branch/jump (inclusive condicionais não tomados). `branch_taken` indica se o branch foi tomado (para o bit BT do CAUSE). `branch_target` continua como antes (só setado quando o desvio é tomado). Isso resolve a nota 5 do STATUS.

2. **ExcCode escrito direto no CAUSE, sem passar por `cop0_write`.** O mecanismo de exceção grava `self.cop0[13]` diretamente, sem usar `cop0_write()`, que tem máscara de escrita limitada aos bits 8-9. Esta era a armadilha 4 do handoff e foi evitada corretamente.

3. **ADDI overflow fecha a dívida da nota 2 do STATUS.** `addi()` agora usa `i32::checked_add` e dispara exceção Ovf em vez de wrapping.

4. **ADD e SUB implementados com overflow.** Secondary 0x20 (ADD) e 0x22 (SUB) agora existem com trap de overflow, fechando a dívida da nota 2.

5. **Dívidas NÃO fechadas (1.8c ou 1.11):** Reserved Instruction (0Ah) para registradores N/A do COP0 e cop0cmd inválido; Coprocessor Unusable (0Bh) para COP1/COP3 e COP2 com SR.CU2=0; address error por acesso a KUSEG em user mode; interrupções externas (IRQ — M3).

6. **BT bit setado para branches tomados.** `branch_taken` é true para J/JAL/JR/JALR (sempre tomam) e para branches condicionais quando a condição é verdadeira. Para branches condicionais não tomados, `delay_slot_pending` é true mas `branch_taken` é false → BD=1, BT=0.
