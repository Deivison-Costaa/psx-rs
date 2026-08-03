<!-- Custo, tokens e duracao ficam em docs/metricas.csv, medidos pelo runner. -->

# 0177 — ci-sem-debuginfo

- **Data:** 2026-08-03
- **Item do roadmap:** 10.105, 10.106 e 10.107
- **Objetivo:** cortar o tempo de CI atacando o custo que a medição mostra, não o que eu tinha
  suposto.
- **Fonte:** orquestrador.

## Spec consultada

Nenhuma: mudança de infraestrutura, não de hardware emulado.

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a medição diz | Como foi pego |
|---|---|---|---|---|
| 1 | medição | Que o gargalo do CI fosse a **execução** da suíte — dez diagnósticos que emulam dezenas de milhões de passos, 189 s de 261 s. Foi com esse argumento que troquei `cargo test` por `cargo nextest` na 0171. | Não é assunto de spec. | Li o log do PR #189 passo a passo. O passo de suíte levou 202 s: `Starting 953 tests` aparece 188 s depois do início, e a execução inteira dura **14 s**. Os diagnósticos pesados terminam em 5 ms **no runner** porque `bios/` e `tests/exes/` são gitignored e não existem lá — eles só são caros na minha máquina. O nextest melhorou o local (449 s → 56 s) e não podia melhorar o CI, porque no CI nunca houve execução para paralelizar. |

## A mudança

`env` no topo do workflow, valendo para os três jobs:

    CARGO_PROFILE_DEV_DEBUG: "0"
    CARGO_PROFILE_TEST_DEBUG: "0"

São 145 binários de teste para linkar a cada rodada, e debuginfo completo domina o link. Fica no
workflow e **não** no `Cargo.toml` de propósito: nenhum backtrace do runner é lido por humano, mas
os diagnósticos locais dependem de linha e símbolo.

O guarda `ci_workflow.rs` ganhou uma asserção para as duas variáveis, no mesmo espírito da
asserção do nextest: sem ela, alguém remove o `env` e o CI só fica lento de novo, sem reprovar.

## 10.106 — o varredor de citações entrava em worktree alheio

Achado no meio desta rodada, por consequência dela. Cinco trabalhadores rodam em paralelo em
worktrees git dentro de `.claude/worktrees/agent-*`, cada um com o acervo inteiro de
`docs/iterations/`. `collect_md_files` pulava `target` e `.git` e mais nada, então varria os
cinco: **295 erros de citação numa árvore limpa**, todos de documentos que nem são desta árvore.

Não é cosmético. O portão fica vermelho por motivo falso exatamente quando há trabalho paralelo
acontecendo, que é quando ninguém tem paciência para investigar um vermelho. Agora `collect_md_files`
pula `worktrees` quando o pai é `.claude`, e há um teste que monta a estrutura num diretório
temporário e exige as duas coisas: achar o doc próprio, não achar o alheio.

## Bateria de mutação

Bateria de mutação: não se aplica — a rodada não toca `crates/*/src/`; muda um workflow de CI e
o escopo de varredura de um meta-teste, e ambos são guardados por asserção própria.

## Placar antes → depois

Workspace: **953 → 955** testes.

CI, job `check` — medido nos dois PRs, mesma máquina, mesmo cache:

| | PR #189 (antes) | PR #190 (depois) | |
|---|---|---|---|
| build + link de ~145 binários de teste | 188 s | **135 s** | −28% |
| execução da suíte | 14,0 s | 12,9 s | ruído |
| job `check` inteiro | 290 s | **239 s** | −18% |

Os 135 s que sobraram continuam sendo link. O candidato seguinte é trocar o linker por `mold`,
em rodada separada para que a medida continue atribuível.

**O que NÃO contar como ganho:** o terceiro empurrão deste mesmo PR mexeu só em documentação e o
job fechou em 65 s, com o passo de suíte em 20 s. Isso é o cache acertando porque nenhuma fonte
mudou — não é efeito do `debug=0`. A comparação honesta é 290 s contra 239 s, as duas com
mudança em `crates/psx-core/tests/` e o cache igualmente quente.

Um erro de processo a registrar: o primeiro empurrão usou `perf(ci):` como tipo de commit. O
`CLAUDE.md` lista `perf` entre os tipos, mas a expressão regular do `commit-lint` aceita só
`(test|feat|fix|refactor|docs|chore)`. Os dois estão em desacordo desde antes desta rodada; virou
o item **10.107**. Reescrevi como `chore(ci):`.

## Revisão cruzada (orquestrador)

Rodada de autoria do orquestrador, feita em paralelo com cinco lotes de trabalhador; a revisão
externa é o próprio CI, que mede o resultado. O risco declarado é baixo: se `debug=0` não render,
o número aparece na tabela acima e o item volta para o ROADMAP com o custo medido.

## Decisões e notas

Não mexi em linker (`mold`/`lld`) na mesma rodada, embora seja o candidato seguinte. Duas
mudanças de uma vez dariam um número só para dois efeitos, e o projeto inteiro depende de saber
qual mudança produziu qual medida.
