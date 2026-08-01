# STATUS

> Memória do projeto entre iterações. O contexto do agente é descartado a cada iteração;
> este arquivo não. **Só handoff:** o que fazer agora e o que a máquina precisa saber para
> julgar uma rodada. Referência estável mora em `docs/invariantes.md` e é citada por número.
> Teto de 6 KB imposto por `status_size.rs`; forma imposta por `status_handoff.rs`.

## Última iteração concluída

**0137** — Diagnostico puro do congelamento pos-boot: 8 hipoteses refutadas POR MEDICAO
(comandos de CD, IRQ preso, TMR2, VBlank do kernel, leitura de CD, GPU/DrawSync, lhu do
I_STAT, vetorizacao IEc). Mecanismo medido: o jogo enfileirou 2 handlers prio 0
(SysEnqIntRP 0x80140004/14) e os REMOVEU (rollback de init falho); o kernel acka o bit6
sozinho (221.520x, padrao 0xFFFFFFBF) e o WaitIntr do jogo (poll cru de 0x40&I_STAT,
orcamento 0x800, $ra=0x8003EAF8) nunca ve o bit. Fila de streaming: 38 paginas em estado
1, promocao 1→2 nunca roda. Dossie completo em docs/iterations/0137-*.md.

## Próxima tarefa

**ROADMAP 4.5 — passo 1: confirmar o gatilho do rollback do init do LIBSN.** Dump da
estrutura dos elementos 0x80140004/0x14/0x24 (verifier/handler) + sonda descartavel no
chain walk do kernel (quem e chamado, o que o verifier le, por que devolve "nao e meu")
na janela do init. Suspeito do painel (juiz adversarial): larco de espera com orcamento
fixo perdendo corrida por ciclos subcustados — classe da 0104; cpu.rs:187 so custa
opcodes 0x20-0x26 (LWC2/SWC2 pagam 1; divida 10.45). SE confirmar: goldens de custo por
instrucao no padrao da 0104 com valor citado de docs/reference/02-cpu.md (NUNCA ajustado
ao sintoma), gate "intr timeout: 2→0", e implementacao do trabalhador
(opencode-go/gpt-5.6-luna, reasoningEffort max ja configurado em ~/.config/opencode).
Armadilhas: (a) sondas sao descartaveis, reverter antes de commitar; (b) rebuild release
antes de medir; (c) o EXE realoca codigo — disasm so da RAM em runtime.
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

Workspace: **861** testes.

## Bloqueios

- **4.4 Boot de jogo**: o motor 4.4ad agora avança setores sequencialmente; a fronteira
  seguinte medida no Crash é VSync/IRQ0 pós-kernel. Imagens de disco ficam fora do
  repositório, em `.../Programacao com agentes/roms/extraido/`.
  **Nunca commitar imagem de disco.**
