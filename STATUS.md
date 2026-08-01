# STATUS

> Memória do projeto entre iterações. O contexto do agente é descartado a cada iteração;
> este arquivo não. **Só handoff:** o que fazer agora e o que a máquina precisa saber para
> julgar uma rodada. Referência estável mora em `docs/invariantes.md` e é citada por número.
> Teto de 6 KB imposto por `status_size.rs`; forma imposta por `status_handoff.rs`.

## Última iteração concluída

**0136** — Motor de respostas do CD-ROM fechado: gate de comando com IRQ pendente, timing
por comando/estado, avanço de MSF em ReadN/ReadS e cancelamento do setor armado quando o
comando é aceito. Bateria: 6/6 mutantes mortos, 2/2 controles verdes; `cargo test --all`
verde. 4.4ad fecha 10.53; o spike confirma que **GTE ainda nao e o muro** — o próximo
diagnóstico é VSync/IRQ0 do jogo.

## Próxima tarefa

**ROADMAP 4.4 — Boot de jogo 2D/menu: diagnóstico puro de VSync/IRQ0 do jogo.** Abrir o
subitem após medir o spike em `docs/spikes/sideload-crash.md`; consultar `docs/reference/03-gpu.md`
§ GP1(07h), L864-873 (Y2 pode interromper IRQ0).
Consultar `docs/reference/11-interrupts.md` § Interrupt Request / Execution, L45-55. Alvos prováveis: `crates/psx-core/src/gpu.rs` e
`bus.rs`, teste novo no padrão de um arquivo por item. Armadilha: não assumir que o caminho
de VBlank que faz o shell avançar também entrega o callback LIBGPU; o Crash chega a
`VSync: timeout` e depois ao vetor `0x80000080`. Não começar GTE antes desse diagnóstico.
Invariantes relevantes: 31, 32, 33.

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

Workspace: **857** testes.

## Bloqueios

- **4.4 Boot de jogo**: o motor 4.4ad agora avança setores sequencialmente; a fronteira
  seguinte medida no Crash é VSync/IRQ0 pós-kernel. Imagens de disco ficam fora do
  repositório, em `.../Programacao com agentes/roms/extraido/`.
  **Nunca commitar imagem de disco.**
