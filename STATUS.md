# STATUS

> Memória do projeto entre iterações. O contexto do agente é descartado a cada iteração;
> este arquivo não. **Só handoff:** o que fazer agora e o que a máquina precisa saber para
> julgar uma rodada. Referência estável mora em `docs/invariantes.md` e é citada por número.
> Teto de 6 KB imposto por `status_size.rs`; forma imposta por `status_handoff.rs`.

## Última iteração concluída

**0161** — o jogo chama `ChangeClearPAD(0)` em 164.110.587 (de `0x801B8BC0`) e instala o hook;
`StartPAD2` em 164.123.374 **religa** o auto-ack por dentro (`ChangeClearPAD(1)` de `0x00004BEC`,
RAM do kernel). Reconhecer IRQ0 no Pad/Card e comportamento documentado do BIOS: **nao ha
correcao de producao nesse caminho**. Teste `rayman_autoack.rs`.

## Próxima tarefa

**ROADMAP 10.88 — por que o caminho de memory card do BIOS conclui `err busy` com o slot vazio,
em vez de `err eject`.**

Handoff: o jogo espera dois descritores de evento — `F1000001h` (slot 1 = `F4000001h,0004h`,
*card done*) e `F1000004h` (slot 4 = `F4000001h,2000h`, *card err eject*) — em 454.122 chamadas de
`TestEvent` entre os passos 86.989.128 e 166.322.304. O BIOS entrega `F4000001h,0100h` (*err
busy*) uma vez, em 166.321.383, e `F0000011h,0100h` tres vezes antes disso. Nenhum dos dois e o
que o jogo espera. Sem card no slot, o desfecho de hardware e *eject*. Alvo: medir o que o
caminho de card do BIOS le do SIO0 antes de concluir *busy* — enderecos `81h`, `JOY_STAT`, e o
IRQ7 (I_MASK.7 so e ligada em 164.754.404).
Spec: § Device addressing (L262-L278) de `docs/reference/10-controllers-memcards.md`, e
§ Address byte (01h) being sent (L381-L389) do mesmo arquivo.
Armadilha: duas correcoes de SIO0 ja foram feitas (0159, 0160) e nenhuma mexeu no boot; o
caminho de card so comeca depois de 164 M passos. Metrica da propria rodada nunca deve ser
fabricada.

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

Workspace: **906** testes.

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
