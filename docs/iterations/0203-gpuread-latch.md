<!-- Custo, tokens e duração NÃO entram aqui: são medidos por scripts/oc-iter.ps1 em
     docs/metricas.csv, casados por head_antes/head_depois. -->

# 0203b — gpuread-latch

- **Data:** 2026-08-05
- **Item do roadmap:** 10.50 (achado legado, iteração de origem 0117)
- **Objetivo:** GPUREAD tem que se comportar como um latch — sem transferência VRAM→CPU em
  curso, uma leitura devolve o último valor lido, não zero fixo.

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| psx-spx | § GPU I/O Ports — GPUREAD recebe respostas de GP0(C0h) e GP1(10h) (L146) | docs/reference/03-gpu.md |
| psx-spx | § GP1(10h) - Read GPU internal register — "the same/latched value can be read multiple times" (L939-940) | docs/reference/03-gpu.md |

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a medição mostrou | Como foi pego |
|---|---|---|---|---|
| 1 | escopo | Que a citação de spec mais direta seria uma seção específica de GP0(C0h) descrevendo o comportamento ocioso | Não existe: a única prosa sobre "GPUREAD é um latch" está na seção de GP1(10h) (ainda não implementado, achado 0193.7), mas GPUREAD é a MESMA porta de hardware para os dois comandos (L146) — a citação vale por extensão, não por estar na seção "certa" | Busquei por "VRAM to CPU blitting" primeiro; a seção não fala de comportamento ocioso, só do protocolo ativo. Achei o texto do latch procurando "latched" no arquivo inteiro |
| 2 | manutenção | Que dava pra reaproveitar `gpu_vram_transfers.rs` pro teste novo | Arquivo já tinha 491 linhas; ficou em 518 com o teste novo, estourando o teto de teste (R8/`file_size.rs`) | `cargo test --test file_size` reprovou; movido pra `gpu_gpuread_latch.rs` novo, dedicado ao achado |
| 3 | ferramenta | Que um `@@DE` de 8 linhas cruzando o fim de uma função e o começo da próxima seria aceito igual aos blocos multi-linha que já usei em manifestos anteriores | `mutation_anchors` reprovou "encontrada 0 vez(es)" apesar do bloco ser byte-idêntico ao fonte (conferido com `diff`); encurtar a âncora para dentro dos limites do match arm (sem cruzar `}` de função) resolveu, mas a causa exata da rejeição não foi identificada — pode ser um limite do casador de blocos longos, não investigado a fundo por não ser o foco desta iteração | `cargo test --test mutation_anchors`; resolvido por tentativa, não por diagnóstico completo |

## Bateria de mutação

Placar da bateria: 5/5 mutantes mortos, 2/2 controles verdes, 0 equivalente —
`docs/mutantes/0203-gpuread-latch.mut`.

- m1 (`gpuread_word` volta a devolver 0 fixo): morto.
- m2 (latch nunca atualizado): morto.
- m3 (latch sempre gravado como 0): morto.
- m4 (`peek_gpuread`, caminho de leitura por byte, volta a devolver 0 fixo): morto.
- m5 (latch inicial = 1 em vez de 0): morto — pega o caso de um GPU novo que nunca fez
  transferência nenhuma.
- c1 (renomeia a variável local `word`→`resultado`, com os dois usos): verde.
- c2 (troca a ordem de declaração do campo `gpuread_latch` na struct/construtor): verde.

## Placar antes → depois

Workspace: **1247** → **1249** testes (2 novos em `gpu_gpuread_latch.rs`).

## Revisão cruzada (orquestrador)

Sem achados — esta iteração foi conduzida pelo próprio orquestrador (exceção vigente em
`docs/orquestracao.md`; ver STATUS.md).

## Decisões e notas

**1. `peek_gpuread` (leitura por byte) também usa o latch.** `region_read_byte` do bus
(`bus.rs:626-631`) lê a porta GPUREAD 32 bits inteira via `peek32` e recorta o byte — sem
sincronizar o fallback ocioso dos dois caminhos (`gpuread_word` e `peek_gpuread`), um byte lido
depois de uma transferência veria um valor diferente de um `read32` no mesmo instante.

**2. Não implementei GP1(10h).** É o achado 0193.7, ainda aberto — o latch que criei aqui
(`gpuread_latch`) é a peça de infraestrutura que GP1(10h) vai precisar (gravar nele em vez de
inventar outro mecanismo), mas populá-lo a partir de GP1(10h) fica pra quando esse achado for
atacado.
