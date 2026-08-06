<!-- Custo, tokens e duração NÃO entram aqui: são medidos por scripts/oc-iter.ps1 em
     docs/metricas.csv, casados por head_antes/head_depois. -->

# 0205a — dicr-bus-error

- **Data:** 2026-08-06
- **Item do roadmap:** 10.49 (achado legado, iteração de origem 0116)
- **Objetivo:** o bit 15 (Bus Error) do DICR tem que ser levantado quando uma transferência de
  DMA sai da RAM (wraparound do contador de endereço), não só ficar gravável por CPU sem nunca
  ser levantado pelo hardware emulado.

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| psx-spx | § DICR — "Bus error flag. Raised when transferring to/from an address outside of RAM. Forces bit 31" (L119-135) | docs/reference/04-dma.md |
| psx-spx | § D#_MADR — "wraps around when counting down from 000000h to FFFFFCh" (L48-50) | docs/reference/04-dma.md |

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a medição mostrou | Como foi pego |
|---|---|---|---|---|
| 1 | processo | Rodada do trabalhador (opencode) fez teste+fix corretamente mas travou no passo 6 (bateria de mutação) e nunca abriu PR | Log da rodada não mostrou erro visível ("ok iter=0204..."); só o `git status`/`git log` local revelou que faltava manifesto, doc da iteração e PR | Verificação manual pós-rodada (mesmo padrão da 0204/0203.4 anterior nesta sessão) |
| 2 | endereçamento | Que a checagem certa de bus error era `(addr & 0x00FF_FFFF) + 4 <= ram.len()` (a versão que a rodada do trabalhador implementou) | `cargo test --workspace` reprovou um teste JÁ EXISTENTE (`dma_otc.rs::dma6_otc_ponteiro_guarda_24_bits_e_nao_dobra_em_21`, do ps1-tests otc-test): endereços com bits 21-23 ligados (ex. `0x00FFFF80`) são mascarados/espelhados pro decodificador de RAM de 21 bits e são **válidos**, não erro — só precisam preservar os bits altos no ponteiro gravado. A condição certa é `addr <= 0x00FF_FFFF` (o campo de 24 bits do MADR não é excedido), não uma segunda máscara de 24 bits comparada contra o tamanho de 2 MB da RAM | `cargo test --workspace` local antes de escrever o manifesto — a suíte inteira, não só o teste novo, é o que pegou a regressão |

## Bateria de mutação

Placar da bateria: 5/5 mutantes mortos, 2/2 controles verdes, 0 equivalente —
`docs/mutantes/0205-dicr-bus-error.mut`.

- m1 (checagem do campo de 24 bits removida): morto.
- m2 (teto trocado por 21 bits — quebraria o alias legítimo): morto por
  `dma_otc::dma6_otc_ponteiro_guarda_24_bits_e_nao_dobra_em_21` (teste pré-existente, override
  de `teste:` no registro).
- m3 (bit15 não levantado): morto.
- m4 (bit31 não recalculado): morto.
- m5 (`&&` vira `||`): morto.
- c1 (ordem dos operandos do `&&`): verde.
- c2 (`0x00FF_FFFF` reescrito como `(1u32 << 24) - 1`): verde.

## Placar antes → depois

Workspace: **1264** → **1266** testes (2 novos em `dma_dicr_bus_error.rs`, do trabalhador).

## Revisão cruzada (orquestrador)

O orquestrador (eu) retomei esta iteração depois que a rodada do trabalhador parou no passo 6
sem PR. Revisão do fix do trabalhador encontrou uma regressão real (erro #2 acima), corrigida
antes de prosseguir — não é uma revisão de PR externo, é a mesma pessoa terminando o trabalho
começado por outra rodada.

## Decisões e notas

**1. Só os caminhos OTC e burst (DMA2) têm teste dedicado.** O helper `ram_transfer_in_bounds`
foi aplicado nos ~11 sítios que já tinham o padrão `if offset + 4 <= ram.len()` (burst, block,
linked-list, e os canais 0/1/3/4/6) — os dois testes cobrem o padrão mecânico compartilhado,
não cada canal individualmente (mesma lógica de escopo já usada no achado 0203.4, máscara de
índice de byte).

**2. Não implementei nenhuma outra semântica de bus error.** O achado é só sobre levantar o
bit 15 quando o endereço sai do campo de 24 bits do MADR — não mudei o que acontece com a
transferência em si (ela continua sendo abortada silenciosamente pro resto das palavras, sem
retry nem interrupção do laço), nem toquei em nenhum outro bit do DICR.
