# STATUS

> Memória do projeto entre iterações. O contexto do agente é descartado a cada iteração;
> este arquivo não. **Só handoff:** o que fazer agora e o que a máquina precisa saber para
> julgar uma rodada. Referência estável mora em `docs/invariantes.md` e é citada por número.
> Teto de 6 KB imposto por `status_size.rs`; forma imposta por `status_handoff.rs`.

## Última iteração concluída

**0127** — Corrida confirmada: os dois eventos de CD-ROM (spec 10h/200h) viram ready
(status=4000h) entre 88 M e 89.9 M passos, no MESMO STEP do ultimo TestEvent do shell
(89 906 602). O `Cpu::step` executa a instrucao ANTES de despachar IRQs: TestEvent le
EvCB busy (2000h), retorna 0; DEPOIS a IRQ2 dispara, DeliverEvent marca os EvCBs ready
(4000h); mas o shell ja desistiu. Nao e defeito no TestEvent — e corrida intra-step.

## Próxima tarefa

**ROADMAP 4.4w — Corrigir corrida intra-step entre TestEvent e IRQ2. O shell desiste de
polling no step 89 906 602; no mesmo step, depois da instrucao, a IRQ2 entrega os eventos.
Candidatos: (a) aumentar timeout do dispatch loop do shell (lento e fragil), (b) adiantar
entrega do evento (reduzir latencia CD-ROM → IRQ2), (c) verificar IRQs ANTES de executar a
instrucao, nao depois (mais proximo do hardware: o pino fisico de IRQ chega entre instrucoes).
Spec: `docs/reference/11-interrupts.md` § Interrupt Request / Execution,
`docs/reference/02-cpu.md` § Exception/Interrupt Processing (L650+). Arquivos-alvo:
`crates/psx-core/src/cpu.rs` (ordem de step), `crates/psx-core/src/irq.rs`.
Armadilha: EvCB[1] (spec=20h) ja estava ready em 88 M — 1.9 M passos ANTES do ultimo
TestEvent. O shell nao espera por spec=20h; se a correcao fizer o shell consumir spec=20h
em vez de 10h/200h, o boot pode avancar por acidente e mascarar o problema real.
Invariantes relevantes: 30, 31 (nova: ordem de IRQ no Cpu::step).
Teste ancora: `evento_consumo_shell.rs` (150 M, asserta WaitEvent>0 apos correcao).

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

Workspace: **834** testes.

## Bloqueios

- **4.4 Boot de jogo**: sem bloqueio conhecido; o boot passa do handshake do controle, do logo
  SONY e agora pede o `GetID`, mas recebe "sem disco" e repete para sempre (4.4q). Imagens de disco
  ficam fora do repositório, em `.../Programacao com agentes/roms/extraido/`.
  **Nunca commitar imagem de disco.**
