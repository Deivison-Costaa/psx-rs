# Mapa do código

> Ponteiros módulo → arquivo → responsabilidade, para achar o alvo sem varrer a árvore (R8).
> Atualize a linha do módulo quando um arquivo nascer ou for fatiado. Sem prosa.

| Módulo | Arquivo(s) | Responsabilidade | Entradas principais |
|---|---|---|---|
| audio | `crates/psx-core/src/audio.rs` | anel de quadros entre o SPU e o frontend, com conversao de taxa | `Ring`, `push_frames`, `fill_interleaved` |
| bus | `crates/psx-core/src/bus.rs` | mapa de memória, RAM 2MB, BIOS, roteamento KUSEG/KSEG0/KSEG1 | `Bus`, `Ram`, `Bios`, `read32`, `write32`, `to_physical` |
| scheduler | `crates/psx-core/src/scheduler.rs` | fila de eventos por timestamp, relógio mestre | `Scheduler`, `EventId`, `ScheduleKey`, `schedule`, `advance_to`, `pending_events` |
| cpu | `crates/psx-core/src/cpu.rs` | R3000A: decode, ALU, delay slots, COP0 | (vazio — item 1.2) |
| gte | `crates/psx-core/src/gte.rs` | COP2: ponto fixo, RTPS/MVMVA, saturação | (vazio — M5) |
| gpu | `crates/psx-core/src/gpu.rs` | GP0/GP1, VRAM, rasterizador | (vazio — M2) |
| dma | `crates/psx-core/src/dma.rs` | 7 canais, OTC, linked-list | (vazio — M3) |
| irq | `crates/psx-core/src/irq.rs` | I_STAT/I_MASK | (vazio — M3) |
| timers | `crates/psx-core/src/timers.rs` | timers 0/1/2 | (vazio — M3) |
| cdrom | `crates/psx-core/src/cdrom.rs` | controller, comandos, BIN/CUE | (vazio — M4) |
| sio | `crates/psx-core/src/sio.rs` | JOY_*, pad digital, roteamento por endereco e /ACK | `Sio`, `send_byte`, `deliver_ack`, `connect_memory_card`, `load_memory_card` |
| memcard | `crates/psx-core/src/memcard.rs` | cartao de 128 KiB, comandos R/W/S e byte FLAG | `MemoryCard`, `exchange`, `begin`, `from_bytes`, `data` |
| spu | `crates/psx-core/src/spu.rs` | registradores, RAM de 512 KiB, transferencia, mixer de 44,1 kHz | `Spu`, `read16`, `write16`, `tick`, `drain_output`, `set_cd_audio` |
| spu/voice | `crates/psx-core/src/spu/voice.rs` | estado das 24 vozes, pitch, key on/off, ENDX | `Voice`, `Volume`, `Phase`, `step` |
| spu/adpcm | `crates/psx-core/src/spu/adpcm.rs` | bloco de 16 bytes -> 28 amostras | `decode_block`, `Flags` |
| spu/envelope | `crates/psx-core/src/spu/envelope.rs` | envoltoria de ADSR e de sweep | `Envelope`, `Rate` |
| spu/reverb | `crates/psx-core/src/spu/reverb.rs` | 32 registradores, formula de reverb a 22,05 kHz | `Reverb`, `run`, `advance`, `set_mbase` |
| spu/gauss | `crates/psx-core/src/spu/gauss.rs` | tabela de 512 entradas e interpolacao de 4 pontos | `TABLE`, `interpolate` |
| cdrom_xa | `crates/psx-core/src/cdrom_xa.rs` | XA-ADPCM, quadros de CD-DA e reamostragem para 44,1 kHz | `decode_sector`, `decode_28_nibbles`, `cdda_frames`, `resample_to_44100` |
| mdec | `crates/psx-core/src/mdec.rs` | decodificador de macroblocos | (vazio — M8) |
| app/library | `crates/psx-core/src/app/library.rs` | ISO 9660: PVD, diretorio raiz, SYSTEM.CNF, serial e regiao | `identifica`, `raiz_do_pvd`, `procura_no_diretorio`, `serial_do_boot`, `dados_do_setor` |
| app/saves | `crates/psx-core/src/app/saves.rs` | diretorio do memory card e nome de cartao por serial | `lista`, `nome_do_cartao` |
| app/input_map | `crates/psx-core/src/app/input_map.rs` | vocabulario de entrada, perfis e palavra do pad | `Entrada`, `Perfil`, `palavra`, `para_texto`, `de_texto` |
| app/config | `crates/psx-core/src/app/config.rs` | configuracao do app: padroes, faixas, validacao, ganho | `Config`, `ajustada`, `valida`, `ganho` |
| app/sessao | `crates/psx-core/src/app/sessao.rs` | recentes, tempo de jogo e multiplicador de velocidade | `Recentes`, `passos_por_quadro`, `proxima_velocidade`, `formata_tempo` |
| snapshot | `crates/psx-core/src/snapshot.rs` | save state: estado do core em bincode, com magico/versao/serial | `salva`, `carrega`, `serial_de`, `SnapshotError` |
| serde_grande | `crates/psx-core/src/serde_grande.rs` | (de)serializacao de `[T; N]` com N > 32, que o serde nao cobre | `serialize`, `deserialize`, `em_cell` |
| psx-cli | `crates/psx-cli/src/main.rs` | runner headless, sideload de EXE, TTY, scoreboard, `--disc-info` | `main`, `run`, `imprime_identidade` |
| psx-desktop | `crates/psx-desktop/src/main.rs` | app egui: estado, argumentos, maquina de telas | `App`, `Tela`, `inicia`, `encerra_partida` |
| psx-desktop/telas | `crates/psx-desktop/src/telas.rs` | as cinco telas (biblioteca, jogando, saves, controles, ajustes) | `tela_biblioteca`, `tela_jogando`, `tela_ajustes` |
| psx-desktop/emulador | `crates/psx-desktop/src/emulador.rs` | maquina em execucao: quadro, entrada, save state, cartao | `Emulador`, `quadro`, `entrada`, `salva_estado` |
| psx-desktop/disco | `crates/psx-desktop/src/disco.rs` | carga do BIN/CUE e identificacao por seek | `carrega`, `identifica` |
| psx-desktop/biblioteca | `crates/psx-desktop/src/biblioteca.rs` | varredura da pasta de jogos | `varre`, `Jogo` |
| psx-desktop/gamepad | `crates/psx-desktop/src/gamepad.rs` | gilrs -> `Entrada`, com zona morta | `Gamepads`, `pressionados` |
| psx-desktop/ajustes | `crates/psx-desktop/src/ajustes.rs` | leitura e gravacao do `psx-rs.toml` | `carrega`, `grava` |

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
