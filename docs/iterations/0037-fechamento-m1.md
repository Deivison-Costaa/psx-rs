<!-- Custo, tokens e duração NÃO entram aqui: são medidos por scripts/oc-iter.ps1 em
     docs/metricas.csv, casados por head_antes/head_depois. -->

# 0037 — fechamento-m1

- **Data:** 2026-07-28
- **Item do roadmap:** 11.1 parcial (relatório incremental) — obrigação de fechamento de marco
- **Objetivo:** fechar o M1 conforme o protocolo manda: consolidar o relatório com os números
  reais e transformar em item de ROADMAP os achados que estavam vivendo só dentro de docs de
  iteração.

## Spec consultada

Nenhuma: iteração de processo.

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que era verdade | Como foi pego |
|---|---|---|---|---|
| 1 | número | Que os 89 erros que eu tinha contado antes de despachar o 1.13 continuavam valendo | São **92** — as iterações 0036 e 0035 acrescentaram três enquanto eu levantava o resto | Recontei antes de publicar, e a soma das categorias (92) não batia com o total que eu tinha escrito (89) |
| 2 | número | Que "50 PRs mergeados" (o número do último PR) fosse a contagem | São **49** merges: a numeração do GitHub conta PRs abertos, não mergeados | `git log main --merges --oneline \| wc -l` |
| 3 | aritmética | Que "outros" na tabela de categorias fosse 27 | 92 menos as nove categorias listadas dá **30** | Somei as parcelas em vez de confiar na conta anterior |

Os três erros são da mesma família e valem mais registrados juntos: **eu publiquei número que
não vinha de medição feita naquele momento.** É o defeito que a revisão vinha cobrando do
trabalhador desde a 0027, cometido pelo revisor no documento cuja função é ser o registro
empírico do projeto. Números do relatório passam a ser recontados no ato da publicação.

## Bateria de mutação

Não se aplica: sem mudança em `crates/`.

## Placar antes → depois

274 → 274 testes (inalterado).

## Revisão cruzada (orquestrador)

Iteração do próprio orquestrador.

## Decisões e notas

1. **O relatório saiu de "(a consolidar)" para números medidos.** As seções 3 e 4 estavam com
   placeholders desde o fechamento do M0. Agora trazem: 59 execuções, US$ 1,87, 20,3% de
   retrabalho, 92 erros de primeira tentativa por categoria, e o placar de EXEs com veredito.
   O comparativo com o gb-rs ficou com uma linha que o piloto não tinha como produzir — *taxa
   de retrabalho medida* —, porque lá a revisão adversarial nunca rodou.

2. **Seção 5 nova: padrões de falha medidos.** É o achado central do experimento até aqui, e
   contraria a expectativa com que o projeto começou: **nenhuma das falhas caras foi de
   emulação.** As três que se repetiram são afirmação sem execução, fechar-um-caminho-abre-o-
   vizinho, e placar que mede o fácil em vez do que importa. As três têm ocorrências datadas e
   contramedida registrada.

3. **Três achados viraram itens 10.3–10.5** em vez de continuarem como nota dentro de doc de
   iteração: o bus error que não levantamos ao executar código do scratchpad (exposto pelo
   `code-in-io` com `given: 0x0, expected: 0x1`), o `CAUSE.CE` não preenchido nas exceções de
   Coprocessor Unusable (`02-cpu.md` L681), e o Amidog que para depois de `args: 0` por causa
   não investigada. Achado que não vira item se perde — e os dois primeiros já estavam
   registrados havia quatro iterações sem endereço no ROADMAP.

4. **A premissa do marco foi reconferida, e a cadência ficou como está.** O M0 fechou decidindo
   revisão adversarial *em lote por marco*; o usuário reverteu para revisão *por PR* em 27/07,
   e os números do M1 confirmam a reversão: 12 rodadas de correção em 59 execuções, todas
   disparadas por achado de revisão. Em lote, esses defeitos chegariam juntos no fim do marco.

5. **Duas das dezessete iterações do M1 consertaram o ferramental, não o emulador** (0030 e a
   parte de CI da 0031). Isso não estava previsto no ROADMAP e é dado sobre o método: um
   pipeline de agentes tem custo de manutenção próprio, e no M1 ele foi de ~12% das iterações.
