# STATUS

> Memória do projeto entre iterações. O contexto do agente é descartado a cada iteração;
> este arquivo não. **Só handoff:** o que fazer agora e o que a máquina precisa saber para
> julgar uma rodada. Referência estável mora em `docs/invariantes.md` e é citada por número.
> Teto de 6 KB imposto por `status_size.rs`; forma imposta por `status_handoff.rs`.

## Última iteração concluída

**0147** — rastreou quem escreve `BFC06FDC` em `mem[$v1+0x18]`. Resposta: o BIOS, durante a
inicialização (passos ~58.500 e ~84.292), pelos endereços `0xBFC00434` (sw) e `0xBFC02B68` (4× sb).
A premissa da 0142 ("o slot muda entre boots") está **refutada**: o slot nunca muda de valor.
O trampolim SEMPRE chama `BFC06FDC` (`SysInitMemory`), desde o primeiro acionamento.
A diferença entre o primeiro e o segundo boot está no MOMENTO da chamada, não no alvo.

## Próxima tarefa

**ROADMAP 4.5 — passo 6: determinar por que `SysInitMemory` apaga a cadeia de ExCB no
segundo boot mas não no primeiro, se o trampolim sempre chama o mesmo endereço.**

Hipóteses a testar:
  - (a) No primeiro boot, `SysInitMemory` é chamada ANTES de os handlers do jogo estarem
    enfileirados em `A000E000h+2000h` — portanto não há nada para apagar.
  - (b) No segundo boot, a chamada acontece DEPOIS de o jogo enfileirar seus handlers,
    e a reinicialização da região os destrói.

Como medir: cruzar o timestamp de cada chamada a `SysInitMemory` (já instrumentado pela 0142)
com o timestamp de cada `SysEnqIntRP` do jogo (já instrumentado pela 0141). Se a primeira
chamada de `SysInitMemory` acontecer antes do primeiro `SysEnqIntRP`, vale (a). Medir com
disco Crash, janela de 0 a 400 M passos.

Invariantes relevantes: 25, 27, 31.

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