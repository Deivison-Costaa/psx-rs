# STATUS

> Memória do projeto entre iterações. O contexto do agente é descartado a cada iteração;
> este arquivo não. **Só handoff:** o que fazer agora e o que a máquina precisa saber para
> julgar uma rodada. Referência estável mora em `docs/invariantes.md` e é citada por número.
> Teto de 6 KB imposto por `status_size.rs`; forma imposta por `status_handoff.rs`.

## Última iteração concluída

**0165** — **load delay slot encadeado corrigido.** Quando dois loads escreviam o mesmo GPR, o
primeiro resultado ficava visivel durante a instrucao seguinte. A condicao agora substitui o
load pendente pelo novo sem commitar o antigo: Amidog `Result: 00000109` e erros
`nop_.*_d` **588 -> 0**. Bateria 5/5 e 2/2; 0111 reexecutada com 5/5 e 2/2.

## Próxima tarefa

**ROADMAP 10.92 — branch encoding: ~4.312 erros de codificacao de branch na suite Amidog.**

Handoff: nao iniciar outro item nesta rodada. Para 10.92, leia a secao de branches da spec
antes de tocar no decode/execucao de branch em `crates/psx-core/src/cpu.rs`; R1 continua
literal. O oraculo Amidog permanece disponivel com o comando do runner no historico da rodada.

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

Workspace: **917** testes.

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
- **Oraculo de hardware disponivel (0164)**: 51 EXEs em `tests/exes/` (gitignored). Amidog CPU
  em `Result: 00000109`. Depurar o CPU contra ele custa menos que inferir de jogo.
- **Rayman parou de ser a frente principal (0164)**: a cadeia esta mapeada mas depende de um
  auto-ack que o BIOS religa por projeto; com o CPU ainda reprovado em branch e load delay, o
  sintoma do jogo nao e o lugar certo para trabalhar.
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
