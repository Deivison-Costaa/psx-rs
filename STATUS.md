# STATUS

> Memória do projeto entre iterações. O contexto do agente é descartado a cada iteração;
> este arquivo não. Teto de 16 KB imposto por `status_size.rs`. Curto e verdadeiro.

## Última iteração concluída

**0014** — Loads/stores + load delay (ROADMAP 1.4): LB/LBU/LH/LHU/LW (loads), SB/SH (stores),
mais o load delay slot. 18 testes em `cpu_load_delay.rs`.
Bateria de mutação: 6/6 pegos, 3/3 controles verdes.
Erro de primeira tentativa: teste `sb_offset_negativo` com endereço de setup errado (0x2004
em vez de 0x2000) — corrigido na primeira execução. Nenhum erro de emulação.
Ver `docs/iterations/0014-cpu-load-store-delay.md`.

## Próxima tarefa

**ROADMAP 1.5** — Branches/jumps + branch delay slot. Implementar J, JAL, JR, JALR
(branches), BEQ, BNE, BLEZ, BGTZ (branches condicionais), BLTZ/BGEZ/BLTZAL/BGEZAL
(BcondZ). O **branch delay slot**: a instrução imediatamente após o branch SEMPRE executa
(seja o branch tomado ou não). Spec: `docs/reference/02-cpu.md` — seções `L379 CPU Jump
Opcodes` (jumps and branches L380, JALR cautions L400). Teste:
`crates/psx-core/tests/cpu_branch_delay.rs` (criar). Armadilha: JALR pode usar o mesmo
reg para rs e rd — o rs original (target address) é lido antes de rd ser escrito com
`pc+8`. BcondZ codifica o subtipo em rt (0=BLTZ, 1=BGEZ, 16=BLTZAL, 17=BGEZAL).
Atenção: J/JAL target tem que preservar os 4 bits mais altos do PC; branches
condicionais usam offset de 16 bits sign-extendido * 4. O branch delay slot já executa
antes do desvio — na nossa CPU instruction-stepped, após executar o branch, a próxima
instrução (delay slot) é executada, e SÓ ENTÃO o PC é redirecionado.

## Repositório

- `main` protegida a partir da iter 0004; 1 PR por item; merge commit (nunca squash);
  commits test→feat→docs; título de PR validado pela CI.
- Iterações são cronológicas e nem sempre na ordem dos itens (0003↔item 0.5, p.ex.);
  o vínculo real está no título do PR e no doc da iteração.

## Placar de testes

Workspace: **98** testes (8 meta-testes + 8 bus_bios + 2 bios_flag + 1 version + 12 bus_scheduler + 8 cpu_fetch_decode + 26 cpu_alu + 14 cpu_shifts + 19 cpu_load_delay + 3 psx-cli/desktop). As duas últimas linhas vieram da revisão da 0014.

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
