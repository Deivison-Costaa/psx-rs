<!-- Custo, tokens e duração NÃO entram aqui: são medidos por scripts/oc-iter.ps1 em
     docs/metricas.csv, casados por head_antes/head_depois. -->

# 0186 — dma-lista-encadeada-longa

> Metade de uma iteração só: a outra metade, o defeito do controle, está em
> [`0186-sio-ordem-dos-switches.md`](0186-sio-ordem-dos-switches.md). Os dois defeitos estão em
> série no mesmo caminho — consertar só um não move o jogo um quadro.

- **Data:** 2026-08-03
- **Item do roadmap:** 4.4ae
- **Objetivo:** a lista encadeada do DMA2 para de ser cortada por um teto artificial de nós.

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| psx-spx | § Linked List DMA (L198-216) | docs/reference/04-dma.md |
| psx-spx | § D#_MADR (L44-58) | docs/reference/04-dma.md |

## O que estava quebrado

**Teto artificial de 4096 nós na lista encadeada do DMA2.** O runtime do jogo imprimia
`GPU timeout:que=0,stat=5604267e,chcr=01000401,madr=00058498` em laço — achado 0185.2.
`chcr=01000401` é DMA2 em SyncMode=2 com bit24 ainda setado. Instrumentei
`execute_linked_list` para reportar, ao bater no teto, quantos nós andou e se algum endereço
se repetiu. A medição foi direta ao ponto: `ciclo_em=None`, `nos=4097`. **Não havia ciclo** —
a cadeia do Crash é só maior que 4096, e o teto a cortava no meio, deixando o canal ocupado
para sempre.

O teto certo não é escolha de projeto: cada nó começa num endereço alinhado a palavra e o
próximo endereço sai inteiro do header, então uma cadeia com mais nós do que há palavras na RAM
repetiu algum endereço por casa dos pombos — ou seja, **tem ciclo**. E ciclo nunca completa em
hardware, que é o que `dma/chain-looping` mede (`finished = false, irq = false`). Trocar 4096
por `ram.len() / 4` mantém o comportamento de ciclo intacto e para de cortar cadeia legítima.

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | nenhum | Cadeia que não termina = cadeia com ciclo, então o teto de 4096 protegia de algo real | § Linked List DMA só conhece o end-marker como parada; não há teto de nós | Instrumentei o laço com detecção de ciclo: `ciclo_em=None` em todas as 4 ocorrências |

## Bateria de mutação

Placar da bateria: 8/8 mutantes mortos, 2/2 controles verdes, 0 equivalente — docs/mutantes/0186-dma-lista-encadeada-longa.mut

| Mutante | Pego por |
|---|---|
| m1 volta o teto de 4096 | `cadeia_de_4097_nos_completa` |
| m2 next_addr truncado em 20 bits | `cadeia_de_4097_nos_completa` (end-marker deixa de ser visto) |
| m3 teto mil vezes menor | `cadeia_de_20000_nos_completa` |
| m4 end-marker não marca fim | `cadeia_de_4097_nos_completa` |
| m5 MADR não recebe o end marker | `cadeia_de_4097_nos_completa` |
| m6 bit24 não é limpo | os três |
| m7 dados começam no header | `cadeia_longa_entrega_todas_as_palavras_ao_gp0` |
| m8 nenhuma palavra de dados sai | `cadeia_longa_entrega_todas_as_palavras_ao_gp0` |

Dois mutantes da primeira versão do manifesto foram descartados antes de rodar, por não serem
mensuráveis pelo teste do item: um teto **maior** que o correto é indistinguível (a cadeia com
ciclo continua não completando, só mais devagar), e ler a contagem de palavras dos bits 16-23
devolve o mesmo valor para os endereços usados no teste. Mutante que não pode morrer não mede
nada — trocados por `next_addr` truncado e `word_count = 0`.

A terceira prova nasceu fraca: `bit24 == 0` só diz que a cadeia terminou. Cada nó passou a
carregar um `GP0(E1h)`, com valor distinto no último, e a prova confere o GPUSTAT final — assim
ela também mede de onde cada palavra foi lida.

## Efeito colateral

Uma âncora antiga envelheceu por causa desta mudança (a outra, do 0092, está no doc irmão):

- **0129-deliverevent-diagnostico** apontava para a assinatura multilinha de `fn run`, que passou
  a caber numa linha quando as sondas viraram um struct. Reexecutada: 5/5, 2/2 (bateria manual —
  o alvo é `psx-cli`, fora do `psx-core`, invariante 29).

O `bateria_manual.py` do scratchpad rodava sempre `-p psx-core`, e existem dois arquivos de teste
com o nome `deliverevent_diagnostico` (um em cada crate): ele mediu o do `psx-core`, que não toca
o alvo, e reportou 5 sobreviventes falsos. Passou a derivar o pacote do campo `alvo:`.

## Placar antes → depois

- Workspace: 1055 → 1060 testes.
- Crash Bandicoot (USA): `PRESS START` sem resposta → **menu, LOADING, N. SANITY BEACH jogável**,
  com câmera, animação, colisão, morte e respawn. TTY final limpo — sem `GPU timeout`, sem
  `intr timeout`.
- Rayman: sem mudança (as provas de passo absoluto seguem verdes).

## Revisão cruzada (orquestrador)

<!-- Preenchido pelo Claude na revisão do PR: achados no formato de docs/prompts/review.md, ou "sem achados". -->

## Decisões e notas

**`--dump-vram-every N PREFIXO`.** Um dump de VRAM por execução obriga uma execução por instante
medido. Com 2,5 bilhões de passos em 5m27s, descobrir em que passo o jogo troca de tela custava
uma execução por palpite. Com a linha do tempo numa execução só, a folha de contato de 25 quadros
mostrou de uma vez a sequência inteira: logo da Sony → menu → cutscene do laboratório → demo de
N. Sanity Beach → menu → demo do segundo nível. Foi ela que provou que o atrator rodava inteiro e
que o problema era só o Start.

**O `intr timeout(0040:004d)` some sozinho.** Ele aparecia quando o jogo ficava preso na tela da
ilha; na execução que entra no nível o TTY não tem nenhuma ocorrência. Não virou achado.

**Uma iteração, dois itens de correção.** O protocolo pede um item por PR; aqui o usuário pediu
explicitamente marcos maiores ("o marco é o crash funcionando"), e os dois defeitos estão em
série no mesmo caminho. Ficam em commits, manifestos, baterias e docs separados — só o PR é um.
