# STATUS

> Memória do projeto entre iterações. O contexto do agente é descartado a cada iteração;
> este arquivo não. **Só handoff:** o que fazer agora e o que a máquina precisa saber para
> julgar uma rodada. Referência estável mora em `docs/invariantes.md` e é citada por número.
> Teto de 6 KB imposto por `status_size.rs`; forma imposta por `status_handoff.rs`.

## Última iteração concluída

**0127** — A "corrida intra-step" era artefato de checkpoint esparso (revisao). Deteccao
continua datou o flip: spec `10h` ready no step **89 702 216**, spec `200h` no **89 702 837**
— ~204 k passos ANTES do ultimo TestEvent (89 906 602). O shell consultou ~17x com os
eventos ja ready e desistiu. A ordem de IRQ do `Cpu::step` esta CORRETA (checa antes do
fetch; invariante 31). Sobra: ou o TestEvent testa OUTRO descritor, ou devolve errado.

## Próxima tarefa

**ROADMAP 4.4w — que descritor o TestEvent do shell testa, e que evento ele espera?** Os
eventos ready (`10h`/`200h`) nao o destravam; os specs `40h`, `80h` e `8000h` ficam busy para
sempre — candidatos ao evento que o shell aguarda e nunca e entregue. **Iteracao de
diagnostico.** Instrumentar os ultimos polls de TestEvent (`0x00001EC8`): capturar `$a0`
(descritor; o handle codifica o indice do EvCB) e `$v0` no retorno; mapear descritor→spec.
O `--trace-pcs` atual despeja `$v0` mas NAO `$a0` — estenda o trace do psx-cli se precisar.
Se o shell espera `40h`/`80h`/`8000h`: qual INT do CD-ROM deveria entrega-lo e por que nao
chega (`docs/reference/13-kernel-bios.md` § BIOS Event Summary (L1735)).
Armadilhas: (a) a ordem de IRQ do `Cpu::step` esta CORRETA — nao mexer (invariante 31);
(b) checkpoint esparso nao data evento — deteccao continua (invariantes 30/31);
(c) NAO reinvestigar `_96_init`, o CD-ROM nem o laco de dispatch (0125–0127 eliminaram).
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

Workspace: **834** testes.

## Bloqueios

- **4.4 Boot de jogo**: sem bloqueio conhecido; o boot passa do handshake do controle, do logo
  SONY e agora pede o `GetID`, mas recebe "sem disco" e repete para sempre (4.4q). Imagens de disco
  ficam fora do repositório, em `.../Programacao com agentes/roms/extraido/`.
  **Nunca commitar imagem de disco.**
