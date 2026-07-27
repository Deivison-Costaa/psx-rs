# STATUS

> Memória do projeto entre iterações. O contexto do agente é descartado a cada iteração;
> este arquivo não. Teto de 16 KB imposto por `status_size.rs`. Curto e verdadeiro.

## Última iteração concluída

**0011** — Fetch/decode + LUI/ORI/SW (ROADMAP 1.2): struct `Cpu` com regs (32×u32, R0
imutável, PC=0xBFC00000), `step(&mut Bus)` que busca instr via `read32`, decodifica pelo
primary opcode (bits 26..31) e executa LUI (imm<<16), ORI (rs|imm, zero-extended) e SW
([rs+imm]=rt). Opcode desconhecido: `unimplemented!`. 7 testes em `cpu_fetch_decode.rs`.

## Próxima tarefa

**ROADMAP 1.3** — ALU: ADD/ADDU/SUB/SUBU/AND/OR/XOR/NOR/SLT/SLTU + imediatos
(ADDI/ADDIU/ANDI/ORI/XORI/SLTI/SLTIU). Em `crates/psx-core/src/cpu.rs`: estender o match
de primary opcode para `00h=SPECIAL` (secondary opcode bits 0..5) e `00xxx=alu-imm`.
Spec: `docs/reference/02-cpu.md` — seções `L285 arithmetic instructions`, `L297 comparison`,
`L305 logical instructions`, `L99 Opcode/Parameter Encoding` (tabela alu-imm em linha 200,
SPECIAL em linha 192). Teste: `crates/psx-core/tests/cpu_alu.rs`. Armadilha: ADDI/ADDIU
sign-extendem o immediate (16→32 bits); SLTI/SLTIU também; ANDI/ORI/XORI zero-extension;
ADD/ADDU trap vs. no-trap — por enquanto implemente ADDU e ignore overflow (ADDU é o que
o BIOS realmente usa). SPECIAL (primary=0x00) requer decodificar o secondary opcode.

## Repositório

- `main` protegida a partir da iter 0004; 1 PR por item; merge commit (nunca squash);
  commits test→feat→docs; título de PR validado pela CI.
- Iterações são cronológicas e nem sempre na ordem dos itens (0003↔item 0.5, p.ex.);
  o vínculo real está no título do PR e no doc da iteração.

## Placar de testes

Workspace: **37** testes (8 meta-testes + 8 bus_bios + 2 bios_flag + 1 version + 11 bus_scheduler + 7 cpu_fetch_decode + 3 psx-cli/desktop).

## Bloqueios

(nenhum)

## Invariantes

(nenhuma ainda — nascem com o código; índice com âncoras quando existirem)

## Notas

1. BIOS local: `bios/SCPH1001.BIN` (MD5 924E392ED05558FFDB115408C263DCCF), gitignored,
   validada na iter 0009 (item 0.9). Nunca commitar.
