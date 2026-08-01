# ROADMAP

Cada item = **1 iteração = 1 PR** (commits test→feat→docs). Uma linha por item, sem prosa —
narrativa mora em `docs/iterations/NNNN-*.md`. Trabalho fora da escada ganha sufixo (`0012b`).
Teto de tamanho imposto por `roadmap_size.rs`.

Marco que fecha 100% sai daqui para `docs/ROADMAP-fechado.md` — hoje M0 (infra), M1 (CPU),
M2 (GPU) e M3 (DMA/IRQ/timers). Regra imposta por `roadmap_arquivo.rs`.

## M4 — CDROM
- [x] 4.1 Regs/FIFOs/IRQs + GetStat/GetID/Test (iter 0062)
- [x] 4.2a Setloc/SeekL/Pause + estado do drive (iter 0063)
- [x] 4.2b Parser BIN/CUE (iter 0064)
- [x] 4.2c ReadN/ReadS + INT1/DRQSTS (máquina de estados) (iter 0065)
- [x] 4.3a DMA canal 3 (registradores + gate BFRD no DRQSTS + transferencia) (iter 0066)
- [x] 4.3b Acoplar DiscLayout + dados do .bin a entrega de setores (iter 0077)
- [x] 4.3c Flag --disc/--cue no psx-cli (iter 0078)
- [ ] 4.4 Boot de jogo 2D/menu
- [x] 4.4h $ra=4: load pendente sobre delay slot (0111)
- [x] 4.4a Boot da BIOS no psx-cli (0079)
- [x] 4.4b scheduler + vblank + IRQ0 (0080)
- [x] 4.4c BIOS nunca escreve I_MASK (0085)
- [x] 4.4d I_MASK=0: IRQs nunca vetoram (0096)
- [x] 4.4e Handler 80000080 despacha p/ tabela do kernel (0095)
- [x] 4.4f IRQ no delay slot sequestrava handler (0103)
- [x] 4.4g Ciclos do load; VSync cobria 69% do frame (0104)
- [x] 4.4i IRQ2 do CD-ROM sobe por borda (0114)
- [x] 4.4j sh/lhu no SIO0 perdiam byte alto (0115)
- [x] 4.4k DICR: flags, bit31 e IRQ3 (0116)
- [x] 4.4l Canal 2 DMA device->RAM (0117)
- [x] 4.4m lhu/sh nos timers caiam no sumidouro (0118)
- [x] 4.4n GetStat correto porto a porto (0119)
- [x] 4.4o Referencia DuckStation: falta GetID (0120)
- [x] 4.4p 1a resposta do CD-ROM pelo scheduler; GetID aparece (0121)
- [x] 4.4q GetID Licensed:Mode2+SCEA; retry acaba (0122)
- [x] 4.4r GetTOC arma 2a resposta INT2 (0123)
- [x] 4.4s Setor Mode2/Form1 offset sai do byte de modo (0124)
- [x] 4.4t Laco de dispatch de eventos do shell (0125)
- [x] 4.4u Evento da montagem do filesystem (0126)
- [x] 4.4v Eventos 10h/200h ready antes do TestEvent (0127)
- [x] 4.4w Descritor do TestEvent do shell (0128)
- [x] 4.4x DeliverEvent(F0000003h,20h); shell desenha (0129)
- [x] 4.4y Tela SCE igual a referencia; congela 120M→200M (0130)
- [x] 4.4z Reads entregavam setor N+150; TMD virava lixo (0131)
- [x] 4.4aa Pregap 150 no read; boot chega a licenca (0132)
- [x] 4.4ab BIOS re-envia Init; INT2 atropela INT3 (0133)
- [x] 4.4ac 2a resposta so apos ack (fila; fecha 10.54) (0134)
- [x] 4.4ad Motor de respostas: flags + timing por comando + avanco de seek (fecha 10.53) (0136)
- [ ] 4.5 1o frame do jogo: rollback do init do LIBSN; poll orfao do TMR2 (diag 0137)

## M5 — GTE
- [x] 5.1 Registradores + MFC2/MTC2/CFC2/CTC2/LWC2/SWC2 (iter 0084)
- [x] 5.2 RTPS/RTPT + divisão UNR (iter 0086)
- [x] 5.3 NCLIP/AVSZ3/AVSZ4/SQR/OP (iter 0088)
- [x] 5.4a MVMVA (iter 0089)
- [ ] 5.4b NCS/NCT/NCCS/NCCT
- [ ] 5.4c NCDS/NCDT/CC/CDP
- [ ] 5.4d DCPL/DPCS/DPCT/INTPL
- [ ] 5.5 Flags de saturação/overflow completos
- [ ] 5.6 Amidog psxtest_gte no scoreboard → jogo 3D jogável

## M6 — SIO: controle e memory card
- [x] 6.1 SIO0 + digital pad (iter 0091)
- [x] 6.2 Input no psx-desktop (teclado/gamepad) (iter 0092)
- [ ] 6.3 Memory card (.mcd)

## M7 — SPU
- [ ] 7.1 Regs de voz + ADPCM
- [ ] 7.2 Pitch/ADSR/volume + mixer
- [ ] 7.3 Saída cpal + ring buffer
- [ ] 7.4 Reverb + noise + CD-DA/XA

## M8 — MDEC
- [ ] 8.1 Regs + DMA canais 0/1
- [ ] 8.2 Macroblocos (RLE, IDCT, YUV→RGB) → FMVs

## M9 — App desktop
- [x] 9.0 psx-desktop boota BIOS (--bios CLI, CPU loop, framebuffer) (iter 0090)
- [ ] 9.1 Biblioteca: scan BIN/CUE, título/serial/região, lista
- [ ] 9.2 Snapshot do core (serde) → save states F5/F8 + slots
- [ ] 9.3 Memory cards automáticos por serial + tela de saves
- [ ] 9.4 Controles PS/Xbox (gilrs) + tela de mapeamento + perfis
- [ ] 9.5 Tela de configurações (BIOS, vídeo, áudio, pasta) em TOML
- [ ] 9.6 Fast-forward + recentes + tempo de jogo

## M10 — Precisão e compatibilidade
- [ ] 10.1 Timings finos (ps1-tests de timing)
- [ ] 10.2 Passe de compatibilidade (bugs viram itens 10.x)
- [x] 10.2a Placar do ps1-tests convertido em itens (iter 0068)
- [ ] 10.3 Bus error ao executar codigo do scratchpad (exposto por ps1-tests/cpu/code-in-io)
- [ ] 10.4 CAUSE.CE nao preenchido nas excecoes de Coprocessor Unusable (`docs/reference/02-cpu.md` L681)
- [ ] 10.5 Amidog psxtest_cpu para apos "args: 0" — causa nao investigada (iter 0032)
- [ ] 10.6 GP0(80h) VRAM->VRAM blit (hoje consumido e ignorado)
- [ ] 10.7 Mask setting GP0(E6h) aplicado a CPU->VRAM e VRAM->VRAM (`docs/reference/03-gpu.md` L590-592)
- [ ] 10.8 SWL/SWR fazem read-modify-write e leem portas de I/O de leitura destrutiva
- [x] 10.9 UV/CLUT dos polígonos texturizados (iter 0045)
- [ ] 10.10 Drawing Area GP0(E3h/E4h) e Drawing Offset GP0(E5h) sem suite de hardware que os meça
- [ ] 10.11 Textura e Texpage de retângulos (hoje UV consumido e ignorado)
- [x] 10.12 Meta-teste do placar pulava em silencio desde a 0041 (iter 0069)
- [ ] 10.28 Tabela por registro nos docs vs `.resultado` — 0071 errou 3/9, 0038 inflou 2 creditos
- [ ] 10.29 `dma_dpcr_gate.rs:141` usa `assert_ne!` como unica assercao numa correcao de defeito
- [ ] 10.30 Habilitar canal no DPCR nao dispara transferencia pendente (so na escrita de CHCR)
- [x] 10.31 Rodada morta seguia escrevendo (daemon) (iter 0101)
- [x] 10.32 GP1(09h) fecha gate nao limpa latch GPUSTAT.15 (iter 0076)
- [ ] 10.25 `unwrap_or(\"\")` sobre caminho nos meta-testes — transformou falha de `strip_prefix` em silencio (10.12)
- [ ] 10.13 GP0(24h) e modulacao, nao raw texture — bit 24 nao e lido (`docs/reference/03-gpu.md` L264/L1610); nao e causa do 2.2e (iter 0110)
- [ ] 10.14 U/V e cor gouraud sao reinterpolados sobre span recortado pela drawing area (`docs/reference/03-gpu.md` L452-454)
- [x] 10.16 `spec_citations.rs` casava titulo de secao por substring (iter 0083)
- [x] 10.15 Reparar ancoras do manifesto 0059 arquivado na 0060 (iter 0067)
- [ ] 10.17 `mutantes.ps1` recusa arvore suja — permitir sujeira restrita a `docs/mutantes/*.mut`
- [x] 10.19 DPCR gate nos tres canais do DMA (iter 0071)
- [x] 10.20 OTC grava o espelho do hardware (iter 0073)
- [x] 10.21 GPUSTAT.15 gateado por GP1(09h) (iter 0074)
- [x] 10.22 Mask-bit em CPU→VRAM (iter 0075)
- [ ] 10.24 Job `scoreboard` da CI sai VERDE medindo zero — sem BIOS rotula 51 suites `sem-bios` (iter 0072)
- [ ] 10.26 Nenhum dos 9 testes de `ci_scoreboard.rs` afirma que o job mede algo
- [ ] 10.27 Placar local (gitignored) e o unico com veredito real e nao e versionado
- [ ] 10.23 45 das 51 suites do scoreboard nao dao veredito (renderizam na VRAM)
- [ ] 10.18 Nada torna `arquivada:` caro — 0052 e 0059 descartaram 17 registros (12 ainda casavam)
- [ ] 10.33 `mutantes.ps1` so roda `cargo test -p psx-core` (linha 290) — outros crates precisam de bateria manual
- [ ] 10.34 Nenhum meta-teste reprova `#[test]` sem assercao — T10 da 0080 era eprintln! e passou
- [ ] 10.35 `mutantes.ps1` grava nome qualificado e `mutation_battery.rs` procura fn literal — nunca casam em modulo
- [x] 10.36 Sideload de PS-EXE sem vetor 0x80000080 (iter 0093)
- [x] 10.37 `oc-loop.ps1` anunciava merge que nao houve (iter 0094)
- [x] 10.38 oc-iter pagava 45 min por rodada de 90 s (iter 0098)
- [x] 10.39 Marco 100% fechado sai da escada para `docs/ROADMAP-fechado.md` (iter 0100)
- [ ] 10.40 `mutantes.ps1` so casa ancora em arquivo LF; em CRLF diz 'encontrada 0 vez(es)'
- [x] 10.41 STATUS handoff puro; invariantes por numero (iter 0102)
- [ ] 10.42 Linhas tremulas: framebuffer capturado sem sincronizar com vblank; 2o suspeito page flip do GP1(05h) (30/07)
- [ ] 10.42 Manifesto trata `#` como comentario dentro de `@@DE`/`@@PARA` — alvo `.md` nao ancora em cabecalho
- [ ] 10.43 Todo texto do TTY sai duplicado (2 linhas 'System ROM' na main e depois do 0103)
- [ ] 10.44 Manifesto com alvo em documento vivo (STATUS.md) envelhece na iteracao seguinte, sempre
- [ ] 10.45 Load shadow: acesso lento se sobrepoe as instrucoes seguintes (`docs/reference/02-cpu.md` L281-296)
- [ ] 10.46 `justificativa:` do equivalente nao aceita continuacao de linha em NENHUM dos dois parsers
- [ ] 10.52 `lhu`/`lbu` no modo do timer nao limpa bits 11/12 (byte usa `peek32`); 32 bits limpa (0118)
- [ ] 10.51 GPU em `region_read_byte`: `(phys & 3) + offset` sem mascara, desalinhado estoura em debug (0118)
- [ ] 10.50 `GP0(C0h)` sem transferencia pendente devolve zero, e dreno alem da janela le zeros (visto na 0117)
- [ ] 10.49 Bit 15 do `DICR` (bus error) e gravavel mas nada o levanta: transferencia fora da RAM e ignorada (visto na 0116)
- [ ] 10.48 `sw` em `1F801044h..1F80104Fh` cai no sumidouro de `region_write32`: JOY_MODE/JOY_CTRL engolidos (visto na 0115)
- [ ] 10.57 Regiao do GetID fixada em SCEA; ler o setor de licenca do `.bin` (0122)
- [x] 10.59 Expansion Region 2 aliasava RAM (POST corrompia 0x2041) (0138)
- [ ] 10.56 Result FIFO do comando anterior continua legivel na janela da primeira resposta (0121)
- [ ] 10.55 Atraso da primeira resposta ignora o motor: spec da `Nop (when stopped) 0x5CF4` (0121)
- [x] 10.54 2a resposta com atraso fisico apos o ack (06-cdrom.md L2066) (0121→0134)
- [ ] 10.53 Comando executa mesmo com INT pendente; spec exige esperar o ack (06-cdrom.md L1984) (0121)
- [ ] 10.47 Lacos de espera da BIOS (`0x80059DA4`/`0x80059D54`): orcamento 0x8000 giros, frame ~230 k passos, saem por timeout (0114)
- [x] 10.60 oc-iter/oc-loop no Linux; trabalhador vira gpt-5.6-luna --variant max (0139)
- [ ] 10.61 Fechado sai da escada mesmo em marco aberto; teto cai para 7 KB
- [ ] 10.58 mutantes.ps1 so roda psx-core; alvo por crate e revalidar 0078/0079 (invariante 29) (0125)

## M11 — Apresentação (incremental desde o M1)
- [x] 11.1 Relatório consolidado (docs/relatorio.md — atualizado a cada marco) (iter 0096)
- [ ] 11.2 Gráficos de metricas.csv + scoreboard-data
- [ ] 11.3 Roteiro de demo
