# STATUS

> Memória do projeto entre iterações. O contexto do agente é descartado a cada iteração;
> este arquivo não. Teto de 16 KB imposto por `status_size.rs`. Curto e verdadeiro.

## Última iteração concluída

**0015** — Branches/jumps + branch delay slot (ROADMAP 1.5): J, JAL, JR, JALR, BEQ, BNE,
BLEZ, BGTZ, BLTZ, BGEZ, BLTZAL, BGEZAL + branch delay slot. 29 testes em
`cpu_branch_delay.rs`.
Bateria de mutação: 6/6 pegos, 2/2 controles verdes.
Erro de primeira tentativa: testes escritos com 1 step em vez de 2 (branch prepara, delay
slot executa e redireciona no step seguinte) — corrigido na primeira execução.
A revisão adversarial achou um pânico de overflow no link (JAL/JALR/BcondZ usavam `+ 4`
em vez de `wrapping_add`) e corrigiu na branch; +2 testes.
Ver `docs/iterations/0015-cpu-branch-delay.md`.

## Próxima tarefa

**ROADMAP 1.6** — MULT/MULTU/DIV/DIVU + HI/LO. (O handoff da 0015 apontava para a 2.1,
pulando 1.6–1.12; corrigido na revisão. O M1 fecha antes de a GPU começar.)

**Antes de escrever qualquer instrução nova, fatie `crates/psx-core/src/cpu.rs`.** Está em
440 linhas e o R3000A ainda deve dobrar de tamanho (MULT/DIV, LWL/LWR, COP0). O motivo é a
R8, não estética: um `cpu.rs` monolítico obriga toda iteração futura a pagar o arquivo
inteiro em contexto. Manter em `cpu/mod.rs` a struct, `new`, `step`, `execute` e o
dispatch; mover as famílias para `cpu/alu.rs`, `cpu/mem.rs`, `cpu/branch.rs` como
`impl Cpu`. Atualizar `docs/mapa.md` junto — é o refactor da iteração, commit
`refactor(cpu):` antes do `feat`. Se algum corte ficar artificial, deixe o arquivo maior e
registre o porquê no doc: coesão vale mais que o número.

Escopo do 1.6: MULT (SPECIAL 0x18), MULTU (0x19), DIV (0x1A), DIVU (0x1B), MFHI (0x10),
MTHI (0x11), MFLO (0x12), MTLO (0x13); registradores `hi`/`lo` na struct `Cpu`.
Spec: `docs/reference/02-cpu.md`, seção **Multiply/divide** (índice: L329) — leia SÓ ela.
Armadilhas que a seção documenta e a intuição erra: DIV por zero e o overflow de
`0x80000000 / -1` **não excetuam**, devolvem valores específicos tabelados; MULT/MULTU
produzem 64 bits divididos entre HI e LO. Os "stalls" da linha do ROADMAP são custo em
ciclos — só serão observáveis quando o scheduler cobrar ciclos da CPU; registre a decisão
no doc da iteração se deixar como dívida. Teste: `crates/psx-core/tests/cpu_mult_div.rs`.

## Repositório

- `main` protegida a partir da iter 0004; 1 PR por item; merge commit (nunca squash);
  commits test→feat→docs; título de PR validado pela CI.
- Iterações são cronológicas e nem sempre na ordem dos itens (0003↔item 0.5, p.ex.);
  o vínculo real está no título do PR e no doc da iteração.

## Placar de testes

Workspace: **129** testes (8 meta-testes + 8 bus_bios + 2 bios_flag + 1 version + 12 bus_scheduler + 8 cpu_fetch_decode + 26 cpu_alu + 14 cpu_shifts + 19 cpu_load_delay + 31 cpu_branch_delay). As 2 últimas linhas vieram da revisão da 0015.

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
