# 0140 — roadmap-arquivo

- **Data:** 2026-08-01
- **Item do roadmap:** 10.61
- **Objetivo:** retirar todos os itens fechados da escada, inclusive os que pertencem a marcos ainda abertos, preservando-os em `docs/ROADMAP-fechado.md`.

## Revisão do PR anterior

A primeira tentativa já tinha movido os itens fechados existentes, mas deixou o próprio item
10.61 como `[ ]` no `ROADMAP.md` e não completou o passo documental. A revisão adversarial também
mostrou que o teste de marco 100% fechado não pegava um `[x]` esquecido dentro de um marco aberto.
Esta continuação acrescentou os testes, viu o vermelho por asserção, e só depois fechou e arquivou
10.61.

## Spec consultada

Nenhuma seção de spec de hardware. O item é organização do roadmap e regra de processo.

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que o comportamento real é | Como foi pego |
|---|---|---|---|---|
| 1 | processo | Mover os `[x]` que já existiam bastaria para concluir o item | O item corrente também precisa ser marcado como concluído e arquivado com `(iter 0140)` | O teste `item_10_61_da_iteracao_0140_foi_arquivado` falhou antes da correção |
| 2 | teste teatral | `marco_totalmente_fechado_nao_fica_no_roadmap` cobria toda a regra nova | Um marco aberto ainda podia carregar itens `[x]` sem falhar | A revisão adversarial; `roadmap_nao_contem_itens_fechados` tornou a propriedade explícita |

## Bateria de mutação

Bateria de mutação: não se aplica — não há código de produção no diff; esta iteração só move documentação e ajusta testes de processo.

## Placar antes → depois

Workspace: **868** → **870** testes (+2 em `roadmap_arquivo`). `ROADMAP.md` fica abaixo do novo teto
de 7 KB; os itens fechados continuam preservados, sem teto, em `docs/ROADMAP-fechado.md`.

## Revisão cruzada (orquestrador)

Revisão adversarial retomada na mesma branch. O caso que antes passava sem concluir o item ficou
vermelho por asserção; após a movimentação de 10.61, os seis testes de `roadmap_arquivo` e o teste
de tamanho passaram.

## Decisões e notas

- `ROADMAP.md` agora contém somente itens abertos; `10.61` foi agrupado sob `## M10` no arquivo histórico.
- A linha do item foi preservada verbatim, com o marcador de iteração acrescentado no fechamento.
- Nenhum item aberto ou cabeçalho de marco foi removido.
