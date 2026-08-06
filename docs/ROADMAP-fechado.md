# ROADMAP — marcos fechados

Historico. Itens fechados saem de `ROADMAP.md` e vêm para cá, para o teto de 7 KB
da escada valer so para o que FALTA. Narrativa de cada item continua em
`docs/iterations/NNNN-*.md`. Regra imposta por `roadmap_arquivo.rs`.

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
- [x] 2.2b VRAM->VRAM copy GP0(80h) — mascara, wrap e coordenadas absolutas (iter 0105)
- [x] 2.2c Endereco do texel 4bpp/8bpp somava a linha duas vezes — logo da BIOS legivel (iter 0106)
- [x] 2.2d Losango do logo — era fotografia de boot morto no 4.4h; com o fix, losango completo e centrado na cena 480i (iter 0111)
- [x] 2.2e Cores do logo — SONY e COMPUTER ENTERTAINMENT azul-escuro; fundo termina em B4B4B4 (fade completo; render de referencia mostra branco — diferenca anotada) (iter 0111)
- [x] 2.2f `COMPUTER ENTERTAINMENT` desenhado apos o 4.4h cair (iter 0111)
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
- [x] 2.10 `framebuffer_for_display` le GPUSTAT.23 invertido — polaridade corrigida; d1/d2/d3 da 0053 e o teste da 0090 virados com citacao (iter 0112)
- [x] 2.11 Altura do display em 480i: `display_height` dobra o range com GPUSTAT.19/22 ligados — tela do logo inteira na janela do app (iter 0113)

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

## M5 — GTE
- [x] 5.1 Registradores + MFC2/MTC2/CFC2/CTC2/LWC2/SWC2 (iter 0084)
- [x] 5.2 RTPS/RTPT + divisão UNR (iter 0086)
- [x] 5.3 NCLIP/AVSZ3/AVSZ4/SQR/OP (iter 0088)
- [x] 5.4a MVMVA (iter 0089)

## M6 — SIO: controle e memory card
- [x] 6.1 SIO0 + digital pad (iter 0091)
- [x] 6.2 Input no psx-desktop (teclado/gamepad) (iter 0092)

## M8 — MDEC
- [x] 8.1 Regs 1F801820h/24h, tabelas de quant/escala, DMA canais 0/1 (iter 0174)

## M9 — App desktop
- [x] 9.0 psx-desktop boota BIOS (--bios CLI, CPU loop, framebuffer) (iter 0090)

## M10 — Precisão e compatibilidade
- [x] 10.2a Placar do ps1-tests convertido em itens (iter 0068)
- [x] 10.9 UV/CLUT dos polígonos texturizados (iter 0045)
- [x] 10.12 Meta-teste do placar pulava em silencio desde a 0041 (iter 0069)
- [x] 10.31 Rodada morta seguia escrevendo (daemon) (iter 0101)
- [x] 10.32 GP1(09h) fecha gate nao limpa latch GPUSTAT.15 (iter 0076)
- [x] 10.16 `spec_citations.rs` casava titulo de secao por substring (iter 0083)
- [x] 10.15 Reparar ancoras do manifesto 0059 arquivado na 0060 (iter 0067)
- [x] 10.19 DPCR gate nos tres canais do DMA (iter 0071)
- [x] 10.20 OTC grava o espelho do hardware (iter 0073)
- [x] 10.21 GPUSTAT.15 gateado por GP1(09h) (iter 0074)
- [x] 10.22 Mask-bit em CPU→VRAM (iter 0075)
- [x] 10.36 Sideload de PS-EXE sem vetor 0x80000080 (iter 0093)
- [x] 10.37 `oc-loop.ps1` anunciava merge que nao houve (iter 0094)
- [x] 10.38 oc-iter pagava 45 min por rodada de 90 s (iter 0098)
- [x] 10.39 Marco 100% fechado sai da escada para `docs/ROADMAP-fechado.md` (iter 0100)
- [x] 10.41 STATUS handoff puro; invariantes por numero (iter 0102)
- [x] 10.59 Expansion Region 2 aliasava RAM (POST corrompia 0x2041) (0138)
- [x] 10.54 2a resposta com atraso fisico apos o ack (06-cdrom.md L2066) (0121→0134)
- [x] 10.60 oc-iter/oc-loop no Linux; trabalhador vira gpt-5.6-luna --variant max (0139)
- [x] 10.61 Fechado sai da escada mesmo em marco aberto; teto cai para 7 KB (iter 0140)
- [x] 10.62 Janela de travamento de 5 min matava rodada no portao do passo 7 (0143)
- [x] 10.67 mutation_battery so procurava fn de teste em crates/psx-core/tests; bateria de outro crate nunca era validada (0144)
- [x] 10.69 Cargo.toml da raiz sem nenhum [profile]: suite inteira em opt-level 0, 528 s contra 76 s no testevent_descritor (0145)
- [x] 10.70 Sondagem da EvCB rodava a cada passo; amostragem a cada 10 k derruba o teste de 100 s para 12,7 s (0146)
- [x] 10.72 `parse_cue` guarda um unico bin_path; cue multi-file fica com a track de audio — registrado, Rayman roda com cue reduzido (0148)
- [x] 10.73 VSync do Rayman: IRQ0 chega e a CPU vetoriza, mas o dispatch da BIOS nao alcanca o handler do jogo (0149)
- [x] 10.74 Rayman instala o VSync por `B(19h) HookEntryInt`, nao por OpenEvent/SetRCnt/vetor (0150)
- [x] 10.75 Rayman desvia em `0x801B8EA0` quando `I_STAT & I_MASK == 0`, antes do caminho de `0x801B8C40` (0151)
- [x] 10.76 Rayman tem `ExcCode=00h` em 1029 hooks; um VBlank chega ao `sw`, que escreve `0x801CF2CC`, nao `0x801DF2CC` (0152)
- [x] 10.79 Enderecos de leitura e escrita do contador convergem em `0x801CF2CC`; 1 de 1029 hooks tem VBlank pendente (0153)
- [x] 10.80 `0x00004A1C` e o handler de IRQ0; acka I_STAT antes de consultar a entrega de evento (0154)
- [x] 10.81 Rayman: os 458 intervalos sem ack são CDROM (bit 2, 173) ou DMA (bit 3, 285) (0155)
- [x] 10.82 Rayman: periodo IRQ0 mediano de 566187 ciclos, igual ao frame NTSC; taxa correta, sem defeito (iter 0156)
- [x] 10.84 SIO0 pedia IRQ7 sem periferico; /ACK agora so do dispositivo enderecado (iter 0159)
- [x] 10.86 /ACK do SIO0 chegava em 0 ciclos, o que a spec proibe emular; agora vem do scheduler (iter 0160)
- [x] 10.87 O auto-ack de IRQ0 do Pad/Card e do BIOS: o jogo desliga e `StartPAD2` religa (iter 0161)
- [x] 10.88 Premissa refutada: os descritores esperados eram de CDROM, nao de memory card (iter 0162)
- [x] 10.89 Premissa refutada: o 2o `KERNEL SETUP` e do BOOTSTRAP LOADER, boot normal (iter 0163)
- [x] 10.91 Fetch desalinhado levanta AdEL; Amidog sai de 00000909 para 00000109 (iter 0164)
- [x] 10.92 Amidog: 4312 erros em codificacoes de branch `b_0xNN` (bltz/bgez/bltzal/bgezal) (iter 0166)
- [x] 10.93 Amidog: ~590 erros de load delay slot encadeado `nop_lX_lY_d` (iter 0165)
- [x] 10.96 psx-cli conecta pad digital e aperta botoes por passo (--pad/--press) (iter 0169)
- [x] 10.95 `--exe` agora boota o kernel de verdade ate 0x80030000 antes de sobrepor o PS-EXE (iter 0170)
- [x] 10.97 TTY duplicado com kernel real (iter 0171)
- [x] 10.43 TTY duplicado ('System ROM' 2x): mesma causa do 10.97, fechado por medicao (iter 0171)
- [x] 10.98 oraculo-tty: alinhamento e prefixo uniforme (iter 0171)
- [x] 10.99 cpu/cop: Coprocessor Unusable pelo bit CU do SR, 19/19 -> 1/19 (iter 0171)
- [x] 10.105 CI: 188 s de link contra 14 s de execucao; debuginfo desligado no workflow (iter 0177)
- [x] 10.110 AutoPause: INT4 no fim da trilha; `[0x801CEEBC]` finalmente vira 1 (iter 0180)
- [x] 10.111 `.cue` com um arquivo por trilha: concatena e resolve LBA absoluto (iter 0180)
- [x] 0182.2 REFUTADO: o hook do jogo ver VBlank 10x em 660 e correto — `VblankIrq` (prio 1) faz ack e chama ReturnFromException, que pula o hook (iter 0183)
- [x] 10.106 spec_citations varria `.claude/worktrees/`: 295 erros de outra arvore (iter 0177)
- [x] 10.3 Bus error (06h) ao buscar instrucao no scratchpad/I_STAT/MDEC (iter 0172)
- [x] 10.100 cpu/cop: `testCop0InvalidOpcode` — so TLBxx lanca reservado, resto e no-op (iter 0172)

## M11 — Apresentação (incremental desde o M1)
- [x] 11.1 Relatório consolidado (docs/relatorio.md — atualizado a cada marco) (iter 0096)

## Achados fechados na iteração 0184
- [x] 10.94 Rayman: laco `0x80132BF0` — era o callback do DMA1 (MDECout) que nunca voltava (0184)
- [x] 0183.2 byte0 de `[0x801CF5F4]`: escrito por `0x80132A30`, fim da cadeia de faixas do MDEC (0184)
- [x] 4.4 Boot de jogo 2D/menu: Rayman entra no primeiro nivel (0184)
- [x] 8.2 Macroblocos coloridos 15/24bpp com yuv_to_rgb (0184)

## Fechados na iteração 0185
- [x] 5.4b NCS/NCT/NCCS/NCCT (0185)
- [x] 5.4c NCDS/NCDT/CC/CDP (0185)
- [x] 5.4d DCPL/DPCS/DPCT/INTPL (0185)

## Fechados na iteração 0186
- [x] 4.4ae Crash Bandicoot joga N. Sanity Beach: menu, load e nivel (0186)
- [x] 0185.2 `GPU timeout` do Crash: teto artificial de 4096 nos cortava a lista encadeada do DMA2 (0186)
- [x] 0186.1 SIO trocava os dois bytes de switches: `swlo` (bit0-7, onde mora Start) vem primeiro (0186)

## Fechados na iteração 0187-0192 (M4, M5, M6 e M7 inteiros)
- [x] 4.5 1o frame: rollback do init do LIBSN; poll orfao do TMR2 (diag 0137-0147) (iter 0192)
- [x] 5.5 Flags de saturação/overflow completos (por parcela, nao no total) (iter 0190)
- [x] 5.6 Placar do GTE contra hardware: 1100/1100 no gte-fuzz do ps1-tests (iter 0190)
- [x] 6.3 Memory card (.mcd) (iter 0191)
- [x] 7.1 Regs de voz + ADPCM (iter 0187)
- [x] 7.2 Pitch/ADSR/volume + mixer (iter 0187)
- [x] 7.3 Saída cpal + ring buffer (iter 0189)
- [x] 7.4 Reverb + noise + CD-DA/XA (iter 0188)
- [x] 10.101 SPU/DMA4 ausente bloqueia dma/dpcr (13/15) (0173) (iter 0187)
- [x] 10.112 SPU sem estado: fetch em 1F801C00h vira NOP infinito (0172) (iter 0187)
- [x] 0185.1 flags de overflow do MAC conferidas no total, nao por parcela (0185) (iter 0190)

## Fechados na iteração 0193-0198 (M9 inteiro)
- [x] 9.1 Biblioteca: scan BIN/CUE, título/serial/região, lista (iter 0193)
- [x] 9.2 Snapshot do core (serde) → save states F5/F8 + slots (iter 0194)
- [x] 9.3 Memory cards automáticos por serial + tela de saves (iter 0195)
- [x] 9.4 Controles PS/Xbox (gilrs) + tela de mapeamento + perfis (iter 0196)
- [x] 9.5 Tela de configurações (BIOS, vídeo, áudio, pasta) em TOML (iter 0197)
- [x] 9.6 Fast-forward + recentes + tempo de jogo (iter 0198)
- [x] 10.23 Scoreboard com veredito de VRAM via diffvram: 13 suítes ganharam veredito gráfico (iter 0199)
- [x] 10.11 Retângulos texturizados (raw+CLUT+STP+wrap); rectangles 11.560px → 7.265px (iter 0200)

## Fechado na iteração 0201
- [x] 10.53 Comando executa com INT pendente; spec exige o ack (06-cdrom.md L1984) (0121) (iter 0201)
- [x] 10.48 `sw` em `1F801044h..104Fh` encaminhado para MODE, CTRL e BAUD do SIO (iter 0201; achado 0115)
- [x] 10.30 Habilitar canal no DPCR nao dispara transferencia pendente (so no CHCR) (iter 0201)
- [x] 10.51 GPU em `region_read_byte`: mascara do indice de byte evita shift alem de `u32` (iter 0201)

## Fechado na iteração 0202
- [x] 10.13 GP0(24h) e modulacao, nao raw texture (03-gpu.md L264/L1610) (0110) (iter 0202)

## Fechado na iteração 0204
- [x] 0203.4 Máscara final do índice de byte em `region_read_byte` aplicada aos 4 sítios
  irmãos do 10.51 (MEM_CTRL, espelho do MEM_CTRL, BCC, DMA) que o PR #215 não cobriu
  (iter 0204)

## Fechado na iteração 0203
- [x] 10.4 CAUSE.CE nao preenchido no Coprocessor Unusable (02-cpu.md L681) (iter 0203)
- [x] 10.6 GP0(80h) VRAM->VRAM blit — achado ficou desatualizado: ja implementado com
  mascara/wrap/coordenadas absolutas desde a iter 0105 (`execute_vram_to_vram`,
  `crates/psx-core/src/gpu.rs`); confirmado sem mudanca de codigo na revisao da 0203
- [x] 10.7 Mask GP0(E6h) em CPU->VRAM e VRAM->VRAM — achado ficou desatualizado: os dois
  caminhos ja checam `force_bit15`/`check_mask` (GPUSTAT bits 11/12) desde a iter 0049/0105;
  confirmado sem mudanca de codigo na revisao da 0203
- [x] 10.50 `GP0(C0h)` sem transferencia devolve zero (0117) (iter 0203)
- [x] 10.14 U/V e gouraud reinterpolados sobre span recortado (03-gpu.md L452) (iter 0203) —
  caminho dither+gouraud+nao-texturizado (`render_triangle_dithered`) tem o mesmo padrao mas
  fica pra depois sem teste dedicado; ver achado 0203.1
- [x] 10.42 Linhas tremulas: captura sem sync com vblank (03-gpu.md L1426) (iter 0203) —
  framebuffer() passa a ler um snapshot de VRAM latchado em enter_vblank(), nao VRAM ao vivo
- [x] 10.117 (parcial) hblank nunca agendado — HBLANK_ENTER/HBLANK_EXIT agora sao eventos
  reais do scheduler, uma vez por scanline (03-gpu.md L826/L1469) (iter 0203); a outra metade
  do achado ("System Clock" diverge ~13-70x, causa raiz desconhecida desde a 0176) continua
  aberta como 0203.3
- [x] 10.109 SyncMode=0 com chopping: MADR e BC atualizados no fim do burst do DMA2
  (04-dma.md L48-50, L80-81) (iter 0203) — so o estado final, sem cycle-stealing real
  (achados 10.102/10.114 continuam abertos)

## Fechado na iteração 0205
- [x] 10.49 Bit 15 do DICR (bus error) levantado quando o endereco do DMA excede o campo de
  24 bits do MADR (04-dma.md L48-50, L119-135) (iter 0205) — trabalhador fez teste+fix,
  orquestrador corrigiu uma regressao real (endereco com bits 21-23 legitimos sendo tratado
  como erro) e completou passos 6-9

## Fechado na iteração 0206
- [x] 10.55 Atraso da 1a resposta usa 0005cf4h quando o motor esta parado, nao sempre
  000c4e1h (06-cdrom.md L2047-2054) (iter 0206)
- [x] 10.56 Result FIFO anterior legivel na janela da 1a resposta — achado ficou desatualizado
  (send_command/result_clear so rodam quando deliver_first() entrega a resposta nova, nao na
  escrita do comando); confirmado sem mudanca de codigo na revisao da 0206
