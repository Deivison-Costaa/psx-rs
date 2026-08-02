# STATUS

> Memória do projeto entre iterações. O contexto do agente é descartado a cada iteração;
> este arquivo não. **Só handoff:** o que fazer agora e o que a máquina precisa saber para
> julgar uma rodada. Referência estável mora em `docs/invariantes.md` e é citada por número.
> Teto de 6 KB imposto por `status_size.rs`; forma imposta por `status_handoff.rs`.

## Última iteração concluída

**0150** — rastreamento da instalacao do VSync do Rayman. O jogo nao chama `OpenEvent` com
`F0000001`, nao habilita Timer1 com sync de VBlank e nao escreve o vetor `0x80000080`. Em vez
disso, chama `B(19h) HookEntryInt` em 164,111,334, instalando contexto em `0x801D0F78` cujo
PC e `0x801B8E60`; o hook roda centenas de vezes. O codigo que grava o contador existe em
`0x801B8C50` (`sw ...,0xF2CC`), mas nao e alcancado antes do spin.

## Próxima tarefa

**ROADMAP 10.75 — por que o hook `0x801B8E60` nao alcanca o incremento em `0x801B8C50`.**

Medido na 0150: o Rayman instala o hook por `B(19h) HookEntryInt` no passo 164.111.334
(`a0=0x801D0F78`, `hook[0]=0x801B8E60`), e o hook **roda 1029 vezes** antes do spin. O codigo do
incremento existe (`0x801B8C40` le `0x801DF2CC`; `0x801B8C50` e `sw ...,0xF2CC` = `0xAC22F2CC`),
mas nao e alcancado. O contador recebe um unico store, de valor ZERO, em 163.969.223 vindo de
`0x801ABCF0` — e a inicializacao.

Medir: seguir calls/branches de `0x801B8E60` ate `0x801B8C40`, ou provar que o incremento depende
de uma chamada que o hook deveria fazer e nao faz. **Nao alterar timers, IRQ nem vetor por
intuicao** — as tres hipoteses do handoff anterior ja foram refutadas por medicao.

Cuidado: existem DOIS `B(19h)`. O do passo 19.258.130 (`hook[0]=0x8005A1D8`) e do kernel; o do
jogo e o de 164.111.334. Nao confundir.

Invariantes relevantes: 25, 27.

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

Workspace: **884** testes.

## Bloqueios

- **4.4 Boot de jogo**: o motor 4.4ad agora avanca setores sequencialmente; a fronteira
  seguinte medida no Rayman e o caminho hook -> incremento. Imagens de disco ficam fora do
  repositorio, em `.../Programacao com agentes/roms/extraido/`.
  **Nunca commitar imagem de disco.**
- **Premissa refutada:** o slot `$v1+0x18` não muda entre boots (0147). O defeito não está
  no valor do slot mas no encaixe temporal entre `SysInitMemory` e o enfileiramento dos
  handlers do jogo.
