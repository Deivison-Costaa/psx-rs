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

## Funcionando (10 de 15 títulos)

Tekken 3, Final Fantasy VII, Final Fantasy VIII, **Resident Evil 2**, Resident Evil 3,
Metal Gear Solid, Crash Team Racing, Crash Bandicoot, Gran Turismo 2 (Arcade **e**
Simulation).

CTR, GT2 e Resident Evil 2 foram destravados nesta sessão. O RE2 chega na tela de título
completa (logo, menu LOAD GAME / NEW GAME / OPTION) — antes ficava 97% do tempo num
busy-wait de 2 instruções em `0x80031D20/24`. O GT2 chega no menu de título no Arcade e numa
corrida em andamento no Simulation.

**"Funcionando" aqui significa: rodou sem congelar pela janela medida, com a VRAM mudando
continuamente e confirmação visual onde deu.** Nenhum foi jogado até o fim. Pode haver bug
de gameplay, áudio, ou travamento mais adiante que a medição não alcança. Os discos
secundários (FF7 2/3, FF8 2/3/4, MGS 2) só passaram por boot sanity check de 300M passos —
ninguém chegou a testar troca de disco. O feedback mais útil é jogar de verdade e mais fundo
do que a medição automática vai.

## Ainda travando (5)

| Jogo | Trava em | Tela no congelamento |
|---|---|---|
| Tomb Raider I | ~200M passos | **preta** — trava antes de desenhar |
| Tomb Raider II | ~275M | congelada |
| Tomb Raider III | ~275M | **preta** |
| Silent Hill | ~500M | título "SILENT HILL" desenhado e parado |
| Final Fantasy IX | ~400M | **branca**, logo já carregado na VRAM (travou num fade) |

Os sintomas são diferentes entre si — **não presuma causa única**. TR1/TR3 travam sem
desenhar nada; RE2/SH travam com a UI inteira pronta; FF9 no meio de uma transição.

**Todos os 6 rodam perfeitamente no DuckStation**, com o mesmo BIOS e as mesmas imagens de
disco (verificado por captura de tela: os 6 passam do ponto de travamento e chegam a jogar).
Ou seja: são bugs nossos, não dos jogos.

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
