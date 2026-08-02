# STATUS

> Memória do projeto entre iterações. O contexto do agente é descartado a cada iteração;
> este arquivo não. **Só handoff:** o que fazer agora e o que a máquina precisa saber para
> julgar uma rodada. Referência estável mora em `docs/invariantes.md` e é citada por número.
> Teto de 6 KB imposto por `status_size.rs`; forma imposta por `status_handoff.rs`.

## Última iteração concluída

**0148** — capturas de execucao versionadas com procedencia (`docs/capturas/`). Um segundo disco
revelou que o emulador **desenha a tela da Ubi Soft com codigo do proprio Rayman** — caminho que
o Crash nunca alcanca — e entao trava repetindo `VSync: timeout`, com VRAM byte a byte identica
de 400 M a 1.500 M passos. Antes disso, a 0147 refutou a premissa da 0142: o slot `$v1+0x18` ja
contem `BFC06FDC` desde a BIOS e nunca muda (1150 ativacoes do trampolim, um unico `$v1`).

## Próxima tarefa

**ROADMAP 10.73 — por que o Rayman fica preso em `VSync: timeout`.**

Reproduzir: `psx-cli --bios bios/SCPH1001.BIN --disc "../roms/extraido/Rayman (USA) DADOS.cue"
--max-steps 400000000 --dump-vram /tmp/ray.raw`. O `.cue` e reduzido, so a track 01 de dados:
o original tem 51 tracks e o `parse_cue` guarda um unico `bin_path` (divida fechada por registro
na 0148, conserto ainda aberto).

`VSync` **nao e funcao do kernel** — nao esta em nenhuma tabela A/B/C nem em
`13-kernel-bios.md`. E da LIBGPU, linkada estaticamente pelo jogo: a mensagem vem do codigo do
proprio Rayman e o timeout e um contador dele.

Hipoteses, TODAS por confirmar: (a) contador de VBlank num handler que o jogo instala e que
depende da IRQ0 chegar; (b) root counter de VBlank ligado a `timers.rs`; (c) `I_MASK` sem o
bit 0 no momento certo, entao a IRQ0 e levantada mas nunca entregue. **Medir qual, antes de
consertar.**

Ja existe VBlank: `irq.rs`, `gpu.rs`, `timers.rs` e 9 testes em `gpu_vblank_irq.rs`. O defeito
nao e ausencia — e entrega, temporizacao ou mascara. Instrumentos: `bus.irq().raise_count(bit)`,
`cpu.irq_handler_entries`, `bus.irq().read_mask()`, `bus.irq().mask_write_count`.

O ROADMAP 4.5 (Crash) segue aberto e nao foi abandonado: a pergunta dele mudou para "o que
`BFC06FDC` FAZ em funcao do estado da maquina", ja que o slot e constante.

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