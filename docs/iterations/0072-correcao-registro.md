# 0072 — correcao-registro

- **Data:** 2026-07-29
- **Item do roadmap:** 10.24 (medição; o conserto fica pendente no próprio item)
- **Objetivo:** corrigir uma afirmação errada da iteração 0068, registrar o que a apuração
  encontrou no lugar dela, e preencher a revisão cruzada da 0071.

## Spec consultada

Nenhuma — item de infraestrutura de CI.

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a apuração diz | Como foi pego |
|---|---|---|---|---|
| 1 | processo | Que `logs/` estar no `.gitignore` bastasse para concluir que o placar "não está no repositório" (nota 2 da 0068) | O job `scoreboard` da CI publica o placar numa branch órfã `scoreboard-data` a cada push na `main`. A afirmação foi publicada num PR mergeado antes de eu conferir o remoto | Ao relançar o loop, o `git fetch` imprimiu `9f56efc..3179fe9 scoreboard-data -> origin/scoreboard-data`. Eu não tinha listado as branches remotas antes de concluir |
| 2 | processo | Que descobrir a branch invalidasse o achado | Invalida a **razão**, não o achado, e o substitui por um pior: das 1982 linhas de `scoreboard-data.csv`, **1981 têm status `sem-bios`** | `git show origin/scoreboard-data:scoreboard-data.csv` e contagem por status |
| 4 | processo | Que `spec_citations.rs` aceitaria a citação certa da seção do CHCR em `04-dma.md` | Ele reprovou `(L84)` — a linha REAL — dizendo "L84 é o offset do ÍNDICE" e no mesmo diagnóstico calculando que a seção começa em L84. `in_index_range` era testado ANTES de `in_real`, e em `04-dma.md`, onde o offset do `CORPO:` é só +25 e as seções distam ~37 linhas, a linha real cai dentro da faixa de índice da própria seção | O portão reprovou uma citação que eu havia conferido à mão contra o arquivo. Consertado neste PR: `if !in_real && in_index_range` |
| 3 | processo | Que matar o `oc-loop`, o `oc-iter` e o `opencode.exe` da rodada parasse o trabalhador. Vi sobrar só o daemon `opencode serve` e concluí que estava parado | O agente roda **dentro** do daemon; o `oc-iter.ps1` é só um cliente HTTP. A sessão server-side continuou, deu `git stash -u` na minha árvore, voltou para `main`, resetou e criou `iter/0071-dma-dpcr-gate` | O commit desta iteração falhou com "nothing to commit, working tree clean" **na branch errada**. O reflog mostrou o `stash` e o `checkout` que eu não tinha feito. Nada foi perdido: as edições estavam em `stash@{0}` |

## O que está acontecendo

`.github/workflows/ci.yml`, job `scoreboard`: baixa os EXEs, compila o `psx-cli`, roda
`pwsh scripts/scoreboard.ps1` e, quando o evento é push na `main`, acumula o CSV na branch órfã
`scoreboard-data`.

O que o script faz sem BIOS (`scripts/scoreboard.ps1`, `$haveBios = Test-Path $BiosPath`): para
cada arquivo encontrado, escreve uma linha com status `sem-bios` e segue. **Não executa suíte
nenhuma, e encerra com código 0.** O job fica verde.

A BIOS não está — e não pode estar — no repositório: é proprietária, e a nota 1 do `STATUS.md`
diz explicitamente "gitignored, nunca commitar". Então **o defeito não é a CI não ter BIOS.**
O defeito é o job reportar sucesso e publicar 51 linhas por execução aparentando medição, quando
o número de suítes executadas é zero.

Consequências concretas:

1. O check `scoreboard` esteve verde em **todos** os PRs do projeto, inclusive nos 28 de hoje, sem
   nunca ter rodado uma suíte. Ele parecia a validação contra hardware no pipeline.
2. Os itens 1.12 ("CI: job scoreboard ligado") e 1.13 ("Veredito real no scoreboard") estão
   marcados `[x]` e ambos são verdade no que prometem: o job existe e o extrator de `pass -`/
   `fail -` funciona — provado localmente na iteração 0068. O que nunca aconteceu foi a CI
   exercitar esse extrator uma única vez.
3. `crates/psx-core/tests/ci_scoreboard.rs` tem 9 testes e nenhum deles afirma que o job mede
   algo; um afirma justamente que `sem-bios` é um rótulo válido a ser preservado. Virou o item
   10.26.
4. O único placar com veredito real é o `logs/scoreboard.csv` local, que é gitignored. Isso é o
   item 10.27, e ele deixa de existir se o 10.24 for resolvido pela via da BIOS em secret.

## Bateria de mutação

Bateria de mutação: não se aplica — esta iteração não altera nenhum arquivo sob `crates/*/src/`
nem sob `scripts/`; ela corrige um documento e reescreve itens de ROADMAP a partir de uma apuração
que se pode repetir com `git show origin/scoreboard-data:scoreboard-data.csv`.

## Placar antes → depois

Workspace: **556** testes, inalterado por esta iteração (os +7 são da 0071).
Suítes efetivamente executadas pela CI, por rodada: **0** antes e depois — o que muda é que agora
está escrito.

Placar de hardware, que esta iteração mediu ao revisar a 0071: `ps1-tests/dma/otc-test` foi de
`6p/34f` para **`7p/30f`** no `8d4267e`, com `testOtcStandardWithMasterDisabled` passando. É a
primeira vez no projeto que uma correção move uma suíte de hardware, e move exatamente os quatro
subtestes que a iteração 0068 havia previsto.

## Revisão cruzada (orquestrador)

<!-- Preenchido na revisão do PR. -->

## Decisões e notas

1. **Tentei parar o loop para escrever isto, e o loop não parou.** O motivo de tentar é a regra do
   `CLAUDE.md`: se avançar rápido conflitar com registrar direito, registrar ganha. Deixar de pé
   uma afirmação falsa num doc mergeado custaria mais que uma rodada de trabalhador, e pior — a
   correção óbvia que alguém tiraria da nota errada seria versionar o CSV, que não resolve nada.
   O que aconteceu está na linha 3 da tabela de erros: matar o cliente não mata o agente. O
   resultado foi bom por acidente — a rodada órfã fez o item 10.19, que era o item certo, e virou
   o PR #85. Mas foi acidente, e a lição operacional é que **conferir o CPU do daemon
   `opencode serve` é obrigatório antes de encostar no repositório.**
2. **Não abri PR competindo com o loop.** Um PR meu aberto enquanto o loop corre é exatamente o
   que matou o loop hoje pela manhã: o `oc-loop` mergeia o PR aberto mais antigo, e o #64 órfão
   sequestrou a vez da rodada. Esperei o #85 mergear antes de abrir este.
3. **Três portões verdes que não mediam nada, em um dia.** `gpu_scoreboard.rs` (afirma
   `contains` sobre um `.ps1`), `bateria_placar_bate_com_resultado` (pulava os 25 manifestos por
   causa do separador) e agora o job `scoreboard` (51 linhas de `sem-bios`, exit 0). Os três
   passaram por revisão adversarial quando entraram, e os três eram falsificáveis em minutos por
   quem tivesse ido olhar o resultado em vez do código. É o achado mais forte do dia para o
   relatório final, e vale mais que qualquer um dos 60 itens verdes.
4. O item 10.24 continua aberto de propósito. Escolher entre BIOS em secret e um status explícito
   de NÃO MEDIDO é decisão com consequência (a primeira coloca material proprietário num secret
   do repositório), e não é decisão de uma iteração de correção de doc.
5. **A revisão cruzada da 0071 está no doc dela**, não aqui: três achados (tabela de mutação
   divergindo do `.resultado` em 3 de 9 linhas, um `assert_ne!` numa correção de defeito, e
   habilitar o DPCR não disparar transferência pendente), que viraram os itens 10.28, 10.29 e
   10.30. Nenhum bloqueia o gate, que está provado por seis dos sete testes novos.
6. **O `ROADMAP.md` fechou esta iteração com 9978 bytes de um teto de 10000.** Para caber os três
   itens novos, encurtei quatro itens cujo detalhe já mora em doc de iteração. O teto está fazendo
   o trabalho dele — obrigar a escada a ser escada —, mas a seção 10 passou de vinte itens e o
   próximo achado não cabe. Não abri item para isso porque não havia espaço, o que é a
   demonstração do problema; a saída provável é a seção 10 virar arquivo próprio.
7. **O portão de citações estava reprovando citação correta**, e isso é pior que o defeito 10.16:
   a 10.16 dá diagnóstico errado, esta forçava escrever a citação **errada** para ficar verde. A
   ordem dos testes foi invertida para `if !in_real && in_index_range`, de modo que "a linha cai
   dentro da seção real" ganha de "a linha cai dentro da faixa de índice". Falsifiquei o conserto
   como a regra do TaskFile agora exige: com a citação trocada para `(L59)`, que é de fato o valor
   do índice, o portão continua acusando "L59 é o offset do ÍNDICE"; com `(L84)`, passa.
8. **Falsifiquei o portão antes de commitar e quase paguei por isso.** Rodei o `sed` de
   falsificação sobre o `STATUS.md` com as edições desta iteração ainda não commitadas, e desfiz
   com `git checkout -- STATUS.md`. O arquivo estava em estado `UU` (merge não resolvido, herdado
   do `git stash apply`), e nesse estado o `checkout` não reverteu nada — só por isso o trabalho
   sobreviveu. A regra "restaure com `git checkout --`" pressupõe que o que você quer de volta
   está commitado. **Falsificar portão é para depois do commit, ou sobre cópia.**
