# STATUS

> Memória do projeto entre iterações. O contexto do agente é descartado a cada iteração;
> este arquivo não. Teto de 16 KB imposto por `status_size.rs`. Curto e verdadeiro.

## Última iteração concluída

**0007** — EXEs de teste (ROADMAP 0.7): 51 EXEs em `tests/exes/` (ps1-tests build-158 +
Amidog cpu/gte), scoreboard esqueleto 0/51 `sem-runner`.

## Próxima tarefa

**ROADMAP 0.8** — orquestração: `.claude/skills/iterate/SKILL.md` (protocolo de 12 passos,
FONTE ÚNICA — bateria de mutação DENTRO do skill), `scripts/oc-iter.ps1` (dispara
`opencode run -m deepseek/deepseek-chat` via `opencode serve`+`--attach` por causa do bug de
sessão headless no Windows, issue opencode#28407; extrai custo/tokens/steps do JSON e appenda
`docs/metricas.csv`), `scripts/oc-loop.ps1 -N` com guardas (árvore limpa, roadmap não vazio).
Smoke test: 1 task trivial de ponta a ponta (branch→commits→PR→checks→merge→métrica).
Executor: orquestrador.

## Repositório

- `main` protegida a partir da iter 0004; 1 PR por item; merge commit (nunca squash);
  commits test→feat→docs; título de PR validado pela CI.
- Iterações são cronológicas e nem sempre na ordem dos itens (0003↔item 0.5, p.ex.);
  o vínculo real está no título do PR e no doc da iteração.

## Placar de testes

Workspace: 8 testes (todos meta-testes de processo). EXEs de teste e scoreboard chegam nos
itens 0.7 e 1.11; ainda não existe emulador.

## Bloqueios

(nenhum)

## Invariantes

(nenhuma ainda — nascem com o código; índice com âncoras quando existirem)

## Notas

1. BIOS local: `bios/SCPH1001.BIN` (MD5 924E392ED05558FFDB115408C263DCCF), gitignored,
   validada na iter 0009 (item 0.9). Nunca commitar.
