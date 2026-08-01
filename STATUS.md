# STATUS

> Memória do projeto entre iterações. O contexto do agente é descartado a cada iteração;
> este arquivo não. **Só handoff:** o que fazer agora e o que a máquina precisa saber para
> julgar uma rodada. Referência estável mora em `docs/invariantes.md` e é citada por número.
> Teto de 6 KB imposto por `status_size.rs`; forma imposta por `status_handoff.rs`.

## Última iteração concluída

**0128** — Diagnostico: o shell testa os descritores **F1000001** (EvCB[1], spec=20h "command
completed") e **F1000004** (EvCB[4], spec=8000h "error happened") em alternancia nos ultimos
polls de `TestEvent(0x00001EC8)`. `$a0` medido: 0xF1000001 (3241x) e 0xF1000004 (3236x),
total 7627 chamadas. Spec=20h nunca e entregue — `DeliverEvent(F0000003h, 20h)` nao ocorre.
O trace do `psx-cli` agora inclui `a0($4)`.

## Próxima tarefa

**ROADMAP 4.4x — Por que `DeliverEvent(F0000003h, 20h)` nunca ocorre?** O shell espera o
evento spec=20h (command completed) no descritor F1000001. A segunda resposta do CD-ROM
(INT2, apos um comando como `GetID` ou `ReadN`) deve disparar o handler da BIOS que chama
`DeliverEvent(F0000003h, 20h)` — mas isso nao acontece. **Duas hipoteses ABERTAS, decidir
por medicao:** (a) nosso CD-ROM nao gera a INT2 do ultimo comando emitido — as dividas
10.53/10.54 do ROADMAP (comando executa com INT pendente; 2a resposta dirigida por ack, nao
por tempo) sao exatamente sobre isso; (b) a INT2 chega mas o handler da BIOS nao invoca
`DeliverEvent(F0000003h, 20h)`. Discriminador: a partir de ~88 M, rastrear cada INT do
CD-ROM entregue (tipo + comando pendente) e cada chamada de `DeliverEvent` (class+spec em
`$a0`/`$a1`, endereco via B-table[07h]) ate o ultimo TestEvent (89 906 602). Nomear o
ultimo comando enviado ao drive e o que aconteceu com a resposta dele.
(`docs/reference/13-kernel-bios.md` § B(07h) - DeliverEvent (L1642), § BIOS Event Summary
(L1735)). Armadilhas: (a) ordem de IRQ do `Cpu::step` CORRETA — nao mexer (invariante 31);
(b) medida negativa exige janela alem de 90 M e deteccao continua (invariantes 30/31).
Invariantes relevantes: 27, 30, 31.

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

Workspace: **836** testes.

## Bloqueios

- **4.4 Boot de jogo**: sem bloqueio conhecido; o boot passa do handshake do controle, do logo
  SONY e agora pede o `GetID`, mas recebe "sem disco" e repete para sempre (4.4q). Imagens de disco
  ficam fora do repositório, em `.../Programacao com agentes/roms/extraido/`.
  **Nunca commitar imagem de disco.**
