# STATUS

> Memória do projeto entre iterações. O contexto do agente é descartado a cada iteração;
> este arquivo não. **Só handoff:** o que fazer agora e o que a máquina precisa saber para
> julgar uma rodada. Referência estável mora em `docs/invariantes.md` e é citada por número.
> Teto de 6 KB imposto por `status_size.rs`; forma imposta por `status_handoff.rs`.

## Última iteração concluída

**0159** — `send_byte` acendia `JOY_STAT.9` e pedia IRQ7 em toda transferencia, sem periferico
algum. Agora o primeiro byte e latchado como endereco (`01h` controle, `81h` card) e so o
dispositivo presente puxa /ACK. Bateria 6/6 e 3/3; 0091 e 0092 reexecutadas por ancora
envelhecida. Efeito no boot do Rayman: **nulo**, medido.

## Próxima tarefa

**ROADMAP 10.85 — qual desvio dentro do hook `0x801B8E60` separa a ativacao que incrementa
`[0x801CF2CC]` das que nao incrementam.**

Handoff: 0159 achou o consumidor. O jogo passa 19.400.685 dos ultimos 20 M passos em
`0x801B9500..0x801B95FF`, num laco com contador de timeout que so sai quando
`[0x801CF2CC] >= 2`; o contador esta parado em **1**. 0157 mediu que a ativacao 0 do hook viu
`I_STAT.bit0=1` e incrementou, e a ativacao 3 viu o bit ja limpo e nao incrementou — desde que o
elemento de prioridade 2 do BIOS acka VBlank antes do hook (0158). Alvo: tracar o corpo do hook
`0x801B8E60` nas duas ativacoes e achar o primeiro desvio divergente ate `0x801B8C50`.
Spec: § B(19h) - HookEntryInt(addr) (L1467-L1482) de `docs/reference/13-kernel-bios.md`.
Armadilha: o hook roda >1000 vezes; comparar ativacao 0 com uma posterior, nao duas posteriores.
Metrica da propria rodada nunca deve ser fabricada.

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

Workspace: **899** testes.

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
- **10.85 (0159)**: o laço final do Rayman é `0x801B9574`, esperando `[0x801CF2CC] >= 2`. A espera
  do memory card NÃO é o bloqueio: ela termina sozinha em 166.321.383 com `F4000001h,0100h`
  (*card err busy*). Correção de /ACK do SIO0 é da spec, mas não mexeu no boot.
- **10.83 diagnóstico (0158, já revisado)**: a ativação 0 não visita `0x4A1C`; a posterior visita
  depois do nó `0x74A8` de prioridade 2, inserido pelo BIOS (não pelo jogo). A caminhada da
  ativação 0 chega ao fim (prioridade 3, `0x2458`) — `0x4A1C` estava fora das cadeias, não pulado.
- **Premissa refutada:** o slot `$v1+0x18` não muda entre boots (0147). O defeito não está
  no valor do slot mas no encaixe temporal entre `SysInitMemory` e o enfileiramento dos
  handlers do jogo.
