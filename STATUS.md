# STATUS

> Memória do projeto entre iterações. O contexto do agente é descartado a cada iteração;
> este arquivo não. Teto de 16 KB imposto por `status_size.rs`. Curto e verdadeiro.

## Última iteração concluída

**0009** — carregamento de BIOS (ROADMAP 0.9): tipo `Bios` em `crates/psx-core/src/bus.rs`
com validação de tamanho (exatos 512 KiB) e `read32` little-endian; 8 testes de integração
em `bus_bios.rs`; flag `--bios <path>` no psx-cli com leitura do arquivo + SHA-256.

## Próxima tarefa

**ROADMAP 1.1** — Scheduler de eventos + bus (KUSEG/KSEG0/KSEG1), RAM 2MB, BIOS ROM.
Em `crates/psx-core/src/scheduler.rs`: struct `Scheduler` com fila de eventos ordenada por
timestamp, `schedule(ticks, callback_id)`, `advance_to(ticks)`, `pending_events()`. Em
`crates/psx-core/src/bus.rs`: struct `Bus` com `RAM([u8; 0x200000])`, `Bios`, e método
`read32<T>(addr: u32) -> T`/`write32<T>(addr: u32, val: T)` que roteia entre KUSEG/KSEG0/KSEG1
(pode ignorar mirrors por enquanto). Spec: `docs/reference/01-memory-map.md` — seções Memory
Map + KUSEG/KSEG0/KSEG1 Memory Regions + Memory Mirrors. Teste:
`psx-core/tests/bus_scheduler.rs` (roteamento KUSEG↔KSEG0↔KSEG1 para RAM, BIOS read32,
eventos em ordem). Armadilha: KSEG1 lê físico limpo sem cache — por ora só máscara de addr
(0x1FFFFF para RAM); escrever scheduler que não avança CPU em loop.

## Repositório

- `main` protegida a partir da iter 0004; 1 PR por item; merge commit (nunca squash);
  commits test→feat→docs; título de PR validado pela CI.
- Iterações são cronológicas e nem sempre na ordem dos itens (0003↔item 0.5, p.ex.);
  o vínculo real está no título do PR e no doc da iteração.

## Placar de testes

Workspace: **19** testes (8 meta-testes + 8 bus_bios + 2 bios_flag + 1 version).

## Bloqueios

(nenhum)

## Invariantes

(nenhuma ainda — nascem com o código; índice com âncoras quando existirem)

## Notas

1. BIOS local: `bios/SCPH1001.BIN` (MD5 924E392ED05558FFDB115408C263DCCF), gitignored,
   validada na iter 0009 (item 0.9). Nunca commitar.
