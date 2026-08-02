# STATUS

> Memória do projeto entre iterações. O contexto do agente é descartado a cada iteração;
> este arquivo não. **Só handoff:** o que fazer agora e o que a máquina precisa saber para
> julgar uma rodada. Referência estável mora em `docs/invariantes.md` e é citada por número.
> Teto de 6 KB imposto por `status_size.rs`; forma imposta por `status_handoff.rs`.

## Última iteração concluída

**0153** — o `lw 0x8C42F2CC` usa `$2=0x801D0000` e lê `0x801CF2CC`, o mesmo endereço do
`sw` que gravou `1`; 1 de 1029 hooks tinha VBlank pendente. O vetor nunca ocorreu sem algum
pending e `0x00004A1C` limpou bit 0 em 570 intervalos antes do hook. Teste: `rayman_vsync_address.rs`.

## Próxima tarefa

**ROADMAP 10.77 — drenar métricas reais do runner.**

Handoff: 10.79 está em `docs/iterations/0153-rayman-vsync.md`; não altere timers, IRQ, vetor
ou imediatos do CPU por intuição. Para 10.77, alvo `logs/metrics-pending.csv` →
`docs/metricas.csv` e o fluxo de `scripts/oc-iter.ps1`; armadilha: nunca fabricar linha,
duplicar campos vazios ou confundir o modelo do runner com o agente trabalhador.

Invariantes relevantes: nenhum.

## Repositório

- `main` protegida a partir da iter 0004; 1 PR por item; merge commit (nunca squash);
  commits test→feat→docs; título de PR validado pela CI.
- **Escopo de commit é UM único identificador `[a-z0-9-]`.** `feat(bus,cpu)` reprova no
  `commit-lint`; quando a mudança toca dois módulos, escolha o principal e cite o outro no
  resumo. Custou uma reescrita de 4 mensagens no PR #36 (ver `0022-scratchpad-isc.md`).
- Iterações são cronológicas e nem sempre na ordem dos itens (0003↔item 0.5, p.ex.);
  o vínculo real está no título do PR e no doc da iteração.
- **`ROADMAP.md` estava a 3 bytes do teto na 0121.** As linhas ja fechadas do 4.4 foram
  comprimidas (o contexto mora em `docs/iterations/`), sobrando ~470 bytes. Encurtar, nunca apagar.

## Placar de testes

Workspace: **888** testes.

## Bloqueios

- **4.4 Boot de jogo**: o motor 4.4ad agora avanca setores sequencialmente; a fronteira
  seguinte medida no Rayman foi o caminho hook -> incremento. Imagens de disco ficam fora do
  repositorio, em `.../Programacao com agentes/roms/extraido/`.
  **Nunca commitar imagem de disco.**
- **10.79 concluído como diagnóstico**: `CAUSE.ExcCode=00h` em 1029 hooks, VBlank pendente em
  1; leitura e escrita convergem em `0x801CF2CC`. Não transformar essa medição em correção de
  produção sem revisão adversarial.
- **Premissa refutada:** o slot `$v1+0x18` não muda entre boots (0147). O defeito não está
  no valor do slot mas no encaixe temporal entre `SysInitMemory` e o enfileiramento dos
  handlers do jogo.
