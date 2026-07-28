# STATUS

> Memória do projeto entre iterações. O contexto do agente é descartado a cada iteração;
> este arquivo não. Teto de 16 KB imposto por `status_size.rs`. Curto e verdadeiro.

## Última iteração concluída

**0020** — COP0: banco de registradores + `MTC0`/`MFC0` + `RFE` (ROADMAP 1.8a): 10 testes,
6/6 mutantes pegos, 2/2 controles verdes. Banco COP0 com 32 registradores inicializado
(PRID=0x2), CAUSE com máscara de escrita nos bits 8-9, RFE com cópia correta dos campos
IE/KU (bit2-3→bit0-1, bit4-5→bit2-3, bits 4-5 inalterados), MFC0 com load delay de 1
opcode, MTC0 sem store delay. EPC e BadVaddr implementados como graváveis (comportamento
ASSUMIDO, resolve no item 1.11). Registradores N/A e garbage não disparam exceção (dívida
do 1.8b). Ver `docs/iterations/0020-cpu-cop0-regs.md`.

## Próxima tarefa

**ROADMAP 1.8b** — Mecanismo de exceção: overflow, syscall, break, AdEL/AdES, bit BD.

A base COP0 está pronta (1.8a). Este item conecta os gatilhos de exceção ao banco de
registradores e implementa o fluxo de entrada em exceção.

**Escopo:** `syscall` (primary 0x00, secondary 0x0C), `break` (primary 0x00, secondary
0x0D), overflow trap em `ADD` (secondary 0x20) e `ADDI` (primary 0x08), address error
(AdEL/AdES) em load/store desalinhados, bit BD no CAUSE para exceções em delay slot,
e o **desvio para o vetor** — o vetor É o mecanismo; sem transferência de controle o item
não tem sentido.

**NÃO inclui** (correção de escopo do orquestrador, 2026-07-27): interrupções externas
(IRQ — M3); handler de exceção em si; **Reserved Instruction (0Ah)** para os registradores
N/A do COP0 e cop0cmd inválido; **Coprocessor Unusable (0Bh)** para COP1/COP3 e para COP2
com `SR.CU2=0`; acesso a KUSEG em user mode. Essas quatro viram dívida com nota própria.
Motivo: BD e delay slot são a parte arriscada deste item; somar mais duas famílias de causa
alarga o raio de explosão, que é exatamente o que a divisão do 1.8 evitou.

**O que muda na CPU:**
- `Cpu` ganha flag `in_delay_slot: bool` e `branch_pc: Option<u32>` para setar
  `CAUSE.BD` e `EPC = branch_pc` (não `pc`) quando a exceção ocorre num delay slot.
- `step()` detecta exceção e, em vez de executar a instrução, seta `CAUSE.ExcCode`,
  `CAUSE.BD`, `CAUSE.CE` (se coprocessor), `EPC` (ou `EPC-4` se BD=1), e
  `BadVaddr` (se AdEL/AdES), e desvia para o vetor de exceção (0x80000080 com BEV=0).
- `ADD` (secondary 0x20) e `ADDI` (primary 0x08): overflow trap → ExcCode=0Ch (Ovf),
  deixa `rt` intacto.
- `syscall` (0x0C): ExcCode=08h.
- `break` (0x0D): ExcCode=09h.
- Load/store desalinhado: ExcCode=04h (AdEL) / 05h (AdES), com `BadVaddr` escrito.

**Spec:** `docs/reference/02-cpu.md` — `exception opcodes` (L409), `COP0 - Exception
Handling` (L589), `Exception Vectors` (L736), `Exception Priority` (L752),
`Coprocessor Instructions` (L422), `arithmetic instructions` (L285), `Illegal Opcodes`
(L148).

### Armadilhas nomeadas

1. **Ordem de prioridade de exceção importa.** A spec lista `Reset > AdEL > AdES > DBE >
   ...`. Se duas condições de exceção ocorrerem na mesma instrução, a de maior
   prioridade vence. Mas na prática do 1.8b, a maioria das exceções são mutuamente
   exclusivas (não dá pra ter overflow E syscall na mesma instrução). Ainda assim,
   implemente a cadeia de `if/else if` na ordem da spec.
2. **BD: EPC aponta para o branch, não para o delay slot.** Quando `CAUSE.BD=1`, `EPC`
   contém o endereço do branch (ou seja, `EPC = pc - 4` relativo ao delay slot). O
   handler de exceção precisa dessa informação para decidir se re-executa o branch ou
   avança. A nota 5 do STATUS já documenta a dívida do flag `in_delay_slot`.
3. **Overflow trap: `rt` fica intacto.** Diferente de `ADDU`, o `ADD` e `ADDI` NÃO
   escrevem `rt` quando há overflow. O registrador destino mantém o valor anterior. A
   nota 2 do STATUS documenta a dívida atual (`ADDI` idêntico a `ADDIU`).
4. **A máscara de escrita do CAUSE, criada na 1.8a, vai engolir o ExcCode se você passar
   por ela.** `cop0_write(13, ...)` só grava os bits 8-9 — é o comportamento correto para
   `MTC0` e está coberto por teste. O mecanismo de exceção **não** é um `MTC0`: ele escreve
   `self.cop0[13]` direto. Passar pelo `cop0_write` faz o ExcCode sumir em silêncio, com
   todos os testes da 1.8a continuando verdes. Esta é a armadilha mais provável do item, e
   ela nasceu de a 1.8a estar certa.
5. **`syscall` e `break` estão no espaço `special` (primary 0x00).** Secondary 0x0C e
   0x0D respectivamente. O `unimplemented!` atual em `special()` os pegaria — troque
   por exceção.

### Testes de aceitação OBRIGATÓRIOS

Literais derivados pelo orquestrador por duas rotas (regra da 0017e). **Exija o valor do
CAUSE inteiro, não só o campo ExcCode** — o erro provável é gravar o ExcCode sem deslocar
(`0x0C` em vez de `0x30`), e um teste que lê só o campo não distingue os dois.
Rota 1: ExcCode ocupa os bits 2-6, logo `CAUSE = ExcCode << 2`. Rota 2 para cada valor abaixo,
em binário: `08h = 01000b` → `0100000b` = `0x20`; `09h = 01001b` → `0100100b` = `0x24`;
`0Ch = 01100b` → `0110000b` = `0x30`; `04h` → `0x10`; `05h` → `0x14`.

**B1 — Overflow em ADD.** `r8 = r9 = 0x7FFF_FFFF`, executar `ADD r10, r8, r9`.
Exigido: `r10` **inalterado**, `CAUSE = 0x0000_0030`, `EPC` = endereço do `ADD`,
`PC = 0x8000_0080`.

**B2 — syscall.** Exigido: `CAUSE = 0x0000_0020`, `EPC` = endereço do `syscall`,
`PC = 0x8000_0080`.

**B3 — break vai para OUTRO vetor.** Exigido: `CAUSE = 0x0000_0024` e
**`PC = 0x8000_0040`**, não `0x8000_0080`. A tabela `Exception Vectors` (L736) dá linha
própria para "COP0 Break" com BEV=0. Mandar `break` para o vetor geral é o erro clássico
deste item e é o único caso em que a suíte inteira pode ficar verde com o desvio errado —
por isso este teste é obrigatório.

**B4 — BD no delay slot.** `JAL` seguido de `syscall` no delay slot. Exigido:
`CAUSE = 0xC000_0020` (BD no bit31, BT no bit30, ExcCode 08h nos bits 2-6) e
**`EPC` = endereço do `JAL`**, não do `syscall` — "EPC is set to the address of the
exception - 4".

**B5 — Load desalinhado dispara AdEL.** `LW` em endereço não múltiplo de 4. Exigido:
`CAUSE = 0x0000_0010`, `rt` inalterado, `BadVaddr` = o endereço desalinhado,
`PC = 0x8000_0080`. Espelhe com `SW` desalinhado: `CAUSE = 0x0000_0014`.
A spec diz que `BadVaddr` é atualizado **só** por AdEL/AdES — os outros casos acima devem
deixá-lo intacto, e isso vale um assert.

### Dívidas que este item fecha

- Nota 2 do STATUS (overflow trap em ADDI/ADD/SUB)
- Nota 5 do STATUS (flag `in_delay_slot` para bit BD)
- `unimplemented!` em `special()` para opcodes 0x0C (syscall) e 0x0D (break)

**Dívidas que este item NÃO fecha** (ficam para um 1.8c ou para o 1.11): Reserved Instruction
(0Ah) nos registradores N/A do COP0 e em cop0cmd inválido; Coprocessor Unusable (0Bh) em
COP1/COP3 e em COP2 com `SR.CU2=0` (nota 9); address error por acesso a KUSEG em user mode.

## Repositório

- `main` protegida a partir da iter 0004; 1 PR por item; merge commit (nunca squash);
  commits test→feat→docs; título de PR validado pela CI.
- Iterações são cronológicas e nem sempre na ordem dos itens (0003↔item 0.5, p.ex.);
  o vínculo real está no título do PR e no doc da iteração.

## Placar de testes

Workspace: **188** testes (10 meta-testes + 8 bus_bios + 2 bios_flag + 1 version + 12 bus_scheduler + 8 cpu_fetch_decode + 26 cpu_alu + 14 cpu_shifts + 19 cpu_load_delay + 24 cpu_branches + 7 cpu_jumps + 20 cpu_mult_div + 27 cpu_unaligned_load_store + 10 cpu_cop0_regs).

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
2. **Dívida de overflow trap (fecha no item 1.8).** `ADDI` está implementado sem trap,
   idêntico a `ADDIU` — a spec (02-cpu.md, `arithmetic instructions`) manda excetuar e
   deixar `rt` intacto no overflow. `ADD` (secondary 0x20) e `SUB` (0x22) nem existem no
   match: dão `unimplemented!`. Quando o 1.8 trouxer o mecanismo de exceção, os três
   entram juntos. Autorizado no handoff da 0012, mas é dívida, não comportamento correto.
3. **Load delay × escrita no mesmo registrador: comportamento ASSUMIDO, não verificado
   (resolve no item 1.11).** Quando a instrução do delay slot escreve o registrador
   destino do load (`lw r10,..` seguido de `ori r10,..`), a nossa implementação faz o
   **load vencer**. A spec local não decide: `02-cpu.md § Caution - Load Delay` só diz que
   o registrador "não é atualizado até o próximo opcode ter completado", o que fala de
   leitura, não de precedência de escrita. Não mudamos sem evidência (R1). O teste
   `load_delay_vs_escrita_no_mesmo_registrador_comportamento_assumido` fixa o que fazemos
   hoje e nomeia a dúvida, para que uma futura mudança seja deliberada. Ponto de
   resolução: Amidog `psxtest_cpu` no item 1.11 — se ele reprovar, inverter a ordem em
   `Cpu::step` (commitar o load antes de executar, escrevendo num banco de saída).
4. **BcondZ com `rt` fora da tabela: comportamento ASSUMIDO (resolve no item 1.11).** O
   opcode 01h só tem `rt`=00h/01h/10h/11h tabelados em `02-cpu.md § Opcode/Parameter
   Encoding`; a spec local não diz o que `rt`=02h..0Fh/12h..1Fh fazem. Assumimos **no-op
   silencioso** (nem desvia nem linka). O teste
   `bcondz_rt_fora_da_tabela_comportamento_assumido` fixa isso e diz na asserção que é
   suposição. Se o Amidog `psxtest_cpu` reprovar, o critério a testar primeiro é o de
   hardware conhecido: bit16 sozinho decide BLTZ/BGEZ e o link ocorre quando os bits
   20..17 valem 1000b — o que faria `rt`=02h agir como BLTZ.
5. **Dívida do bit BD / delay slot para o item 1.8.** `Cpu` sinaliza desvio pendente com
   `branch_target: Option<u32>`, consumido em `step` ANTES de executar a instrução. Isso
   basta para o desvio, mas apaga a informação "a instrução atual está num delay slot",
   que o 1.8 precisa para setar `CAUSE.BD` e apontar `EPC` para o branch (e não para o
   delay slot) — a própria spec cita o caso em `§ JALR cautions`. Quem fizer o 1.8 tem de
   guardar esse flag junto com o endereço do branch, não deduzi-lo depois.
6. **EPC e BadVaddr são graváveis via MTC0 — comportamento ASSUMIDO (resolve no item
   1.11).** A spec marca ambos como (R), mas o comportamento sob escrita não está
   documentado localmente. Implementados como R/W na 1.8a. Os testes
   `epc_gravavel_comportamento_assumido` e `badvaddr_gravavel_comportamento_assumido`
   fixam o comportamento atual. Se o Amidog `psxtest_cpu` reprovar, adicionar `if reg ==
   8 || reg == 14 { return; }` em `cop0_write`.
7. **TAR (cop0r6) é R/W — comportamento ASSUMIDO (resolve no item 1.11).** Mesmo
   critério de EPC/BadVaddr: spec marca (R), implementado como R/W sem evidência
   contrária.
8. **Registradores N/A do COP0 (r0-r2, r4, r10, r32-r63) não disparam exceção — dívida
   do 1.8b.** Leitura retorna 0, escrita é ignorada. O comportamento correto é Reserved
   Instruction Exception (excode=0Ah).
9. **Acesso ao COP0 em User mode com COP0 disabled — dívida do 1.8b.** Acessar qualquer
   registrador do COP0 que não seja garbage (r16-r31), ou executar RFE, em User mode com
   COP0 disabled (SR.bit1=1 e SR.bit28=0) gera Coprocessor Unusable Exception (excode=0Bh).
   Os registradores garbage r16-r31 podem ser acessados nesse estado sem exceção. Fonte:
   `docs/reference/02-cpu.md`, seção cop0r16-r31 - Garbage (L805).
