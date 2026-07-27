# STATUS

> Memória do projeto entre iterações. O contexto do agente é descartado a cada iteração;
> este arquivo não. Teto de 16 KB imposto por `status_size.rs`. Curto e verdadeiro.

## Última iteração concluída

**0013** — Shifts (ROADMAP 1.3b): SLL/SRL/SRA (shift-imm, campo `sa`) e SLLV/SRLV/SRAV
(shift-reg, quantidade em `rs & 0x1F`). 14 testes em `cpu_shifts.rs`.
Bateria de mutação: 3/3 pegos, 2/2 controles verdes.
Erro de primeira tentativa: nenhum na implementação; teste `opcode_desconhecido_especial_panics`
da ALU usava secondary=0x00 que agora é SLL válido — atualizado para secondary=0x08.
Ver `docs/iterations/0013-cpu-shifts.md`.

## Próxima tarefa

**ROADMAP 1.4** — Loads/stores + load delay slot. Expandir `step()` para LB/LBU/LH/LHU/LW
(loads) e SB/SH (stores). Implementar o **load delay slot**: o resultado de um load SÓ fica
disponível uma instrução depois; ler o register destino no ciclo imediatamente seguinte
retorna o valor anterior. Spec: `docs/reference/02-cpu.md` — seções `L156 Load/Store Opcodes`
(load instructions linha 157, store instructions linha 299), `L171 Caution - Load Delay`,
`L180 Load Timing`, `L201 Load Shadow`. Teste: `crates/psx-core/tests/cpu_load_delay.rs`.
Armadilha: o load delay é um fenômeno do pipeline real; na nossa CPU instruction-stepped
precisamos marcar o registrador como "locked" e adiar a escrita. Atenção: o próprio store
não tem delay — o write-queue esconde. R0 locked nunca deve afetar nada.

## Repositório

- `main` protegida a partir da iter 0004; 1 PR por item; merge commit (nunca squash);
  commits test→feat→docs; título de PR validado pela CI.
- Iterações são cronológicas e nem sempre na ordem dos itens (0003↔item 0.5, p.ex.);
  o vínculo real está no título do PR e no doc da iteração.

## Placar de testes

Workspace: **75** testes (8 meta-testes + 8 bus_bios + 2 bios_flag + 1 version + 11 bus_scheduler + 8 cpu_fetch_decode + 26 cpu_alu + 14 cpu_shifts + 3 psx-cli/desktop).

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
