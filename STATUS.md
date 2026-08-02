# STATUS

> Memória do projeto entre iterações. O contexto do agente é descartado a cada iteração;
> este arquivo não. **Só handoff:** o que fazer agora e o que a máquina precisa saber para
> julgar uma rodada. Referência estável mora em `docs/invariantes.md` e é citada por número.
> Teto de 6 KB imposto por `status_size.rs`; forma imposta por `status_handoff.rs`.

## Última iteração concluída

**0151** — as 20 primeiras entradas do hook `0x801B8E60` têm `r2=1`; `I_MASK` mantém o bit 0,
mas `I_STAT` varia. Quando `I_STAT & I_MASK == 0`, o `beq` em `0x801B8EA0` toma o ramo para
`0x801B8F94`; a hipótese do `r2` errado foi refutada e a do ack prematuro de `I_STAT` foi
confirmada. O teste permanente é `rayman_hook_flow.rs`; nenhuma produção foi alterada.

## Próxima tarefa

**ROADMAP 4.4 — Boot de jogo 2D/menu.**

Handoff: o 10.75 confirmou que o caminho do Rayman depende de `I_STAT`, não de `r2`; consulte
`docs/iterations/0151-hook-flow.md` e não altere timers, IRQ ou vetor por intuição. O contador
agregado de hooks inclui o hook do kernel em `0x8005A1D8`; desambiguar sempre por `hook[0]`.
O próximo alvo de produção deve ser escolhido pelo orquestrador após a revisão deste diagnóstico.

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

Workspace: **885** testes.

## Bloqueios

- **4.4 Boot de jogo**: o motor 4.4ad agora avanca setores sequencialmente; a fronteira
  seguinte medida no Rayman foi o caminho hook -> incremento. Imagens de disco ficam fora do
  repositorio, em `.../Programacao com agentes/roms/extraido/`.
  **Nunca commitar imagem de disco.**
- **Premissa refutada:** o slot `$v1+0x18` não muda entre boots (0147). O defeito não está
  no valor do slot mas no encaixe temporal entre `SysInitMemory` e o enfileiramento dos
  handlers do jogo.
