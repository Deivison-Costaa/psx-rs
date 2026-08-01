# STATUS

> Memória do projeto entre iterações. O contexto do agente é descartado a cada iteração;
> este arquivo não. **Só handoff:** o que fazer agora e o que a máquina precisa saber para
> julgar uma rodada. Referência estável mora em `docs/invariantes.md` e é citada por número.
> Teto de 6 KB imposto por `status_size.rs`; forma imposta por `status_handoff.rs`.

## Última iteração concluída

**0141** — Diagnostico puro. **A conclusao da 0137 esta REFUTADA por medicao**: nao houve
rollback do init pelo jogo. Sonda no gancho de C0 com `head_antes` mostrou que o handler do jogo
(`0x80140004`, prio 0) SOBREVIVE as duas chamadas de SysDeqIntRP — a que remove `0x80140014` e
correta (era a cabeca), e a que "remove" `0x80140024` nao remove nada (o elemento nunca foi
enfileirado; a spec marca a funcao como bugged). Quem destroi o handler e uma **SEGUNDA execucao
da sequencia de boot**: entre as chamadas 10 e 11 a cabeca pula de A00091E0 para 00006DA8, as
chamadas 9-10 se repetem como 11-12, e o TTY traz `reading file system` e `Inited and Allocated`
DUAS vezes. Sonda revertida; nenhum codigo de producao mudou.

## Próxima tarefa

**ROADMAP 4.5 — passo 2: achar o que dispara o SEGUNDO boot.** Rodar pelo ORQUESTRADOR
(trabalhador bloqueado por 10.62). O handler do jogo so some porque a cadeia de excecoes e
resetada; o alvo agora e a causa desse reset, nao mais a corrida de timing.
Medir: (a) quantas vezes o PC entra em `0xBFC00000` (reset vector) e em `0x80030000`; (b) quem
escreve na cabeca da cadeia (`[[0x100]] + prio*8`) entre a 10a e a 11a chamada de C0(02h/03h) —
sonda descartavel no caminho de escrita do bus; (c) se o segundo boot e espurio (nosso) ou pedido
pelo jogo. SE espurio: some com ele. SE legitimo: o defeito e o kernel nao repor os handlers.
**Testar junto o item 10.43** (TTY duplicado, hoje catalogado como defeito de TTY): se o boot roda
2x de fato, o TTY duplicado e SINTOMA, nao causa — a 0141 e a primeira medicao que da outra leitura
para aquele item, e ela e barata de conferir.
NAO implementar goldens de custo de ciclo ainda: a hipotese do painel da 0137 (`cpu.rs:187`
subcusta LWC2/SWC2; divida 10.45) continua ABERTA mas deixou de explicar o sintoma.
Armadilhas: (a) sondas sao descartaveis, reverter antes de commitar; (b) rebuild release antes de
medir; (c) o EXE do Crash REALOCA codigo — disasm so da RAM em runtime.
Invariantes relevantes: 25, 27, 30, 31.

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
