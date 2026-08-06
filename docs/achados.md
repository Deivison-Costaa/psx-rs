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
- [ ] 10.4 CAUSE.CE nao preenchido no Coprocessor Unusable (02-cpu.md L681)
- [ ] 10.5 Amidog psxtest_cpu para apos "args: 0" — causa nao investigada (iter 0032)
- [ ] 10.6 GP0(80h) VRAM->VRAM blit (hoje consumido e ignorado)
- [ ] 10.7 Mask GP0(E6h) em CPU->VRAM e VRAM->VRAM (`docs/reference/03-gpu.md` L590)
- [ ] 10.8 SWL/SWR fazem read-modify-write e leem portas de I/O de leitura destrutiva
- [ ] 10.10 Drawing Area GP0(E3h/E4h) e Offset GP0(E5h) sem suite que os meça
- [ ] 10.28 Tabela por registro nos docs vs `.resultado`: 0071 errou 3/9, 0038 inflou 2
- [ ] 10.29 `dma_dpcr_gate.rs:141`: `assert_ne!` como unica assercao de uma correcao
- [ ] 10.25 `unwrap_or` de caminho nos meta-testes silencia `strip_prefix` que falha (10.12)
- [ ] 10.14 U/V e gouraud reinterpolados sobre span recortado (03-gpu.md L452)
- [ ] 10.17 `mutantes.ps1` recusa arvore suja; permitir `docs/mutantes/*.mut`
- [ ] 10.24 Job `scoreboard` da CI sai VERDE medindo zero (0072)
- [ ] 10.26 Nenhum dos 9 testes de `ci_scoreboard.rs` afirma que o job mede algo
- [ ] 10.27 Placar local (gitignored) e o unico com veredito real e nao e versionado
- [ ] 10.18 Nada torna `arquivada:` caro: 0052 e 0059 largaram 17 registros, 12 casavam
- [ ] 10.33 `mutantes.ps1` so roda `cargo test -p psx-core` (L290); outros crates a mao
- [ ] 10.34 Nenhum meta-teste reprova `#[test]` sem assercao (T10 da 0080 passou assim)
- [ ] 10.35 `mutantes.ps1` grava nome qualificado, `mutation_battery` busca fn literal
- [ ] 10.40 `mutantes.ps1` so casa ancora em LF; CRLF da 'encontrada 0 vez(es)'
- [ ] 10.42 Linhas tremulas: captura sem sync com vblank; suspeito GP1(05h) (30/07)
- [ ] 10.42b Manifesto trata `#` como comentario em `@@DE`/`@@PARA` (alvo `.md`)
- [ ] 10.44 Manifesto com alvo em documento vivo (STATUS.md) envelhece sempre
- [ ] 10.45 Load shadow sobrepoe as instrucoes seguintes (`docs/reference/02-cpu.md` L281)
- [ ] 10.46 `justificativa:` do equivalente nao aceita continuacao de linha
- [ ] 10.52 `lhu`/`lbu` no modo do timer nao limpa bits 11/12 (0118)
- [ ] 10.50 `GP0(C0h)` sem transferencia devolve zero (0117)
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
- [ ] 10.109 SyncMode=0 com chopping: MADR e BC parados (04-dma.md L48) (0173)
- [ ] 10.113 step-by-step-log: divergencia de endereco do proprio EXE e status (yuv2rgb feito na 0184)
- [ ] 10.114 DMA sem custo por ciclo: SPU testDMA*Timing exigem poll (0174)
- [ ] 10.115 Provas do Rayman fixam passo absoluto: melhoria legitima reprova (0174)
- [ ] 10.116 gpu/bandwidth sem timing de desenho; spec omissa (03-gpu.md L1107)
- [ ] 10.117 timers: hblank nunca agendado; System Clock diverge ~13-70x (0176)

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
