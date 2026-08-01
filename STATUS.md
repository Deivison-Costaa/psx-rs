# STATUS

> Memória do projeto entre iterações. O contexto do agente é descartado a cada iteração;
> este arquivo não. **Só handoff:** o que fazer agora e o que a máquina precisa saber para
> julgar uma rodada. Referência estável mora em `docs/invariantes.md` e é citada por número.
> Teto de 6 KB imposto por `status_size.rs`; forma imposta por `status_handoff.rs`.

## Última iteração concluída

**0131** — Nenhum candidato do handoff: o shell nao espera hardware, espera DADO. Ele varre
em spin infinito (~113M+) o TMD do logo PlayStation carregado em 0x80010000 procurando
primitivos 0x20/0x30 — e o conteudo e lixo (contador de primitivas 0x2E0E1E1E) porque
**todo read de CD entrega o setor N+150**: bytes da RAM localizados no `.bin` no setor 155;
o TMD real esta no setor 5. 155-5 = pregap de 00:02:00. `--sample-pcs` novo no psx-cli.
Invariante 33 (inclui: amostragem exige passo primo; nunca Set-Content em fonte).

## Próxima tarefa

**ROADMAP 4.4aa — Subtrair o pregap no read do CD.** Defeito nomeado:
`read_sector_from_disc` (`crates/psx-core/src/cdrom.rs:518-520`) faz
`offset = abs_sector * 2352`; o correto e `(abs_sector - 150) * 2352` porque o MSF de
Setloc e absoluto (docs/reference/06-cdrom.md § Setloc - Command 02h (L787)) e a trilha 1
comeca em 00:02:00 (docs/reference/06-cdrom.md L850) — o `.bin` comeca 150 setores depois
do zero. Teste com golden do disco real: setor 5 do `.bin` comeca `41 00 00 00` (ID de TMD)
apos os 24 bytes de header Form1; pedir MSF 00:02:05 tem que devolver esses bytes. Guardar
tambem o caso abs_sector < 150 (retornar None, nao underflow — e usize!). Criterio de
sucesso de sistema: rodar o boot 130M steps e o spin em 0x8004205C NAO existir mais
(sample-pcs), e/ou a VRAM sair da tela SCE. Armadilhas: (a) invariantes 31/32 seguem
valendo; (b) exe release stale — rebuild antes de medir (invariante 30, corolario rlib).
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

Workspace: **844** testes.

## Bloqueios

- **4.4 Boot de jogo**: sem bloqueio conhecido; com disco montado o shell consome os eventos
  do CD, lê ~86 setores e desenha (0129) — fronteira atual é o 4.4y. Imagens de disco
  ficam fora do repositório, em `.../Programacao com agentes/roms/extraido/`.
  **Nunca commitar imagem de disco.**
