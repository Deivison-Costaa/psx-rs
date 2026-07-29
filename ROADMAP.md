# ROADMAP

Cada item = **1 iteração = 1 PR** (commits test→feat→docs). Uma linha por item, sem prosa —
narrativa mora em `docs/iterations/NNNN-*.md`. Trabalho fora da escada ganha sufixo (`0012b`).
Teto de tamanho imposto por `roadmap_size.rs`.

## M0 — Infra e processo
- [x] 0.1 Repo público, merge-commit-only, template de PR (iter 0001)
- [x] 0.2 Workspace 3 crates + esqueleto de módulos (iter 0002)
- [x] 0.3 Meta-testes de processo (7) (iter 0005)
- [x] 0.4 CI check + commit-lint + proteção de branch (iter 0004)
- [x] 0.5 Docs de gestão (iter 0003)
- [x] 0.6 psx-spx fatiado em docs/reference com índice de seções (iter 0006)
- [x] 0.7 fetch de EXEs de teste + scoreboard esqueleto (iter 0007)
- [x] 0.8 Orquestração opencode/DeepSeek + smoke test de ponta a ponta (iter 0008/0008b)
- [x] 0.9 Carregamento de BIOS com validação de hash (1ª iteração do trabalhador) (iter 0009)
- [x] 0.10 Formato de manifesto de mutação + meta-teste (iter 0040)
- [x] 0.11 scripts/mutantes.ps1 + job de CI + reconciliação do placar (iter 0041)
- [x] 0.12 Verificador de citações de spec (iter 0043)

## M1 — CPU R3000A até o BIOS falar
- [x] 1.1 Scheduler de eventos + bus (KUSEG/KSEG0/KSEG1), RAM 2MB, BIOS ROM (iter 0010)
- [x] 1.2 Fetch/decode + LUI/ORI/SW (iter 0011)
- [x] 1.3 ALU: ADDU/SUBU/AND/OR/XOR/NOR/SLT/SLTU + imediatos (iter 0012)
- [x] 1.3b Shifts SLL/SRL/SRA/SLLV/SRLV/SRAV (fatiado de 1.3 na revisão da 0012) (iter 0013)
- [x] 1.4 Loads/stores + load delay slot (iter 0014)
- [x] 1.5 Branches/jumps + branch delay slot (iter 0015)
- [x] 1.6 MULT/MULTU/DIV/DIVU + HI/LO com stalls (iter 0016)
- [x] 1.7 LWL/LWR/SWL/SWR (iter 0018)
- [x] 1.8a COP0: registradores SR/CAUSE/EPC/BadVaddr/PRID + MTC0/MFC0 + RFE (sem exceções) (iter 0020)
- [x] 1.8b Mecanismo de exceção: overflow, syscall, break, AdEL/AdES, bit BD (iter 0021)
- [x] 1.9 Cache isolation + scratchpad + memory control stubs (iter 0022)
- [x] 1.10 Hook de TTY (A0h/B0h) → BIOS imprimindo no console (iter 0025)
- [x] 1.11 Sideload de PS-EXE no psx-cli + Amidog psxtest_cpu no scoreboard (iter 0027)
- [x] 1.11b Hook de printf A(3Fh) com expansão de % → Amidog imprimindo no TTY (iter 0029)
- [x] 1.12 CI: job scoreboard ligado (iter 0031)
- [x] 1.13 Veredito real no scoreboard: ler a saida de cada suite e extrair pass/fail (depende do 2.1 — GPUSTAT + decodificacao GP0/GP1) (iter 0036)
- [x] 1.14 Opcode nao implementado gera excecao (RI 0Ah / CpU 0Bh) em vez de panic (iter 0033)

## M2 — GPU (rasterizador por software)
- [x] 2.1 GPUSTAT + decodificação GP0/GP1 (iter 0035)
- [x] 2.2 VRAM 1MB + transfers (fill, CPU↔VRAM) (iter 0038)
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
- [x] 2.8 psx-desktop mínimo (eframe/egui) → logo do BIOS na tela (iter 0052)
- [x] 2.8b psx-desktop janela eframe/egui (deferido da 2.8 por incompatibilidade Rust 1.85 vs eframe 0.35) (iter 0053)
- [x] 2.9 Suíte GPU do ps1-tests no scoreboard (iter 0054)

## M3 — DMA, IRQ, timers
- [ ] 3.1 Interrupt controller (I_STAT/I_MASK) + COP0
- [ ] 3.2 DMA regs + canal 6 (OTC)
- [ ] 3.3 DMA canal 2 GPU (block + linked-list)
- [ ] 3.4 Timers 0/1/2

## M4 — CDROM
- [ ] 4.1 Regs/FIFOs/IRQs + GetStat/GetID/Test
- [ ] 4.2 Parser BIN/CUE + Setloc/SeekL/ReadN/ReadS/Pause/Init
- [ ] 4.3 DMA canal 3 + entrega de setores
- [ ] 4.4 Boot de jogo 2D/menu

## M5 — GTE
- [ ] 5.1 Registradores + MFC2/MTC2/CFC2/CTC2/LWC2/SWC2
- [ ] 5.2 RTPS/RTPT + divisão UNR
- [ ] 5.3 NCLIP/AVSZ3/AVSZ4/SQR/OP
- [ ] 5.4 MVMVA + comandos de iluminação (NCS/NCT/NCDS/NCCS...)
- [ ] 5.5 Flags de saturação/overflow completos
- [ ] 5.6 Amidog psxtest_gte no scoreboard → jogo 3D jogável

## M6 — SIO: controle e memory card
- [ ] 6.1 SIO0 + digital pad
- [ ] 6.2 Input no psx-desktop (teclado/gamepad)
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
- [ ] 9.1 Biblioteca: pasta de jogos, scan BIN/CUE, título/serial/região, lista
- [ ] 9.2 Snapshot do core (serde) → save states F5/F8 + slots
- [ ] 9.3 Memory cards automáticos por serial + tela de saves
- [ ] 9.4 Controles PS/Xbox (gilrs) + tela de mapeamento + perfis
- [ ] 9.5 Tela de configurações (BIOS, vídeo, áudio, pasta) em TOML
- [ ] 9.6 Fast-forward + recentes + tempo de jogo

## M10 — Precisão e compatibilidade
- [ ] 10.1 Timings finos (ps1-tests de timing)
- [ ] 10.2 Passe de compatibilidade (bugs viram itens 10.x)
- [ ] 10.3 Bus error ao executar codigo do scratchpad (exposto por ps1-tests/cpu/code-in-io)
- [ ] 10.4 CAUSE.CE nao preenchido nas excecoes de Coprocessor Unusable (02-cpu.md L681)
- [ ] 10.5 Amidog psxtest_cpu para apos "args: 0" — causa nao investigada (medido na iter 0032)
- [ ] 10.6 GP0(80h) VRAM->VRAM blit (hoje consumido e ignorado)
- [ ] 10.7 Mask setting GP0(E6h) aplicado a CPU->VRAM e VRAM->VRAM (03-gpu.md L590-592)
- [ ] 10.8 SWL/SWR fazem read-modify-write e leem portas de I/O de leitura destrutiva
- [x] 10.9 UV/CLUT dos polígonos texturizados (hoje consumidos e ignorados) (iter 0045)
- [ ] 10.10 Drawing Area GP0(E3h/E4h) e Drawing Offset GP0(E5h) sem suite de hardware que os meça
- [ ] 10.11 Textura e Texpage de retângulos (hoje UV consumido e ignorado)
- [ ] 10.12 Meta-teste do placar pula em silêncio quando o doc não é legível (`Err(_) => continue`) — passou local e reprovou na CI na iter 0042
- [ ] 10.13 GP0(24h) é modulação, não raw texture: bit 24 do comando não é lido e o texel vai cru (03-gpu.md L264 e L1610)
- [ ] 10.14 U/V e cor gouraud são reinterpolados sobre o span já recortado pela drawing area — a textura estica em vez de só perder os pixels de fora (03-gpu.md L452-454)

## M11 — Apresentação (incremental desde o M1)
- [ ] 11.1 Relatório consolidado (docs/relatorio.md — atualizado a cada marco)
- [ ] 11.2 Gráficos de metricas.csv + scoreboard-data
- [ ] 11.3 Roteiro de demo
