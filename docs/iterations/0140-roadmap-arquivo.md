# 0140 — roadmap-arquivo

- **Data:** 2026-08-01
- **Item do roadmap:** 10.61
- **Objetivo:** retirar todos os itens fechados da escada, inclusive os que pertencem a marcos ainda abertos, preservando-os em `docs/ROADMAP-fechado.md`.

## Revisão do PR anterior

A primeira tentativa já tinha movido os itens fechados existentes, mas deixou o próprio item
10.61 como `[ ]` no `ROADMAP.md` e não completou o passo documental. Esta continuação acrescentou
os testes, viu o vermelho por asserção, e só depois fechou e arquivou 10.61.

> **Correção do orquestrador.** A versão original deste parágrafo afirmava que "a revisão
> adversarial mostrou que o teste não pegava um `[x]` esquecido". **Essa revisão não aconteceu**: a
> primeira rodada não foi reprovada, foi **morta por travamento** (ver nota 2). O trabalhador
> reproduziu fielmente o texto do envoltório do `-ContinueBranch`, que afirma tratar-se de "um item
> que JA foi reprovado na revisao adversarial" — a flag foi usada fora do caso para o qual existe.
> A observação sobre o teste é do próprio trabalhador, e é boa; a atribuição é que estava errada.

## Spec consultada

Nenhuma seção de spec de hardware. O item é organização do roadmap e regra de processo.

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que o comportamento real é | Como foi pego |
|---|---|---|---|---|
| 1 | processo | Mover os `[x]` que já existiam bastaria para concluir o item | O item corrente também precisa ser marcado como concluído e arquivado com `(iter 0140)` | O teste `item_10_61_da_iteracao_0140_foi_arquivado` falhou antes da correção |
| 2 | teste teatral | `marco_totalmente_fechado_nao_fica_no_roadmap` cobria toda a regra nova | Um marco aberto ainda podia carregar itens `[x]` sem falhar | Percebido pelo próprio trabalhador na rodada 2 (não por revisão externa — ver correção acima); `roadmap_nao_contem_itens_fechados` tornou a propriedade explícita |

## Bateria de mutação

Bateria de mutação: não se aplica — não há código de produção no diff; esta iteração só move documentação e ajusta testes de processo.

## Placar antes → depois

Workspace: **868** → **870** testes (+2 em `roadmap_arquivo`). `ROADMAP.md` fica abaixo do novo teto
de 7 KB; os itens fechados continuam preservados, sem teto, em `docs/ROADMAP-fechado.md`.

## Revisão cruzada (orquestrador)

Feita por Claude sobre o diff completo. O parágrafo que antes ocupava esta seção fora escrito pelo
próprio trabalhador — o template reserva esta seção ao orquestrador, e auto-revisão não a cumpre.

**Substância aprovada.** Conservação verificada por contagem independente: 188 itens antes e 188
depois, nenhum ID sumiu nem apareceu, e as 66 linhas de item aberto são **byte a byte idênticas**.
Foi movimentação, não poda. A sequência `test → feat` foi respeitada. O trabalhador **melhorou o
desenho pedido**: além de baixar o teto para 7 KB, escreveu `roadmap_nao_contem_itens_fechados`, que
declara a regra explicitamente em vez de deixá-la implícita no limite de bytes — e esse teste passou
a matar também m1 e m2 da bateria 0100, fortalecendo cobertura alheia.

**Achado 1 (alta) — manifesto vivo arquivado. CORRIGIDO pelo orquestrador.** O commit `9de728b` pôs
`arquivada:` no `0100-roadmap-arquivo.mut` inteiro, alegando mudança estrutural do alvo. Medi os 7
registros contra o `ROADMAP.md` novo: **5 ainda casavam** (m1, m2, m4, m5, c1). Só m3 e c2 haviam
quebrado, ambos ancorados no texto do ponteiro que esta iteração reescreveu. Arquivar destruía 5
registros vivos para contornar 4 linhas de reparo. É a reencenação do item **10.18** ("nada torna
`arquivada:` caro — 0052 e 0059 descartaram 17 registros, 12 ainda casavam"), com precedente de
conserto no item 10.15 (iter 0067). Reparei as duas âncoras para o texto novo do ponteiro e removi o
`arquivada:`; a bateria 0100 voltou a rodar inteira: **5/5 mutantes mortos, 2/2 controles verdes**.

**Achado 2 (média) — proveniência falsa, causa no ferramental do orquestrador.** Ver a correção na
seção "Revisão do PR anterior". Não é falha do trabalhador.

**Achado 3 (baixa) — aceito como dívida, não corrigido.** `item_10_61_da_iteracao_0140_foi_arquivado`
afirma uma string exata de um item específico e viverá na suíte para sempre, sem poder falhar de
forma útil de novo; a regra geral já cobre o caso. Mantido: remover teste alheio por estilo é
intervenção pesada demais, e ele documenta o requisito "verbatim + marcador de iteração".

## Decisões e notas

- `ROADMAP.md` agora contém somente itens abertos; `10.61` foi agrupado sob `## M10` no arquivo histórico.
- A linha do item foi preservada verbatim, com o marcador de iteração acrescentado no fechamento.
- Nenhum item aberto ou cabeçalho de marco foi removido.

- **Nota 2 — as duas rodadas morreram por travamento, e a causa é infraestrutura, não o modelo.**
  Rodada 1 (`falha:travamento`, US$ 0,0313, 27 steps, 10 min): último comando foi o portão do passo
  7, `cargo fmt ... && cargo test --all`. A suíte leva **842 s** nesta máquina e o detector mata a
  rodada após 5 min sem evento — o trabalhador ficou calado porque estava obedecendo o protocolo.
  O `$TravamentoMin = 5` foi calibrado quando os testes de BIOS e disco **pulavam por ausência de
  arquivo**; com a BIOS e o Crash presentes desde a 0139 eles executam de verdade e a suíte
  triplicou. O parâmetro ficou obsoleto no instante em que os arquivos chegaram.
  Rodada 2 (`falha:travamento`, US$ 0,1047, 60 steps, 52 min), já com `-TravamentoMin 25`: rodou
  `cargo test --all` **três vezes** (~42 min dos 52). Morreu no terceiro.
  O PR foi aberto pelo orquestrador: o trabalho estava completo em 7 commits, incluindo o
  `docs(iter)`, e faltava só o `gh pr create` — uma terceira rodada custaria ~50 min e levaria ao
  limite de 3 falhas no mesmo passo.

- **Dívidas abertas por esta iteração** (não consertadas aqui, R4):
  1. `$TravamentoMin = 5` é incompatível com a suíte atual; mudar o default esbarra na âncora viva
     `[int]$TravamentoMin = 5,` de `0098-oc-iter-travamento.mut`, que precisa de reparo junto.
  2. O envoltório do `-ContinueBranch` afirma "JA foi reprovado na revisao adversarial" mesmo quando
     a rodada anterior morreu por travamento, e o trabalhador escreve isso no registro permanente.
  3. `nextest` roda a mesma suíte em **449 s** contra 842 s do `cargo test` (1,87×), mas 447 s dos
     449 são **um único teste** (`evcb_descritor_mapeia_para_spec_correto`): o piso é aquele teste,
     não o runner.
