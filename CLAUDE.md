# psx-rs — emulador de PlayStation 1 em Rust

Projeto final da cadeira de Programação com Agentes. Objetivo duplo, com o mesmo peso:

1. Um emulador de PS1 correto e completo, com app desktop (biblioteca, saves, controles).
2. Um **registro empírico** de como o trabalho foi conduzido por agentes: métricas, erros de
   primeira tentativa, decisões e falhas. Se "avançar rápido" conflitar com "registrar
   direito", **registrar ganha**.

Quem escreve código de emulação é o trabalhador (opencode/DeepSeek). Quem orquestra, revisa
adversarialmente e faz merge é o orquestrador (Claude). Papéis em `docs/orquestracao.md`.

## Regras invioláveis

- **R1 — Nunca implemente hardware de memória.** O R3000A tem load/branch delay slots, o GTE
  satura em limites não óbvios, o rasterizador da GPU tem regras próprias de preenchimento.
  Sua intuição de MIPS/3D **não é confiável aqui**. Leia a seção da spec em `docs/reference/`
  ANTES de implementar; se faltar, baixe do psx-spx (`scripts/fetch-reference-docs.ps1`),
  commite, e só então implemente. Erro descoberto depois não é vergonha — é o dado mais
  valioso do projeto: registre no doc da iteração.
- **R2 — Scheduler de eventos desde o dia 1.** Componentes avançam por timestamps no
  `scheduler`; a CPU é instruction-stepped com contagem de ciclos. Nada de "cada componente
  se atualiza sozinho quando lembrado".
- **R3 — `psx-core` puro:** sem I/O, sem dependências fora da allowlist de `purity.rs`.
- **R4 — Uma micro-funcionalidade por iteração, e PARE.**
- **R5 — Teste antes de implementar.** Sem exceção.
- **R6 — Sem `unsafe`/`unwrap()` fora de teste;** `#![forbid(unsafe_code)]`; clippy `-D warnings`.
- **R7 — Comentários ≤5% por arquivo `.rs` (reprova em 10%).** Justificativa, narrativa e
  citação de spec moram em `docs/iterations/`, não no código.
- **R8 — Orçamento de contexto é recurso do projeto.** Leia SÓ: `STATUS.md` → a linha do item
  no `ROADMAP.md` ou em `docs/achados.md` → a(s) seção(ões) da spec apontadas pelo handoff → o(s) arquivo(s)-alvo
  achados via `docs/mapa.md` → o teste do item atual. NUNCA leia `tests/` inteiro, specs
  inteiras nem arquivos fora do item. **Arquivo de TESTE com >500 linhas reprova em
  `file_size.rs`**: um arquivo de teste por item do ROADMAP, não um por subsistema.
  Arquivo fonte pode ser grande se for coeso — cortar módulo por contagem de linha é pior
  que um arquivo longo, e não há teto para `src/`.

## Mapa de ponteiros (abra só o que o passo pedir)

| Preciso de | Arquivo |
|---|---|
| Estado atual + próxima tarefa | `STATUS.md` |
| O que fazer e como (protocolo, 12 passos) | `.claude/skills/iterate/SKILL.md` |
| Escada de itens (o que construir) | `ROADMAP.md` |
| Defeitos achados por medição (o que consertar) | `docs/achados.md` |
| Onde mora cada módulo/arquivo | `docs/mapa.md` |
| Spec de hardware (com índice de seções no topo) | `docs/reference/NN-*.md` |
| Template do doc de iteração | `docs/iterations/TEMPLATE.md` |
| Prompt de revisão adversarial | `docs/prompts/review.md` |
| Como o processo é conduzido + diário | `docs/orquestracao.md` |
| Métricas por execução | `docs/metricas.csv` |
| Rascunho do relatório final | `docs/relatorio.md` |

## Commits e PRs

- Branch `iter/NNNN-slug`; PR título `iter NNNN: resumo (ROADMAP X.Y)` — validado pela CI.
- Commits separados por papel: `test(escopo):` → `feat|fix|refactor(escopo):` → `docs(iter):`.
  Merge sempre por merge commit (nunca squash) para preservá-los na main.
- Identificadores em inglês; docs, commits e mensagens de teste em português.
