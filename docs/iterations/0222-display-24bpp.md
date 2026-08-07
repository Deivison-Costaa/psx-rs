<!-- Custo, tokens e duração NÃO entram aqui: são medidos por scripts/oc-iter.ps1 em
     docs/metricas.csv, casados por head_antes/head_depois. -->

# 0222 — display-24bpp

- **Data:** 2026-08-07
- **Item do roadmap:** 0222.1
- **Objetivo:** `Gpu::framebuffer()` passa a respeitar GPUSTAT.21 e ler a área de display
  com 3 bytes por pixel quando o jogo pede 24 bits — que é o modo de toda FMV.

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| psx-spx | § GP1(08h) - Display mode | docs/reference/03-gpu.md |
| psx-spx | § GPU Video Memory (VRAM) / Framebuffer | docs/reference/03-gpu.md |

`03-gpu.md` L890 e L1019: o bit 4 de GP1(08h) é a "Display Area Color Depth" e vira
GPUSTAT.21 (0=15bit, 1=24bit). L1280-1282 dão os canais de 8 bits e L1284-1285 dizem que
"the 24bit pixels occupy 3 bytes (not 4 bytes with unused MSBs), so each 6 bytes contain two
24bit pixels" e que o modo "is used mostly for MDEC videos".

## O defeito, medido

O handoff descrevia FMVs "granuladas": forma reconhecível sobre fundo ruidoso, sintoma
clássico de IDCT ou zigzag errados no MDEC. A medição desmentiu o diagnóstico.

1. Oráculo `mdec/frame` 15bpp (`tests/exes/ps1-tests/mdec/frame/frame-15bit.exe` contra
   `vram-15bit.png`): a imagem sai correta — 61.373 dos 76.800 pixels idênticos ao hardware,
   e das divergências 11.653 são de exatamente um degrau de 5 bits. Nada de "granulado".
2. Oráculo `mdec/movie` 15bpp: silhueta limpa, sem ruído.
3. Oráculo `mdec/frame` 24bpp comparado **byte a byte** (e não como PNG de 16 bits, que
   embaralha a comparação): os bytes saem alinhados com o hardware e divergem por 1-3 no
   valor do canal. Ou seja, o MDEC entrega o macrobloco certo também em 24 bits.
4. Instrumentando `write_vram_dump` com o GPUSTAT, os dumps 10, 11 e 12 do Silent Hill saem
   com `GPUSTAT=54222640`, **bit 21 = 1**, `display_start=(0,16)` e `(0,256)`.
5. Renderizando a mesma VRAM crua como 24bpp (3 bytes por pixel) aparece o retrato da menina
   do Silent Hill, nítido e sem grão nenhum.

Conclusão: os dados na VRAM sempre estiveram certos. Quem estava errado era o observador —
`Gpu::framebuffer()`, que é o que o app desktop (`emulador.rs`, via
`framebuffer_for_display`) mostra na tela, decodificava cada halfword como um pixel 5:5:5
independentemente de GPUSTAT.21. Em 24 bits isso corta 1/3 dos bytes e desalinha os outros
dois terços a cada pixel — exatamente o "chuvisco colorido" relatado.

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a medição diz | Como foi pego |
|---|---|---|---|---|
| 1 | diagnóstico herdado | o ruído vinha do MDEC (IDCT/zigzag/quant) | os três oráculos de MDEC batem com o hardware | rodar `mdec/frame` e `mdec/movie` antes de tocar em `mdec.rs` |
| 2 | instrumento | `--vram-to-png` mostra o que o console mostraria | ele fixa 15bpp; a VRAM crua não carrega a profundidade | comparar o PNG de 24 bits do oráculo com o gabarito byte a byte, não pixel a pixel |

## Bateria de mutação

Placar da bateria: 5/5 mutantes mortos, 2/2 controles verdes, 0 equivalente — docs/mutantes/0222-display-24bpp.mut

| Registro | Mutação | Teste que pegou |
|---|---|---|
| m1 | volta a ignorar GPUSTAT.21 | `t2_pixels_de_24_bits_ocupam_tres_bytes_cada` |
| m2 | lê o bit 20 (Video Mode) no lugar do 21 | `t3_24bpp_usa_os_oito_bits_de_cada_canal` |
| m3 | pixel de 24 bits ocupa 4 bytes | `t2_pixels_de_24_bits_ocupam_tres_bytes_cada` |
| m4 | byte alto e byte baixo do halfword trocados | `t4_display_start_x_continua_em_halfwords_no_modo_24bpp` |
| m5 | canais em BGR em vez de RGB | `t3_24bpp_usa_os_oito_bits_de_cada_canal` |

Os manifestos 0113 (m5) e 0203 (m2) foram reancorados — a indentação das duas linhas mudou
com o `if cor24` — e rerrodados: 5/5 mortos e 2/2 controles verdes em cada um.

## Placar antes → depois

Workspace: 1400 → 1405 testes, todos verdes. `clippy --all-targets --workspace -D warnings`
limpo; `rustfmt --check` limpo nos 252 arquivos de `crates/`.

Medição a 600M passos com linha de base compilada do mesmo HEAD **sem** a mudança de
`gpu.rs` (hashes conferidos: `CAB539D2…` contra `AD365930…`), lendo a tela pelo novo
`<dump>.tela.png`:

| Jogo | Antes | Depois |
|---|---|---|
| Silent Hill (dumps 10-12) | chuvisco verde/magenta ocupando a tela toda | retrato da menina, gola branca e gravata legíveis, moldura e papel de parede visíveis |
| Tomb Raider III (dump 11) | mancha branca no meio de um padrão de listras | campo estelar com sol brilhando e planetas |
| Tomb Raider I (dump 12) | ruído colorido | céu azul com nuvem e o anel da Eidos |

Regressão a 400M nos 13 jogos que funcionam (Tekken 3, FF7 D1, FF8 D1, RE2 D1, RE3, MGS D1,
CTR, Crash, GT2 Arcade, GT2 Simulation, TR1, TR3, Silent Hill), sempre contra a mesma linha
de base: **os 52 dumps de VRAM saem idênticos ao byte** e a contagem de PCs distintos nos
últimos 10% é igual nos 13 (173, 95, 141, 100, 44, 41, 86, 87, 251, 73, 138, 111, 49).
Nenhum passou a travar — a mudança é só de leitura e não realimenta a emulação. Sete dos 52
`*.tela.png` mudaram, todos em modo 24bpp: Tekken 3 (FMV de tochas), RE2 (logo da Capcom),
GT2 (logo da Sony), TR1 e TR3.

## Revisão cruzada (orquestrador)

<!-- Preenchido pelo Claude na revisão do PR: achados no formato de docs/prompts/review.md, ou "sem achados". -->

## Decisões e notas

- **`--vram-to-png` continua em 15bpp de propósito.** Os gabaritos do ps1-tests são retratos
  crus da VRAM em 16 bits; mudar esse conversor quebraria a comparação com o oráculo. O
  instrumento novo é outro: `--dump-vram-every` passa a gravar `<dump>.tela.png` a partir de
  `Gpu::framebuffer()`, que sabe a profundidade porque tem o GPUSTAT em mãos.
- **Achado 0222.2 fica aberto:** no oráculo `mdec/frame` 15bpp a vigésima e última chamada
  de `mdec_readDecoded(0x1e00)` nunca retorna, mesmo com 300M passos e com as 38.400 palavras
  (300 macroblocos) já na fifo de saída — 36.480 saem e a leitura da última tira de 16px
  trava antes do primeiro `pop`. Custa 2.112 pixels na coluna de macrobloco 19. O software
  espera algo no MDEC1 que não entregamos; candidatos ainda **não medidos**: o bit 30
  ("Data-In Fifo Full", que nunca setamos) e o contador de palavras restantes, que devolvemos
  como 0xFFFF fora de comando.
- **Achado 0222.3 fica aberto:** fora dessa tira, 11.653 pixels saem um degrau de 5 bits
  fora do hardware. É arredondamento de `real_idct_core`/`yuv_to_rgb`, que a própria spec
  (09-mdec.md, "the results aren't perfect") declara desconhecido. Invisível a olho nu.
- `cargo fmt --all` **falha neste worktree** com `os error 206` (caminho longo do Windows);
  a verificação foi feita chamando `rustfmt --edition 2024 --check` arquivo por arquivo.
