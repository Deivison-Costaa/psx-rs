# 0003 — docs de gestão

- **Data:** 2026-07-27
- **Item do roadmap:** 0.5
- **Objetivo:** arquitetura de documentos com fonte única e ponteiros.

## O que foi feito

CLAUDE.md (regras R1–R8 + mapa de ponteiros, sem protocolo — protocolo terá fonte única no
SKILL, item 0.8), STATUS.md, ROADMAP.md (só checkboxes, teto por teste no item 0.3),
TEMPLATE.md, mapa.md, orquestracao.md (papéis + diagnóstico do gb-rs), relatorio.md
(incremental), prompts/review.md, README.md, BOOTSTRAP.md.

## Decisões

- Ordem de execução do M0 ajustada: 0.5 → 0.4 → 0.3 (meta-testes exigem docs e CI
  existentes). Iteração é cronológica; item é temático — vínculo no título do PR.
- Categorias de erro de primeira tentativa adaptadas ao PS1: `delay-slot`, `saturação-gte`
  no lugar das específicas do SM83; enum registrado no TEMPLATE (no gb-rs categorias ad hoc
  derivaram do vocabulário e sujaram a agregação).

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que era | Como foi pego |
|---|---|---|---|---|
| — | nenhum | | | |
