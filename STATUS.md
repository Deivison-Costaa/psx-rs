# STATUS

> Memória do projeto entre iterações. O contexto do agente é descartado a cada iteração;
> este arquivo não. Teto de 16 KB imposto por `status_size.rs`. Curto e verdadeiro.

## Última iteração concluída

**0012** — ALU completa (ROADMAP 1.3): SPECIAL (primary=0x00) com secondary opcode para
ADDU/SUBU/AND/OR/XOR/NOR/SLT/SLTU; alu-imm para ADDI/ADDIU/ANDI/ORI/XORI/SLTI/SLTIU. 26
testes em `cpu_alu.rs`. Bateria de mutação: 6/6 pegos, 2/2 controles verdes.
Erro de primeira tentativa: campo rs/rt trocado no helper de encode (API-Rust); expectativa
errada de SLTIU com imm alto (flags). Ver `docs/iterations/0012-cpu-alu.md`.

## Próxima tarefa

**ROADMAP 1.3b** — Shifts, e SÓ isso (a 0012 entregou o resto da ALU; loads são 1.4).
Em `crates/psx-core/src/cpu.rs`, adicionar ao `special()`: SLL (secondary 0x00),
SRL (0x02), SRA (0x03) — quantidade de shift no campo `sa` (bits 6..10) — e as variantes
por registrador SLLV (0x04), SRLV (0x06), SRAV (0x07), que usam `rs & 0x1F`.
Spec: `docs/reference/02-cpu.md` — seção `shifting instructions` (linha 316).
Teste: `crates/psx-core/tests/cpu_shifts.rs`. Armadilhas: SRA é aritmético (propaga o bit
de sinal — use `as i32 >> n`), SRL é lógico; nas variantes V a quantidade vem de `rs`
mascarada com 0x1F (shift de 32 não existe); `sll $0,$0,0` é o NOP canônico e precisa
continuar não fazendo nada.

Depois dela, **ROADMAP 1.4** — Loads/stores + load delay slot. Expandir `step()` para LB/LBU/LH/LHU/LW
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

Workspace: **67** testes (8 meta-testes + 8 bus_bios + 2 bios_flag + 1 version + 11 bus_scheduler + 8 cpu_fetch_decode + 26 cpu_alu + 3 psx-cli/desktop).

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
