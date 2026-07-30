# 0110 — clut-pisoteada

- **Data:** 2026-07-30
- **Item do roadmap:** 2.2e
- **Objetivo:** achar por que "SONY" sai vermelho quando deveria ser azul-escuro. **Resultado: o
  suspeito nomeado (10.13, modulação) foi refutado por medição, e a causa real foi medida ponta a
  ponta: o losango com Y dobrado (2.2d) rasteriza por cima das CLUTs do texto na linha 480 da
  VRAM.** De quebra, o fundo cinza também deixou de ser bug de cor: é o fade congelado pelo crash
  do 4.4h. Iteração de diagnóstico — nenhum código de produção mudou.

## Revisão do PR anterior

PR #126 (iter 0109b), do próprio orquestrador: quatro checks verdes, só documentos, mergeado no
início desta iteração.

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| psx-spx | § GPU Render Polygon Commands (L254) — bit 24 = raw/modulação | docs/reference/03-gpu.md |
| psx-spx | § Modulation (also known as Texture Blending) (L1604) — `(texel*cor)/128`; 0x808080 é identidade | docs/reference/03-gpu.md |
| psx-spx | § GP1(08h) - Display mode (L885) — vres 480 só com bit5=1; "Interlace must be enabled to see all lines in 480-lines mode… a pretty bad example is the intro screens shown by the BIOS"; "the Vertical Interlace flag DOES affect GP0 draw commands" | docs/reference/03-gpu.md |

## A cadeia causal medida (harness `vramshot.rs` + instrumentação descartável, 85,5 M passos)

1. **Modulação identidade.** Todos os **357** quads texturizados do boot chegam com cor de comando
   `0x808080` — pela spec, modulação identidade. Implementar o 10.13 não mudaria um pixel desta
   tela. O 10.13 continua sendo defeito real, mas **não é a causa do 2.2e**.
2. **As CLUTs do texto moram na linha 480 da VRAM** (`780C`→x=192, `7810`→x=256, `7814`→x=320),
   carregadas uma única vez por CPU→VRAM com os azuis corretos.
3. **O losango as pisoteia.** A passada de desenho com offset (0,241) e área (0,241)-(639,480)
   recebe o losango com vértices absolutos y=353..609; o recorte para o triângulo até a linha 480
   **inclusive** — exatamente onde estão as CLUTs. 16 740 escritas rasterizadas na região
   (192..336, 480), com o gradiente vermelho/laranja do losango (`0x0016`, `0x0115`, `0x01F6`…).
4. **O texto é desenhado depois, amostrando a CLUT pisoteada** — e sai com o gradiente do losango.
   "SONY" vermelho não é cor errada: é textura certa lendo paleta destruída.
5. **O fundo cinza é o fade congelado.** Os quads de fundo sobem `000000 → 030303 → … → B4B4B4` e
   param em `B4B4B4` = (180,180,180) — o cinza medido na 0109 — porque o boot morre no 4.4h no
   passo 85 544 264. No hardware real o fade seguiria até branco.
6. **O display está DESLIGADO durante tudo isso** (GPUSTAT.23=0 até o crash). A tela que vínhamos
   julgando contra a referência nunca seria mostrada pelo hardware real: é estado intermediário
   de uma animação que a BIOS roda de tela apagada e só exibiria depois — e o "depois" nunca chega
   por causa do 4.4h.

Sobre o 2.2d, a medição estreitou mais: a lista da BIOS é uma **cena de 480 linhas** (SONY y≈69,
losango 112..368 centro 240, "COMPUTER ENTERTAINMENT" y≈397..427 — proporções exatas da
referência), desenhada duas vezes por frame com offsets (0,1) e (0,241) e display start alternando
1/241. Do jeito que rasterizamos, **as duas metades recebem a mesma metade superior da cena** — e
é isso que o despejo mostra, duplicado. GP1(08h) recebe `param=03` (640×240, sem entrelaçamento)
248 vezes. A spec cita a tela de boot da BIOS como o exemplo de 480 linhas entrelaçadas, e diz que
o flag de interlace afeta o desenho; como a BIOS ainda não ligou o display quando morre, o modo
final é desconhecido. O caminho do 2.2d passa por destravar o 4.4h.

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que o comportamento real é | Como foi pego |
|---|---|---|---|---|
| 1 | processo | Que o suspeito nomeado no handoff (10.13, modulação) fosse a causa e dava para ir direto ao teste | A cor de comando é 0x808080 em todos os 357 quads — modulação identidade. Teria implementado o item, visto a tela igual, e perdido a iteração | Instrumentação antes do teste: um `eprintln` em `render_polygon` derrubou a hipótese em uma rodada |
| 2 | endereçamento | Que a CLUT vermelha na VRAM significasse upload errado (troca R↔B em algum canal) | O upload está correto e os azuis chegam; quem escreve vermelho é o rasterizador do losango, frames depois, dentro da própria área de desenho programada pela BIOS | Log de quem escreve na linha 480: zero suspeitos em upload, 16 740 escritas vindas de `render_triangle` com as cores do gradiente do losango |
| 3 | timing | Que a tela aos 50 M passos fosse "a tela do logo" (a 0109 dizia que 30–85 M eram idênticos) | Aos 50 M o texto ainda nem foi rasterizado (zero quads texturizados); tudo acontece entre 50 M e 85,5 M. E o display está desligado o tempo todo | Contador de quads texturizados por faixa de passos: 0 em 50 M, 357 em 85,5 M |
| 4 | processo | Que "fundo cinza vs branco" fosse defeito de cor do GPU (era metade do enunciado do 2.2e) | O cinza (180,180,180) é literalmente o último quad do fade antes do crash: `B4B4B4`. O enunciado do item embutia uma comparação entre um frame intermediário e a tela final | O log do fade termina em `B4B4B4` — o mesmo valor da medição da 0109 |

## Bateria de mutação

Bateria de mutação: não se aplica — esta iteração não altera código de produção. Ela entrega a
refutação do suspeito do 2.2e, a cadeia causal medida do texto vermelho, a redefinição dos itens
2.2d/e/f como bloqueados pelo 4.4h, e a invariante 22.

## Placar antes → depois

Workspace: **735** → **735** testes. Nenhum código mudou.

## Revisão cruzada (orquestrador)

Iteração inteira do orquestrador.

## Decisões e notas

1. **Não implementar o 10.13 nesta iteração**, mesmo sendo defeito real e fácil: nesta tela ele é
   invisível (0x808080), então o teste de aceitação viraria teatro. Quando um jogo modular de
   verdade, o item tem tela para ser julgado.
2. **A prioridade vira o 4.4h.** Fundo, texto e provavelmente o modo de vídeo final são
   incognoscíveis antes de o boot sobreviver ao crash — o display nunca liga. Julgar cor de novo
   só depois.
3. **Achado colateral, já coberto pelo item 10.11:** retângulo texturizado é parseado e descartado
   sem desenhar nada (gpu.rs, `RectStage::AwaitDims` com `textured`). Não está no caminho do logo
   (o texto usa polígonos 2Ch), mas reforça o item.
4. **A imagem de referência agora existe em lugar durável:** o arquivo do usuário está em
   `Programacao com agentes/ps1-sony-computer-entertainment-boot-screen-16k-v0-5e3diayxmyp71.webp`
   e uma cópia em `psx-estado/referencias/tela-de-boot.webp`. `docs/referencias/tela-de-boot.md`
   atualizado para apontar para elas.
5. **O harness `vramshot.rs` ganhou despejo das três CLUTs** e continua descartável, fora do repo,
   em `psx-estado/instrumentacao/vramshot.rs`.
