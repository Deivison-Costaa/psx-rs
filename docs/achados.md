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
- [ ] 10.5 Amidog psxtest_cpu para apos "args: 0" — causa nao investigada (iter 0032)
- [ ] 10.8 SWL/SWR fazem read-modify-write e leem portas de I/O de leitura destrutiva
- [ ] 10.10 Drawing Area GP0(E3h/E4h) e Offset GP0(E5h) sem suite que os meça
- [ ] 10.45 Load shadow sobrepoe as instrucoes seguintes (`docs/reference/02-cpu.md` L281)
- [ ] 10.49 Bit 15 do `DICR` gravavel mas nada o levanta; DMA fora da RAM ignorada (0116)
- [ ] 10.57 Regiao do GetID fixada em SCEA; ler o setor de licenca do `.bin` (0122)
- [ ] 10.56 Result FIFO anterior legivel na janela da primeira resposta (0121)
- [ ] 10.55 Atraso da 1a resposta ignora o motor: `Nop (when stopped) 0x5CF4` (0121)
- [ ] 10.47 Espera da BIOS por timeout: 0x8000 giros < ~230 k (0114)
- [ ] 10.90 296x `VSync: timeout` antes do executavel assumir; o jogo roda depois (0184: nao e bloqueio)
- [ ] 10.102 DMA sincrono: ticks medidos ao redor leem overhead do poll (0173)
- [ ] 10.108 Oraculo roda cdrom sem `--disc`; com disco: GetStat sem bit1 (0175)
- [ ] 10.113 step-by-step-log: divergencia de endereco do proprio EXE e status (yuv2rgb feito na 0184)
- [ ] 10.114 DMA sem custo por ciclo: SPU testDMA*Timing exigem poll (0174)
- [ ] 10.116 gpu/bandwidth sem timing de desenho; spec omissa (03-gpu.md L1107)

## Iteração 0181 em diante (`NNNN.k`)

- [ ] 0189.1 Anel de audio nao tem controle de fluxo: se o emulador roda fora do tempo real o anel enche ou esvazia (0189)
- [ ] 0193.1 Fila interna do SPU (`output`, teto 8192) descarta o quadro mais novo sem contador (0193)
- [ ] 0193.3 Toggle de SPUCNT bits 14/15 zera a saida sem rampa: pop audivel (0193)
- [ ] 0193.5 Mixer do SPU: hard clip duplo sem headroom nem saturacao por voz (0193)
- [ ] 0193.6 Saida de audio: underrun em degrau a 0.0 e resampler vizinho-mais-proximo duplica quadros a 48 kHz (0193)
- [ ] 0198.5 `saves::lista` nao confere o magico `MC` da imagem de cartao; imagem lixo lista saves fantasmas (0198)
- [ ] 0201.1 Silent Hill (SLUS-00707) trava ~150-200M passos apos a tela de abertura: VRAM para, so sobra vblank; ultimo evento de CDROM e um INT2 sem sequencia (0201)
- [ ] 0203.1 render_triangle_dithered tem o mesmo bug de 10.14 (reinterpola gouraud sobre o span ja recortado pela drawing area), caminho dither+gouraud+nao-texturizado; sem teste dedicado ainda (0203)
- [ ] 0203.3 "System Clock" diverge ~13-70x do gabarito do oraculo `timers` (ex-10.117); a iteracao 0176 ja tentou achar a causa raiz (inclusive corrigindo a propagacao de timing da GPU pros timers) e nao moveu esse numero — hblank agora e agendado de verdade (0203, ex-10.117 parcial) mas isso tambem nao deve mudar o "System Clock" (nao depende de GPU/hblank per a nota da 0176); causa raiz ainda desconhecida (0203)
- [ ] 0208.2 FF7 trava em 0x80059DFC-0x80059E10 esperando RAM 0x80089D9C mudar; so a BIOS escreve la (pc=0xBFC02B7C/0xBFC0D864), nao o jogo nem um handler de IRQ identificavel — quem deveria escrever alem da BIOS nao foi isolado (0208)
- [ ] 0208.3 Tomb Raider: ~900 iteracoes de laco de frame procurando \FMV\CORELOGO.FMV;1 (existe no .bin, confirmado por grep) antes de desistir e imprimir "not found" — falso negativo do driver ISO9660 do CD-ROM, ou contador de retentativas do jogo estourando por descasamento de ciclos (liga com 0193.4/10.102/10.114/10.116/0203.3) (0208)
- [ ] 0208.5 FF8 e Silent Hill travavam na tela SCEA/PlayStation apos o boot; sem ROM nesta maquina, nao retestados (os outros 6 da lista original rodam desde o PR #256) (0208)
- [ ] 0214.2 FF7 (atualiza 0208.2): apos os degraus 1-6 de timing CPU/barramento, o CPU nao trava mais em 0x80059DFC-0x80059E10 — passa por ali e segue executando (amostrado ate 219 PCs distintos aos 300M passos, incluindo tabela do kernel 0x0000xxxx e codigo do jogo 0x8003C000-0x80041xxx). RAM 0x80089D9C recebe uma escrita NOVA no passo 154.872.307 (pc=0xBFC0D864, mesma familia BIOS de antes, mas agora dispara onde antes aparentemente nunca disparava). Nenhum marco novo de TTY apareceu em 300M passos — progresso real, mas destino ainda nao identificado (0214)
- [ ] 0214.5 Tomb Raider (atualiza 0208.3): a retentativa de `\FMV\CORELOGO.FMV;1` ainda ocorre, mas o limiar de passos ate desistir saltou de ~900 iteracoes (modelo de ciclos antigo bugado) para algo entre 2,4 e 4 bilhoes de passos brutos apos os degraus 1-6 — mais de uma ordem de grandeza, evidencia forte a favor da hipotese (b) do 0208.3 (descasamento de ciclos, nao bug do driver ISO9660). Depois do CORELOGO falhar, o jogo passa a procurar um segundo FMV (`\FMV\CAFE.FMV;1`), que tambem falha entre 4 e 6 bilhoes de passos — marco nunca visto na investigacao original (teto testado entao era 1,6B). Tela permanece preta o tempo todo (VRAM zerada), sem indicio de renderizacao (0214)
- [ ] 0227.2 A taxa de DMA do SPU que a spec da (`04-dma.md` L223: `0420h clks per 100h words` = 4,125 clk/word) fica ABAIXO da janela que o gabarito de hardware exige: `spu/memory-transfer` transfere 1024 bytes (256 palavras) e aceita measuredCycles entre `writeSize*16*0.1` = 1638 e `writeSize*16*1.1` = 18022, isto e, 6,4 a 70,4 clk/word. A propria spec marca o valor como incerto ("SPU transfer is unknown (may have some extra delays)", "XXX is SPU really only 4 clks?") e o teste registra em comentario "The transfer speed does not match those mentioned in NoCash docs (4cycles/word)". Sem numero confiavel na spec o valor NAO foi inventado: resolver 0227.1 sozinho trocaria a reprovacao "too slow" pela "too fast" (0227)
- [ ] 0240.1 CD-ROM rapido demais contra o DuckStation: seek medio (~370 setores) 52 ms vs ~100 ms, releitura 15 ms vs 20-60 ms, Pause em 2x 34 ms vs 39 ms; cargas do GT2/TR2/TR3 terminam 0,5-1,2 s antes (0240). Tentativa na branch `onda8/cd-seek` (modelo por geometria, 1.098 de 1.105 seeks do DuckStation dentro de 10%) derruba a suite para 8/12: o Crash ja fica ~0,5 s atras do DuckStation entre 17,5 e 21 s na main e perde o Start do roteiro com o CD mais lento; achar essa deriva antes de reintegrar
- [ ] 0240.3 Tekken 3: palco vazio ~1 s depois do replay do round contra <=0,25 s no DuckStation; adversarios diferentes, nao confirmado (0240)
- [ ] 0240.4 CTR: previa da pista na SELECT TRACK preta num unico dump (89 s) que o DuckStation nao tem (0240)
- [ ] 0240.5 Sincronia de FMV: RE3 abre ~1,2 s depois do DuckStation e o video do Mr. Dark do Rayman ~0,75 s antes (0240)
- [ ] 0240.6 Branch `onda6/dma-custo` acerta dma/chopping (44,9% -> 2,5%) e spu/memory-transfer (9/9) mas leva gpu/texture-overflow de 0 a 93.184 px; fora da main (0240)
- [ ] 0241.1 Suite converte quadro do DuckStation em tempo com 59,94 Hz, mas o NTSC progressivo do PS1 roda a ~59,82 Hz (263 linhas): 0,2% de deriva, ~0,24 s aos 120 s, que entra em toda comparacao de sincronia (0241)
