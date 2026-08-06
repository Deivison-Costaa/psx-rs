<!-- Custo, tokens e duração NÃO entram aqui: são medidos por scripts/oc-iter.ps1 em
     docs/metricas.csv, casados por head_antes/head_depois. -->

# 0203f — dma-chopping

- **Data:** 2026-08-06
- **Item do roadmap:** 10.109 (achado legado, iteração de origem 0173)
- **Objetivo:** DMA2 (GPU) em SyncMode=0 (Burst) com o bit de chopping ligado (CHCR.8) tem
  que atualizar MADR e zerar o campo BC ao final da transferência — hoje ficam sempre
  congelados, mesmo com chopping ligado.

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| psx-spx | § D#_MADR — "unless Chopping is enabled, in that case it does update MADR" (L48-50) | docs/reference/04-dma.md |
| psx-spx | § D#_BCR — "SyncMode=0 with chopping enabled decrements BC to zero" (L80-81) | docs/reference/04-dma.md |

## Erros de primeira tentativa

Nenhum — a spec já dizia exatamente o que fazer (MADR e BC congelados por padrão, exceto com
chopping), o teste sem-chopping já existente (`dma_burst_sync0.rs`, da iteração 0173) já
confirmava metade do contrato, e `execute_burst` já calculava o endereço final numa variável
local — só faltava escrevê-la de volta condicionalmente ao chopping.

## Bateria de mutação

Placar da bateria: 5/5 mutantes mortos, 2/2 controles verdes, 0 equivalente —
`docs/mutantes/0203-dma-chopping.mut`.

- m1 (condição do chopping invertida): morto.
- m2 (MADR não escrito mesmo com chopping): morto.
- m3 (BC não zerado mesmo com chopping): morto.
- m4 (máscara zera os bits altos em vez do campo BC): morto.
- m5 (MADR final grava um endereço a mais, off-by-one word): morto.
- c1 (ordem das duas atribuições dentro do `if`): verde.
- c2 (`&= !0xFFFF` reescrito como `&= 0xFFFF_0000`, equivalente): verde.

## Placar antes → depois

Workspace: **1251** → **1252** testes (1 novo em `dma_burst_sync0_chopping.rs`).

## Revisão cruzada (orquestrador)

Sem achados — esta iteração foi conduzida pelo próprio orquestrador (exceção vigente em
`docs/orquestracao.md`; ver STATUS.md).

## Decisões e notas

**1. Não implementei o cycle-stealing real do chopping.** A spec descreve chopping como
literalmente interromper a transferência a cada N palavras pra devolver M ciclos à CPU
(campos "Chopping DMA/CPU window size" do CHCR) — isso exigiria um modelo de DMA incremental
(passo a passo, coordenado com o scheduler), enquanto este projeto tem DMA síncrono (a
transferência inteira acontece numa chamada só). Essa lacuna já está registrada nos achados
10.102/10.114 ("DMA sem custo por ciclo") — este fix só corrige o ESTADO FINAL de MADR/BCR
depois que a transferência (síncrona) termina, que é o que um jogo lendo os registradores
depois do fato observaria, sem fingir que a temporização intermediária está certa.

**2. Só o canal 2 (GPU) tem burst mode com chopping neste código.** `execute_burst` é
específico do DMA2 (usa `self.madr[2]`/`self.bcr[2]`/`self.chcr[2]` diretamente, sem
parâmetro de canal) — nenhum outro canal chama essa função, então o achado 10.109 e este fix
são inerentemente sobre DMA2, não uma lacuna genérica de todos os canais.
