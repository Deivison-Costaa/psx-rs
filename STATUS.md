# STATUS

> Memória do projeto entre iterações. O contexto do agente é descartado a cada iteração;
> este arquivo não. **Só handoff:** o que fazer agora e o que a máquina precisa saber para
> julgar uma rodada. Referência estável mora em `docs/invariantes.md` e é citada por número.
> Teto de 6 KB imposto por `status_size.rs`; forma imposta por `status_handoff.rs`.

## Última iteração concluída

**0125** — O laco `0x8004205C..0x800422DC` e um **dispatch de eventos** do kernel: varre uma
tabela de entradas, despacha as de tipo `0x20` e `0x30` (registradores `$t4`/`$t5`), e retorna.
Nao e um laco de espera — e um laco de varredura que o shell chama a cada quadro. O evento que
faria o shell montar o sistema de arquivos e ler `SYSTEM.CNF` **nao esta sendo postado na tabela**.
`psx-cli` agora tem `--max-steps`, `--trace-pcs` e `--dump-mem` para diagnostico.

## Próxima tarefa

**ROADMAP 4.4u — o evento que falta esta a montante do dispatch.** O laco de dispatch (4.4t) esta
entendido: ele varre a tabela de eventos, despacha tipos `0x20` e `0x30`, e retorna. Quem deveria
postar um evento que dispara a montagem do sistema de arquivos nao o faz. A referencia do
DuckStation carrega `SCUS_949.00` depois de `SetGraphDebug`; nos nao lemos `SYSTEM.CNF`.
**Iteracao de diagnostico.** Rastrear o fluxo de eventos entre o CD-ROM e o kernel: quem produz
o evento que faz o shell sair do `SetGraphDebug`? Candidatos: o handler de interrupcao do CD-ROM
nao traduz INT para evento de kernel, ou o scheduler de threads do kernel nunca acorda a thread
do sistema de arquivos. O discriminador barato e o TTY contra a referencia (invariante 27).
Armadilha conhecida: nao instrumente o loop de dispatch de novo — ele ja e entendido. O alvo e
QUEM alimenta a tabela, nao QUEM a consome.
Invariantes relevantes: 26, 27.

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

Workspace: **823** testes.

## Bloqueios

- **4.4 Boot de jogo**: sem bloqueio conhecido; o boot passa do handshake do controle, do logo
  SONY e agora pede o `GetID`, mas recebe "sem disco" e repete para sempre (4.4q). Imagens de disco
  ficam fora do repositório, em `.../Programacao com agentes/roms/extraido/`.
  **Nunca commitar imagem de disco.**
