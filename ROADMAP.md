# ROADMAP

Cada item = **1 iteração = 1 PR** (commits test→feat→docs). Uma linha por item, sem prosa —
narrativa mora em `docs/iterations/NNNN-*.md`. Trabalho fora da escada ganha sufixo (`0012b`).
Teto de tamanho imposto por `roadmap_size.rs`.

Itens fechados saem daqui para `docs/ROADMAP-fechado.md`; a escada mantém só o que FALTA.
Regra imposta por `roadmap_arquivo.rs`.

## M4 — CDROM
- [ ] 4.4 Boot de jogo 2D/menu
- [ ] 4.5 1o frame do jogo: rollback do init do LIBSN; poll orfao do TMR2 (diag 0137)

## M5 — GTE
- [ ] 5.4b NCS/NCT/NCCS/NCCT
- [ ] 5.4c NCDS/NCDT/CC/CDP
- [ ] 5.4d DCPL/DPCS/DPCT/INTPL
- [ ] 5.5 Flags de saturação/overflow completos
- [ ] 5.6 Amidog psxtest_gte no scoreboard → jogo 3D jogável

## M6 — SIO: controle e memory card
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
- [ ] 9.1 Biblioteca: scan BIN/CUE, título/serial/região, lista
- [ ] 9.2 Snapshot do core (serde) → save states F5/F8 + slots
- [ ] 9.3 Memory cards automáticos por serial + tela de saves
- [ ] 9.4 Controles PS/Xbox (gilrs) + tela de mapeamento + perfis
- [ ] 9.5 Tela de configurações (BIOS, vídeo, áudio, pasta) em TOML
- [ ] 9.6 Fast-forward + recentes + tempo de jogo

## M10 — Precisão e compatibilidade
- [ ] 10.1 Timings finos (ps1-tests de timing)
- [ ] 10.2 Passe de compatibilidade (bugs viram itens 10.x)
- [ ] 10.3 Bus error ao executar codigo do scratchpad (exposto por ps1-tests/cpu/code-in-io)
- [ ] 10.4 CAUSE.CE nao preenchido nas excecoes de Coprocessor Unusable (`docs/reference/02-cpu.md` L681)
- [ ] 10.5 Amidog psxtest_cpu para apos "args: 0" — causa nao investigada (iter 0032)
- [ ] 10.6 GP0(80h) VRAM->VRAM blit (hoje consumido e ignorado)
- [ ] 10.7 Mask setting GP0(E6h) aplicado a CPU->VRAM e VRAM->VRAM (`docs/reference/03-gpu.md` L590-592)
- [ ] 10.8 SWL/SWR fazem read-modify-write e leem portas de I/O de leitura destrutiva
- [ ] 10.10 Drawing Area GP0(E3h/E4h) e Drawing Offset GP0(E5h) sem suite de hardware que os meça
- [ ] 10.11 Textura e Texpage de retângulos (hoje UV consumido e ignorado)
- [ ] 10.28 Tabela por registro nos docs vs `.resultado` — 0071 errou 3/9, 0038 inflou 2 creditos
- [ ] 10.29 `dma_dpcr_gate.rs:141` usa `assert_ne!` como unica assercao numa correcao de defeito
- [ ] 10.30 Habilitar canal no DPCR nao dispara transferencia pendente (so na escrita de CHCR)
- [ ] 10.25 `unwrap_or(\"\")` sobre caminho nos meta-testes — transformou falha de `strip_prefix` em silencio (10.12)
- [ ] 10.13 GP0(24h) e modulacao, nao raw texture — bit 24 nao e lido (`docs/reference/03-gpu.md` L264/L1610); nao e causa do 2.2e (iter 0110)
- [ ] 10.14 U/V e cor gouraud sao reinterpolados sobre span recortado pela drawing area (`docs/reference/03-gpu.md` L452-454)
- [ ] 10.17 `mutantes.ps1` recusa arvore suja — permitir sujeira restrita a `docs/mutantes/*.mut`
- [ ] 10.24 Job `scoreboard` da CI sai VERDE medindo zero — sem BIOS rotula 51 suites `sem-bios` (iter 0072)
- [ ] 10.26 Nenhum dos 9 testes de `ci_scoreboard.rs` afirma que o job mede algo
- [ ] 10.27 Placar local (gitignored) e o unico com veredito real e nao e versionado
- [ ] 10.23 45 das 51 suites do scoreboard nao dao veredito (renderizam na VRAM)
- [ ] 10.18 Nada torna `arquivada:` caro — 0052 e 0059 descartaram 17 registros (12 ainda casavam)
- [ ] 10.33 `mutantes.ps1` so roda `cargo test -p psx-core` (linha 290) — outros crates precisam de bateria manual
- [ ] 10.34 Nenhum meta-teste reprova `#[test]` sem assercao — T10 da 0080 era eprintln! e passou
- [ ] 10.35 `mutantes.ps1` grava nome qualificado e `mutation_battery.rs` procura fn literal — nunca casam em modulo
- [ ] 10.40 `mutantes.ps1` so casa ancora em arquivo LF; em CRLF diz 'encontrada 0 vez(es)'
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
- [ ] 10.56 Result FIFO do comando anterior continua legivel na janela da primeira resposta (0121)
- [ ] 10.55 Atraso da primeira resposta ignora o motor: spec da `Nop (when stopped) 0x5CF4` (0121)
- [ ] 10.53 Comando executa mesmo com INT pendente; spec exige esperar o ack (06-cdrom.md L1984) (0121)
- [ ] 10.47 Lacos de espera da BIOS (`0x80059DA4`/`0x80059D54`): orcamento 0x8000 giros, frame ~230 k passos, saem por timeout (0114)
- [ ] 10.58 mutantes.ps1 so roda psx-core; alvo por crate e revalidar 0078/0079 (invariante 29) (0125)
- [ ] 10.66 Nenhum meta-teste re-executa bateria: .resultado verde pode mentir apos mudanca de constante (0143 achou m4 do 0098 assim)
- [ ] 10.63 Envoltorio do `-ContinueBranch` afirma "reprovado na revisao" mesmo apos morte por travamento; trabalhador grava isso no doc (0140)
- [ ] 10.65 Revisor via opencode isolado em /tmp nao le o repo e nao verifica citacao; precisa de acesso r/o (0141)
- [ ] 10.64 `evcb_descritor_mapeia_para_spec_correto` sozinho custa 447 s dos 449 s da suite sob nextest (0140)
- [ ] 10.68 `cdrom_evento_kernel` queima 150 M passos sem saida antecipada em 2 testes; o alvo aparece antes de 100 M (0144)

## M11 — Apresentação (incremental desde o M1)
- [ ] 11.2 Gráficos de metricas.csv + scoreboard-data
- [ ] 11.3 Roteiro de demo
