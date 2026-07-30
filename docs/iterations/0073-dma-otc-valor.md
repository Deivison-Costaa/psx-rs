# 0073 — dma-otc-valor

- **Data:** 2026-07-29
- **Item do roadmap:** 10.20
- **Objetivo:** gravar em cada palavra do OTC o endereço que será visitado a seguir, com o
  terminador na última palavra escrita e ponteiro de 24 bits.

## Revisão do PR anterior

Revisão do PR anterior (#86, iter 0072): sem achados novos. Os nove padrões conferidos: teste que
não mede (o portão novo foi falsificado antes de entrar); parâmetro não consumido (sem comando
GP0); regra de borda (sem rasterização); campo de bit (sem leitura de bits); panic ou laço
ilimitado (só mudança de ordem de dois booleanos); citação de spec (a do CHCR foi conferida à mão
e é o motivo do PR); escopo transbordado (nenhum); portão que não mede (era exatamente o assunto);
manifesto arquivado (nenhum).

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| psx-spx | 1F801088h+N\*10h - D#\_CHCR - DMA Channel Control (Channel 0..6) (R/W) (L84) | docs/reference/04-dma.md |
| psx-spx | DMA Register Summary (L27) | docs/reference/04-dma.md |

A passagem que decide o item está nas L117-119: o D6\_CHCR tem o bit 1 sempre em 1, com
`increment=-4`. O OTC **desce** a partir do MADR — o sentido de varredura da implementação
anterior já estava certo, e foi por isso que o handoff mandou não mexer nele. O canal aparece
como "reverse clear OT" na L35, dentro do sumário de registradores.

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | endereçamento | Que o terminador ia na primeira palavra escrita (em MADR) e cada palavra recebia o endereço da palavra ANTERIOR — o que dá ponteiros subindo | Descendo com `increment=-4`, cada palavra recebe o endereço que será visitado a seguir, o de baixo, e o terminador cai na ÚLTIMA palavra escrita, que é a mais baixa | `ps1-tests/dma/otc-test`, que exige `buf[0]=0xFFFFFF` e `buf[i]=&buf[i-1]`. O teste próprio afirmava o espelho disso desde a iteração 0056 e certificava o defeito em vez de pegá-lo |
| 2 | endereçamento | Que dobrar o ponteiro gravado na máscara de RAM de 21 bits (`0x1F_FFFC`) fosse inofensivo | O valor guardado tem 24 bits. O `otc-test` roda com o buffer acima de 2 MB e compara contra `(uint32_t)&buf[i] & 0xffffff`; com a dobra de 21 bits os bits 21-23 somem | A iteração 0068 levantou isso como **hipótese**, sem confirmar. Confirmado aqui pelo hardware |
| 3 | processo | Que a bateria fecharia com o teste reescrito | m4 (máscara 24 → 21 bits) **sobreviveu**: todos os endereços do arquivo de teste ficam abaixo de 2 MB, faixa onde as duas máscaras coincidem. O teste media o comportamento certo e era cego para o defeito 2 | `scripts/mutantes.ps1 -Iter 0073`, placar 5/6. Fortalecido com `dma6_otc_ponteiro_guarda_24_bits_e_nao_dobra_em_21`, que usa MADR em `0x00FF_FF80` — a faixa que o hardware exercita |
| 4 | processo | Que uma rodada morta deixasse o repositório em estado reciclável | A sessão travou às 20:21 deixando o manifesto **não rastreado**. `git reset --hard` não remove arquivo não rastreado, e o `oc-iter.ps1` recusa árvore suja: as 19 rodadas seguintes do encadeamento morreram em segundos na mesma checagem | `logs/loop-noite2.err.log`, dezenove vezes `Arvore suja - commit ou descarte antes de iterar`. Consertado no `oc-loop.ps1` nesta iteração |

## Bateria de mutação

Placar da bateria: 6/6 mutantes mortos, 2/2 controles verdes, 0 equivalente — docs/mutantes/0073-dma-otc-valor.mut

| Registro | Rótulo | Teste que pegou |
|---|---|---|
| m1 | terminador na primeira palavra em vez da última | `dma6_otc_preenche_ram_com_linked_list` |
| m2 | grava `addr+4` em vez de `addr-4` | `dma6_otc_preenche_ram_com_linked_list` |
| m3 | não grava terminador nenhum | `dma6_otc_preenche_ram_com_linked_list` |
| m4 | máscara de 21 bits em vez de 24 | `dma6_otc_ponteiro_guarda_24_bits_e_nao_dobra_em_21` |
| m5 | condição do último índice trocada por `i == 0` | `dma6_otc_preenche_ram_com_linked_list` |
| m6 | não decrementa o endereço no laço | `dma6_otc_bcr_zero_equivale_a_10000h` |
| c1 | local sem uso antes do laço | sobreviveu |
| c2 | formatação numérica do literal | sobreviveu |

As atribuições acima foram lidas do `.resultado` gerado pela máquina, não preenchidas por
inspeção — a iteração 0071 errou 3 de 9 linhas exatamente por preencher à mão (item 10.28).

## Placar antes → depois

Workspace: **556** → **557** testes (+1: o teste de 24 bits em `dma_otc`).

**Hardware, que é o ponto deste item:** `ps1-tests/dma/otc-test` foi de **7p/30f** para
**15p/0f**. A suíte inteira passa. Antes da pausa de análise de hoje ela estava em `3p/35f`, e
ninguém havia lido o número.

| Momento | otc-test |
|---|---|
| 28/07 15:38 a 29/07 06:53, 14 execuções | 3p/35f |
| depois do item 3.2 (iter 0056) | 6p/34f |
| depois do gate do DPCR (iter 0071) | 7p/30f |
| **depois deste item** | **15p/0f** |

## Revisão cruzada (orquestrador)

Esta iteração foi **começada pelo trabalhador e terminada pelo orquestrador**, e isso é um dado de
processo que vale registrar. A sessão do trabalhador escreveu o teste, a implementação e o
manifesto, e morreu às 20:21 sem abrir PR — o transcript parou de crescer e o daemon ficou idle. A
branch `iter/0073-dma-otc-valor` ficou com três commits e um manifesto sem commit.

O que o trabalhador acertou sozinho: o valor gravado, a ponta do terminador, a máscara de 24 bits
e a reescrita das asserções do `dma_otc.rs`, que ficaram **mais fortes** do que eram — passaram a
conferir as quatro palavras em vez de três, com a máscara certa. Também achou um defeito no teste
antigo, no endereço do terminador do caso BCR=0.

O que o orquestrador fez: commitou o manifesto que ficou solto, rodou a bateria, viu m4 sobreviver,
escreveu o teste que o mata, mediu contra o hardware, e consertou o `oc-loop.ps1`.

A alternativa era descartar a branch e deixar o encadeamento redoer o item. Custaria uma rodada e
arriscaria uma implementação pior — e a implementação parada ali já fechava a suíte de hardware.

## Decisões e notas

1. **O `oc-loop.ps1` agora para quando a árvore continua suja depois do reset**, imprimindo os
   arquivos. Não usa `git clean`: apagar não rastreado sem olhar foi o que destruiu quatro commits
   na iteração 0038. Uma parada honesta vale mais que dezenove falhas idênticas.
2. **A hipótese da 0068 sobre a máscara estava certa**, e é o segundo caso do dia em que um número
   de hardware resolveu uma dúvida que a leitura da spec sozinha havia deixado aberta. Vale para o
   relatório: a suíte não só encontra defeito, ela decide entre leituras plausíveis da spec.
3. **`otc-test` verde não significa OTC correto.** A suíte tem 15 asserções e cobre o padrão da
   lista, o gate de habilitação e o comportamento com chopping e sync modes; não cobre timing nem
   a interação com prioridade entre canais. O item 10.1 continua valendo.
4. O teste novo usa `assert_ne!` de propósito, como **segunda** asserção depois de um `assert_eq!`
   que fixa o valor exato: ele nomeia o valor que a máscara errada produziria. Não é o padrão
   proibido, que é usar `assert_ne!` como única afirmação (item 10.29).
