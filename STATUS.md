# STATUS

> Memória do projeto entre iterações. O contexto do agente é descartado a cada iteração;
> este arquivo não. **Só handoff:** o que fazer agora e o que a máquina precisa saber para
> julgar uma rodada. Referência estável mora em `docs/invariantes.md` e é citada por número.
> Teto de 6 KB imposto por `status_size.rs`; forma imposta por `status_handoff.rs`.

## Última iteração concluída

**0158** — a ativacao 0 percorre sete handlers e nao visita `0x4A1C`; a ativacao 3
visita `0x4A4C -> 0x49BC -> 0x4A1C`. Teste `rayman_exception_chain.rs`. Revisao cruzada leu
o `ExCB` em `[0x100]` e **refutou** que o enfileiramento de `0x74A8` seja do jogo (`$ra` em
RAM do kernel, `0x4BC8`) — quem insere e o proprio BIOS.

## Próxima tarefa

**ROADMAP 10.83 — por que o elemento de prioridade 1 que responde a VBlank executa e nao
reconhece IRQ0.**

Handoff: a revisao de 0158 mostrou que a cadeia de prioridade 1 tem quatro elementos
(`0x6D88`→`0x6D78`→`0x6D68`→`0x6D58`) e que o primeiro (`first=0x18BC`, `second=0x19C8`) roda
o handler `0x19C8` **na ativacao 0**, com `I_STAT.bit0` ainda pendente no hook; quem acka e o
caminho de prioridade 2 (`0x4A1C`, `0xFFFFFFFE`, medido em 0157). Alvo: tracar as instrucoes de
`0x19C8` na ativacao 0 — cada load/store, e o valor lido — ate a saida, e dizer qual leitura o
faz nao reconhecer. Spec: `docs/reference/13-kernel-bios.md` § Priority Chains (L1484-L1502),
§ Exception Control Blocks ExCB (L2885) do mesmo arquivo, e
§ Interrupt Acknowledge (L52-L55) de `docs/reference/11-interrupts.md`.
Armadilha: `0x18BC` roda em toda ativacao, `0x19C8` so quando
o bit 0 esta setado — nao confundir verificador com handler. Metrica da propria rodada nunca
deve ser fabricada.

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

Workspace: **893** testes.

## Bloqueios

- **4.4 Boot de jogo**: o motor 4.4ad agora avanca setores sequencialmente; a fronteira
  seguinte medida no Rayman foi o caminho hook -> incremento. Imagens de disco ficam fora do
  repositorio, em `.../Programacao com agentes/roms/extraido/`.
  **Nunca commitar imagem de disco.**
- **10.79 concluído como diagnóstico**: `CAUSE.ExcCode=00h` em 1029 hooks, VBlank pendente em
  1; leitura e escrita convergem em `0x801CF2CC`. Não transformar essa medição em correção de
  produção sem revisão adversarial.
- **10.80 concluído como diagnóstico**: `0xBFC00448` instala `0x4A1C` antes de `C(00h)`;
  `0x4A1C` limpa IRQ0 antes da consulta de entrega, e o hook observa `I_STAT` diretamente.
- **10.81 concluído como diagnóstico**: nos 458 intervalos sem ack do balanço, `I_STAT` tinha
  somente bit 2 (173 CDROM) ou bit 3 (285 DMA); não há defeito de VBlank a corrigir nesta rodada.
- **10.83 diagnóstico (0158, já revisado)**: a ativação 0 não visita `0x4A1C`; a posterior visita
  depois do nó `0x74A8` de prioridade 2, inserido pelo BIOS (não pelo jogo). A caminhada da
  ativação 0 chega ao fim (prioridade 3, `0x2458`) — `0x4A1C` estava fora das cadeias, não pulado.
- **Premissa refutada:** o slot `$v1+0x18` não muda entre boots (0147). O defeito não está
  no valor do slot mas no encaixe temporal entre `SysInitMemory` e o enfileiramento dos
  handlers do jogo.
