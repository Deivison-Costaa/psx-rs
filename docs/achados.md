# ACHADOS

> Defeitos e divergências descobertos por medição, um por linha. **Não é a escada** — a escada
> é `ROADMAP.md`, e ela responde "o que construir a seguir". Este arquivo responde "o que está
> errado e ainda não foi consertado".
>
> **Numeração: `NNNN.k`**, onde `NNNN` é a iteração que ACHOU o item. Nunca reaproveite número,
> nunca renumere item alheio. Rodadas paralelas colidiam com o esquema antigo `10.x` — na noite
> de 02-03/08 dois lotes escolheram `10.108` e dois escolheram `10.102`, e o orquestrador teve
> de renumerar na hora do merge. Com o número da iteração isso é impossível por construção.
>
> **Acrescente no FIM da sua seção**, nunca no meio: append não dá conflito de merge, inserção dá.
>
> Itens fechados saem daqui para `docs/ROADMAP-fechado.md`. O teto de 24 KB é de
> `achados_size.rs`; contexto e narrativa moram em `docs/iterations/NNNN-*.md`.

## Legado (numeração `10.x`, anterior à iteração 0181)
- [ ] 10.1 Timings finos (ps1-tests de timing)
- [ ] 10.2 Passe de compatibilidade (bugs viram itens 10.x)
- [ ] 10.5 Amidog psxtest_cpu para apos "args: 0" — causa nao investigada (iter 0032)
- [ ] 10.8 SWL/SWR fazem read-modify-write e leem portas de I/O de leitura destrutiva
- [ ] 10.10 Drawing Area GP0(E3h/E4h) e Offset GP0(E5h) sem suite que os meça
- [ ] 10.28 Tabela por registro nos docs vs `.resultado`: 0071 errou 3/9, 0038 inflou 2
- [ ] 10.29 `dma_dpcr_gate.rs:141`: `assert_ne!` como unica assercao de uma correcao
- [ ] 10.25 `unwrap_or` de caminho nos meta-testes silencia `strip_prefix` que falha (10.12)
- [ ] 10.17 `mutantes.ps1` recusa arvore suja; permitir `docs/mutantes/*.mut`
- [ ] 10.24 Job `scoreboard` da CI sai VERDE medindo zero (0072)
- [ ] 10.26 Nenhum dos 9 testes de `ci_scoreboard.rs` afirma que o job mede algo
- [ ] 10.27 Placar local (gitignored) e o unico com veredito real e nao e versionado
- [ ] 10.18 Nada torna `arquivada:` caro: 0052 e 0059 largaram 17 registros, 12 casavam
- [ ] 10.33 `scripts/mutantes.ps1` so roda `cargo test -p psx-core`; outros crates a mao
- [ ] 10.34 Nenhum meta-teste reprova `#[test]` sem assercao (T10 da 0080 passou assim)
- [ ] 10.35 `mutantes.ps1` grava nome qualificado, `mutation_battery` busca fn literal
- [ ] 10.40 `mutantes.ps1` so casa ancora em LF; CRLF da 'encontrada 0 vez(es)'
- [ ] 10.42b Manifesto trata `#` como comentario em `@@DE`/`@@PARA` (alvo `.md`)
- [ ] 10.44 Manifesto com alvo em documento vivo (STATUS.md) envelhece sempre
- [ ] 10.45 Load shadow sobrepoe as instrucoes seguintes (`docs/reference/02-cpu.md` L281)
- [ ] 10.46 `justificativa:` do equivalente nao aceita continuacao de linha
- [ ] 10.52 `lhu`/`lbu` no modo do timer nao limpa bits 11/12 (0118)
- [ ] 10.49 Bit 15 do `DICR` gravavel mas nada o levanta; DMA fora da RAM ignorada (0116)
- [ ] 10.57 Regiao do GetID fixada em SCEA; ler o setor de licenca do `.bin` (0122)
- [ ] 10.56 Result FIFO anterior legivel na janela da primeira resposta (0121)
- [ ] 10.55 Atraso da 1a resposta ignora o motor: `Nop (when stopped) 0x5CF4` (0121)
- [ ] 10.47 Espera da BIOS por timeout: 0x8000 giros < ~230 k (0114)
- [ ] 10.58 mutantes.ps1 so roda psx-core; alvo por crate (invariante 29) (0125)
- [ ] 10.66 Meta-teste nao reexecuta bateria antiga: `.resultado` mente (0143)
- [ ] 10.63 `-ContinueBranch` diz "reprovado na revisao" apos travamento (0140)
- [ ] 10.65 Revisor isolado em /tmp nao le o repo nem confere citacao (0141)
- [ ] 10.64 `evcb_descritor_mapeia_para_spec_correto` custa 447 s dos 449 s sob nextest (0140)
- [ ] 10.68 `cdrom_evento_kernel`: 150 M passos sem saida antecipada em 2 testes (0144)
- [ ] 10.71 `mutantes.ps1`: duas ramificacoes `teste` no mesmo switch (0146)
- [ ] 10.77 Trabalhador inventa metrica em vez de drenar metrics-pending.csv (0150)
- [ ] 10.78 oc-iter.ps1: exit 0 sem commits vira ok (0151)
- [ ] 10.83 Rayman: ~89/660 IRQ0 sem hook (0158)
- [ ] 10.85 Rayman: laco `0x801B9574` espera `[0x801CF2CC]>=2` (0159)
- [ ] 10.90 296x `VSync: timeout` antes do executavel assumir; o jogo roda depois (0184: nao e bloqueio)
- [ ] 10.102 DMA sincrono: ticks medidos ao redor leem overhead do poll (0173)
- [ ] 10.103 GetlocL/GetlocP stub; disc-swap exige tray scriptavel (06-cdrom.md L1052) (0175)
- [ ] 10.107 `CLAUDE.md` lista `perf`/`ci` mas o commit-lint so aceita 6 tipos (0177)
- [ ] 10.108 Oraculo roda cdrom sem `--disc`; com disco: GetStat sem bit1 (0175)
- [ ] 10.113 step-by-step-log: divergencia de endereco do proprio EXE e status (yuv2rgb feito na 0184)
- [ ] 10.114 DMA sem custo por ciclo: SPU testDMA*Timing exigem poll (0174)
- [ ] 10.115 Provas do Rayman fixam passo absoluto: melhoria legitima reprova (0174)
- [ ] 10.116 gpu/bandwidth sem timing de desenho; spec omissa (03-gpu.md L1107)

## Iteração 0181 em diante (`NNNN.k`)

- [ ] 0181.1 `docs/relatorio.md` e `docs/orquestracao.md` ainda descrevem o ROADMAP unico (0181)
- [ ] 0184.1 MDEC: 35 de 512 palavras do gabarito divergem em 1 passo de 5 bits (0184)
- [ ] 0188.1 XA reamostrado por vizinho mais proximo; o hardware usa zigzag de 25 pontos (0188)
- [ ] 0189.1 Anel de audio nao tem controle de fluxo: se o emulador roda fora do tempo real o anel enche ou esvazia (0189)
- [ ] 0190.1 `gte_fuzz_hardware` cobre 22 dos 27 comandos; NCLIP/DPCS/... sem caso de fuzz ficam so nos testes unitarios (0190)
- [ ] 0193.1 Fila interna do SPU (`output`, teto 8192) descarta o quadro mais novo sem contador (0193)
- [ ] 0193.2 GPUSTAT bit 21 (display 24bpp) nunca lido: FMV/MDEC exibida listrada (0193)
- [ ] 0193.3 Toggle de SPUCNT bits 14/15 zera a saida sem rampa: pop audivel (0193)
- [ ] 0193.4 CPU cobra 1 ciclo/instrucao sem custo de RAM/ROM e GPU desenha em 0 ciclos: jogo de 30 fps roda a 60 (0193)
- [ ] 0193.5 Mixer do SPU: hard clip duplo sem headroom nem saturacao por voz (0193)
- [ ] 0193.6 Saida de audio: underrun em degrau a 0.0 e resampler vizinho-mais-proximo duplica quadros a 48 kHz (0193)
- [ ] 0193.7 GPU: texture flip E1 bits 12/13 e GP1(10h) GPU-info ausentes (0193)
- [ ] 0198.1 R3 "sem I/O" nao tem teste que varra src/; `bus.rs` tem 3 `eprintln!` de sonda dentro do core (0198)
- [ ] 0198.2 Justificativa da allowlist (`purity.rs`: "save states") mais estreita que o uso real de serde em `app/` (0198)
- [ ] 0198.3 `app/saves.rs` a 5,5% de comentario, acima do alvo de 5% da R7 (0198)
- [ ] 0198.4 Teste com BIOS+disco reais do snapshot nunca roda na CI; so a maquina sintetica (de NOPs) cobre o save state la (0198)
- [ ] 0198.5 `saves::lista` nao confere o magico `MC` da imagem de cartao; imagem lixo lista saves fantasmas (0198)
- [ ] 0198.6 Maquina sintetica dos testes de snapshot executa so NOPs: scheduler/IRQ/DMA nunca exercitados no roundtrip (0198)
- [ ] 0201.1 Silent Hill (SLUS-00707) trava ~150-200M passos apos a tela de abertura: VRAM para, so sobra vblank; ultimo evento de CDROM e um INT2 sem sequencia (0201)
- [ ] 0202.1 Crash Bandicoot: triangulos aparecem e somem rapido durante o jogo (relato do usuario, nao reproduzido por medicao ainda) — suspeita: ordem de desenho/pintor sem Z-buffer, ou lixo de VRAM entre frames; precisa de dump-vram-every com passo fino pra flagrar o frame do artefato (0202)
- [ ] 0203.1 render_triangle_dithered tem o mesmo bug de 10.14 (reinterpola gouraud sobre o span ja recortado pela drawing area), caminho dither+gouraud+nao-texturizado; sem teste dedicado ainda (0203)
- [ ] 0203.2 PR #214 (10.30, retrigger de DMA no DPCR): so o canal OTC e exercitado; mutantes m3/m4/m5 do manifesto "matam" por efeito colateral (return precoce tambem pula o OTC, que vem depois no codigo), nao porque testam os canais 0/1/2 de fato — CDROM (dma3) e SPU (dma4) nunca sao exercitados nem pela bateria nem pelo teste (revisao do orquestrador no PR #214)
- [ ] 0203.3 "System Clock" diverge ~13-70x do gabarito do oraculo `timers` (ex-10.117); a iteracao 0176 ja tentou achar a causa raiz (inclusive corrigindo a propagacao de timing da GPU pros timers) e nao moveu esse numero — hblank agora e agendado de verdade (0203, ex-10.117 parcial) mas isso tambem nao deve mudar o "System Clock" (nao depende de GPU/hblank per a nota da 0176); causa raiz ainda desconhecida (0203)
- [ ] 0208.2 FF7 trava em 0x80059DFC-0x80059E10 esperando RAM 0x80089D9C mudar; so a BIOS escreve la (pc=0xBFC02B7C/0xBFC0D864), nao o jogo nem um handler de IRQ identificavel — quem deveria escrever alem da BIOS nao foi isolado (0208)
- [ ] 0208.3 Tomb Raider: ~900 iteracoes de laco de frame procurando \FMV\CORELOGO.FMV;1 (existe no .bin, confirmado por grep) antes de desistir e imprimir "not found" — falso negativo do driver ISO9660 do CD-ROM, ou contador de retentativas do jogo estourando por descasamento de ciclos (liga com 0193.4/10.102/10.114/10.116/0203.3) (0208)
- [ ] 0208.4 CTR trava esperando RAM 0x8007DD9C (contador de vsync do proprio jogo) incrementar; IRQ0/VBlank dispara e e reconhecido corretamente (I_STAT/I_MASK confirmados via watch-mem), mas o elo IRQ->contador do jogo nunca executa — handler encadeado ou EvCB especifico do CTR nao identificado (0208)
- [ ] 0208.5 8 dos 14 jogos comerciais testados (FF8, FF9, GT2, MGS, RE3, Silent Hill, Tomb Raider II, Tomb Raider III) travam na tela SCEA/PlayStation apos o boot, mas nao foram investigados individualmente — so ff7/tekken3/re2/tomb-raider/ctr tiveram o PC do travamento caçado nesta rodada (0208)
- [ ] 0214.2 FF7 (atualiza 0208.2): apos os degraus 1-6 de timing CPU/barramento, o CPU nao trava mais em 0x80059DFC-0x80059E10 — passa por ali e segue executando (amostrado ate 219 PCs distintos aos 300M passos, incluindo tabela do kernel 0x0000xxxx e codigo do jogo 0x8003C000-0x80041xxx). RAM 0x80089D9C recebe uma escrita NOVA no passo 154.872.307 (pc=0xBFC0D864, mesma familia BIOS de antes, mas agora dispara onde antes aparentemente nunca disparava). Nenhum marco novo de TTY apareceu em 300M passos — progresso real, mas destino ainda nao identificado (0214)
- [ ] 0214.3 Tekken 3 (atualiza 0208.1): a trava antiga de dupla-leitura do Timer 1 esta resolvida (fix 0208 + degraus 1-6) — o jogo anima ate a tela cheia da PlayStation (dumps de VRAM distintos aos passos 50M/100M/150M) e entao CONGELA ali por 250M passos (62% do orcamento de 400M), com IRQ2 de CD-ROM disparando continuamente a cada ~20-21 mil ciclos e o PC circulando por ~20 enderecos em faixas 0x80083Bxx/0x800859xx/0x8008F5xx/0x80091xxx sem produzir frame novo — travamento novo, distinto do antigo, com cara de espera de streaming/CD-ROM (0214)
- [ ] 0214.4 RE2 (atualiza 0208.1): mesma trava antiga resolvida — o jogo inicializa a lib do pad (`PS-X Control PAD Driver Ver 3.0`), anima ate ~150-200M passos e entao trava num laco em 0x80081A54-0x80081AB0 (dentro do executavel do jogo, nao kernel): um contador de retentativas decrescente guarda um segundo bloco que compara RAM 0x800A5110 (global, cresce de verdade: 0x8C aos 200M passos -> 0x3B2 aos 400M) contra um alvo, sem alcanca-lo dentro do orcamento testado. Padrao "espera contador atingir alvo com timeout de retentativa" que 0193.4 e a suspeita original do usuario ja previam — primeira vez caracterizado com PC e endereco exatos (0214)
- [ ] 0214.5 Tomb Raider (atualiza 0208.3): a retentativa de `\FMV\CORELOGO.FMV;1` ainda ocorre, mas o limiar de passos ate desistir saltou de ~900 iteracoes (modelo de ciclos antigo bugado) para algo entre 2,4 e 4 bilhoes de passos brutos apos os degraus 1-6 — mais de uma ordem de grandeza, evidencia forte a favor da hipotese (b) do 0208.3 (descasamento de ciclos, nao bug do driver ISO9660). Depois do CORELOGO falhar, o jogo passa a procurar um segundo FMV (`\FMV\CAFE.FMV;1`), que tambem falha entre 4 e 6 bilhoes de passos — marco nunca visto na investigacao original (teto testado entao era 1,6B). Tela permanece preta o tempo todo (VRAM zerada), sem indicio de renderizacao (0214)
- [ ] 0214.6 CTR (confirma 0208.4 inalterado): os degraus 1-6 nao mudam nada. A unica escrita nova em RAM 0x8007DD9C (passo 160.575.277, valor 0xA4A000EE, pc de BIOS 0xBFC06618) nao parece incremento genuino de contador (valor nao sequencial, parece bytes de uma copia em bloco) e fica congelada pelo resto dos 400M passos; VRAM tambem congela a partir de ~150M passos. IRQ0/VBlank seguem com cadencia saudavel (583 bordas de subida, ~669K passos de intervalo medio, confirmado no orcamento inteiro) — a falha nao e de entrega de hardware, e' na cadeia software IRQ->contador do proprio jogo, ainda nao isolada; nenhum degrau da escada de timing CPU/barramento (1-9) deveria afetar isso (0214)
