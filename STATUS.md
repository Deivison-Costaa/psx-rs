# STATUS

> Memória do projeto entre iterações. O contexto do agente é descartado a cada iteração;
> este arquivo não. **Só handoff:** o que fazer agora e o que a máquina precisa saber para
> julgar uma rodada. Referência estável mora em `docs/invariantes.md` e é citada por número.
> Teto de 6 KB imposto por `status_size.rs`; forma imposta por `status_handoff.rs`.

## Última iteração concluída

**0126** — Diagnostico da corrente CD-ROM → kernel, **corrigido na revisao**: a conclusao
original ("IRQ2 nunca sobe") era artefato da janela de 80 M. Medido em build limpo: IRQ2 da
107 raises entre 80 M e 100 M e depois silencia; 6 EvCBs `class=F0000003h` registrados; e
`DeliverEvent` deixa **DOIS eventos ready** (`status=4000h`, specs `10h` e `200h`) que ninguem
consome. Elos IRQ2→handler→DeliverEvent→EvCB **intactos**; o defeito e o CONSUMO.

## Próxima tarefa

**ROADMAP 4.4v — dois eventos de CD-ROM ficam READY e o shell nao age.** Medidos a 150 M e
estaveis ate 700 M: `EvCB[0] class=F0000003h spec=10h` e `EvCB[5] spec=200h`, ambos
`status=4000h`, `mode=2000h` (sem callback — cabe a alguem testar/esperar). O TTY para em
`SetGraphDebug` e `SYSTEM.CNF` nunca e lido (invariante 27; DuckStation carrega `SCUS_949.00`
aqui). **Iteracao de diagnostico.** Rastrear o CONSUMO: quem deveria consumir esses eventos
(`docs/reference/13-kernel-bios.md`: § B(0Ah) - WaitEvent (L1625), § B(0Bh) - TestEvent
(L1637), § BIOS Event Summary (L1735)) e por que nao
chega la. Relacionar com o laco de dispatch da 0125 (`0x8004205C`, compara tipos `20h`/`30h`
numa tabela que NAO e o EvCB — achar o elo entre EvCB ready e essa tabela). Teste ancora:
`cdrom_evento_kernel.rs` (150 M; asserta IRQ2>0 e EvCBs registrados).
Armadilhas: (a) NAO reinvestigar `_96_init` nem o CD-ROM — 107 IRQ2 e 6 EvCBs provam que
rodam; (b) medida negativa com janela <150 M nao vale (invariante 30).
Invariantes relevantes: 26, 27, 30.

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

Workspace: **828** testes.

## Bloqueios

- **4.4 Boot de jogo**: sem bloqueio conhecido; o boot passa do handshake do controle, do logo
  SONY e agora pede o `GetID`, mas recebe "sem disco" e repete para sempre (4.4q). Imagens de disco
  ficam fora do repositório, em `.../Programacao com agentes/roms/extraido/`.
  **Nunca commitar imagem de disco.**
