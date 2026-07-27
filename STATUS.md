# STATUS

> Memória do projeto entre iterações. O contexto do agente é descartado a cada iteração;
> este arquivo não. Teto de 16 KB imposto por `status_size.rs`. Curto e verdadeiro.

## Última iteração concluída

**0017** — LWL/LWR/SWL/SWR (ROADMAP 1.7): LWL primary 0x22, LWR 0x26, SWL 0x2A, SWR 0x2E.
18 testes em `cpu_unaligned_load_store.rs`. Bateria de mutação: 5/5 pegos, 2/2 controles verdes.
Erro de primeira tentativa: (1) teste SWL com valor de rt igual ao byte correspondente na
memória mascarava o merge; corrigido com rt=0xDEAD_BEEF em memória 0xAABB_CCDD; (2) teste
LWL+LWR no mesmo rt (par clássico) falhava por causa do load delay — o LWR lia o valor
antigo de rt em vez do recém-carregado pelo LWL; trocado para rts diferentes. Ver
`docs/iterations/0017-cpu-unaligned-load-store.md`.

## Próxima tarefa

**ROADMAP 1.8** — COP0: SR/CAUSE/EPC, exceções, RFE. Spec: `docs/reference/02-cpu.md`,
seções **Exception Handling** (índice: L462) e **System Control (COP0)** (índice: L426).
Leia ambas — a primeira define os registradores COP0 (SR, CAUSE, EPC), a segunda o fluxo
de exceção e o RFE. Armadilhas: (a) `CAUSE.BD` (bit 31) precisa ser setado quando a
instrução que causou a exceção está num delay slot; (b) o fluxo de exceção precisa ler o
PC da instrução causadora, não o PC incrementado; (c) a dívida de overflow trap do ADDI
(fechar junto, nota 2 do STATUS); (d) a dívida do bit BD (nota 5). Atenção: este item
fecha 3 dívidas ao mesmo tempo. Candidato natural a fatiar `cpu.rs` — COP0 ganha módulo
próprio em `src/cpu/cop0.rs` se exceder coesão do arquivo único.

Teste: `crates/psx-core/tests/cpu_exceptions.rs`.

`cpu.rs` tem ~620 linhas — dentro da tolerância (sem teto), mas o orquestrador autorizou
corte quando a coesão pedir. COP0 é o candidato certo: se o módulo `cop0` ficar >100
linhas, extraia para `src/cpu/` com `mod cop0;` em `cpu.rs`.

`cpu_unaligned_load_store.rs` tem 259 linhas — dentro do teto de 500.

## Repositório

- `main` protegida a partir da iter 0004; 1 PR por item; merge commit (nunca squash);
  commits test→feat→docs; título de PR validado pela CI.
- Iterações são cronológicas e nem sempre na ordem dos itens (0003↔item 0.5, p.ex.);
  o vínculo real está no título do PR e no doc da iteração.

## Placar de testes

Workspace: **167** testes (8 meta-testes + 8 bus_bios + 2 bios_flag + 1 version + 12 bus_scheduler + 8 cpu_fetch_decode + 26 cpu_alu + 14 cpu_shifts + 19 cpu_load_delay + 24 cpu_branches + 7 cpu_jumps + 20 cpu_mult_div + 18 cpu_unaligned_load_store).

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
