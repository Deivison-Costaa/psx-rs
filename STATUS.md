# STATUS

> Memória do projeto entre iterações. O contexto do agente é descartado a cada iteração;
> este arquivo não. Teto de 16 KB imposto por `status_size.rs`. Curto e verdadeiro.

## Última iteração concluída

**0010** — Scheduler de eventos + Bus (ROADMAP 1.1): struct `Scheduler` com fila ordenada por
timestamp, `schedule(ticks, callback_id)`, `advance_to(ticks)`, `pending_events()`. Struct `Bus`
com `Ram([u8; 0x200000])`, `Bios`, `read32<T>(addr)`/`write32<T>(addr, val)` com roteamento
KUSEG/KSEG0/KSEG1 via `to_physical()`. 11 testes de integração em `bus_scheduler.rs`.

## Próxima tarefa

**ROADMAP 1.2** — CPU R3000A: decode de instruções, ALU, load delay slot.
Em `crates/psx-core/src/cpu.rs`: struct `Cpu` com registradores (32×u32 + PC + HI/LO),
`step()` que busca instrução na `Bus`, decode e executa instruções base (ADDU, SUBU, AND,
OR, XOR, NOR, SLL, SRL, SRA, LW, SW, ADDIU, ORI, ANDI, XORI, LUI, SLTI, SLTIU, BEQ,
BNE, J, JAL, JR, BLTZ, BLEZ, BGTZ, BGEZ). Implementar load delay slot: instrução após LW
lê o registrador destino antes da escrita. Spec: `docs/reference/02-cpu-specifications.md`
— seções Instruction Overview + CPU Registers + Load Delay Slot + Instruction Set.
Teste: `psx-core/tests/cpu_instructions.rs`. Armadilha: delay slot existe para loads E
branches; testar LW seguido de uso imediato do registrador.

## Repositório

- `main` protegida a partir da iter 0004; 1 PR por item; merge commit (nunca squash);
  commits test→feat→docs; título de PR validado pela CI.
- Iterações são cronológicas e nem sempre na ordem dos itens (0003↔item 0.5, p.ex.);
  o vínculo real está no título do PR e no doc da iteração.

## Placar de testes

Workspace: **33** testes (8 meta-testes + 8 bus_bios + 2 bios_flag + 1 version + 11 bus_scheduler + 3 psx-cli/desktop).

## Bloqueios

(nenhum)

## Invariantes

(nenhuma ainda — nascem com o código; índice com âncoras quando existirem)

## Notas

1. BIOS local: `bios/SCPH1001.BIN` (MD5 924E392ED05558FFDB115408C263DCCF), gitignored,
   validada na iter 0009 (item 0.9). Nunca commitar.
