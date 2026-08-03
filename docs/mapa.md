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
| spu | `crates/psx-core/src/spu.rs` | registradores, RAM de 512 KiB, transferencia, mixer de 44,1 kHz | `Spu`, `read16`, `write16`, `tick`, `drain_output`, `set_cd_audio` |
| spu/voice | `crates/psx-core/src/spu/voice.rs` | estado das 24 vozes, pitch, key on/off, ENDX | `Voice`, `Volume`, `Phase`, `step` |
| spu/adpcm | `crates/psx-core/src/spu/adpcm.rs` | bloco de 16 bytes -> 28 amostras | `decode_block`, `Flags` |
| spu/envelope | `crates/psx-core/src/spu/envelope.rs` | envoltoria de ADSR e de sweep | `Envelope`, `Rate` |
| spu/reverb | `crates/psx-core/src/spu/reverb.rs` | 32 registradores, formula de reverb a 22,05 kHz | `Reverb`, `run`, `advance`, `set_mbase` |
| spu/gauss | `crates/psx-core/src/spu/gauss.rs` | tabela de 512 entradas e interpolacao de 4 pontos | `TABLE`, `interpolate` |
| cdrom_xa | `crates/psx-core/src/cdrom_xa.rs` | XA-ADPCM, quadros de CD-DA e reamostragem para 44,1 kHz | `decode_sector`, `decode_28_nibbles`, `cdda_frames`, `resample_to_44100` |
| mdec | `crates/psx-core/src/mdec.rs` | decodificador de macroblocos | (vazio — M8) |
| psx-cli | `crates/psx-cli/src/main.rs` | runner headless, sideload de EXE, TTY, scoreboard | stub |
| psx-desktop | `crates/psx-desktop/src/main.rs` | app egui: biblioteca, emulação, saves, controles, config | stub |

## Testes

Um arquivo de integração por item, nome começando pelo módulo e espelhando o item
(`cpu_load_delay.rs` ↔ 1.4). O item 1.5 rendeu dois (`cpu_jumps.rs` e `cpu_branches.rs`):
quando um item passa das 500 linhas de teste, ele vira mais de um arquivo — o teto de
`file_size.rs` vale aqui, não em `src/`.

Suporte compartilhado em `crates/psx-core/tests/support/`: `mod.rs` (raiz do repo, varredura
de fontes e de testes, usado pelos meta-testes) e `asm.rs` (montagem de opcodes e bus com
BIOS vazia, usado pelos testes de CPU).
Meta-testes de processo: `crates/psx-core/tests/{purity,comment_density,file_size,status_size,roadmap_size,ci_workflow,metrics_freshness,toolchain_pin,mutation_manifest,mutation_anchors}.rs`.
O parser de manifesto de mutação mora em `crates/psx-core/tests/support/mutation_format.rs`
(incluído via `#[path]`, NÃO declarado em `mod.rs` — evita 28 compilações a mais).
A gramática e a semântica estão em `docs/mutantes/README.md`.
A versão do compilador mora em `rust-toolchain.toml` (raiz) e em nenhum outro lugar.
