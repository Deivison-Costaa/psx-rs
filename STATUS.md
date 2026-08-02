# STATUS

> Memória do projeto entre iterações. O contexto do agente é descartado a cada iteração;
> este arquivo não. **Só handoff:** o que fazer agora e o que a máquina precisa saber para
> julgar uma rodada. Referência estável mora em `docs/invariantes.md` e é citada por número.
> Teto de 6 KB imposto por `status_size.rs`; forma imposta por `status_handoff.rs`.

## Última iteração concluída

**0149** — diagnostico do `VSync: timeout` do Rayman. Das tres hipoteses do handoff, **(a) esta
CONFIRMADA** e (b) e (c) refutadas: a IRQ0 e levantada (660x), a CPU vetoriza para `0x80000080`
(1470x) e o `I_MASK` tem o bit 0 ligado (0x000D) — mas o contador do jogo em `0x801DF2CC`
permanece **zero**. O defeito esta DEPOIS do vetor: a cadeia de dispatch da BIOS nao alcanca o
handler do jogo. A ExCB tem 1 entrada com `class=0x00006DA8` (invalido) e a EvCB nao tem nenhuma
entrada `F0000001` (callback de VBlank).

## Próxima tarefa

**Rastrear COMO o jogo instala seu handler de VBlank.**

O spin do `VSync()` da LIBGPU esta em `0x801B958C` (`bne $v0, $v1, loop`, timeout de 0xFFFF
iteracoes) e le o contador global `0x801DF2CC` (`lui $2, 0x801D; lw $2, 0xF2CC($2)` em
`0x801B95AC`). Esse contador nunca sai de zero.

Descobrir por qual via o jogo registra o incremento: `VSyncCallback()` (que usa `F0000001`),
`SetRCnt`, ou substituicao direta do vetor. Sondar a INSTALACAO durante a inicializacao, nao o
momento do timeout.

**Ligacao com o ROADMAP 4.5, a conferir:** a ExCB com `class=0x00006DA8` e o mesmo tipo de falha
que trava o Crash, onde o `SysInitMemory` de `BFC06F4C` apaga o array de ExCB em
`A000E000h`+`2000h`. Se a raiz for a mesma cadeia de dispatch, um conserto fecha os dois. Vale
medir antes de assumir.

Armadilhas: (a) sondas sao descartaveis, reverter antes de commitar; (b) reconstruir release
antes de medir; (c) o `.cue` do Rayman e o reduzido, so track 01.

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

Workspace: **882** testes.

## Bloqueios

- **4.4 Boot de jogo**: o motor 4.4ad agora avança setores sequencialmente; a fronteira
  seguinte medida no Crash é VSync/IRQ0 pós-kernel. Imagens de disco ficam fora do
  repositório, em `.../Programacao com agentes/roms/extraido/`.
  **Nunca commitar imagem de disco.**
- **Premissa refutada:** o slot `$v1+0x18` não muda entre boots (0147). O defeito não está
  no valor do slot mas no encaixe temporal entre `SysInitMemory` e o enfileiramento dos
  handlers do jogo.