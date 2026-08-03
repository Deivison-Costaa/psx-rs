<!-- Custo, tokens e duracao ficam em docs/metricas.csv, medidos pelo runner. -->

# 0181 — achados-separados

- **Data:** 2026-08-03
- **Item do roadmap:** 0181.1 (o próprio processo).
- **Objetivo:** o `ROADMAP.md` não sobreviveu ao trabalho paralelo. Consertar o artefato, não
  continuar contornando-o.
- **Fonte:** orquestrador, a pedido do usuário.

## Spec consultada

Nenhuma: mudança de processo, não de hardware emulado.

## O que estava errado

Na noite de 02-03/08 rodaram cinco lotes em paralelo mais três rodadas do orquestrador. O
`ROADMAP.md` cobrou três pedágios, todos por causa da mesma confusão de papéis:

1. **Colisão de numeração.** Dois lotes escolheram `10.108` e dois escolheram `10.102`, cada um
   olhando a lista antes de o outro mergear. Renumerei quatro itens à mão no merge.
2. **Teto de 7000 bytes estourado em cinco merges seguidos.** A cada um comprimi linhas de itens
   **que não tinham nada a ver com a rodada**, para caber. Isso é perda de informação diagnóstica
   como pedágio de merge — o oposto do que o teto existe para proteger.
3. **Conflito garantido.** Todos os oito ramos editavam as mesmas linhas do mesmo arquivo.

A causa: o `ROADMAP.md` fazia dois trabalhos. A **escada** (M4-M11, "o que construir") ocupava
1000 bytes. O **backlog de achados** (M10, defeitos descobertos por medição) ocupava **5188 de
6812 bytes — 76% do arquivo**. O teto de R8 protege a escada, que é o que se lê a cada rodada;
os achados são um depósito que cresce com a medição e não deveria disputar aqueles bytes.

## As mudanças

**Separação.** O M10 virou `docs/achados.md`. `ROADMAP.md` caiu para **1778 bytes** e o teto dele
apertou de 7000 para **3000** — se a escada voltar a estourar, é sinal de achado disfarçado de
degrau. Achados têm teto próprio de 24 KB.

**Numeração `NNNN.k`, pelo número da iteração que achou.** Colisão vira impossível por
construção: duas rodadas paralelas nunca têm o mesmo `NNNN`. O acervo `10.x` fica como está,
numa seção "Legado" — renumerar 90 itens quebraria toda citação existente em docs e manifestos.

**Convenção de append.** Achado novo entra no FIM da seção. Append não conflita; inserção no meio
conflita.

**Guardas.** `achados_arquivo.rs` com cinco asserções, e as antigas ensinadas sobre o arquivo
novo: `status_handoff` aceita item de achado na "Próxima tarefa", e a checagem de duplicata do
`roadmap_arquivo` passou a cobrir os três arquivos.

**Ferramentas.** `scripts/oc-loop.ps1` procurava o checkbox só no `ROADMAP.md` — a auto-remediação
teria ficado silenciosamente inerte para todo defeito achado por medição, que é a maioria das
rodadas. Agora varre os dois. `SKILL.md`, `CLAUDE.md` e a tarefa-modelo dos lotes atualizados.

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a medição diz | Como foi pego |
|---|---|---|---|---|
| 1 | processo | Que comprimir o ROADMAP a cada merge fosse manutenção normal. | Não é assunto de spec. | Cinco merges seguidos, cada um encurtando linhas de itens alheios. Estava pagando com informação um problema de estrutura, e a estrutura era o que precisava mudar. |
| 2 | teste | Que estes guardas estivessem verificados por passarem. | Não é assunto de spec. | Passaram na primeira execução — o que não prova nada. Violei cada propriedade de propósito (id duplicado, item fechado no arquivo de abertos, teto estourado, ponteiro removido) e conferi que os quatro reprovam. Sem isso, seriam guardas decorativos. |

## Bateria de mutação

Bateria de mutação: não se aplica — a rodada não toca `crates/*/src/`; mexe em documentos de
processo, meta-testes e um script, e cada guarda novo foi verificado por violação deliberada.

## Placar antes → depois

Workspace: **1019 → 1024** testes.

| | antes | depois |
|---|---|---|
| `ROADMAP.md` | 6812 bytes (76% deles achados) | **1778 bytes** |
| Teto da escada | 7000 | **3000** |
| Colisão de número entre lotes paralelos | 4 renumerações manuais numa noite | impossível por construção |

## Revisão cruzada (orquestrador)

Rodada de autoria do orquestrador. A verificação que sustenta os guardas não é a suíte verde, é
a tabela de violação deliberada acima.

## Decisões e notas

**Não renumerei o acervo.** Os 90 itens `10.x` são citados em docs de iteração, manifestos de
mutação e mensagens de commit. Renumerar quebraria tudo isso para ganhar uniformidade estética.
O corte fica visível na seção "Legado", que é honesto sobre o projeto ter mudado de esquema no
meio.
