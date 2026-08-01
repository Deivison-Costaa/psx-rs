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

**ROADMAP 4.5 — passo 4: identificar a funcao do kernel em `0x2DB8` e quem a chama.**
Rodar pelo ORQUESTRADOR (trabalhador bloqueado por 10.62).
Ja provado: (i) `SysInitMemory` de `BFC06F4C` (ciclo 354.241.830) apaga o array de ExCB, que vive em
`A000E004`, dentro da regiao `A000E000h`+`2000h` que a spec manda ele reinicializar; (ii) a entrada
no BIOS e um `jal` LEGITIMO de `0x2DB8` para `BFC06FDC` (`ra=0x2DBC`, `cause=0`) — a leitura
"desvio espurio nosso" esta DESCARTADA por medicao.
Falta: que funcao mora em `0x2DB8` e quem a chama. Medir: (a) sonda de `jal`/`jr` com alvo
`0x2DB8` na janela, registrando `$ra` do chamador — se vier de `0x800xxxxx` e o jogo; (b) conferir
se algum A/B/C-function do kernel tem entrada que caia em `0x2DB8` (a tabela A0 fica em `0x200`,
B0 em `0x874`, C0 em `0x674` segundo o mapa de RAM da spec).
`A(9Ch) SetConf` ja foi sondado e NAO dispara — nao e ele.
NAO implementar goldens de ciclo: divida 10.45 aberta, mas nao explica este sintoma.
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
