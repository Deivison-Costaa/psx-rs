# Mapa do código

> Ponteiros módulo → arquivo → responsabilidade, para achar o alvo sem varrer a árvore (R8).
> Atualize a linha do módulo quando um arquivo nascer ou for fatiado. Sem prosa.

| Módulo | Arquivo(s) | Responsabilidade | Entradas principais |
|---|---|---|---|
| bus | `crates/psx-core/src/bus.rs` | mapa de memória, RAM 2MB, BIOS, roteamento KUSEG/KSEG0/KSEG1 | `Bus`, `Ram`, `Bios`, `read32`, `write32`, `to_physical` |
| scheduler | `crates/psx-core/src/scheduler.rs` | fila de eventos por timestamp, relógio mestre | `Scheduler`, `EventId`, `ScheduleKey`, `schedule`, `advance_to`, `pending_events` |
| cpu | `crates/psx-core/src/cpu.rs` | R3000A: decode, ALU, delay slots, COP0 | (vazio — item 1.2) |
| gte | `crates/psx-core/src/gte.rs` | COP2: ponto fixo, RTPS/MVMVA, saturação | (vazio — M5) |
| gpu | `crates/psx-core/src/gpu.rs` | GP0/GP1, VRAM, rasterizador | (vazio — M2) |
| dma | `crates/psx-core/src/dma.rs` | 7 canais, OTC, linked-list | (vazio — M3) |
| irq | `crates/psx-core/src/irq.rs` | I_STAT/I_MASK | (vazio — M3) |
| timers | `crates/psx-core/src/timers.rs` | timers 0/1/2 | (vazio — M3) |
| cdrom | `crates/psx-core/src/cdrom.rs` | controller, comandos, BIN/CUE | (vazio — M4) |
| sio | `crates/psx-core/src/sio.rs` | pad e memory card | (vazio — M6) |
| spu | `crates/psx-core/src/spu.rs` | vozes ADPCM, mixer, reverb | (vazio — M7) |
| mdec | `crates/psx-core/src/mdec.rs` | decodificador de macroblocos | (vazio — M8) |
| psx-cli | `crates/psx-cli/src/main.rs` | runner headless, sideload de EXE, TTY, scoreboard | stub |
| psx-desktop | `crates/psx-desktop/src/main.rs` | app egui: biblioteca, emulação, saves, controles, config | stub |

## Testes

Um arquivo de integração por item, nome começando pelo módulo e espelhando o item
(`cpu_load_delay.rs` ↔ 1.4). Suporte compartilhado em `crates/psx-core/tests/support/mod.rs`.
Meta-testes de processo: `crates/psx-core/tests/{purity,comment_density,file_size,status_size,roadmap_size,ci_workflow,metrics_freshness}.rs`.
