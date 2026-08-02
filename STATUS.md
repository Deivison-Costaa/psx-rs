# STATUS

> Memória do projeto entre iterações. O contexto do agente é descartado a cada iteração;
> este arquivo não. **Só handoff:** o que fazer agora e o que a máquina precisa saber para
> julgar uma rodada. Referência estável mora em `docs/invariantes.md` e é citada por número.
> Teto de 6 KB imposto por `status_size.rs`; forma imposta por `status_handoff.rs`.

## Última iteração concluída

**0160** — o /ACK do SIO0 chegava em 0 ciclos, o que § Emulation Note proibe emular. Agora e
evento do scheduler (`SIO_ACK`, 338 ciclos: fora dos 100 ignorados, dentro dos 100 us de
timeout). Bateria 6/6 e 3/3; a do 0091 caiu para 5/6 e denunciou um teste que virou vacuo —
corrigido, 6/6. Efeito no boot do Rayman: **nulo**, medido.

## Próxima tarefa

**ROADMAP 10.87 — decidir se reconhecer VBlank no caminho de prioridade 2 antes do hook do jogo
e defeito nosso ou comportamento do BIOS que outra coisa deveria evitar.**

Handoff: 0160 traçou o corpo do hook. A divergencia e uma instrucao:
`0x801B8E78 lhu` le `I_STAT` (ativacao 0: `0x0001`; ativacao 3: `0x0000`), `0x801B8E98` aplica a
mascara `[0x801CF2E4] = 0x0009` (VBlank|DMA) e `0x801B8EA0 beq` desvia para `0x801B8F94` quando
da zero. A ativacao 0 roda 277 instrucoes e alcanca `0x801B8C50` (o incremento); as ativacoes 1 e
2 (`I_STAT=0x08`, DMA) rodam 444 e nao incrementam; a 3 sai em 26. Ou seja, o hook so conta VBlank
ainda pendente, e desde 0158 o elemento de prioridade 2 (`0x74A8` → `0x4A4C` → `0x49BC` →
`0x4A1C`, `0xFFFFFFFE`) acka antes dele. Alvo:
§ Priority Chains (L1494-L1502) de `docs/reference/13-kernel-bios.md`
diz que um handler que reconhece a IRQ **pode chamar
`ReturnFromException`**, pulando prioridades menores E o hook — medir se o caminho de prioridade 2
deveria ter feito isso, e o que o nosso emulador faz de diferente que o leva a seguir ate
`0x2458` e depois ao hook. Armadilha: duas ativacoes posteriores sao iguais entre si; compare
sempre com a ativacao 0. Metrica da propria rodada nunca deve ser fabricada.

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

Workspace: **905** testes.

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
- **Duas correções de SIO0 (0159, 0160) são da spec e NÃO mexeram no boot** — o histograma de PC
  dos últimos 20 M passos é idêntico byte a byte. Não gastar rodada nova no SIO0 esperando boot.
- **10.83 diagnóstico (0158, já revisado)**: a ativação 0 não visita `0x4A1C`; a posterior visita
  depois do nó `0x74A8` de prioridade 2, inserido pelo BIOS (não pelo jogo). A caminhada da
  ativação 0 chega ao fim (prioridade 3, `0x2458`) — `0x4A1C` estava fora das cadeias, não pulado.
- **Premissa refutada:** o slot `$v1+0x18` não muda entre boots (0147). O defeito não está
  no valor do slot mas no encaixe temporal entre `SysInitMemory` e o enfileiramento dos
  handlers do jogo.
