<!-- Custo, tokens e duracao ficam em docs/metricas.csv, medidos pelo runner. -->

# 0182 — diagnostico-endereco-errado

- **Data:** 2026-08-03
- **Item do roadmap:** 0182.1 (o defeito no diagnóstico) e 0182.2 (o que sobra de verdade).
- **Objetivo:** entender por que o Rayman ainda dá `VSync: timeout` depois da 0180.
- **Fonte:** orquestrador.

**O diagnóstico estava lendo o endereço errado, e vinha "confirmando" uma hipótese falsa.**

## Spec consultada

Nenhuma: o defeito é de instrumentação, não de hardware.

## O que estava errado

`vsync_timeout_diag.rs` lia o contador de VBlank do jogo em **`0x801DF2CC`** e afirmava, com a
suíte verde:

> "A hipotese (a) esta confirmada: IRQ0 e levantada, a CPU entra no handler, I_MASK tem bit0
> habilitado, mas o handler do jogo nunca incrementou o contador. (...) A cadeia ExCB/EvCB nao
> contem entrada para classe F0000001 (VBlank callback)."

O executável do Rayman ocupa `0x80125000..0x801CF800`. **`0x801DF2CC` cai fora dele** — é RAM
intocada, e vale zero para sempre. O contador de verdade é `0x801CF2CC`, um dígito hex de
diferença, e é o mesmo que o achado 10.85 já citava corretamente desde a iteração 0159.

As duas afirmações da conclusão são falsas, e medi as duas:

| Afirmação do diagnóstico | Medido em 700 M passos |
|---|---|
| "o handler nunca incrementou o contador" | `[0x801CF2CC]` = **1469** |
| "a cadeia não contém entrada para F0000001" | `DeliverEvent(F0000001)` chamado **1723 vezes** |

Esse teste passou em toda rodada desde que foi escrito, e a conclusão dele foi copiada para o
`STATUS.md` e citada em iterações seguintes. Um teste verde medindo o lugar errado é pior que
teste nenhum: ele fecha a pergunta.

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a medição diz | Como foi pego |
|---|---|---|---|---|
| 1 | diagnóstico | Que o `VSync: timeout` fosse consequência da cadeia de eventos, como o diagnóstico afirmava havia várias iterações. | `F0000001` é entregue 1723 vezes e o contador chega a 1469. | Fui verificar a afirmação em vez de reusá-la, e o endereço não batia com o do achado 10.85. Dois endereços para o mesmo contador, um dígito de diferença. |
| 2 | medição | Que corrigir o endereço fosse fechar o item. | No instante do **primeiro** timeout o contador está em **1**, com 660 IRQ0 e 1470 entradas de handler. | O contador anda — mas tarde. A defasagem inicial é o defeito que sobra, e agora tem número em vez de teoria. |

## A mudança

Endereço corrigido e as asserções reescritas para afirmar o que é verdade: o contador **anda**
(refuta a hipótese antiga) e, no instante do timeout, ainda está **atrás** do número de VBlanks
(fixa o defeito que sobra). Se algum dia passar, o teste reprova e manda atualizar o achado —
em vez de continuar verde medindo o vazio.

## Bateria de mutação

Bateria de mutação: não se aplica — a rodada corrige um teste de diagnóstico e não toca
`crates/*/src/`.

## Placar antes → depois

Workspace: **1024 → 1024** testes (o mesmo teste, agora medindo o lugar certo).

| | antes | depois |
|---|---|---|
| endereço lido | `0x801DF2CC` (fora do executável) | `0x801CF2CC` |
| valor observado | `0` (sempre) | `1` no timeout, `1469` em 700 M |
| conclusão registrada | cadeia de eventos quebrada | defasagem inicial do contador (0182.2) |

## Revisão cruzada (orquestrador)

Rodada de autoria do orquestrador. A verificação é aritmética e independente de julgamento: o
endereço antigo está fora da faixa do executável do jogo, que o próprio `STATUS.md` documenta
como `0x80125000..0x801CF800`.

## Onde a defasagem realmente está (0182.2)

Fui atrás em vez de arquivar. O caminho, medido:

O Rayman **não usa o sistema de eventos da BIOS para IRQ**. Ele tem despachante próprio em
`0x801B8E98`, com tabela de handlers por número de IRQ em `0x801C9290` (IRQ0 = `0x801B8C3C`, o
incrementador do contador; IRQ2 = `0x801A9198`, o poller de CD-ROM da 0178). O despachante lê
`I_STAT` e `I_MASK` por ponteiro (`[0x801CF2DC]=0x1F801070`, `[0x801CF2E0]=0x1F801074`), tem
máscara própria em `[0x801CF2E4]=0x0D` (VBlank, CDROM, DMA) e confirma cada IRQ escrevendo
`~bit` em `I_STAT` com `sh`.

Em 300 M passos: o despachante passa do portão de "há IRQ pendente" **492 vezes**, e o handler de
VBlank roda **3**. Ou seja, quando o código do jogo chega lá, o **bit 0 de `I_STAT` já foi
apagado por outro** — o handler da própria BIOS, antes na cadeia.

Duas causas candidatas eliminadas por inspeção:

- **A semântica de ack do `I_STAT` está correta.** `write_stat` faz `stat &= val | !0x7FF`, que é
  escrever-zero-para-limpar, e `write_stat_half` cobre o `sh` de 16 bits que o jogo usa.
- **Não há ack automático no emulador.** `self.stat` só é limpo por escrita explícita do
  programa; nada em `bus.rs` ou no scheduler apaga IRQ0 por conta própria.

Resta a ordem da cadeia de handlers da BIOS — território do achado 10.83, agora com número: 492
oportunidades para 3 execuções.

Também registro o que **não** é a causa: o VBlank é agendado e levanta IRQ0 (`VBLANK_ENTER` em
`bus.rs`), `I_MASK` tem o bit 0 ligado (`mask=0x000D`), o Timer1 não está sincronizado com VBlank
(`tmr1_sync=false`), e o modelo de ciclos já cobra os 7 ciclos de RAM principal que § Load Timing
(L262-268) de docs/reference/02-cpu.md pede.
