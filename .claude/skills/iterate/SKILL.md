---
name: iterate
description: Executa exatamente UMA iteração do psx-rs (um item do ROADMAP) sob TDD, bateria de mutação e PR padronizado. Fonte única do protocolo — vale para qualquer modelo (DeepSeek trabalhador ou outro).
---

# Protocolo de iteração (fonte única)

Regras de fundo em `CLAUDE.md` (R1–R8). Este arquivo é o ÚNICO lugar onde o protocolo mora;
se algo aqui conflitar com outro doc, este ganha e o outro deve ser corrigido.

## Passo 0 — Contexto mínimo

Leia `STATUS.md` inteiro (é curto). A seção **Próxima tarefa** define o item e as seções de
spec. Abra no `ROADMAP.md` SÓ a linha do item. Não leia mais nada "de graça" (R8) — nem
`tests/` inteiro, nem specs inteiras, nem iterações antigas.

## Passo 1 — Bloqueios

Se houver caixa `**BLOQUEADO por X**` no STATUS cujo X já fechou, reavalie-a antes de pegar
item novo (no gb-rs um item ficou pronto uma hora e três iterações passaram ao lado).

## Passo 2 — Branch

`git checkout main && git pull --ff-only && git checkout -b iter/NNNN-slug`
(NNNN = número da iteração, sequencial ao último doc em `docs/iterations/`; slug curto em
minúsculas). Árvore precisa estar limpa antes.

**Se a branch JÁ EXISTIR, a rodada é de continuação e este passo muda inteiro:**
`git checkout iter/NNNN-slug && git pull` — e mais nada. É **proibido** apagar, recriar,
resetar, rebasear ou fazer force-push da branch, e proibido abrir PR novo se já houver um.
Ela pode conter commits do orquestrador (correções, métricas, revisão) que não estão em
lugar nenhum; recriá-la os destrói. Se o `git checkout -b` falhar com "already exists", a
resposta é trocar para a branch, nunca removê-la.
Medido na iter 0038 rodada 3: o trabalhador recriou a branch a partir da `main`, jogou fora
quatro commits (dois deles do orquestrador) e reimplementou o item do zero, perdendo as
correções G1–G4 que já estavam revisadas.

## Passo 3 — Spec primeiro (R1)

Abra o arquivo de `docs/reference/` apontado pelo handoff, leia o **índice de seções** no
topo e pule DIRETO à(s) seção(ões) do item (offsets relativos à marca `CORPO:`). Se a spec
necessária não estiver em docs/reference, rode `scripts/fetch-reference-docs.ps1` com o
capítulo adicionado, commite (`chore(scripts)` + `docs(reference)`) e prossiga.

**Convenção de citação:** use `§ Título (L<n>)` — o título sobrevive à regeneração da spec,
o número não. Antes de commitar o doc da iteração, rode `scripts/confere-citacoes.ps1` para
validar que as citações de spec estão corretas (linha real, não offset do índice).

## Passo 4 — Teste que falha (R5)

Um arquivo de integração por item em `crates/psx-core/tests/`, nome `modulo_slug.rs`
espelhando o item (ex.: `cpu_load_delay.rs` ↔ 1.4). Valores esperados vêm DA SPEC (golden
values), nunca do output da implementação. Rode, veja falhar (vermelho por afirmação, não
por erro de compilação de API inexistente — crie os stubs mínimos que compilam).
Commit: `test(escopo): resumo`.

## Passo 5 — Implementação mínima

Só o necessário para o teste passar. Sem generalizar, sem "aproveitar que estou aqui" (R4).
Arquivo passando de 500 linhas: fatie e atualize `docs/mapa.md` no mesmo commit.
Commit: `feat(escopo): resumo` (ou `fix`/`refactor`).

## Passo 6 — Bateria de mutação (obrigatória)

Verde não prova que o teste mede: mutante que sobrevive é teste que não olha.

### 6.1 — Manifesto de mutação

Crie `docs/mutantes/NNNN-slug.mut` (formato em `docs/mutantes/README.md`) com ≥5 mutantes
e ≥2 controles. **É proibido mutar arquivo de teste** — a âncora deve apontar para
`crates/*/src/` (asserção F do meta-teste `mutation_manifest.rs` reprova).

Cada mutante declara o `@@DE` (linha(s) original(is)) e o `@@PARA` (linha(s) substituída(s))
com casamento por **linha inteira**. `ocorrencias: N` é contrato, dica: declarou 2 e achou
3 no fonte → erro duro, nada é mutado.

### 6.2 — Validação do manifesto

`cargo test --test mutation_manifest --test mutation_anchors` valida forma, unicidade de
pares (de,para), volume, âncoras reais, não-trivialidade, alvo em src/ e equivalência.
Rode ANTES de commitar o manifesto.

### 6.3 — Script de bateria (item 0.11)

O script `scripts/mutantes.ps1` que aplica cada mutante, roda o teste e registra o placar
**ainda não existe** — será implementado na iteração 0041 (item 0.11). Até lá, o placar
no doc da iteração é preenchido por inspeção (aplicar cada mutante manualmente, rodar o
teste, reverter).

Quando o script existir, ele lerá `docs/mutantes/NNNN-slug.mut` e produzirá o placar
canônico. O passo 6.3 será reescrito para `scripts/mutantes.ps1 NNNN`.

## Passo 7 — Verificação completa

`cargo fmt --all` → `cargo fmt --all -- --check` → `cargo clippy --all-targets -- -D warnings`
→ `cargo test --all`. Quando o runner existir (item 1.11+): `scripts/scoreboard.ps1` e anote
o placar. Nada de prosseguir com amarelo.

## Passo 8 — Documentar

Copie `docs/iterations/TEMPLATE.md` para `docs/iterations/NNNN-slug.md` e preencha TODOS os
campos. **Erros de primeira tentativa** é o campo mais importante do projeto: o que você
assumiu, o que a spec diz, como foi pego. Registrado não é vergonha — um log onde tudo sempre
deu certo é um log inútil. Atualize `STATUS.md`: última iteração (1 linha), **Próxima tarefa**
(handoff denso: item, arquivo de spec + seções, arquivos-alvo, armadilha conhecida), placar,
invariantes/notas novas (índice numerado, nunca renumere). Marque o checkbox do item no
`ROADMAP.md` com `(iter NNNN)` — a 0009 esqueceu e o orquestrador teve que fechar depois.
Se `logs/metrics-pending.csv` existir, mova suas linhas para o fim de `docs/metricas.csv`
(mesmo formato, sem cabeçalho) e apague o pendente — são as métricas das execuções
anteriores, appendadas pelo runner.

Rode `scripts/confere-citacoes.ps1` antes do commit `docs(iter)` para garantir que as
citações de spec no doc da iteração estão corretas.

## Passo 9 — PR (sem merge)

```
git add <docs> ; git commit -m 'docs(iter): NNNN — resumo'
git push -u origin iter/NNNN-slug
gh pr create --title 'iter NNNN: resumo (ROADMAP X.Y)' --body '<template preenchido>'
```

Armadilhas conhecidas do `gh` (todas já causaram defeito real):
- Título/corpo SEMPRE entre aspas simples — aspas duplas comem `$` e parênteses
  (commit permanente `CALL cc,u16 (    )` no gb-rs).
- Nunca `--fill`.
- O corpo segue `.github/PULL_REQUEST_TEMPLATE.md` (checkboxes + placar da bateria).
- `gh pr checks` sai com erro antes de os runs registrarem: espere em loop com retry.

Antes do push: `git status` tem que estar LIMPO — fmt rodado depois do commit deixa formatação
pendente fora do PR e derruba o check da CI (aconteceu na 0008b; o passo 7 vem antes do 9).

**NÃO faça merge.** O merge é do orquestrador, após revisão adversarial. Abriu o PR → passo 10.

## Passo 10 — Falhou 3× o mesmo passo?

Não insista. Registre em **Bloqueios** do STATUS (o que travou, o que foi tentado), marque o
PR como draft (`gh pr ready --undo`), e pare. Decisão sobe para o orquestrador/humano.

## Passo 11 — FIM

Uma iteração = um item = um PR aberto. Não comece o próximo item "já que está aqui" (R4).

**Este passo proíbe COMEÇAR item novo — ele não diz que um PR aberto significa item pronto.**
Se a rodada é de continuação (a branch já existe, o PR já está aberto e o `STATUS.md` descreve
o item como reprovado na revisão), o trabalho é TERMINAR este item: acrescente commits à
branch, `git push`, e pare. Não abra PR, não feche o existente, não recrie a branch.
Medido na iter 0038, rodadas 4 e 5: as duas leram o PR aberto, concluíram "já foi concluída" e
devolveram a rodada sem escrever uma linha — US$ 0,056 e 4 min de parede em nada. O texto que
as induziu vinha do envoltório do `oc-iter.ps1` ("ao abrir o PR, PARE"), hoje corrigido pelo
modo `-ContinueBranch`.
