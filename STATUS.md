# STATUS

> Memória do projeto entre iterações. O contexto do agente é descartado a cada iteração;
> este arquivo não. Teto de 16 KB imposto por `status_size.rs`. Curto e verdadeiro.

## Última iteração concluída

**0010** — Scheduler de eventos + Bus (ROADMAP 1.1): struct `Scheduler` com fila ordenada por
timestamp, `schedule(ticks, callback_id)`, `advance_to(ticks)`, `pending_events()`. Struct `Bus`
com `Ram([u8; 0x200000])`, `Bios`, `read32<T>(addr)`/`write32<T>(addr, val)` com roteamento
KUSEG/KSEG0/KSEG1 via `to_physical()`. 11 testes de integração em `bus_scheduler.rs`.

## Próxima tarefa

**ROADMAP 1.2** — Fetch/decode + LUI/ORI/SW, e SÓ isso (R4: ALU é 1.3, loads/delay é 1.4,
branches é 1.5 — NÃO implemente agora). Em `crates/psx-core/src/cpu.rs`: struct `Cpu` com
regs (32×u32, R0 sempre 0, + PC iniciando em 0xBFC00000), `step(&mut Bus)` que busca a
instrução via `read32`, decodifica (primary opcode bits 26..31, secondary bits 0..5) e
executa APENAS LUI, ORI e SW — as três primeiras instruções que o BIOS executa. Opcode
desconhecido: `unimplemented` explícito por enquanto (exceções são 1.8). Spec:
`docs/reference/02-cpu.md` — seções `L19 CPU Registers`, `L74 CPU Opcode Encoding`,
`L305 logical instructions` (LUI/ORI), `L219 Store instructions` (SW).
Teste: `psx-core/tests/cpu_fetch_decode.rs` (golden values da spec; verificar R0
imutável e SW escrevendo na RAM via bus). Armadilha: ORI é zero-extended, não
sign-extended; LUI zera os 16 bits baixos.

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
