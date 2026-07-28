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
| 4 | COP0-mask | `self.cop0[13] = cause` sobrescrevia o registrador inteiro, zerando os bits Sw (8-9) e IP (10-15). | A spec diz que os bits 8-9 são "Write to these bits to manually cause an exception. Clear them before returning from the exception handler" — a instrução de limpar em software só faz sentido se o hardware não limpar sozinho. | Revisão adversarial do orquestrador (PR #35). Corrigido com máscara: `self.cop0[13] = (self.cop0[13] & !0xC000_007C) \| cause`. |
| 5 | SR-stack | Não havia empilhamento de SR na entrada da exceção — `RFE` desempilha mas nada empilhava. | Comportamento ASSUMIDO: o inverso exato do RFE. A spec local só documenta o RFE, não o push. | Revisão adversarial do orquestrador (PR #35). Implementado push: bits 0-1 → 2-3, bits 2-3 → 4-5, bits 0-1 zerados. |
| 6 | load-delay | Load pendente era descartado pela exceção — o `return` antecipado no `step()` ocorria antes de `self.load_delay` ser commitado. | Comportamento ASSUMIDO: o acesso à memória do `lw` já ocorreu quando a exceção da instrução seguinte é reconhecida, então o valor pendente deve ser commitado antes de entrar na exceção. | Revisão adversarial do orquestrador (PR #35). Corrigido commitando o load delay antes do `return`.

## Bateria de mutação

Placar: **10/10 mutantes pegos, 3/3 controles verdes.**

### Mutantes originais (iteração inicial)

| # | Tipo | Mutação | Teste que pegou |
|---|---|---|---|
| 1 | Mutante | ADD sem overflow check (`wrapping_add` em vez de `checked_add`) | `overflow_em_add_seta_cause_ovf_e_nao_escreve_rt` |
| 2 | Mutante | ADDI sem overflow check | `addi_overflow_seta_cause_ovf_e_nao_escreve_rt` |
| 3 | Mutante | break vai para vetor geral (0x80000080) em vez de 0x80000040 | `break_desvia_para_vetor_cop0_break_nao_para_vetor_geral` |
| 4 | Mutante | BD sempre = 0 (não seta bit 31 no CAUSE) | `excecao_em_delay_slot_seta_bd_e_epc_aponta_para_o_branch` |
| 5 | Mutante | EPC no delay slot = `instr_pc` em vez de `instr_pc - 4` | `excecao_em_delay_slot_seta_bd_e_epc_aponta_para_o_branch` |
| 6 | Mutante | LW sem verificação de alinhamento | `load_word_desalinhado_dispara_adel` |

### Mutantes novos (correções pós-revisão adversarial)

| # | Tipo | Mutação | Teste que pegou |
|---|---|---|---|
| 7 | Mutante | CAUSE sobrescrito sem máscara (`self.cop0[13] = cause`) | `excecao_preserva_bits_sw_do_cause` |
| 8 | Mutante | Push do SR removido (comentado) | `sr_e_empilhado_na_entrada_da_excecao` + `sr_push_seguido_de_rfe_restaura_os_bits_0_3` |
| 9 | Mutante | Shift errado no push (`iep_kup << 2` em vez de `(sr & 0xC) << 2` — IEp vai para bits 2-3, não 4-5) | `sr_push_seguido_de_rfe_restaura_os_bits_0_3` (o E2 simples com SR=0x03 sobrevive, prova que o E2b é necessário) |
| 10 | Mutante | Load delay não commitado antes da exceção (removido o `if let Some`) | `load_pendente_e_commitado_antes_da_excecao_comportamento_assumido` |

### Controles

| # | Tipo | Mutação | Resultado |
|---|---|---|---|
| C1 | Controle | Reordenar match arms em `special()` (MFHI/MFLO ↔ MTHI/MTLO) | Todos passaram |
| C2 | Controle | Reordenar match arms em `cop0_op()` (MFC0 ↔ MTC0) | Todos passaram |
| C3 | Controle | Renomear local `in_delay_slot` para `is_delay` | Todos passaram |

## Placar antes → depois

Workspace: **188 → 202** testes (+14). Os 10 originais + 4 novos (E1, E2, E2b, E3).

## Por que a suite original não pegou as três lacunas

Os 10 testes originais cobrem exatamente os 5 casos de aceitação do handoff do STATUS (B1-B5 + variações). As três lacunas estão **fora** do escopo desses testes:

- **E1 (Sw bits):** Nenhum teste original escrevia nos bits Sw do CAUSE antes de disparar uma exceção. O B1-B5 verificam CAUSE após exceção partindo do estado resetado (cop0[13] = 0). É o formato esperado: o handoff lista o que o item deve fazer, e os testes de aceitação cobrem exatamente esse escopo, nada além.
- **E2 (SR push):** Nenhum teste original lia o SR após uma exceção. Os testes B1-B5 verificam CAUSE, EPC, BadVaddr e PC — mas não o SR. A spec local só documenta o RFE, não o push; o push é uma inferência lógica (se RFE desempilha, algo precisa empilhar), mas não estava no escopo declarado.
- **E3 (load delay):** Nenhum teste original combinava load + exceção na instrução seguinte. O B4 (BD + syscall) usa `JAL` + `syscall`, sem load pendente.

A lição é sobre o formato do handoff dirigido por testes de aceitação: ele garante o que foi pedido e nada além. As lacunas de borda (estado prévio do CAUSE, SR significativo, load pendente) são exatamente o tipo de coisa que testes de integração mais amplos (como o Amidog `psxtest_cpu`, item 1.11) exercitariam. Isso não é falha do trabalhador — o código passava em todos os testes pedidos. É uma característica do processo: o handoff define o "o que", os testes de aceitação verificam o "o que", e revisões adversariais encontram o "o que mais".

## Decisões e notas

1. **Flags de delay slot separados de branch_target.** `delay_slot_pending` é setado por TODO branch/jump (inclusive condicionais não tomados). `branch_taken` indica se o branch foi tomado (para o bit BT do CAUSE). `branch_target` continua como antes (só setado quando o desvio é tomado). Isso resolve a nota 5 do STATUS.

2. **ExcCode escrito direto no CAUSE, sem passar por `cop0_write`.** O mecanismo de exceção grava `self.cop0[13]` diretamente, sem usar `cop0_write()`, que tem máscara de escrita limitada aos bits 8-9. Esta era a armadilha 4 do handoff e foi evitada corretamente.

3. **ADDI overflow fecha a dívida da nota 2 do STATUS.** `addi()` agora usa `i32::checked_add` e dispara exceção Ovf em vez de wrapping.

4. **ADD e SUB implementados com overflow.** Secondary 0x20 (ADD) e 0x22 (SUB) agora existem com trap de overflow, fechando a dívida da nota 2.

5. **Dívidas NÃO fechadas (1.8c ou 1.11):** Reserved Instruction (0Ah) para registradores N/A do COP0 e cop0cmd inválido; Coprocessor Unusable (0Bh) para COP1/COP3 e COP2 com SR.CU2=0; address error por acesso a KUSEG em user mode; interrupções externas (IRQ — M3).

6. **BT bit setado para branches tomados.** `branch_taken` é true para J/JAL/JR/JALR (sempre tomam) e para branches condicionais quando a condição é verdadeira. Para branches condicionais não tomados, `delay_slot_pending` é true mas `branch_taken` é false → BD=1, BT=0.

7. **E1 — CAUSE preserva bits Sw (8-9) e IP (10-15).** Correção aplicada na revisão adversarial (PR #35). A gravação do ExcCode agora usa máscara: `self.cop0[13] = (self.cop0[13] & !0xC000_007C) | cause`, preservando os bits que a exceção não define. O erro original (`self.cop0[13] = cause`) zerava os bits Sw, o que contradiz a spec: "Clear them before returning from the exception handler" só faz sentido se o hardware não limpar sozinho.

8. **E2 — Empilhamento de SR na entrada da exceção (comportamento ASSUMIDO).** O inverso exato do RFE: bits 0-1 (IEc/KUc) → bits 2-3 (IEp/KUp), bits 2-3 (IEp/KUp) → bits 4-5 (IEo/KUo), bits 0-1 zerados. A spec local NÃO documenta o push, apenas o RFE (que desempilha). Ponto de resolução: Amidog `psxtest_cpu` no item 1.11. Ver nota 10 do STATUS.

9. **E3 — Load delay commitado antes da exceção (comportamento ASSUMIDO).** Escolha (a): commitar o load pendente antes de entrar na exceção, com o argumento de que o acesso à memória do `lw` já ocorreu quando a exceção da instrução seguinte é reconhecida. A spec local não tem evidência sobre este caso (R1). Ponto de resolução: Amidog `psxtest_cpu` no item 1.11. Ver nota 11 do STATUS.
