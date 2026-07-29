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
- [x] 4.3a DMA canal 3 (registradores + gate BFRD no DRQSTS + transferência) (iter 0066)
- [ ] 4.3b Acoplar DiscLayout + dados do .bin à entrega de setores
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
- [x] 10.2a Primeiro passe: placar do ps1-tests lido e convertido nos itens 10.19-10.23 (iter 0068)
- [ ] 10.3 Bus error ao executar codigo do scratchpad (exposto por ps1-tests/cpu/code-in-io)
- [ ] 10.4 CAUSE.CE nao preenchido nas excecoes de Coprocessor Unusable (02-cpu.md L681)
- [ ] 10.5 Amidog psxtest_cpu para apos "args: 0" — causa nao investigada (medido na iter 0032)
- [ ] 10.6 GP0(80h) VRAM->VRAM blit (hoje consumido e ignorado)
- [ ] 10.7 Mask setting GP0(E6h) aplicado a CPU->VRAM e VRAM->VRAM (03-gpu.md L590-592)
- [ ] 10.8 SWL/SWR fazem read-modify-write e leem portas de I/O de leitura destrutiva
- [x] 10.9 UV/CLUT dos polígonos texturizados (hoje consumidos e ignorados) (iter 0045)
- [ ] 10.10 Drawing Area GP0(E3h/E4h) e Drawing Offset GP0(E5h) sem suite de hardware que os meça
- [ ] 10.11 Textura e Texpage de retângulos (hoje UV consumido e ignorado)
- [x] 10.12 Meta-teste do placar pula em silêncio quando o doc não é legível (`Err(_) => continue`) — passou local e reprovou na CI na iter 0042. Causa real: `relative()` devolvia separador nativo, então o portão pulava os 25 manifestos na máquina local desde a 0041 (iter 0069)
- [ ] 10.25 Varrer os outros `unwrap_or("")` sobre caminho nos meta-testes: foi o que transformou a falha de `strip_prefix` em silêncio na 10.12, e o separador foi só o gatilho
- [ ] 10.13 GP0(24h) é modulação, não raw texture: bit 24 do comando não é lido e o texel vai cru (03-gpu.md L264 e L1610)
- [ ] 10.14 U/V e cor gouraud são reinterpolados sobre o span já recortado pela drawing area — a textura estica em vez de só perder os pixels de fora (03-gpu.md L452-454)
- [ ] 10.16 `spec_citations.rs` casa mal título e referência quando a mesma linha tem 2+ títulos entre aspas e 2+ refs: ele usa o primeiro título para todos os refs em vez de parear pelo mais próximo. Na iteração 0066 isso produziu o diagnóstico "L940 não corresponde à seção 'ReadN/ReadS'" quando a L940 pertencia à seção seguinte e estava citada com o texto certo ao lado. Diagnóstico errado é pior que ambiguidade declarada — ou pareia por proximidade, ou falha como ambíguo, como já faz quando há 2+ arquivos na linha
- [x] 10.15 Reparar as âncoras do manifesto 0059 (`timers-sync`), arquivado na 0060 quando o `tick()` foi reescrito: 4 dos 9 registros não casam mais, e a bateria daquele item está sem rodar. O 0052 foi reparado no mesmo dia com quatro caracteres (`fn` → `pub fn`), o que sugere que arquivar foi resposta cara demais para o problema (iter 0067)
- [ ] 10.17 `mutantes.ps1` recusa árvore suja, então reparo de âncora só pode ser verificado depois de commitado às cegas — permitir sujeira restrita a `docs/mutantes/*.mut`
- [ ] 10.19 DPCR nunca é consultado: os três `try_execute_*` do DMA olham só CHCR bits 24/28 e transferem com o canal desabilitado — `otc-test` reprova 4 subtestes só por isso
- [ ] 10.20 A lista que o OTC escreve é o espelho da do hardware (terminador e sentido dos ponteiros); `dma_otc.rs:79-81` afirma o espelho, então 13 testes verdes certificam o defeito
- [ ] 10.21 GP0(E1h) escreve o bit 15 do GPUSTAT (Texture Disable) sem o gate de GP1(09h) — `gpu/gp0-e1` reprova 3 de 10
- [ ] 10.22 `gpu/mask-bit` reprova 2 de 5 desde que passou a dar veredito; provável sobreposição com 10.7
- [ ] 10.24 `logs/` é gitignored, então `scoreboard.csv` — a única medida contra hardware real — não está no repositório e some com a máquina. Versionar o placar (ou um digest por commit)
- [ ] 10.23 45 das 51 suites do scoreboard não dão veredito porque renderizam na VRAM: 88% do placar não mede nada. A `diffvram` do ps1-tests já está baixada
- [ ] 10.18 Nada torna `arquivada:` caro: em 0052 e 0059, arquivar descartou 17 registros dos quais 12 ainda casavam. Exigir no header quantos casam e falhar se algum casar

## M11 — Apresentação (incremental desde o M1)
- [ ] 11.1 Relatório consolidado (docs/relatorio.md — atualizado a cada marco)
- [ ] 11.2 Gráficos de metricas.csv + scoreboard-data
- [ ] 11.3 Roteiro de demo
