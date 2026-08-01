# STATUS

> Memória do projeto entre iterações. O contexto do agente é descartado a cada iteração;
> este arquivo não. **Só handoff:** o que fazer agora e o que a máquina precisa saber para
> julgar uma rodada. Referência estável mora em `docs/invariantes.md` e é citada por número.
> Teto de 6 KB imposto por `status_size.rs`; forma imposta por `status_handoff.rs`.

## Última iteração concluída

**0140** — Itens fechados foram movidos de `ROADMAP.md` para `docs/ROADMAP-fechado.md`, inclusive
o item 10.61 dentro do M10 aberto. A revisão adversarial encontrou o item corrente ainda aberto e
um teste que só cobria marcos 100% fechados; a continuação adicionou duas asserções, viu o vermelho
e corrigiu a movimentação. Bateria de mutação: não se aplica — nenhum código de produção.

## Próxima tarefa

**ROADMAP 4.5 — 1o frame do jogo: rollback do init do LIBSN e poll orfao do TMR2.** Confirmar
primeiro a corrida de timing medida na 0137; so depois escrever goldens e implementar. Spec:
`docs/reference/02-cpu.md` § Load Timing (L260) e § Load Shadow (L281);
`docs/reference/05-timers.md` § Timer 0..2 Counter Mode (L30) e § Dotclock/Hblank (L79);
`docs/reference/11-interrupts.md` § Interrupt Request / Execution (L45) e § Interrupt Acknowledge (L52).
Arquivos-alvo:
`crates/psx-core/src/cpu.rs`, `timers.rs`, `irq.rs` e teste novo em
`crates/psx-core/tests/cpu_game_frame.rs`. Armadilha: o sintoma `VSync: timeout` antigo nao e a
causa medida; nao inverter a ordem de IRQ do `Cpu::step`, nao atualizar timer por chamada manual
e nao aceitar checkpoint esparso como data do evento. Invariantes relevantes: 17, 28, 31.

**Meta em vigor (ordem do usuario, 31/07):** emendar as iteracoes ate o M4 fechar, sem parar entre
PRs. Pronto = **menu navegavel no `psx-desktop`**. Parada: 5 iteracoes fechadas sem o jogo bootar,
ou falha 3x no mesmo passo. Risco anotado: o unico disco disponivel e o Crash Bandicoot, que e 3D —
5.4b/5.4c/5.4d e 5.5 (GTE) estao abertos e podem entrar na conta.

**Referencia externa (30/07):** captura canonica do DuckStation em
`psx-estado/referencias/tela-de-boot-duckstation.png`; fundo (180,180,180) e cores do losango
CONFIRMADOS iguais aos nossos; sem "®" na tela real. Diferenca visual restante no logo: costuras
de gouraud no losango (candidato 10.14).

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

Workspace: **870** testes.

## Bloqueios

- **4.4 Boot de jogo**: o motor 4.4ad agora avança setores sequencialmente; a fronteira
  seguinte medida no Crash é VSync/IRQ0 pós-kernel. Imagens de disco ficam fora do
  repositório, em `.../Programacao com agentes/roms/extraido/`.
  **Nunca commitar imagem de disco.**
