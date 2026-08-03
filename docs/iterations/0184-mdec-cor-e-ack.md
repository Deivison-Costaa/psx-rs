<!-- Custo, tokens e duracao ficam em docs/metricas.csv, medidos pelo runner. -->

# 0184 — mdec-cor-e-ack

- **Data:** 2026-08-03
- **Itens:** 0183.2 (a alça `0x80132BF0`), 10.90 (`VSync: timeout`), 10.113 (yuv2rgb), 0184.1.
- **Objetivo:** o Rayman roda. Autorização explícita do usuário para cruzar quantos itens fosse
  preciso numa iteração só.
- **Fonte:** orquestrador.

**O Rayman entra no jogo.** Ele carrega, toca a intro em MDEC, mostra a tela de "insira o
controle", aceita o controle e chega ao primeiro nível — Rayman voando entre as nuvens, com HUD
e sprites animados nos dois framebuffers.

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| psx-spx | § decode_colored_macroblock (L172-180) | docs/reference/09-mdec.md |
| psx-spx | § yuv_to_rgb(xx,yy) (L269-285) | docs/reference/09-mdec.md |
| psx-spx | § MDEC Data/Response Register (L70-78) | docs/reference/09-mdec.md |
| psx-spx | § Colored Macroblocks (L376-393) | docs/reference/09-mdec.md |
| psx-spx | § MDEC(2) - Set Quant Table(s) (L141-149) | docs/reference/09-mdec.md |
| psx-spx | § Address byte (01h) being sent (L379-386) | docs/reference/10-controllers-memcards.md |
| psx-spx | § Emulation Note (L316-320) | docs/reference/10-controllers-memcards.md |

## A cadeia, elo por elo

A 0183 tinha deixado o jogo parado em `0x80132BF0`, esperando o byte `[0x801CF5F4]` que nenhum
`sb` de deslocamento fixo da imagem escrevia. Desmontando a partir do endereço, o byte é escrito
em `0x80132A30`, no fim de uma rotina registrada por `0x801B8A90` — que é `DMACallback(1, f)`,
callback de fim do **canal 1 de DMA (MDECout)**. A rotina desenha uma faixa de 16 pixels e só
levanta o byte quando a faixa 320 termina. Três defeitos quebravam essa cadeia:

**1. O MDEC não decodificava em cor.** `run_decode` saía sem produzir nada quando a profundidade
era 24 ou 15 bits. O Rayman manda `MDEC(1)` com `depth=3` e 12352 palavras; a fifo de saída
ficava vazia e o DMA1 não tinha o que levar.

**2. O DMA1 desistia.** O jogo arma o canal 1 **antes** de mandar os dados pelo canal 0 — no
console os dois correm juntos, servidos por DREQ. Nosso DMA é síncrono: o canal 1 encontrava a
fifo vazia, transferia zero palavra e ficava armado para sempre. Sem fim de canal não há IRQ3, e
sem IRQ3 o callback nunca é chamado de volta.

**3. O DMA1 não costurava o macrobloco.** § MDEC Data/Response Register (L74-78) de docs/reference/09-mdec.md
diz que o registrador entrega quatro bitmaps 8x8 em fila e que "usually, the data is received via
DMA1, which is doing the re-ordering automatically". Copiando linear, cada macrobloco 16x16 saía como
quatro faixas de 16x4. A VRAM mostrou exatamente isso: a imagem certa, listrada.

Com os três corrigidos o jogo passou a intro e parou em **"PLEASE INSERT A STANDARD SONY
PLAYSTATION CONTROLLER INTO PORT 1"** — com `--pad` ligado. O quarto defeito:

**4. O /ACK do SIO0 chegava no meio do byte.** § Address byte (01h) being sent (L379-386) de docs/reference/10-controllers-memcards.md:
o /ACK vem depois do **último pulso de SCK**, e a taxa do kernel é ~250 kHz — 136 ciclos por bit,
1088 pelo byte. § Emulation Note (L316-320) de docs/reference/10-controllers-memcards.md explica por que o instante importa:

> "After sending a byte, the Kernel waits 100 cycles or so, and does THEN acknowledge any old
> IRQ7, and does then wait for the new IRQ7. Due to that bizarre coding, emulators can't trigger
> IRQ7 immediately within 0 cycles after sending the byte."

Entregávamos o /ACK 338 ciclos depois do byte. Ele chegava antes de o driver limpar a IRQ7 velha
(`I_STAT &= ~80h` em `0x800045A8`), era apagado por essa limpeza, e o driver concluía que a porta
estava vazia. Cada rodada imprimia **483** vezes `PS-X Control PAD Driver Ver 3.0`, reiniciando o
driver. Com o /ACK depois do byte: **5**.

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a medição diz | Como foi pego |
|---|---|---|---|---|
| 1 | endereçamento | Que `0x801B8A90(f)` registrasse o callback na **IRQ 1 (GPU)** e que o jogo dependesse de `GP0(1Fh)`. Instrumentei a GPU para contar `GP0(1Fh)`. | Zero em 250 M passos. O número é o **canal de DMA**, não a IRQ: `0x801B93B0` grava `1<<(16+n)` e `1<<23` no DICR. | A sonda deu zero. Ler o registrador que a função realmente escreve custou menos que a hipótese. |
| 2 | arredondamento | Que o desvio sistemático de −1 em todos os canais de cor fosse ruído do IDCT, e varri 8 combinações de arredondamento do `real_idct_core` e do `fast_idct_core`. Nenhuma passou de 181/512. | O erro era **sempre −1, nunca +1**: 774 de 3072 canais. Isso é assinatura de truncamento, não de ruído. A redução de 8 para 5 bits **arredonda**: `(v+4)>>3`. De 245 divergências para 43. | Olhei a distribuição do erro em vez de continuar chutando variantes. Ruído dá os dois sinais; viés dá um só. |
| 3 | teste | Que os testes verdes de `mdec/4bit` e `mdec/8bit` provassem o caminho monocromático. | As constantes vinham de um script Python que seguia a spec, não do console. Contra os `psx.log` reais elas erram **16 de 64 bytes** (8bit) e **16 de 32** (4bit) — o mesmo truncamento do erro #2, num caminho que estava "coberto" desde a 0174. | Minha correção "quebrou" os dois testes. A quebra estava certa; o verde é que era falso. |
| 4 | teste | Que a bateria cobrisse a tabela de quantização de cor e o alinhamento do DMA1. | **m7 e m9 sobreviveram.** No gabarito do ps1-tests as duas metades da tabela de quantização são iguais, e meu teste de swizzle pedia um múltiplo exato de macroblocos. | Dois testes novos: tabela de cor dobrada tem de mudar o quadro; fifa com macrobloco e meio não pode ser levada pela metade. 10/12 → 12/12. |
| 5 | ferramenta | Que o `teste:` do cabeçalho do manifesto valesse para os registros sem `teste:` próprio. | O `mutantes.ps1` herda o **último** `teste:` lido (achado 10.71): oito registros rodaram contra `sio_ack_atrasado` e a bateria reportou **4/12 falso**. | O placar não batia com o que eu sabia dos testes. Explicitei `teste:` em todos os 14 registros. |

Registro também que a 0174 escreveu, no próprio doc, que cor "fica para uma próxima iteração:
nenhuma suíte deste lote exercita esse caminho (R5)". Errado: `mdec/step-by-step-log`,
`mdec/frame` e `mdec/movie` exercitam, e estavam em `tests/exes/` desde a 0164.

## Bateria de mutação

Placar da bateria: **12/12 mutantes mortos, 2/2 controles verdes, 0 equivalente** —
docs/mutantes/0184-mdec-cor-e-ack.mut

| # | O que quebra | Quem pega |
|---|---|---|
| m1 | bloco não fecha em `k==63` | `mdec_15bpp_reproduz_o_gabarito_de_hardware_dentro_de_um_passo` |
| m2 | Cr e Cb trocados | idem |
| m3 | quadrante ignora a coluna | `dma1_reordena_os_quatro_blocos_8x8_em_macroblocos_16x16` |
| m4 | croma sem subamostragem 2x2 | gabarito de 15bpp |
| m5 | redução para 5 bits trunca | gabarito de 15bpp |
| m6 | redução para 4 bits trunca | `mdec_decode_4bit_bloco_heart_bate_com_o_gabarito_de_hardware` |
| m7 | tabela de cor ignorada | `tabela_de_quantizacao_de_cor_e_usada_por_cr_e_cb` |
| m8 | DMA1 sem swizzle | `dma1_reordena_...` |
| m9 | DMA1 leva macrobloco pela metade | `dma1_nao_leva_macrobloco_pela_metade` |
| m10 | CPU alimentando o MDEC não retoma o DMA1 | `mdec_dma1_armado_antes_dos_dados_...` |
| m11 | /ACK ignora o tempo do byte | `ack_nao_chega_antes_de_o_byte_terminar_de_sair` |
| m12 | fator de baud ignorado | `baud_mais_lento_atrasa_o_ack_na_mesma_proporcao` |

A bateria da **0174** foi reexecutada com duas âncoras renovadas (o empacotamento de 4 bits e o
`Current Block` do status mudaram nesta rodada): **6/6 e 2/2**.

## Placar antes → depois

| | antes | depois |
|---|---|---|
| Rayman | parado em `0x80132BF0`, VRAM com 2 cores | **jogando**, HUD e sprites, 153 600 pixels redesenhados a cada quadro |
| `PS-X Control PAD Driver` | 483 impressões | 5 |
| MDEC colorido | não existia | 93% das 512 palavras do gabarito batem byte a byte; nenhum canal desvia mais de um passo |
| `mdec/4bit` | 16 de 32 bytes errados (teste verde) | 2 nibbles, com teto no teste |
| `mdec/8bit` | 16 de 64 bytes errados (teste verde) | 16 de 64, todos dentro de ±2, com piso de exatos no teste |

## Revisão cruzada (orquestrador)

Rodada de autoria do orquestrador. O que sustenta a rodada não é a suíte verde — a de
`mdec/4bit` e `mdec/8bit` era verde e errada — e sim o gabarito palavra a palavra do
`mdec/step-by-step-log`, que é saída de console real e não foi derivado de nenhuma implementação
nossa.

## Decisões e notas

**Nenhuma das quatro correções é específica do Rayman.** MDEC colorido vale para todo jogo com
FMV; o DMA1 retomar vale para qualquer produtor assíncrono; o swizzle é do hardware; o /ACK
atrasado vale para todo jogo que lê o controle pelo driver do kernel — isto é, praticamente
todos. O Rayman foi o instrumento de medida, não o alvo.

**O resíduo de arredondamento fica.** § real_idct_core (L241-267) de docs/reference/09-mdec.md diz que o hardware
"isn't perfect" e que a resolução exata do `yuv_to_rgb` é desconhecida. 35 das 512 palavras divergem, e
os testes fixam o que se pode afirmar: nenhum canal desvia mais de um passo, e o número de
palavras exatas não pode cair. Quem quiser fechar os 7% tem o gabarito e o teto no teste.

**`VSync: timeout` continua em 296 e agora é suspeito de ser normal.** Eles acontecem antes do
executável do jogo assumir; o jogo roda depois disso. Fica como 0184.1 até alguém medir contra
console, e não como bloqueio.
