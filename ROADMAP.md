# ROADMAP

Cada item = **1 iteração = 1 PR** (commits test→feat→docs). Uma linha por item, sem prosa —
narrativa mora em `docs/iterations/NNNN-*.md`. Trabalho fora da escada ganha sufixo (`0012b`).
Teto de tamanho imposto por `roadmap_size.rs`.

## M0 — Infra e processo
- [x] 0.1 Repo, merge-commit-only, PR template (iter 0001)
- [x] 0.2 Workspace 3 crates + esqueleto (iter 0002)
- [x] 0.3 Meta-testes de processo (7) (iter 0005)
- [x] 0.4 CI check + commit-lint + proteção de branch (iter 0004)
- [x] 0.5 Docs de gestão (iter 0003)
- [x] 0.6 psx-spx fatiado em docs/reference com índice de seções (iter 0006)
- [x] 0.7 fetch de EXEs de teste + scoreboard esqueleto (iter 0007)
- [x] 0.8 Orquestração opencode/DeepSeek + smoke test (iter 0008/0008b)
- [x] 0.9 Carregamento de BIOS com validação de hash (1ª iteração do trabalhador) (iter 0009)
- [x] 0.10 Formato de manifesto de mutação + meta-teste (iter 0040)
- [x] 0.11 scripts/mutantes.ps1 + job de CI + reconciliação do placar (iter 0041)
- [x] 0.12 Verificador de citações de spec (iter 0043)

## M1 — CPU R3000A até o BIOS falar
- [x] 1.1 Scheduler + bus (KUSEG/KSEG0/KSEG1), RAM 2MB, BIOS ROM (iter 0010)
- [x] 1.2 Fetch/decode + LUI/ORI/SW (iter 0011)
- [x] 1.3 ALU: ADDU/SUBU/AND/OR/XOR/NOR/SLT/SLTU + imediatos (iter 0012)
- [x] 1.3b Shifts SLL/SRL/SRA/SLLV/SRLV/SRAV (fatiado de 1.3 na revisão da 0012) (iter 0013)
- [x] 1.4 Loads/stores + load delay slot (iter 0014)
- [x] 1.5 Branches/jumps + branch delay slot (iter 0015)
- [x] 1.6 MULT/MULTU/DIV/DIVU + HI/LO com stalls (iter 0016)
- [x] 1.7 LWL/LWR/SWL/SWR (iter 0018)
- [x] 1.8a COP0: SR/CAUSE/EPC/BadVaddr/PRID + MTC0/MFC0 + RFE (iter 0020)
- [x] 1.8b Mecanismo de exceção: overflow, syscall, break, AdEL/AdES, bit BD (iter 0021)
- [x] 1.9 Cache isolation + scratchpad + memory control stubs (iter 0022)
- [x] 1.10 Hook de TTY (A0h/B0h) → BIOS imprimindo no console (iter 0025)
- [x] 1.11 Sideload de PS-EXE no psx-cli + Amidog psxtest_cpu no scoreboard (iter 0027)
- [x] 1.11b Hook de printf A(3Fh) com expansão de % → Amidog imprimindo no TTY (iter 0029)
- [x] 1.11c printf: flags de largura e zero-pad (iter 0087)
- [x] 1.12 CI: job scoreboard ligado (iter 0031)
- [x] 1.13 Veredito no scoreboard: parse de saida das suites (iter 0036)
- [x] 1.14 Opcode nao implementado gera excecao (RI 0Ah / CpU 0Bh) em vez de panic (iter 0033)

## M2 — GPU (rasterizador por software)
- [x] 2.1 GPUSTAT + decodificação GP0/GP1 (iter 0035)
- [x] 2.2 VRAM 1MB + transfers (fill, CPU↔VRAM) (iter 0038)
- [ ] 2.2b VRAM->VRAM copy GP0(80h) — no-op hoje, bloqueio de boot de jogo
- [x] 2.3 Triângulos flat + gouraud (iter 0039)
- [x] 2.4 Quads, retângulos, linhas (iter 0042)
- [x] 2.5a Texpage GP0(E1h) + amostragem de textura 15bpp (iter 0044)
- [x] 2.5b Texturas 4bpp e 8bpp + CLUT (iter 0045)
- [x] 2.5c Texture window GP0(E2h) (iter 0046)
- [x] 2.6a Semi-transparência (blend B/2+F/2, B+F, B-F, B+F/4) (iter 0047)
- [x] 2.6b Dithering 24→15 bit (matriz 4x4) (iter 0048)
- [x] 2.6c Mask bit (proteção de pixel bit15=1) (iter 0049)
- [x] 2.7a Display registers GP1(05h-07h) (iter 0050)
- [x] 2.7b Timing NTSC/PAL, vblank IRQ (dividido da 2.7 por R4 — display regs ja implementado) (iter 0051)
- [x] 2.8 psx-desktop eframe/egui (iter 0052/0053)
- [x] 2.9 Suíte GPU do ps1-tests no scoreboard (iter 0054)

## M3 — DMA, IRQ, timers
- [x] 3.1 Interrupt controller (I_STAT/I_MASK) + COP0 (iter 0055)
- [x] 3.2 DMA regs + canal 6 (OTC) (iter 0056)
- [x] 3.3 DMA canal 2 GPU (block + linked-list) (iter 0057)
- [x] 3.4 Timers 0/1/2 — registradores e contagem básica (iter 0058)
- [x] 3.4b Timers — modos de sync Hblank/Vblank (iter 0059)
- [x] 3.4c Timers — fontes de clock Dotclock/Hblank (iter 0060)
- [x] 3.4d Timers — conexão de IRQ4/IRQ5/IRQ6 ao controlador (deferido da 3.4) (iter 0061)

## M4 — CDROM
- [x] 4.1 Regs/FIFOs/IRQs + GetStat/GetID/Test (iter 0062)
- [x] 4.2a Setloc/SeekL/Pause + estado do drive (iter 0063)
- [x] 4.2b Parser BIN/CUE (iter 0064)
- [x] 4.2c ReadN/ReadS + INT1/DRQSTS (máquina de estados) (iter 0065)
- [x] 4.3a DMA canal 3 (registradores + gate BFRD no DRQSTS + transferencia) (iter 0066)
- [x] 4.3b Acoplar DiscLayout + dados do .bin a entrega de setores (iter 0077)
- [x] 4.3c Flag --disc/--cue no psx-cli (iter 0078)
- [ ] 4.4 Boot de jogo 2D/menu
- [x] 4.4a Boot da BIOS no psx-cli (--bios sozinho) (iter 0079)
- [x] 4.4b Base de tempo: scheduler + vblank + IRQ0 (iter 0080)
- [x] 4.4c BIOS nunca escreve I_MASK (iter 0085)
- [ ] 4.4d I_MASK=0x0000 por todo o boot — bloqueio real, IRQs nunca vetoram
- [x] 4.4e Handler de excecao em 0x80000080 despacha para tabela de eventos do kernel (ehk) — VSync callbacks nao rodam (iter 0095)

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
- [ ] 10.31 `oc-loop` nao parqueia rodada morta (iter 0073)
- [x] 10.32 GP1(09h) fecha gate nao limpa latch GPUSTAT.15 (iter 0076)
- [ ] 10.25 `unwrap_or(\"\")` sobre caminho nos meta-testes — transformou falha de `strip_prefix` em silencio (10.12)
- [ ] 10.13 GP0(24h) e modulacao, nao raw texture — bit 24 do comando nao e lido (`docs/reference/03-gpu.md` L264 e L1610)
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
- [ ] 10.35 `mutantes.ps1` escreve nome qualificado no .resultado e `mutation_battery.rs` procura fn literal — nunca casam em modulo
- [x] 10.36 Interrupcoes nao funcionam em sideload de PS-EXE — vetor 0x80000080 nao configurado (iter 0093)
- [x] 10.37 `oc-loop.ps1` anunciava merge que nao aconteceu: `Wait-Checks` lia estado do commit anterior e falha de `gh pr merge` passava por sucesso (iter 0094)

## M11 — Apresentação (incremental desde o M1)
- [ ] 11.1 Relatório consolidado (docs/relatorio.md — atualizado a cada marco)
- [ ] 11.2 Gráficos de metricas.csv + scoreboard-data
- [ ] 11.3 Roteiro de demo
