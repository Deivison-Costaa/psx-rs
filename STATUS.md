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
Nenhum erro de emulação.
Ver `docs/iterations/0015-cpu-branch-delay.md`.

## Próxima tarefa

**ROADMAP 2.1** — GPU: interface de barramento + registradores GP0/GP1. Implementar o
mapeamento de I/O da GPU na faixa `0x1F80_1810..0x1F80_181C` (GP0, GP1, GPUSTAT, etc.),
com respostas stub (zero) para leituras e captura de writes. A GPU precisa de uma struct
`Gpu` com estado interno mínimo (GPUSTAT, modo de desenho, etc.), registrada no `Bus`.
Spec: `docs/reference/03-gpu.md` — seção GPU I/O Ports (registradores, offsets e
comportamento esperado de leitura/escrita). Arquivos-alvo: `crates/psx-core/src/gpu.rs`
(criar), `crates/psx-core/src/bus.rs` (adicionar mapeamento GPU). Teste:
`crates/psx-core/tests/gpu_io_ports.rs` (criar). Armadilha: GP0 é write-only (leitura
retorna o último valor escrito em GP1, ou zero); GP1 é write-only (leitura retorna zero);
GPUSTAT é read-only (escrita ignorada). O reset de GPU deve setar GPUSTAT para um valor
conhecido (pelo menos bit 0 = ready to receive command).

## Repositório

- `main` protegida a partir da iter 0004; 1 PR por item; merge commit (nunca squash);
  commits test→feat→docs; título de PR validado pela CI.
- Iterações são cronológicas e nem sempre na ordem dos itens (0003↔item 0.5, p.ex.);
  o vínculo real está no título do PR e no doc da iteração.

## Placar de testes

Workspace: **127** testes (8 meta-testes + 8 bus_bios + 2 bios_flag + 1 version + 12 bus_scheduler + 8 cpu_fetch_decode + 26 cpu_alu + 14 cpu_shifts + 19 cpu_load_delay + 29 cpu_branch_delay + 0 psx-cli/desktop).

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
