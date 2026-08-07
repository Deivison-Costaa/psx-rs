# Estado dos jogos — o que funciona, o que trava, o que já foi descartado

Resultado da sessão de 2026-08-07 (branch `iter/fix-jogos-loop`, 21 commits). Este doc
existe para **outros membros testarem sem repetir caminho já andado**. Se você for
investigar um travamento, leia a seção "Hipóteses já refutadas" antes — várias delas
custaram horas e foram fechadas com medição.

## Como testar

```
cargo build --release -p psx-cli
./target/release/psx-cli.exe --bios bios/SCPH1001.BIN \
  --disc "../roms/extraido/<jogo>.cue" --max-steps 600000000 --pad \
  --dump-vram-every 25000000 pref
```
Depois `md5sum pref-*.vram`. Hash que para de mudar = travado.

**Duas armadilhas de medição que já fizeram relatório mentir:**

1. **Ordene os dumps NUMERICAMENTE, não como texto.** `pref-1, pref-10, pref-11, ..., pref-2`
   embaralha a linha do tempo e inverte a conclusão.
2. **Meça além do horizonte onde o fenômeno aparece.** O FF9 foi declarado "não travado"
   porque a medição parou em 400M — que é exatamente onde ele congela. Com 1.5B o hash é
   idêntico do dump 4 ao 15.

Hash mudando ainda pode ser ruído de textura na VRAM. Confirme com
`--vram-to-png entrada.vram saida.png` antes de comemorar.

## Classificação honesta

Estão em três níveis, porque "não trava" **não** é a mesma coisa que "joga".

### Jogável confirmado (2) — testado por humano

**Crash Bandicoot** e **Rayman**. Renderizam a gameplay e respondem a input.

### Navegação por menu confirmada (2) — exigiu input real

- **Tekken 3** — chega em `STAGE 1 — EDDY VS LAW`, ou seja, passou por menu, seleção de
  personagem e entrou numa luta.
- **Tomb Raider II** — chega no seletor de fase `Lara's Home` com `Select / Go Back`.

Não é o mesmo que jogar até o fim, mas exigiu navegar, então o caminho de input funciona.

### Renderizando corretamente, progressão não confirmada (8)

Validado por inspeção visual do framebuffer (2B passos, com presses sintéticos):

| Jogo | O que aparece na tela |
|---|---|
| Resident Evil 2 | prompt `Use memory card` (passou do título) |
| Resident Evil 3 | menu `NEW GAME / LOAD GAME / GAME CONFIG` |
| Final Fantasy VII | menu de título com a Buster Sword |
| Final Fantasy VIII | cutscene de abertura em 640x480, preto-e-branco |
| Tomb Raider I | tela de título com o rosto da Lara |
| Tomb Raider III | cena 3D dentro do veículo |
| Silent Hill | FMV de abertura (veículo na estrada) |
| Metal Gear Solid | cena escura de abertura, coerente |
| Crash Team Racing | personagem 3D no cenário |
| Gran Turismo 2 (Arcade) | FMV de corrida em 24bpp |

Alguns pararam em tela de título. **Isso não prova que não respondem a input** — os presses
sintéticos podem simplesmente não ter acertado a sequência de botões daquele jogo. Precisa
de teste humano para separar as duas coisas.

### Quebrado (1)

**Final Fantasy IX.** Morre logo depois da tela `Published by Square Electronic Arts L.L.C.`
Framebuffer **byte-idêntico** (PNG de 782 bytes, preto sólido) do passo 600M ao 2B — oito
amostras iguais ao longo de 1,4 bilhão de passos, então não é fade nem transição.

Mecanismo já rastreado: gira em `0x800A9A6C` esperando o bit1 do byte em `0x80076B14`
zerar. Esse byte **não é o stat do CD-ROM** (verificado) — é campo do próprio jogo,
sobrescrito por um `memcpy` em `pc=0x800226CC` cuja origem passou a conter `0x80015509` no
passo 408.411.249.

## A armadilha da métrica — leia antes de confiar em qualquer número aqui

A medição automática desta sessão (**hash de VRAM mudando + histograma de PC**) prova
**ausência de um travamento específico**, e nada além disso. Ela **não distingue jogável de
lixo**: medidos lado a lado, Crash Bandicoot (jogável) e Tekken 3 (renderiza ruído) produzem
o mesmo veredito — VRAM mudando até o fim e áudio do mesmo tamanho.

O que separa os dois só aparece **olhando a imagem** ou jogando. Foi um teste humano que
corrigiu o placar de "13 de 15 funcionam" para "2 jogáveis".

Pior: a régua de diagnóstico tinha o **mesmo** defeito do emulador. O `--vram-to-png`
renderizava sempre como 15bpp, então em jogo que estava em 24bpp era a própria ferramenta
que gerava o ruído — e o sintoma parecia corrupção de VRAM. Use `-fb.png` (gerado a cada
`--dump-vram`), que respeita área de display e formato de cor reais.

**Um quadro preto não prova travamento.** Amostre vários pontos e compare o hash entre eles:
o FF9 já foi declarado "não travado" por medição feita num único ponto que era justamente a
transição.

Isso é o caso clássico de métrica que mede o que é fácil medir em vez do que importa — o
mesmo padrão que o projeto já registrou no scoreboard da 1.11b. **Antes de declarar um jogo
bom, renderize o framebuffer e olhe.**

Os discos secundários (FF7 2/3, FF8 2/3/4, MGS 2) só passaram por boot sanity check de 300M
passos; ninguém testou troca de disco.

## Nota: o Tomb Raider II nunca esteve travado

Ele foi listado como travado por boa parte da sessão. Estava errado: chega no seletor de
fase `Lara's Home`. A classificação vinha da régua de 15bpp, que mostrava ruído onde havia
imagem. **Reclassificado depois do fix de 24bpp.**

Serve de aviso: reconfira classificação antiga sempre que a ferramenta de medição mudar.

O único quebrado hoje é o **Final Fantasy IX** (detalhe na classificação acima). Ele roda
perfeitamente no DuckStation com o mesmo BIOS e o mesmo disco, então é bug nosso.

## Corrigido nesta sessão

Todos com teste vermelho→verde, citação de spec conferida com `grep -n`, e regressão medida
nos jogos que já funcionavam.

**CD-ROM**
- Sector Size do Setmode (bit5): 800h=2048 DataOnly vs 924h=2340 WholeSector.
- Setfilter consumia os parâmetros da fila e vazava pro próximo comando; e não filtrava de
  verdade por file/channel.
- Cadência de INT1 durante streaming usava constante fixa em vez da fórmula real de
  velocidade (`SystemClock*930h/4/44100Hz`).
- Setor Audio+RealTime ia pra CPU como dado; deve ir **exclusivamente** ao decoder XA.
- GetTN, GetTD, GetlocL, GetlocP não existiam (caíam num handler genérico de 1 byte).
- Avanço de setor de áudio dependia de um ACK de INT1 que nunca chega.
- **O drive não girava sozinho**: o avanço do setor de dados esperava o ACK da CPU. No
  hardware o drive despeja 150 setores/s independente disso, e setor não consumido some
  silenciosamente (`06-cdrom.md` L2118-2126).
- **Buffer de setor virou ring de 8 slots** (era 1 slot). `06-cdrom.md` L2109-2117, com golden values
  medidos em hardware real (o caso `1+8=9` prova as 8 posições).
- **Comando que não aborta a leitura não pode cancelar a entrega de setor em voo.** A regra
  estava invertida: só uma lista curta de "passivos" preservava, e Setmode/Setloc caíam no
  resto — matando o streaming pra sempre. `06-cdrom.md` L471-473 mede literalmente
  `ReadN -> INT3 -> SetMode/SetLoc` e diz "will not drop any of the two commands".

**Bus / scheduler**
- `CDROM_SECOND` era reagendado a partir do fim da fatia do `tick_timers` em vez do
  vencimento do evento — o drive rodava **abaixo** dos 75/150 setores/s, com atraso
  acumulando. Estava mascarado porque o ACK reancorava o relógio a cada setor.

**CPU**
- `bus.tick_timers()` era a última linha de `step()` e havia 5 `return` antes dela: IRQ,
  fetch desalinhado, bus error e todo `pending_exception` (inclusive SYSCALL). Esses
  caminhos custavam **zero ciclo**. Corrigido — mas medido: o efeito real é de 2-3 ppm do
  relógio, então a hipótese antiga (achado 0193.4) superestimava muito o impacto.

**SPU**
- Escrita nos capture buffers não disparava IRQ9 ao cruzar o endereço configurado
  (resolveu o CTR).

**GPU**
- Semi-transparência não era aplicada a primitiva **sem textura** (bit25 sozinho).

**DMA / barramento**
- **Escrita de byte nos offsets 1-3 dos registradores de DMA era descartada, e a leitura
  devolvia zero fixo.** O jogo liga/desliga a máscara do canal no DICR por read-modify-write
  de 1 byte em `1F8010F6h` (bits 16-23 = máscara por canal + master enable), para que a
  interrupção de fim de DMA exista **só no setor que carrega o último chunk do quadro**.
  Sem isso o IRQ subia a cada setor, o quadro era marcado pronto com 1/9 dos dados, e o
  decoder de FMV consumia 18.144 bytes de onde só havia 2.016 — saindo pela RAM até zerar o
  vetor de exceção. **Destravou Tomb Raider I e III e Silent Hill**, e fez Tekken 3 e RE2
  avançarem (o Tekken agora sai da tela de título e entra no FMV de atração).
  Não era timing: era decodificação de endereço de I/O.

**Barramento**
- Acesso a região não mapeada caía na RAM mascarada por `0x1FFFFF` nos **seis** caminhos
  (`write32/16/8`, `read32/16/8`). Um ponteiro lixo que no hardware não faz nada corrompia
  RAM do kernel silenciosamente. Dois buracos extras junto: SIO1 (`0x1F801050-5F`) e fetch
  de instrução além do fim da BIOS, que lia zeros da RAM e executava como NOP.
  Achado colateral: o teste `desktop_boot.rs` **dependia** desse último bug — andava 1M
  passos quando a BIOS só tem 131.072 instruções.

## Hipóteses já refutadas — não reabrir sem dado novo

- **"O CD-ROM entrega dados errados/incompletos."** Refutado com sonda: a entrega ao jogo é
  idêntica à do DuckStation, setor a setor. Mesmo `Setmode 0xC0`, mesmos submodes (`0x42`
  vídeo, `0x64` áudio), mesmo roteamento do setor de áudio XA intercalado no meio do vídeo.
  A sequência de setores é limpa e sequencial, sem buraco nem repetição.
- **"O buffer de 1 slot é o gargalo."** Refutado: o código do ring só diverge do antigo
  quando existe setor bufferizado no momento do ACK, e isso **nunca ocorreu** em 400M passos
  em nenhum dos 14 jogos — a VRAM saiu byte-a-byte idêntica à baseline.
- **"A posição de leitura atrasada quebra o sincronismo de FMV."** Corrigido e medido:
  nenhum dos 6 mudou de comportamento.
- **"Exceções custando zero ciclo fazem o kernel rodar de graça."** O defeito era real mas
  o déficit é de 2-3 ppm (+822 ciclos em 100M passos). Só a *instrução de entrada* custava
  zero; o corpo do handler sempre foi contabilizado.
- **"A tela branca do FF9 é um retângulo de fade semi-transparente virando opaco."**
  Refutado por inspeção da VRAM: não existe retângulo branco opaco, nem antes nem depois da
  correção de blending.
- **"O DMA3 fica preso esperando o CD e nunca retoma."** Refutado por medição direta: em 600M
  passos, nos 5 jogos travados, `try_execute_dma3` **nunca** terminou com o canal ainda
  ocupado — zero ocorrências. Os jogos só armam o canal quando o setor já está no buffer.
  (O canal 1/MDECout fica pendente no TR2 e no Silent Hill, mas ele já tem gancho de
  retomada: fica pendente porque nada volta a alimentar o MDEC — sintoma, não causa.)
- **"Escrita em endereço não mapeado corrompe a RAM e mata o Tomb Raider."** O bug era real e
  foi corrigido (ver abaixo), mas **não é a causa**: medi que o TR não escreve em nenhum
  endereço fora da RAM. O `0x80200080` que ele escreve é espelho legítimo dos 2 MB.
- **"O custo de DMA do canal 3 está alto demais."** Rejeitado: a spec dá `24 clks/word` em
  duas formas consistentes (`04-dma.md` L217-222) e é o que implementamos. Aqui o DuckStation
  é que se afasta da spec — não trocamos citação clara por imitação.
- **"É bug do próprio jogo"** — foi a conclusão anterior para estes 6, e estava **errada**.
  Foi derrubada ao rodá-los no DuckStation. Lição: prova interna consistente (dados batendo
  byte-a-byte, spec citada, teste passando) não substitui comparação com uma referência
  externa.

## Mecanismo do Tomb Raider (o mais mapeado)

```
passo=79384      pc=0x00000ED8   BIOS instala o handler de exceção em 0x80000080
passo=228974889  pc=0x80060E30   escreve ZERO sobre o handler
depois           CPU presa oscilando em 0x80000080-90 pra sempre
```

`0x80060E30` é o loop do decoder de vídeo **do próprio jogo**: o ponteiro de saída dele
desce até a RAM baixa e apaga o vetor de exceção. Como o dado de entrada está provadamente
correto, o que resta é timing.

## RESOLVIDO: o "ruído" era o modo de display de 24 bits nunca implementado

Vários jogos bootavam bem e depois passavam a renderizar **ruído puro no framebuffer**. A
causa não estava na VRAM, no MDEC, no CD nem na rasterização: **`Gpu::framebuffer()`
decodificava todo pixel como 5:5:5, ignorando GPUSTAT.21**. Quando o jogo entra em FMV ele
liga o modo 24 bits por GP1(08h).4; a partir daí a mesma VRAM correta era lida com o
formato errado, e 3 bytes de RGB888 interpretados como halfwords 5:5:5 dão exatamente
estática colorida.

`03-gpu.md` L1279-1285: em 24bpp os bits 0-7 são R, 8-15 G, 16-23 B, e **cada 6 bytes
contêm dois pixels** — o pixel não está alinhado a halfword. L1019: GPUSTAT.21 é
`Display Area Color Depth (0=15bit, 1=24bit)`, escrito por GP1(08h).4.

**Como foi medido.** Sonda de `GPUSTAT` a cada dump de VRAM, com o mesmo dump renderizado
nas duas interpretações:

| Jogo | Entra em 24bpp | Como 15bpp (antes) | Como 24bpp (depois) |
|---|---|---|---|
| Tekken 3 | entre 300M e 350M, e fica | estática colorida | Heihachi nítido, FMV limpo |
| Silent Hill | ~525M, e fica | estática colorida | retrato da menina, limpo |
| Tomb Raider III | ~300M, e fica | estática colorida | logo CORE Design com starfield |
| Resident Evil 2 | ~375M-600M, depois volta | estática colorida | barril Umbrella legível |
| Resident Evil 3 | ~450M-600M, depois volta | estática colorida | — |
| **Crash Bandicoot** | **nunca** | limpo | limpo (byte-a-byte igual) |

O controle positivo fecha o argumento: **Crash é jogável justamente porque nunca liga o
bit21**. A correlação é perfeita nos 6 jogos — quem entra em 24bpp exibia ruído, quem não
entra exibia imagem.

Medição A/B com dois binários de hash diferente, 12 quadros por jogo pelo caminho real de
`framebuffer()`: os jogos afetados passam de estática a FMV legível, e **Crash sai com os
12 quadros byte-a-byte idênticos** (zero regressão).

**Duas armadilhas de medição que essa investigação expôs:**

1. `--vram-to-png` renderiza a VRAM crua **sempre como 15bpp**. Em jogo que está em 24bpp,
   a ferramenta de diagnóstico produzia o ruído por conta própria — o emulador e a régua
   erravam igual, o que fez o defeito parecer "corrupção de VRAM" por uma sessão inteira.
   Por isso `--dump-vram-every` agora grava também `<prefixo>-N-fb.png`, gerado pelo
   `framebuffer()` de verdade, com a profundidade correta e o GPUSTAT no log.
2. A "FMV granulada" de Silent Hill e Tomb Raider III nunca foi granulação: era o mesmo
   defeito, visto num quadro escuro.

**O MDEC já tinha sido descartado, e corretamente.** Os quatro oráculos de hardware passam
limpos: `tests/exes/ps1-tests/mdec/frame/{15,24}bit{,-dma}.exe` e `.../movie/movie-15bit.exe`
(4.255 linhas `ok`, zero falhas). O MDEC vinha entregando pixels certos o tempo todo — só
eram exibidos errado.

## Corrigido depois: tempo de seek (destravou o RE2)

O tempo de seek era uma **constante fixa**: `second_response_cycles_for` devolvia `0x4A00`
(18.944 ciclos ≈ 0,56 ms) para praticamente todo comando, incluindo SeekL/SeekP —
independente da distância e idêntico em toda repetição. Agora depende da distância em
quadros e varia por busca (LCG determinístico no estado, para não quebrar save state).

Dois problemas:
1. A própria spec mede um `Pause` em `0x21181C` (~2,1 milhões de ciclos). Fazemos um *seek
   inteiro* ~100× mais rápido que um pause. Seek real vai de ~10 ms a ~900 ms.
2. A spec documenta que toda temporização de CD-ROM tem faixa Min..Max (`06-cdrom.md` L2070-2076), não
   valor único.

O autor do DuckStation registrou em comentário que jogos com código de disco sensível a
timing entram em **loop infinito** quando o emulador devolve sempre o mesmo tempo de seek, e
cita nominalmente a série Resident Evil. Bate com o RE2, que trava num busy-wait de 2
instruções depois que um contador de retentativa esgota.

## Nota sobre o DuckStation como referência

Foi usado **só como oráculo de comportamento** — para comparar traces e entender o
hardware. A licença dele é CC BY-NC-ND (NonCommercial, **NoDerivatives**): copiar código, ou
traduzir C++ para Rust linha a linha, seria obra derivada e violaria a licença. Toda
correção nossa foi escrita do zero e justificada por citação de `docs/reference/`.

Para reproduzir a comparação: o binário fica em `tools/duckstation/`, com log configurado em
`%LOCALAPPDATA%\DuckStation\settings.ini` (`LogLevel = Dev`, canal `CDROM = true`,
`LogToFile = true`). O nível `Dev` registra cada setor entregue com LBA e submode. `Debug` e
`Trace` são removidos em tempo de compilação no build release e não aparecem.
