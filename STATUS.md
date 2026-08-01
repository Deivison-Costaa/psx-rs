# STATUS

> Memória do projeto entre iterações. O contexto do agente é descartado a cada iteração;
> este arquivo não. **Só handoff:** o que fazer agora e o que a máquina precisa saber para
> julgar uma rodada. Referência estável mora em `docs/invariantes.md` e é citada por número.
> Teto de 6 KB imposto por `status_size.rs`; forma imposta por `status_handoff.rs`.

## Última iteração concluída

**0142** — Nomeado o mecanismo que a 0141 mediu. **Nao ha reset de CPU**: `reset_vector` dispara
UMA vez, no ciclo 0. No ciclo 354.241.830 — dentro da janela em que a cadeia e resetada — codigo do
BIOS em `BFC06F4C` chama `C(08h) SysInitMemory`, que pela spec reinicializa a regiao `A000E000h`
tamanho `2000h`. E `[0x100] = A000E004`: **o array de ExCB vive dentro dessa regiao**. Reinicializa-la
apaga as 4 cabecas de cadeia e leva junto o handler do jogo. Cadeia causal fechada ponta a ponta.
Falta saber POR QUE o BIOS re-executa esse caminho aos 354 M.

## Próxima tarefa

**ROADMAP 4.5 — passo 3: de ONDE se entra no BIOS aos 354 M.** Rodar pelo ORQUESTRADOR
(trabalhador bloqueado por 10.62). Esta provado O QUE apaga a cadeia (`SysInitMemory` de
`BFC06F4C`, ciclo 354.241.830) e POR QUE apaga (regiao contem o array de ExCB). Falta a entrada.
`ra=BFC06F4C` diz so que a CHAMADA a C0(08) partiu do BIOS — nao diz quem entrou no BIOS.
Medir: rastro de PC nos ~2000 passos ANTES do ciclo 354.241.830 (`--trace-pcs` ja existe no
psx-cli), procurando a transicao `0x800xxxxx` → `0xBFC0xxxx`.
Duas leituras a distinguir: (a) ESPURIA — exceção mal vetorizada, salto com alvo errado ou `$ra`
corrompido levam o PC para o BIOS; conserto e no nosso codigo. (b) LEGITIMA — o jogo chama de
proposito uma rotina que reinicializa memoria, e o defeito e o kernel nao repor os handlers.
Se a entrada vier de codigo do jogo com `$ra` coerente, e (b).
NAO implementar goldens de ciclo: a divida 10.45 continua aberta mas nao explica este sintoma.
Armadilhas: (a) sondas descartaveis, reverter antes de commitar; (b) rebuild release antes de
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
