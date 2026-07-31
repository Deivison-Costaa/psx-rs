# STATUS

> Memória do projeto entre iterações. O contexto do agente é descartado a cada iteração;
> este arquivo não. **Só handoff:** o que fazer agora e o que a máquina precisa saber para
> julgar uma rodada. Referência estável mora em `docs/invariantes.md` e é citada por número.
> Teto de 6 KB imposto por `status_size.rs`; forma imposta por `status_handoff.rs`.

## Última iteração concluída

**0114** — IRQ2 do CD-ROM: o drive pedia interrupcao (`irq_pending()` certo desde a 0062) e
ninguem levantava o bit; agora `service_cdrom_irq` sobe `I_STAT.2` por BORDA de
`(HINTMSK & HINTSTS)`. O boot passou do logo: TTY imprime a versao do controlador do CD e o
driver do PAD (ROADMAP 4.4i).

## Próxima tarefa

**ROADMAP 4.4j — resposta do controle no SIO0 (proximo degrau para a shell da BIOS).**
Medido na 0114 com `psx-estado/instrumentacao/shellwait.rs`: o boot agora para em
`PC=0x000045C4`, laco `lhu $t4,4($s1) / andi $t5,$t4,2 / beq` com `$s1=0x1F801040` — espera
`JOY_STAT.1` (RX FIFO nao vazia), a resposta do controle, que nunca chega. Perto dali
(`0x00004560`) ha um laco irmao esperando `JOY_STAT.0`, e `$s0=0x1F801070` (I_STAT), entao
IRQ7 tambem entra na conta: `sio.rs` ja tem `take_irq7`/`service_sio_irq` ligados.
Spec: `docs/reference/10-controllers-memcards.md` (§ JOY_STAT, § Controller sequence de um pad
digital SCPH-1080: resposta `hi-Z, 41h, 5Ah, botoes`). Arquivo-alvo: `crates/psx-core/src/sio.rs`.
Armadilha conhecida: `I_STAT` e de borda (invariante 24) — a mesma regra que custou dois erros
na 0114 vale aqui, e o ack do JOY_CTRL.bit4 vem DEPOIS do ack do I_STAT
(§ Interrupt Acknowledge, 11-interrupts.md).
Critério de aceitação: o boot sai de `0x000045C4` e o TTY avanca alem de
`PS-X Control PAD Driver Ver 3.0`.
Invariantes relevantes: 23, 24.

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

## Placar de testes

Workspace: **758** testes.

## Bloqueios

- **4.4 Boot de jogo**: sem bloqueio conhecido; desde a 0114 o boot passa do logo e para no
  handshake do controle (4.4j). Imagens de disco ficam fora do repositório, em
  `.../Programacao com agentes/roms/extraido/`. **Nunca commitar imagem de disco.**
