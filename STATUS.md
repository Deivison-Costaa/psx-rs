# STATUS

> Memória do projeto entre iterações. O contexto do agente é descartado a cada iteração;
> este arquivo não. Teto de 16 KB imposto por `status_size.rs`. Curto e verdadeiro.

## Última iteração concluída

**0006** — psx-spx fatiado (ROADMAP 0.6): 15 capítulos em `docs/reference/NN-*.md`, commit
pinado `035c765`, índice de seções no topo de cada arquivo.

## Próxima tarefa

**ROADMAP 0.7** — `scripts/fetch-test-exes.ps1`: baixar EXEs de teste para `tests/exes/`
(gitignored) com procedência: releases do `JaCzekanski/ps1-tests` (zip com EXEs por suíte) e
Amidog `psxtest_cpu`/`psxtest_gte`. `scripts/scoreboard.ps1` esqueleto: varre `tests/exes/`,
gera CSV `suite,exe,status,ciclos` (por ora tudo `sem-runner` — o runner real chega no item
1.11). Executor: orquestrador.

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
