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
| 7 | cobertura (teste) | O teste `sr_e_empilhado_na_entrada_da_excecao` usava apenas SR=0x03, que tem os bits 2-5 zerados. O literal 0x03 não distingue a limpeza correta dos bits 2-5 (máscara `!0x3F`) da limpeza insuficiente (`!0x3`), porque o OR dos bits shiftados esconde o defeito quando os bits 2-5 de origem são zero. **O defeito era de TESTE, não de implementação.** | O segundo caso com SR=0x0040_0031 (bits 4-5=11, bit 22=1) expõe o mutante: o resultado correto é 0x0040_0004, o mutante produz 0x0040_0034. | Segunda rodada de revisão adversarial (PR #35, mutante M-A). Adicionado segundo caso SR=0x0040_0031 → 0x0040_0004 ao teste existente.
| 8 | cobertura (teste) | Nenhum teste do item disparava duas exceções em sequência. Se a primeira exceção ocorre em delay slot (BD=1, BT=1), os bits 31-30 do CAUSE podem ficar velhos na segunda exceção se a máscara de pré-limpeza esquecer de limpá-los. **O defeito era de TESTE, não de implementação.** | O código usa máscara `!0xC000_007C` que limpa BD e BT corretamente. O mutante `!0x0000_007C` mantém os bits 31-30 entre exceções. | Segunda rodada de revisão adversarial (PR #35, mutante M-B). Adicionado teste `excecao_sequencial_limpa_bd_e_bt`.

## Bateria de mutação

Placar: **12/12 mutantes pegos, 3/3 controles verdes.**

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

### Mutantes da segunda rodada de revisão adversarial (PR #35)

| # | Tipo | Mutação | Teste que pegou |
|---|---|---|---|
| 11 | Mutante | Máscara do push do SR `!0x3F` → `!0x3` (limpa só bits 0-1, não bits 2-5) | `sr_e_empilhado_na_entrada_da_excecao` (segundo caso: SR=0x0000_FF31 → 0x0000_FF04) |
| 12 | Mutante | Máscara do CAUSE `!0xC000_007C` → `!0x0000_007C` (não limpa BD/BT entre exceções) | `excecao_sequencial_limpa_bd_e_bt` |

### Controles

| # | Tipo | Mutação | Resultado |
|---|---|---|---|
| C1 | Controle | Reordenar match arms em `special()` (MFHI/MFLO ↔ MTHI/MTLO) | Todos passaram |
| C2 | Controle | Reordenar match arms em `cop0_op()` (MFC0 ↔ MTC0) | Todos passaram |
| C3 | Controle | Renomear local `in_delay_slot` para `is_delay` | Todos passaram |

## Placar antes → depois

Workspace: **188 → 203** testes (+15). Os 10 originais + 5 novos (E1, E2, E2b, E3, M-B sequencial).

## Por que a suite original não pegou as três lacunas

Os 10 testes originais cobrem exatamente os 5 casos de aceitação do handoff do STATUS (B1-B5 + variações). As três lacunas estão **fora** do escopo desses testes:

- **E1 (Sw bits):** Nenhum teste original escrevia nos bits Sw do CAUSE antes de disparar uma exceção. O B1-B5 verificam CAUSE após exceção partindo do estado resetado (cop0[13] = 0). É o formato esperado: o handoff lista o que o item deve fazer, e os testes de aceitação cobrem exatamente esse escopo, nada além.
- **E2 (SR push):** Nenhum teste original lia o SR após uma exceção. Os testes B1-B5 verificam CAUSE, EPC, BadVaddr e PC — mas não o SR. A spec local só documenta o RFE, não o push; o push é uma inferência lógica (se RFE desempilha, algo precisa empilhar), mas não estava no escopo declarado.
- **E3 (load delay):** Nenhum teste original combinava load + exceção na instrução seguinte. O B4 (BD + syscall) usa `JAL` + `syscall`, sem load pendente.

A lição é sobre o formato do handoff dirigido por testes de aceitação: ele garante o que foi pedido e nada além. As lacunas de borda (estado prévio do CAUSE, SR significativo, load pendente) são exatamente o tipo de coisa que testes de integração mais amplos (como o Amidog `psxtest_cpu`, item 1.11) exercitariam. Isso não é falha do trabalhador — o código passava em todos os testes pedidos. É uma característica do processo: o handoff define o "o que", os testes de aceitação verificam o "o que", e revisões adversariais encontram o "o que mais".

A primeira rodada de revisão adversarial achou três lacunas (E1/E2/E3) porque nenhum teste combinava exceção com ESTADO PRÉVIO; esta segunda rodada achou duas (M-A/M-B) porque nenhum teste combinava exceção com ESTADO DEIXADO POR OUTRA EXCECÃO. O eixo é o mesmo — teste de aceitação verifica transição a partir do reset.

**Regra do literal (terceira ocorrência no projeto).** Um literal que verifica limpeza-antes-de-OR precisa de um bit em 1 na origem onde o resultado correto exige 0, senão o OR esconde a máscara errada. Ocorrências: (1) iteração 1.8a com SR=0x34 na verificação de `rfe`; (2) esta iteração com SR=0x03 no push do SR (E2 inicial); (3) esta mesma iteração com SR=0x3F no round-trip (E2b). Em todos os casos, o literal tinha os bits-alvo zerados na origem, e o OR de `ie_ku_shifted` (ou `rfe` shift) colocava os valores corretos nos bits de destino, mascarando o defeito. A correção é sempre a mesma: escolher um literal que tenha 1 nos bits que a máscara deve limpar, para que a falha de limpeza apareça como diferença no resultado.

## Revisão cruzada (orquestrador)

Três comentários na PR #35 e **duas rodadas de correção** — único item do M1 até aqui que
precisou de duas. O motivo é registrável, e não é o óbvio.

### Como a implementação foi verificada

1. **Sondas escritas antes de ler o código do PR**, não commitadas. Rodada 1: três cenários
   deliberadamente fora dos casos de aceitação B1–B5. **Os três falharam.** Rodada 2: os mesmos
   três re-executados sobre a correção.

   | Sonda | Rodada 1 | Rodada 2 |
   |---|---|---|
   | `cop0[13] & 0x300` após `syscall` | `0x000` | `0x300` |
   | `SR = 0x3` → `syscall` | `0x3` | `0xC` |
   | `lw r5` → `syscall` | `0` | `0xCAFE_BABE` |

2. **Álgebra do empilhamento conferida contra o `rfe` da 1.8a.** `(sr & !0x3F) | ((sr & 0x3) <<
   2) | ((sr & 0xC) << 2)` é o inverso exato do desempilhamento, e o `sr_push_seguido_de_rfe_...`
   prova isso empiricamente.
3. **Mutação re-executada pelo orquestrador**, não conferida na tabela. Dos três mutantes
   próprios, dois escapavam da suíte inteira (11 e 12 da tabela acima).
4. **Sonda de escopo:** `syscall` não executa o opcode seguinte e `EPC` aponta para o próprio
   `syscall`. Passa. A garantia é estrutural — `step()` executa uma instrução por chamada — mas
   a spec cita o caso ("immediately executed, ie. without executing the following opcode") e a
   arquitetura pode mudar.

### A leitura do processo

**A implementação convergiu em uma rodada; a suíte de testes não.** A separação importa, porque
a leitura preguiçosa ("duas rodadas, logo a revisão não converge") é falsa: os achados da rodada
2 são de classe estritamente mais fraca. Nenhum comportamento errado, só rede de regressão
furada. Um mutante que escapa não quebra o emulador hoje — tira a capacidade de detectar quem o
quebrar amanhã.

Regra nova de handoff, válida a partir da 1.9: **todo item que escreve em registrador
persistente precisa de um caso de aceitação com o registrador sujo antes.** As duas rodadas
acharam no mesmo eixo (estado prévio; estado deixado por outra exceção) porque teste de
aceitação verifica transição a partir do reset — é assim que se escreve um caso mínimo.

Regra nova de registro: **nota ASSUMIDA do STATUS é referenciada por título, não por número.**
A seção Notas foi renumerada neste PR (as dívidas 2 e 5 fecharam com o 1.8b) e as quatro
referências cruzadas do doc e das mensagens de `assert` ficaram apontando para a numeração
antiga. Zero efeito em comportamento, mas quem for resolver as notas ASSUMIDAS no item 1.11
abriria a nota errada — e o registro é entregável de mesmo peso que o emulador aqui. Renumerar
lista referenciada por ponteiro tem raio maior que o arquivo editado, e o número é volátil por
construção: toda dívida fechada renumera as de baixo.

### Mérito do trabalho

- Os três erros de primeira tentativa da tabela original são honestos e específicos, não
  "nenhum". O nº 1 (`branch_target.is_some()` não detecta delay slot de branch condicional não
  tomado) é achado genuíno do próprio trabalhador, e separar `delay_slot_pending` de
  `branch_taken` é a correção certa.
- A armadilha 4 do handoff — a máscara de escrita do CAUSE criada na 1.8a engolindo o ExcCode —
  **não** foi pisada.
- O mutante 9 da bateria admite que o teste E2 simples sobrevive e que só o E2b o pega. Bateria
  que reporta o próprio ponto cego vale mais que placar liso.

### Erros do orquestrador nesta iteração

| # | Categoria | O que eu errei | Como foi pego |
|---|---|---|---|
| 1 | derivação | O literal `SR = 0x0040_0031` que especifiquei na rodada 2 usa o **bit 22 (BEV)** como "bit que prova que o resto do SR sobrevive". BEV tem semântica: com BEV=1 os vetores mudam para `0xBFC0_0180`. O teste só afirma sobre `cop0[12]`, então está correto hoje — mas é armadilha plantada para quando o BEV for implementado. | Reli a tabela do `cop0r12 - SR` ao preparar o handoff do 1.9. Trocado para `0x0000_FF31` → `0x0000_FF04`, usando o campo Im (8-15), que precisa sobreviver de qualquer jeito e não muda fluxo. Mesma força contra o mutante 11. |
| 2 | processo | Despachei a rodada 2 antes de conferir as referências cruzadas do STATUS, então o achado M-C chegou com a rodada já em voo e não pôde entrar nela. | Óbvio ao reler o doc para preencher esta seção. |

**Edição direta do orquestrador nesta branch, declarada:** a troca do literal do mutante 11 e as
quatro referências cruzadas foram aplicadas por mim, não pelo trabalhador — são um literal
derivado por mim (erro meu) e quatro strings sem efeito em comportamento. Uma terceira rodada
custaria mais do que registra. O papel geral segue valendo: código de emulação é do trabalhador.

## Decisões e notas

1. **Flags de delay slot separados de branch_target.** `delay_slot_pending` é setado por TODO branch/jump (inclusive condicionais não tomados). `branch_taken` indica se o branch foi tomado (para o bit BT do CAUSE). `branch_target` continua como antes (só setado quando o desvio é tomado). Isso resolve a nota 5 do STATUS.

2. **ExcCode escrito direto no CAUSE, sem passar por `cop0_write`.** O mecanismo de exceção grava `self.cop0[13]` diretamente, sem usar `cop0_write()`, que tem máscara de escrita limitada aos bits 8-9. Esta era a armadilha 4 do handoff e foi evitada corretamente.

3. **ADDI overflow fecha a dívida da nota 2 do STATUS.** `addi()` agora usa `i32::checked_add` e dispara exceção Ovf em vez de wrapping.

4. **ADD e SUB implementados com overflow.** Secondary 0x20 (ADD) e 0x22 (SUB) agora existem com trap de overflow, fechando a dívida da nota 2.

5. **Dívidas NÃO fechadas (1.8c ou 1.11):** Reserved Instruction (0Ah) para registradores N/A do COP0 e cop0cmd inválido; Coprocessor Unusable (0Bh) para COP1/COP3 e COP2 com SR.CU2=0; address error por acesso a KUSEG em user mode; interrupções externas (IRQ — M3).

6. **BT bit setado para branches tomados.** `branch_taken` é true para J/JAL/JR/JALR (sempre tomam) e para branches condicionais quando a condição é verdadeira. Para branches condicionais não tomados, `delay_slot_pending` é true mas `branch_taken` é false → BD=1, BT=0.

7. **E1 — CAUSE preserva bits Sw (8-9) e IP (10-15).** Correção aplicada na revisão adversarial (PR #35). A gravação do ExcCode agora usa máscara: `self.cop0[13] = (self.cop0[13] & !0xC000_007C) | cause`, preservando os bits que a exceção não define. O erro original (`self.cop0[13] = cause`) zerava os bits Sw, o que contradiz a spec: "Clear them before returning from the exception handler" só faz sentido se o hardware não limpar sozinho.

8. **E2 — Empilhamento de SR na entrada da exceção (comportamento ASSUMIDO).** O inverso exato do RFE: bits 0-1 (IEc/KUc) → bits 2-3 (IEp/KUp), bits 2-3 (IEp/KUp) → bits 4-5 (IEo/KUo), bits 0-1 zerados. A spec local NÃO documenta o push, apenas o RFE (que desempilha). Ponto de resolução: Amidog `psxtest_cpu` no item 1.11. Ver a nota "E2 — empilhamento de SR" do STATUS.

9. **E3 — Load delay commitado antes da exceção (comportamento ASSUMIDO).** Escolha (a): commitar o load pendente antes de entrar na exceção, com o argumento de que o acesso à memória do `lw` já ocorreu quando a exceção da instrução seguinte é reconhecida. A spec local não tem evidência sobre este caso (R1). Ponto de resolução: Amidog `psxtest_cpu` no item 1.11. Ver a nota "E3 — load delay commitado" do STATUS.
