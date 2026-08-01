# STATUS

> Memória do projeto entre iterações. O contexto do agente é descartado a cada iteração;
> este arquivo não. **Só handoff:** o que fazer agora e o que a máquina precisa saber para
> julgar uma rodada. Referência estável mora em `docs/invariantes.md` e é citada por número.
> Teto de 6 KB imposto por `status_size.rs`; forma imposta por `status_handoff.rs`.

## Última iteração concluída

**0132** — Fix de 1 linha: `read_sector_from_disc` agora faz `checked_sub(150)` (pregap)
antes de indexar o `.bin`. Criterio de sistema CUMPRIDO nos dois termos: o spin em
0x80042xxx sumiu (0 amostras em 110M-200M) e a VRAM aos 200M mostra a **tela de licenca**
("PlayStation TM / Licensed by SCEA"). Achado: o teste da 4.4o codificava o defeito
(round-trip com o mapeamento errado) — corrigido. Bateria pelo mutantes.ps1 (5/5+2/2).

## Próxima tarefa

**ROADMAP 4.4ab — Pos-licenca: o boot continua?** Aos 200M a tela de licenca esta na VRAM e
o histograma de PCs (passo primo 100003, janela 110M-200M) da 61% em 0xBFC04xxx (BIOS ROM)
e 23% em 0x80059xxx (shell). Decidir por medicao se e o fluxo normal (tempo de tela +
proximas leituras) ou novo bloqueio: (1) estender a janela (300M+, invariante 30) com
--dump-vram e --sample-pcs e ver se a tela vira o logo PS / boot do jogo (TTY deve ganhar
linhas novas, p.ex. SYSTEM.CNF); (2) se travar, desassemblar o loop quente de 0xBFC04xxx
via --dump-mem (e ROM: ler o offset correspondente da BIOS) + --trace-pcs para registrar o
que ele le. Ferramentas ja existem no psx-cli. Armadilhas: (a) invariantes 31/32/33; (b)
rebuild release apos bateria (corolario rlib); (c) passo de amostragem PRIMO (invariante
33a); (d) o hot em BFC04xxx pode ser so o idle do kernel entre IRQs — conferir contra o
TTY/VRAM antes de declarar bloqueio.
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
