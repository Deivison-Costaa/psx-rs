# STATUS

> Memoria do projeto entre iteracoes. O contexto do agente e descartado a cada iteracao;
> este arquivo nao. **So handoff:** o que fazer agora e o que a maquina precisa saber para
> julgar uma rodada. Referencia estavel mora em `docs/invariantes.md` e e citada por numero.
> Teto de 6 KB imposto por `status_size.rs`; forma imposta por `status_handoff.rs`.

## Ultima iteracao concluida

**0148** — diagnosticou por que o Rayman fica preso em `VSync: timeout`. Hipotese (a)
**confirmada**: o contador de VBlank do jogo em `0x801DF2CC` nunca e incrementado. IRQ0 e
levantada (660x), a CPU vetoriza para `0x80000080` (1470x), I_MASK tem bit0=1 (0x000D),
mas o handler do jogo nunca e alcancado — a cadeia ExCB/EvCB nao contem entrada para
classe F0000001 (VBlank callback). Hipotese (b - Timer1 VBlank) **refutada** (Timer1 em
Free Run, bit0=0). Hipotese (c - I_MASK bit0) **refutada** (bit0=1 no timeout).

## Proxima tarefa

**ROADMAP 10.74 — investigar COMO o jogo instala o handler de VBlank (e por que falha).**

O teste `vsync_timeout_diag.rs` (0148) prova que o handler do jogo que deveria incrementar
`0x801DF2CC` nunca executa, apesar de IRQ0 ser entregue ate `0x80000080`. Hipoteses:

  - (d) O jogo substitui o handler da BIOS em `0x80000080` por um proprio, que le I_STAT
    e despacha para o contador — mas a substituicao falha (endereco errado, RAM nao
    inicializada, etc.).
  - (e) O jogo usa `VSyncCallback()` da LIBGPU, que chama `OpenEvent(HwRCnt, ...)` ou
    equivalente — e o `OpenEvent` falha porque a tabela de handlers nao foi alocada.
  - (f) O jogo instala o handler via `SysEnqIntRP(0, ...)` diretamente, e a funcao
    escreve em endereco errado porque `A000E000h+2000h` foi pisoteada.

Como medir: rastrear escritas em `0x80000080` (vetor de interrupcao) e em `0x801DF2CC`
(contador do jogo) durante a inicializacao (0 a 200M passos). Verificar se a RAM em
`0x80000080` contem codigo da BIOS ou do jogo apos a inicializacao. Instrumentar
`region_write32` para logar escritas no vetor de interrupcao.

Invariantes relevantes: 32.

## Repositorio

- `main` protegida a partir da iter 0004; 1 PR por item; merge commit (nunca squash);
  commits test->feat->docs; titulo de PR validado pela CI.
- **Escopo de commit e UM unico identificador `[a-z0-9-]`.** `feat(bus,cpu)` reprova no
  `commit-lint`; quando a mudanca toca dois modulos, escolha o principal e cite o outro no
  resumo. Custou uma reescrita de 4 mensagens no PR #36 (ver `0022-scratchpad-isc.md`).
- Iteracoes sao cronologicas e nem sempre na ordem dos itens (0003<->item 0.5, p.ex.);
  o vinculo real esta no titulo do PR e no doc da iteracao.
- **`ROADMAP.md` estava a 3 bytes do teto na 0121.** As linhas ja fechadas do 4.4 foram
  comprimidas (o contexto mora em `docs/iterations/`), sobrando ~470 bytes. Encurtar, nunca apagar.

## Placar de testes

Workspace: **883** testes.

## Bloqueios

- **4.4 Boot de jogo**: o motor 4.4ad agora avanca setores sequencialmente; a fronteira
  seguinte medida no Crash e VSync/IRQ0 pos-kernel. Imagens de disco ficam fora do
  repositorio, em `.../Programacao com agentes/roms/extraido/`.
  **Nunca commitar imagem de disco.**
- **Premissa refutada:** o slot `$v1+0x18` nao muda entre boots (0147). O defeito nao esta
  no valor do slot mas no encaixe temporal entre `SysInitMemory` e o enfileiramento dos
  handlers do jogo.
- **10.73 VSync timeout Rayman:** o handler de VBlank do jogo nao e alcancado. IRQ0 chega
  ate `0x80000080` mas a cadeia ExCB/EvCB nao tem callback de VBlank. Proximo passo (10.74):
  rastrear como o jogo instala o handler e por que a instalacao falha.
