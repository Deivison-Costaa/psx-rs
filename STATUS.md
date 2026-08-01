# STATUS

> Memória do projeto entre iterações. O contexto do agente é descartado a cada iteração;
> este arquivo não. **Só handoff:** o que fazer agora e o que a máquina precisa saber para
> julgar uma rodada. Referência estável mora em `docs/invariantes.md` e é citada por número.
> Teto de 6 KB imposto por `status_size.rs`; forma imposta por `status_handoff.rs`.

## Última iteração concluída

**0133** — Medicao pura, zero codigo: o pos-licenca E bloqueio. A BIOS re-envia **Init
(0x0A)** para sempre (laco em 0xBFC04A48-90, 99,85% das amostras 200M-400M; tela identica
aos 400M) fazendo poll da flag RAM 0x91C4 que fica em 0. O drive RESPONDE (1317 IRQs), mas
o **INT2 chega 383 cycles apos o INT3 — antes do ack** — violando a fila da spec
(docs/reference/06-cdrom.md L333-337). E a divida 10.54 medida em campo.

## Próxima tarefa

**ROADMAP 4.4ac — 2a resposta so apos o ack da 1a.** Mecanismo: hoje `cdrom.rs` agenda a
2a resposta (INT2) por TEMPO; a spec manda ENFILEIRAR — "if the 1st response is INT3 ...
the second is not delivered until INT3 is acknowledged" (docs/reference/06-cdrom.md § HINTSTS
(L313), regra em L333-337). Implementar: resposta secundaria pendente fica numa fila; o ack
via HCLRCTL (escrita em bank1 reg3) do INT corrente e o gatilho que promove a proxima
resposta a INTSTS. Teste (golden da spec): mandar Init (0x0A); INT3 aparece; INTSTS NAO
muda para 2 enquanto nao houver ack; apos HCLRCTL=0x07, INT2 aparece com stat correto.
Criterio de sistema: boot 400M+ — flag 0x91C4 vira 1, BIOS sai do retry de Init e a tela
passa da licenca (logo PS / leitura do SYSTEM.CNF no TTY). Fecha tambem 10.53/10.54 se a
implementacao for a fila. Armadilhas: (a) NAO mexer na ordem de IRQ do Cpu::step
(invariante 31); (b) o caminho CDROM_RESPONSE do bus.rs tambem entrega INTs — cobrir os
dois; (c) rebuild release antes de medir (corolario rlib); (d) passo primo na amostragem.
Invariantes relevantes: 30, 31, 32, 33.

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

Workspace: **847** testes.

## Bloqueios

- **4.4 Boot de jogo**: sem bloqueio conhecido; com disco montado o shell consome os eventos
  do CD, lê ~86 setores e desenha (0129) — fronteira atual é o 4.4y. Imagens de disco
  ficam fora do repositório, em `.../Programacao com agentes/roms/extraido/`.
  **Nunca commitar imagem de disco.**
