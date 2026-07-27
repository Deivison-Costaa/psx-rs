# STATUS

> Memória do projeto entre iterações. O contexto do agente é descartado a cada iteração;
> este arquivo não. Teto de 16 KB imposto por `status_size.rs`. Curto e verdadeiro.

## Última iteração concluída

**0005** — meta-testes de processo (ROADMAP 0.3): 8 testes guardando R3/R6/R7/R8, tetos de
STATUS/ROADMAP, CI sem condicional e frescor de métricas. Bateria 7/7.

## Próxima tarefa

**ROADMAP 0.6** — `scripts/fetch-reference-docs.ps1`: baixar capítulos do psx-spx
(repo `psx-spx/psx-spx.github.io`, commit pinado) para `docs/reference/NN-slug.md` com
cabeçalho de índice de seções (título § → âncora) gerado pelo script, e `docs/reference/README.md`
com procedência (SHA, data, licença). Capítulos: memory map, CPU, GPU, DMA, timers, CDROM,
GTE, SPU, MDEC, controllers/memcards, interrupts. Executor: orquestrador.

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
