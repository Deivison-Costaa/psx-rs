# STATUS

> Memória do projeto entre iterações. O contexto do agente é descartado a cada iteração;
> este arquivo não. Teto de 16 KB imposto por `status_size.rs`. Curto e verdadeiro.

## Última iteração concluída

**0021 (revisão adversarial)** — Correções do PR #35: CAUSE preserva bits Sw (máscara de escrita só nos campos ExcCode/BD/BT), empilhamento de SR na entrada da exceção (bits 0-1→2-3, 2-3→4-5, 0-1 zerados — inverso do RFE), load delay commitado antes da exceção (o acesso à memória já ocorreu). +4 testes, 4/4 mutantes novos pegos, 1/1 controles verdes. Ver `docs/iterations/0021-cpu-exception-mechanism.md`.

## Próxima tarefa

**ROADMAP 1.8c** — Reserved Instruction (0Ah) e Coprocessor Unusable (0Bh).

A base de exceção e COP0 estão prontas (1.8a + 1.8b). Este item adiciona duas famílias
de exceção que completam o tratamento de opcodes e acessos a coprocessadores.

**Escopo:**
- Reserved Instruction (ExcCode 0Ah) para registradores N/A do COP0 (r0-r2, r4, r10, r32-r63)
  e para cop0cmd inválido (valores diferentes de 0x10 na faixa `co=0x10..=0x1F`).
- Coprocessor Unusable (ExcCode 0Bh) para acesso a COP1/COP3 e para COP2 com SR.CU2=0.
  Os registradores garbage r16-r31 podem ser acessados em user mode com COP0 disabled
  sem exceção (spec L805).
- `CE` (bits 28-29 do CAUSE) setado com o número do coprocessador que causou CpU.
- `BEV` (bit 22 do SR) influencia o vetor de exceção: BEV=0 → KSEG0 (0x8000_xxxx),
  BEV=1 → KSEG1 (0xBFC0_01xx).

**NÃO inclui:** interrupções externas (IRQ — M3); acesso a KUSEG em user mode (AdEL/AdES
por região de memória inválida).

**Spec:** `docs/reference/02-cpu.md` — `cop0r0..r2, cop0r4, cop0r10, cop0r32-r63 - N/A` (L790),
`cop0r16-r31 - Garbage` (L805), `cop0cmd=01h,02h,06h,08h` (L796), `Exception Vectors` (L736),
`cop0r12 - SR` (L624).

### Armadilhas nomeadas

1. **Acesso a registrador garbage (r16-r31) NÃO dispara CpU.** A spec diz explicitamente que
   esses registradores podem ser acessados em user mode com COP0 disabled "sem exceção".
   Um teste que acessa r16 esperando CpU está errado.
2. **Reserved Instruction só para registradores explicitamente marcados N/A.** r0-r2, r4,
   r10, r32-r63. r3 (BPC), r5 (BDA), r6 (TAR), r7 (DCIC), r8 (BadVaddr), r9 (BDAM), r11 (BPCM),
   r12 (SR), r13 (CAUSE), r14 (EPC), r15 (PRID) são válidos.
3. **cop0cmd inválido dispara RI, não CpU.** O match `co=0x10..=0x1F` atualmente ignora
   cop0cmd != 0x10 com `None` — deve disparar Reserved Instruction.

## Repositório

- `main` protegida a partir da iter 0004; 1 PR por item; merge commit (nunca squash);
  commits test→feat→docs; título de PR validado pela CI.
- Iterações são cronológicas e nem sempre na ordem dos itens (0003↔item 0.5, p.ex.);
  o vínculo real está no título do PR e no doc da iteração.

## Placar de testes

Workspace: **202** testes (10 meta-testes + 8 bus_bios + 2 bios_flag + 1 version + 12 bus_scheduler + 8 cpu_fetch_decode + 26 cpu_alu + 14 cpu_shifts + 19 cpu_load_delay + 24 cpu_branches + 7 cpu_jumps + 20 cpu_mult_div + 27 cpu_unaligned_load_store + 10 cpu_cop0_regs + 14 cpu_exception_mechanism).

## Bloqueios

(nenhum)

## Invariantes

1. **Imediato de endereçamento é SINALIZADO.** Todo load/store (`lw/sw/lb/lh/...`) e todo
   `addi/addiu/slti/sltiu` sign-extendem o campo de 16 bits: `(instr & 0xFFFF) as u16 as
   i16 as u32`. Só a família lógica (`andi/ori/xori`) zero-extende. Violado na iter 0011
   (SW), pego na revisão adversarial; qualquer item novo que leia um imediato reconfere
   esta linha antes de escolher a extensão.

## Notas

1. BIOS local: `bios/SCPH1001.BIN` (MD5 924E392ED05558FFDB115408C263DCCF), gitignored,
   validada na iter 0009 (item 0.9). Nunca commitar.
2. **Load delay × escrita no mesmo registrador: comportamento ASSUMIDO, não verificado
   (resolve no item 1.11).** Quando a instrução do delay slot escreve o registrador
   destino do load (`lw r10,..` seguido de `ori r10,..`), a nossa implementação faz o
   **load vencer**. A spec local não decide: `02-cpu.md § Caution - Load Delay` só diz que
   o registrador "não é atualizado até o próximo opcode ter completado", o que fala de
   leitura, não de precedência de escrita. Não mudamos sem evidência (R1). O teste
   `load_delay_vs_escrita_no_mesmo_registrador_comportamento_assumido` fixa o que fazemos
   hoje e nomeia a dúvida, para que uma futura mudança seja deliberada. Ponto de
   resolução: Amidog `psxtest_cpu` no item 1.11 — se ele reprovar, inverter a ordem em
   `Cpu::step` (commitar o load antes de executar, escrevendo num banco de saída).
3. **BcondZ com `rt` fora da tabela: comportamento ASSUMIDO (resolve no item 1.11).** O
   opcode 01h só tem `rt`=00h/01h/10h/11h tabelados em `02-cpu.md § Opcode/Parameter
   Encoding`; a spec local não diz o que `rt`=02h..0Fh/12h..1Fh fazem. Assumimos **no-op
   silencioso** (nem desvia nem linka). O teste
   `bcondz_rt_fora_da_tabela_comportamento_assumido` fixa isso e diz na asserção que é
   suposição. Se o Amidog `psxtest_cpu` reprovar, o critério a testar primeiro é o de
   hardware conhecido: bit16 sozinho decide BLTZ/BGEZ e o link ocorre quando os bits
   20..17 valem 1000b — o que faria `rt`=02h agir como BLTZ.
4. **EPC e BadVaddr são graváveis via MTC0 — comportamento ASSUMIDO (resolve no item
   1.11).** A spec marca ambos como (R), mas o comportamento sob escrita não está
   documentado localmente. Implementados como R/W na 1.8a. Os testes
   `epc_gravavel_comportamento_assumido` e `badvaddr_gravavel_comportamento_assumido`
   fixam o comportamento atual. Se o Amidog `psxtest_cpu` reprovar, adicionar `if reg ==
   8 || reg == 14 { return; }` em `cop0_write`.
5. **TAR (cop0r6) é R/W — comportamento ASSUMIDO (resolve no item 1.11).** Mesmo
   critério de EPC/BadVaddr: spec marca (R), implementado como R/W sem evidência
   contrária.
6. **Registradores N/A do COP0 (r0-r2, r4, r10, r32-r63) não disparam exceção — dívida
   do 1.8c.** Leitura retorna 0, escrita é ignorada. O comportamento correto é Reserved
   Instruction Exception (excode=0Ah).
7. **Acesso ao COP0 em User mode com COP0 disabled — dívida do 1.8c.** Acessar qualquer
   registrador do COP0 que não seja garbage (r16-r31), ou executar RFE, em User mode com
   COP0 disabled (SR.bit1=1 e SR.bit28=0) gera Coprocessor Unusable Exception (excode=0Bh).
   Os registradores garbage r16-r31 podem ser acessados nesse estado sem exceção. Fonte:
   `docs/reference/02-cpu.md`, seção cop0r16-r31 - Garbage (L805).
8. **E1 — Entrada de exceção preserva bits Sw (8-9) e IP (10-15) do CAUSE.** A escrita
   do ExcCode agora usa máscara: `self.cop0[13] = (self.cop0[13] & !0xC000_007C) | cause`,
   gravando apenas BD (bit31), BT (bit30) e ExcCode (bits 2-6). O erro original
   (`self.cop0[13] = cause`) zerava os bits Sw, contradizendo a spec que diz "clear them
   before returning from the exception handler" — instrução de software que só faz sentido
   se o hardware não limpar sozinho. Corrigido na revisão adversarial do PR #35.
9. **E2 — Empilhamento de SR na entrada da exceção: comportamento ASSUMIDO (resolve no
   item 1.11).** O inverso exato do RFE: bits 0-1 (IEc/KUc) → bits 2-3 (IEp/KUp), bits 2-3
   (IEp/KUp) → bits 4-5 (IEo/KUo), bits 0-1 zerados. A spec local NÃO documenta o push,
   apenas o RFE que desempilha. Sem o push, um handler que execute RFE restaura lixo nos
   bits IEc/KUc. Ponto de resolução: Amidog `psxtest_cpu`. Testes:
   `sr_e_empilhado_na_entrada_da_excecao` e `sr_push_seguido_de_rfe_restaura_os_bits_0_3`.
10. **E3 — Load delay commitado antes da exceção: comportamento ASSUMIDO (resolve no
   item 1.11).** O acesso à memória do `lw` já ocorreu quando a exceção da instrução
   seguinte é reconhecida; o valor pendente é commitado antes do desvio para o handler.
   A spec local não tem evidência sobre este caso (R1). Escolha (a) entre duas opções
   igualmente plausíveis. Teste:
   `load_pendente_e_commitado_antes_da_excecao_comportamento_assumido`.
