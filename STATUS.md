# STATUS

> Memória do projeto entre iterações. O contexto do agente é descartado a cada iteração;
> este arquivo não. **Só handoff:** o que fazer agora e o que a máquina precisa saber para
> julgar uma rodada. Referência estável mora em `docs/invariantes.md` e é citada por número.
> Teto de 6 KB imposto por `status_size.rs`; forma imposta por `status_handoff.rs`.

## Última iteração concluída

**0126** — Diagnostico da corrente de entrega de evento CD-ROM → kernel. `Irq::raise_counts[2]`
= 0 apos 80 M passos: o CD-ROM **nunca** levanta IRQ2 durante o boot completo com disco. A
cadeia arrebenta no elo 1. EvCB alocado (16 blocos em 0xE028) mas totalmente vazio (status=0).
O TTY termina em `ResetCallback: _96_remove ..`. O problema nao e o dispatch (4.4t) nem o CD-ROM
isolado — e a INICIALIZACAO do CD-ROM pela BIOS (`_96_init` / `A(96h)`).

## Próxima tarefa

**ROADMAP 4.4v — por que o `_96_init()` nao dispara comandos de CD-ROM?** O kernel chama
`_96_init()` durante o boot, que deveria enviar comandos ao CD-ROM (Test, GetStat) e receber
respostas com IRQ2. A prova: `raise_counts[2]` = 0 em 80 M passos. Candidatos: (a) o drive esta
em estado que o BIOS interpreta como "sem disco" e pula a init; (b) a flag de shell-open (bit 4
do stat do CD-ROM) esta bloqueando; (c) o `_96_init()` executa mas os comandos sao rejeitados
antes de produzir resposta. **Iteracao de diagnostico.** Instrumentar o caminho do `_96_init`:
quais portas do CD-ROM sao escritas, quais comandos sao enviados, e por que nenhum produz IRQ2.
Ferramentas: `--max-steps`, `--trace-pcs`, `--dump-mem` + contadores `raise_counts`.
Invariantes relevantes: 24, 26, 27. Armadilha conhecida: o CD-ROM isolado FUNCIONA (iters
0114–0124); o defeito esta na integracao BIOS↔CD-ROM, nao no dispositivo em si.

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
