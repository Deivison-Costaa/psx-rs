# 0068 — placar-hardware

- **Data:** 2026-07-29
- **Item do roadmap:** 10.2a
- **Objetivo:** ler o placar do ps1-tests, que nunca foi lido por ninguém, e converter o que
  ele mede em itens do ROADMAP.

## Spec consultada

Nenhuma — este passe **mede e registra**, não conserta. Cada defeito virou item (10.19 a 10.23)
justamente para que a correção seja feita com a seção de spec aberta, por R1. Diagnosticar DMA
ou GPU a partir dos números aqui, sem abrir `04-dma.md` / `03-gpu.md`, é o erro que R1 proíbe;
onde este doc arrisca uma hipótese, ela está marcada como hipótese.

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a medição diz | Como foi pego |
|---|---|---|---|---|
| 1 | processo | Que o item 2.9 ("suíte GPU do ps1-tests no scoreboard") tinha ligado uma medida contra hardware real | O item não mediu nada. Os 6 testes de `gpu_scoreboard.rs` só fazem `script.contains("tty")`, `contains("-Recurse")`, `contains("tests/exes")` — afirmam propriedades do texto de um `.ps1`, não do emulador. O próprio doc da 0054 admite que a suíte já estava integrada desde a 0007 | Leitura de `crates/psx-core/tests/gpu_scoreboard.rs` nesta pausa |
| 2 | processo | Que o placar em `logs/scoreboard.csv` refletia a `main` | Estava no commit `c947ea9` (PR #68, 06:47), **13 merges atrás** — todo o M3 e todo o M4 entraram depois. O `scoreboard.ps1` só roda quando alguém o chama; nada no loop o chamava | `git log c947ea9..main` durante esta análise |
| 3 | emulação | Que 13 testes verdes em `dma_otc.rs` significassem OTC correto | O hardware reprova 34 de 40 subtestes, e o teste próprio afirma o **espelho** do que o hardware exige (item 10.20). Verde contra o teste errado | `ps1-tests/dma/otc-test` executado no `465192e` |
| 4 | processo | Que eu pudesse escrever o handoff citando a seção do DPCR de memória | A linha real é **L121**, não a L127 que eu havia escrito: L127 é a linha de prioridade do DMA2 *dentro* da seção. O índice diz `L96` e o offset do `CORPO:` deste arquivo é +25 (marca na L24) | Conferido à mão contra `docs/reference/04-dma.md` antes de commitar, porque citação inventada foi o erro mais frequente do dia. O portão não teria pegado: a linha existe e o título casaria |
| 5 | processo | Que referir linha de código-fonte em prosa (`try_execute_dma3` (L90)) fosse neutro | `spec_citations.rs` lê qualquer `(L<n>)` como citação de spec e exige o nome do arquivo na mesma linha; não distingue fonte de referência, e não tem como | O portão reprovou com "sem menção a arquivo anterior" em 5 pontos. Corrigido nomeando o arquivo inline, que ficou mais claro de ler |

## O placar, no commit 465192e

`51` arquivos varridos: **5 com veredito** (1 pass, 4 fail), **45 com saída mas sem veredito**
(status `tty`), 1 binário de host.

| Suíte | Placar | Item que deveria cobrir |
|---|---|---|
| `ps1-tests/cpu/cop` | 2p/0f | — (único verde) |
| `ps1-tests/cpu/code-in-io` | 1p/2f | já é a dívida 10.3 |
| `ps1-tests/dma/otc-test` | **6p/34f** | 3.2, marcado `[x]` |
| `ps1-tests/gpu/gp0-e1` | 7p/3f | 2.5a, marcado `[x]` |
| `ps1-tests/gpu/mask-bit` | 3p/2f | 2.6c, marcado `[x]` |

### O que 13 merges de M3 e M4 moveram

`otc-test` ficou em `3p/35f` em **14 execuções consecutivas**, de 28/07 15:38 a 29/07 06:53, e
está em `6p/34f` agora. O item 3.2 (DMA canal 6 OTC, iter 0056) é o que mexeu nesse número:
**+3 subtestes**, de 38 alcançados para 40. `gpu/mask-bit` está em `3p/2f` nas 11 execuções desde
28/07 18:47, sem uma única variação. `gpu/gp0-e1` saiu de `5p/5f` para `7p/3f` entre 28/07 23:33
e 29/07 01:34 e não se moveu nas 5 execuções seguintes.

Ou seja: os 58 itens verdes movimentaram o veredito de hardware em exatamente **3 subtestes**
desde ontem à tarde. Não porque o emulador não avance — porque quase nada do que ele avança está
sendo medido contra hardware.

## Os defeitos, com a evidência de cada um

### 10.19 — DPCR não é gate em nenhum canal (4 subtestes)

`testOtcStandardWithMasterDisabled:259-262` desabilita o canal e espera o buffer intacto
(`0x11111111`..`0x44444444`); nós entregamos a lista transferida. Em
`crates/psx-core/src/dma.rs` o campo `dpcr` só tem `read_dpcr`/`write_dpcr`, e as três funções
`try_execute_otc`, `try_execute_dma3` e `try_execute_dma2` olham apenas CHCR.
**Não há gate de habilitação em canal nenhum.**

Vale registrar que o handoff do item 3.2 já apontava para esta seção do `04-dma.md` e mencionava
o master enable: a citação foi lida e o comportamento não foi implementado. O verificador de
citações confere que o ponteiro existe, não que ele foi obedecido — e nenhum portão pode conferir
isso, porque exigiria entender a spec.

Uma consequência que só apareceu ao abrir a spec para escrever o handoff: o valor de reset do
DPCR é `07654321h`, cujos nibbles vão de 1 a 7, então **o bit 3 de todos eles é zero** e todo
canal nasce desabilitado. Ligar o gate vai deixar vermelha parte dos 39 testes de `dma_otc`,
`dma_gpu` e `cdrom_dma`, que disparam CHCR sem nunca habilitar o canal. Está anotado no handoff
para que essa vermelhidão seja lida como resultado esperado, e não como motivo para relaxar o
gate.

### 10.20 — a lista do OTC é o espelho da do hardware (30 subtestes)

Para um buffer de 4 palavras, `testOtcStandard:28-31` espera
`buf[0]=0xffffff`, `buf[1]=&buf[0]`, `buf[2]=&buf[1]`, `buf[3]=&buf[2]`: terminador no índice
mais baixo, cada palavra apontando para a de baixo. Nós produzimos o contrário — terminador no
índice mais alto e cada palavra apontando para a de cima —, porque `try_execute_otc` escreve o
terminador em MADR e depois **desce** (`addr.wrapping_sub(4)`, `dma.rs:85`).

O ponto que interessa ao projeto: `dma_otc.rs:79-81` afirma literalmente `"ultimo slot = end
marker"` e `"slot N-1 aponta para slot N"` — o espelho exato do que o hardware exige. O teste
próprio não pode ser o árbitro aqui, e os 13 verdes do item 3.2 certificam o defeito em vez de
pegá-lo. Duas leituras ainda cabem nos dados (o hardware sobe a partir de MADR e escreve o
terminador primeiro; ou desce e escreve o terminador por último) e **só a seção de OTC de
`04-dma.md` mais a fonte do teste decidem qual**. Isso é trabalho do item, não deste doc.

Há uma segunda diferença, separada: os ponteiros absolutos que gravamos (`0x1fff88`) não são os
do hardware (`0xffff80`). O endereço esperado pelo teste fica acima de 2 MB, e nós dobramos o
ponteiro gravado com `& 0x1F_FFFC` (`dma.rs:84`). **Hipótese**, não conclusão: a dobra de 21 bits
no valor armazenado é indevida. Confirmar pela spec antes de mexer.

### 10.21 — bit 15 do GPUSTAT escrito sem o gate de GP1(09h) (3 subtestes)

As três falhas de `gpu/gp0-e1` são o mesmo defeito, e todas diferem por exatamente `0x8000`:

```
testWriteOnesToE1:41                          given 0x87ff, expected 0x7ff
testTexturedPolygons:60                       given 0x81ff, expected 0x1ff
testTextureDisableBitIsIgnoredWhenNotAllowed  given 0x8000, expected 0x0
```

O nome do terceiro entrega o critério: o bit de Texture Disable é **ignorado quando não
permitido**. Nós o gravamos sempre. Uma correção, três subtestes.

### 10.22 — mask bit (2 subtestes)

`testSetBit:24` espera `0x8000` na VRAM e lê `0x0`; `testCheckMaskBit:40` espera `0x8000` e lê
`0x1234`, isto é, sobrescrevemos um pixel que deveria estar protegido. Provável sobreposição com
a dívida 10.7 (mask de GP0(E6h) não aplicado a CPU→VRAM e VRAM→VRAM); confirmar qual caminho de
escrita a suíte usa antes de tratar como o mesmo item.

### 10.23 — 88% do placar não mede nada

45 das 51 suítes saem como `tty`: executam sem crash e não emitem veredito, porque desenham na
VRAM. Isso inclui `gpu/triangle`, `gpu/quad`, `gpu/lines`, `gpu/rectangles`,
`gpu/uv-interpolation`, `gpu/transparency`, `gpu/clipping`, `gpu/texture-flip` — exatamente os
itens 2.3 a 2.6 do marco 2. O caminho existe: `tests/exes/ps1-tests/tools/diffvram` já está
baixado (é o binário de host que aparece no placar como `host-bin`). Sem comparação de VRAM, o
marco 2 inteiro está verde apenas contra testes escritos pelo próprio trabalhador.

## Bateria de mutação

Bateria de mutação: não se aplica — esta iteração não altera nenhum arquivo sob `crates/*/src/`;
ela lê um placar já produzido e escreve documento e itens de ROADMAP. Os defeitos que ela nomeia
ganham bateria quando forem corrigidos, cada um no seu item.

## Placar antes → depois

Workspace: **548** testes, inalterado. Scoreboard: reexecutado no `465192e` (antes: `c947ea9`,
13 merges atrás) — 5 com veredito, 1p/4f; era 5 com veredito, 1p/4f, com `otc-test` em 3p/35f.

## Revisão cruzada (orquestrador)

<!-- Este doc É o produto da revisão. -->

## Decisões e notas

1. **O placar não é rodado por ninguém automaticamente na `main`.** A CI tem job `scoreboard`,
   mas quem escreve o CSV é o script local, e ele ficou 13 merges atrás. Enquanto isso não mudar,
   todo número de hardware é histórico, não corrente.
2. **CORRIGIDO NA ITERAÇÃO 0072 — esta nota estava errada.** Eu escrevi aqui que, por `logs/`
   estar no `.gitignore`, o placar "nunca esteve no repositório". Está no `.gitignore`, e as
   tabelas acima realmente moram no corpo deste doc por isso — mas a conclusão era falsa: o job
   `scoreboard` da CI **publica** o placar numa branch órfã `scoreboard-data` a cada push na
   `main`. O que eu não tinha visto é pior do que a ausência: das 1982 linhas publicadas lá,
   **1981 têm status `sem-bios`**, porque a CI não tem BIOS e o script encerra sem rodar nada.
   Ver `docs/iterations/0072-correcao-registro.md`. O item 10.24 foi reescrito.
3. **`gpu_scoreboard.rs` é o exemplo mais limpo do projeto de teste que satisfaz o portão sem
   medir o objeto.** Seis testes verdes, todos sobre o texto de um `.ps1`. Nenhum é falso; todos
   juntos não excluem um emulador completamente quebrado. Vale citar no relatório final ao lado
   da bateria de mutação, que existe precisamente para essa classe de vazio — e que não foi
   aplicada aqui porque o alvo não estava sob `crates/*/src/`.
4. **A ordem de valor mudou.** O handoff atual manda seguir para 4.3b (mais CDROM). Com 34
   subtestes de hardware reprovando em um item já marcado `[x]`, e com o DPCR sem gate em três
   canais, a próxima tarefa deveria ser 10.19 — é a menor mudança com evidência de hardware mais
   direta. A decisão é do usuário; o handoff em `STATUS.md` está anotado com as duas opções.
5. Nenhum item foi desmarcado. `[x]` neste projeto significa "o item foi entregue e sua bateria
   fecha", não "bate com hardware"; reescrever o significado a posteriori apagaria o registro.
   Os itens 10.19-10.23 são o registro de que a diferença existe.
