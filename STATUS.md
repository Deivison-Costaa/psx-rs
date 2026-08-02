# STATUS

> Memória do projeto entre iterações. O contexto do agente é descartado a cada iteração;
> este arquivo não. **Só handoff:** o que fazer agora e o que a máquina precisa saber para
> julgar uma rodada. Referência estável mora em `docs/invariantes.md` e é citada por número.
> Teto de 6 KB imposto por `status_size.rs`; forma imposta por `status_handoff.rs`.

## Última iteração concluída

**0163** — o TTY do BIOS resolveu a leitura: o segundo `KERNEL SETUP!` e do
`BOOTSTRAP LOADER` lendo o `SYSTEM.CNF`, **boot normal**. O jogo so comeca em
`Execute !` no passo 164.000.000 (`T_ADDR(80125000) T_SIZE(000aa800)`) e ja em 167.000.000
imprime `VSync: timeout`, 142 vezes. **Experimento:** forcando `ChangeClearPAD(0)` nas duas
religadas do kernel depois do `Execute !`, o contador `[0x801CF2CC]` vai de **1 para 145** e os
142 timeouts viram **0**. O corpo do `StartPAD2` na RAM bate byte a byte com a ROM (offset
`0x14680`), entao nao e corrupcao nossa. Itens 10.88 e 10.89 fechados como premissa refutada.

## Próxima tarefa

**ROADMAP 10.90 — por que o jogo chama `ChangeClearPAD(0)` ANTES do `StartPAD2`, que o desfaz.**

Handoff: o experimento de 0163 provou o elo final — com o auto-ack desligado depois do
`Execute !`, o contador de VSync anda (1 → 145) e os 142 `VSync: timeout` somem. O codigo do
`StartPAD2` que religa e ROM autentica (RAM `0x4B80..0x4C10` == ROM offset `0x14680`), entao o
defeito **nao** esta na cadeia de excecao nem no BIOS que executamos. Restam duas pontas, as duas
mediveis: (a) o fluxo real do jogo chamaria `ChangeClearPAD(0)` de novo depois do `StartPAD2`, e
nao chega la por uma divergencia nossa anterior — tracar o que o jogo faz entre `0x801A7958`
(retorno do StartPad) e a primeira espera de VSync; (b) o handler de pad de prioridade 2 alcanca
o ack aqui em condicoes em que no hardware nao alcancaria — comparar quantas transferencias de
pad ele completa por IRQ. Armadilha: **nada antes do passo 164.000.000 e do jogo**. Metrica da
propria rodada nunca deve ser fabricada.

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

Workspace: **909** testes.

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
  do memory card NÃO é o bloqueio: termina sozinha em 166.321.383 com `F4000001h,0100h`.
- **Janela util do Rayman: depois do passo 164.000.000** (`Execute !`). Antes disso e boot do
  BIOS + BOOTSTRAP LOADER; `0x8003xxxx`/`0x8005xxxx` sao do carregador. O executavel do jogo ocupa
  `0x80125000..0x801CF800`.
- **10.89 fechado como premissa refutada (0163)**: o 2o `KERNEL SETUP` e do bootstrap.
- **10.88 fechado como premissa refutada (0162)**: os descritores que o jogo consulta eram de
  CDROM no momento da espera. Não procurar defeito no caminho de card por causa dessa espera.
- **10.87 fechado sem correção (0161)**: o auto-ack de IRQ0 no handler de Pad/Card é do BIOS, e
  quem religa depois do `ChangeClearPAD(0)` do jogo é o próprio `StartPAD2`. Não procurar defeito aí.
- **Duas correções de SIO0 (0159, 0160) são da spec e NÃO mexeram no boot** — o histograma de PC
  dos últimos 20 M passos é idêntico byte a byte. Não gastar rodada nova no SIO0 esperando boot.
- **10.83 diagnóstico (0158, já revisado)**: a ativação 0 não visita `0x4A1C`; a posterior visita
  depois do nó `0x74A8` de prioridade 2, inserido pelo BIOS (não pelo jogo). A caminhada da
  ativação 0 chega ao fim (prioridade 3, `0x2458`) — `0x4A1C` estava fora das cadeias, não pulado.
- **Premissa refutada:** o slot `$v1+0x18` não muda entre boots (0147). O defeito não está
  no valor do slot mas no encaixe temporal entre `SysInitMemory` e o enfileiramento dos
  handlers do jogo.
