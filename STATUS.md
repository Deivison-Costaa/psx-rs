# STATUS

> Memória do projeto entre iterações. O contexto do agente é descartado a cada iteração;
> este arquivo não. Teto de 16 KB imposto por `status_size.rs`. Curto e verdadeiro.

## Última iteração concluída

**0018** — LWL/LWR/SWL/SWR — SEGUNDA TENTATIVA (ROADMAP 1.7): 25 testes com golden
values ancorados em bytes literais (não derivados da implementação), 7/7 mutantes pegos,
1/1 controles verdes. Correção dos dois defeitos da PR #27: (1) vias de byte por
deslocamento, não máscara; (2) `reg_with_pending` para o idioma LWL+LWR sem delay.
Ver `docs/iterations/0018-cpu-unaligned-load-store.md`. PR #32 aberta para revisão
adversarial.

## Próxima tarefa

**ROADMAP 1.8a** — COP0: banco de registradores + `MTC0`/`MFC0` + `RFE`. **SEM mecanismo de
exceção** — isso é o 1.8b. Se um teste só faz sentido com uma exceção acontecendo, ele pertence
ao 1.8b; não o escreva aqui.

O item 1.8 do ROADMAP foi dividido pelo orquestrador em 1.8a e 1.8b. Motivo registrado na
iteração 0019: o 1.8 original juntava banco de registradores, três opcodes de move, RFE,
mecanismo de entrada em exceção, cinco causas e o bit BD. A 1.7, com quatro opcodes de uma
família só, custou US$ 0,16 e 23min e ainda saiu com a bateria de mutação irreproduzível. R4
manda uma micro-funcionalidade por iteração.

**Escopo:** registradores `r12` (SR), `r13` (CAUSE), `r14` (EPC), `r8` (BadVaddr), `r15` (PRID);
opcodes `MFC0` (cop0cmd=00h), `MTC0` (cop0cmd=04h) e `RFE` (cop0cmd=10h).

**Spec:** `docs/reference/02-cpu.md` — `COP0 - Register Summary` (L568), `cop0r13 - CAUSE`
(L590), `cop0r12 - SR` (L624), `cop0r14 - EPC` (L670), `cop0cmd=10h - RFE opcode` (L712),
`cop0r8 - BadVaddr` (L730), `Coprocessor Instructions` (L422), `Coprocessor Opcode/Parameter
Encoding` (L127).

### Armadilhas nomeadas

1. **CAUSE é quase todo read-only.** A spec titula: "Read-only, except, Bit8-9 are R/W". Um
   `MTC0` em CAUSE só altera os bits 8-9 (Sw); ExcCode, IP, BD e CE ficam intactos. Tratar
   CAUSE como registrador comum é o erro esperado neste item.
2. **RFE não é um shift do campo.** A spec: "bit2-3 are copied to bit0-1, and bit4-5 are copied
   to bit2-3, all other bits (**including bit4-5**) are left unchanged". Os bits 4-5 permanecem.
3. **`MFC0` tem load delay de UM opcode; `MTC0` NÃO tem store delay.** `Caution - Load Delay`
   (L438) diz que o próximo opcode não pode usar o registrador destino, e derruba
   explicitamente o boato dos dois opcodes: "the PSX does finish both COP0 and COP2 reads after
   ONE opcode". `Caution - Store Delay` (L446): "COP0 is more or less free of store delays (eg.
   one can read from a cop0 register immediately after writing to it)". Não implemente delay no
   MTC0 por simetria.
4. **EPC e BadVaddr são marcados (R) na spec.** O que o hardware faz numa escrita via MTC0 não
   está documentado localmente. Se implementar como gravável, registre como **comportamento
   ASSUMIDO** nas Notas, no formato das notas 3 e 4, com ponto de resolução no Amidog
   `psxtest_cpu` (item 1.11).

### Testes de aceitação OBRIGATÓRIOS

Derivados pelo orquestrador **por dois caminhos independentes**, conforme a regra da 0017e. A
derivação vai abaixo justamente para você poder reprovar o orquestrador se ela estiver errada —
foi o que faltou na 0017 e custou uma iteração inteira.

**A1 — RFE.** `SR = 0x0000_0034` (binário `110100`). Executar `rfe`. Exigido: **`SR = 0x0000_003D`**.
*Rota 1 (bit a bit):* bit0←bit2=1; bit1←bit3=0; bit2←bit4=1; bit3←bit5=1; bits 4-5 inalterados
= 1,1. `bit5..bit0` = `111101` = `0x3D`.
*Rota 2 (por campos):* bits 3:2 = `01` viram bits 1:0; bits 5:4 = `11` viram bits 3:2; bits 5:4
seguem `11`. Concatenando `11 11 01` = `0x3D`.
O literal é assimétrico de propósito: os dois erros prováveis — shift do campo inteiro, e copiar
4-5 para 2-3 zerando 4-5 — dão ambos `0x0D`; inverter a ordem das cópias dá `0x3F`.

**A2 — CAUSE só aceita escrita nos bits 8-9.** Semeie `CAUSE = 0x0000_0020` direto no banco do
COP0 (na 1.8a não há exceção que o produza) e execute `mtc0 rt, $13` com `rt = 0xFFFF_FFFF`.
Exigido: **`CAUSE = 0x0000_0320`**.
*Rota 2:* máscara gravável `0x300`; `(0x20 & !0x300) | (0xFFFF_FFFF & 0x300)` = `0x320`.
Se o banco não for acessível ao teste, use `CAUSE = 0` e exija `0x0000_0300` — mais fraco, mas
ainda pega "CAUSE é registrador comum".

**A3 — load delay do `MFC0`.** `mfc0 r2, $12` seguido **imediatamente** de uma instrução que leia
`r2`: ela vê o valor **antigo**.

### Regra nova da bateria de mutação (vale deste item em diante)

Método auxiliar compartilhado por N pontos de chamada rende **N mutantes independentes**. Mutar
só a definição do helper testa 1 deles. Na 0018 isso escondeu uma lacuna real: trocar
`reg_with_pending` por `reg` apenas dentro de `fn lwl` não quebrava nenhum dos 25 testes.
Mute cada ponto de chamada, não o helper.

## Repositório

- `main` protegida a partir da iter 0004; 1 PR por item; merge commit (nunca squash);
  commits test→feat→docs; título de PR validado pela CI.
- Iterações são cronológicas e nem sempre na ordem dos itens (0003↔item 0.5, p.ex.);
  o vínculo real está no título do PR e no doc da iteração.

## Placar de testes

Workspace: **178** testes (10 meta-testes + 8 bus_bios + 2 bios_flag + 1 version + 12 bus_scheduler + 8 cpu_fetch_decode + 26 cpu_alu + 14 cpu_shifts + 19 cpu_load_delay + 24 cpu_branches + 7 cpu_jumps + 20 cpu_mult_div + 27 cpu_unaligned_load_store).

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
