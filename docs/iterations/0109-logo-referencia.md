# 0109 — logo-referencia

- **Data:** 2026-07-30
- **Item do roadmap:** 2.2d
- **Objetivo:** com a referência da tela real em mãos, achar por que o losango do logo sai grande
  demais e deslocado. **Resultado: três hipóteses medidas e descartadas, sem correção.** A
  iteração entrega a referência, a correção de uma conclusão anterior errada, e o próximo passo.

## Revisão do PR anterior

PR #124 (iter 0108), do próprio orquestrador: quatro checks verdes, `headRefOid` conferido,
iteração de diagnóstico sem código.

## A referência, e o que ela derruba

O usuário forneceu a tela oficial "SONY COMPUTER ENTERTAINMENT". Comparando com o nosso despejo:

| | Real | Nosso |
|---|---|---|
| Fundo | branco | cinza (180,180,180) |
| "SONY" | azul-escuro, acima | **vermelho**, acima |
| Losango | **completo**, quatro pontas, "S" vazado | **metade de baixo**, "S" vazado correto |
| "COMPUTER ENTERTAINMENT" | abaixo, azul-escuro | **ausente** |

**Isto derruba a conclusão da iteração 0107**, que dizia que o losango cortado talvez não fosse
defeito nosso. É defeito. O que a 0107 mediu continua valendo — a BIOS emite triângulos de 256
linhas para uma área de desenho de 240 que ela mesma programou, e nós recortamos certo — então o
erro está **antes** do recorte. A invariante 20 foi reescrita.

Três defeitos separados, porque as causas são diferentes: **2.2d** (geometria), **2.2e** (cor),
**2.2f** (texto que falta).

## O que foi medido nesta iteração

Sobre o 2.2d, o desvio é preciso: o losango ocupa `y=112..368`, centro 240, altura 256. Num
framebuffer de 640×240 — que é o que a BIOS programa — o centro deveria ser 120 e a altura ~128.
**Y é exatamente o dobro; X está certo** (centro 320 num buffer de 640).

Três hipóteses, todas descartadas por medição:

1. **Projeção do GTE.** Um fator de escala com deslocamento junto é assinatura de projeção
   perspectiva. Instrumentei a divisão do `rtps`: **zero** chamadas em 85 M passos. O losango não
   passa pelo GTE.
2. **Resolução vertical mal reportada.** Se a BIOS perguntasse o modo e recebesse 480, calcularia
   o centro em 240 — explicaria o dobro só em Y. Medido: `GPUSTAT=0x1406260D`, bit19 = 0 (**240
   linhas**), bit22 = 0 (sem entrelaçamento), bits16-18 = 110 (**640** de largura). Está certo.
3. **Escrita direta do vértice pela CPU.** Procurei qualquer `sw` cujo registrador de origem
   valesse `0x00700140` (o vértice cru). Nenhuma. A lista de display vai por **DMA**, montada em
   RAM antes.

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que o comportamento real é | Como foi pego |
|---|---|---|---|---|
| 1 | processo | Que "sem referência não dá para julgar" fosse conclusão suficiente (iteração 0107) | Bastava **pedir a referência**. O usuário forneceu em um minuto o que eu tratei como bloqueio | O usuário mandou a imagem. A lição não é sobre o losango: é que "falta referência externa" é um pedido a fazer, não um beco |
| 2 | hardware | Que geometria 3D no boot implicasse GTE | O logo é desenhado sem uma única instrução de GTE. A forma 3D vem de coordenadas pré-computadas | Contador na divisão do `rtps`: zero. Descartou a hipótese antes de eu escrever qualquer teste |
| 3 | ferramenta | Que a região de textura no despejo da VRAM mostrasse as cores da textura | Ali cada halfword são **quatro índices de CLUT**, não uma cor; pintá-los como RGB de 15 bits produz rosa/azul sem significado. Cheguei a desconfiar de cor errada onde não havia | Comparação com a referência: o texto rasterizado sai vermelho, mas a "cor" da texpage crua não diz nada. Virou a invariante 21 |

## Bateria de mutação

Bateria de mutação: não se aplica — esta iteração não altera código de produção. O que ela entrega
é a referência, a correção da invariante 20 e três hipóteses eliminadas para o item 2.2d.

## Placar antes → depois

Workspace: **735** → **735** testes. Nenhum código mudou.

## Revisão cruzada (orquestrador)

Iteração inteira do orquestrador.

## Decisões e notas

1. **Não mexer no recorte pela área de desenho.** Ele tem teste próprio, está certo, e é o
   caminho mais tentador para deixar o losango bonito escondendo a causa. Dito na invariante 20.
2. **O próximo passo do 2.2d é a lista de display por DMA:** interceptar o canal 2 (GPU) e ler os
   pacotes de polígono direto da RAM, achando quem escreve o `Y` dobrado. É onde as três
   hipóteses descartadas deixam de estreitar e a medição precisa continuar.
3. **2.2e e 2.2f são independentes** e podem ser atacados sem o 2.2d. O 2.2e tem um suspeito
   nomeado: o item 10.13 (modulação vs raw texture, bit 24 do comando não lido) explicaria cor
   errada em texto texturizado.
4. **O boot ainda morre no 4.4h** (passo 85 544 264). A tela que estamos julgando já está completa
   antes disso — conferido despejando a VRAM em 30 M, 50 M, 70 M, 85 M e 85,54 M passos: idêntica.
   Então os defeitos de tela e o crash são independentes.
