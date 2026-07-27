# STATUS

> Memória do projeto entre iterações. O contexto do agente é descartado a cada iteração;
> este arquivo não. Teto de 16 KB imposto por `status_size.rs`. Curto e verdadeiro.

## Última iteração concluída

**0003** — docs de gestão (ROADMAP 0.5): CLAUDE.md, STATUS, ROADMAP, TEMPLATE, mapa,
orquestracao, relatorio, review, README, BOOTSTRAP.

## Próxima tarefa

**ROADMAP 0.4** — CI (`.github/workflows/ci.yml`): job `check` (fmt → clippy `-D warnings` →
`cargo test --all`, sem condicionais nos passos) e job `commit-lint` (título do PR
`iter NNNN: ... (ROADMAP X.Y)`; prefixos `test|feat|fix|refactor|docs|chore(escopo):` nos
commits). Depois de verde na main, ativar proteção de branch (PR obrigatório, checks
`check`+`commit-lint`, enforce admins). Executor: orquestrador.

## Repositório

- `main` protegida a partir da iter 0004; 1 PR por item; merge commit (nunca squash);
  commits test→feat→docs; título de PR validado pela CI.
- Iterações são cronológicas e nem sempre na ordem dos itens (0003↔item 0.5, p.ex.);
  o vínculo real está no título do PR e no doc da iteração.

## Placar de testes

Workspace: 0 testes (esqueleto). EXEs de teste e scoreboard chegam nos itens 0.7 e 1.11;
ainda não existe emulador.

## Bloqueios

(nenhum)

## Invariantes

(nenhuma ainda — nascem com o código; índice com âncoras quando existirem)

## Notas

1. BIOS local: `bios/SCPH1001.BIN` (MD5 924E392ED05558FFDB115408C263DCCF), gitignored,
   validada na iter 0009 (item 0.9). Nunca commitar.
