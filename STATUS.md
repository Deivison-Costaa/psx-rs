# STATUS

> Memória do projeto entre iterações. O contexto do agente é descartado a cada iteração;
> este arquivo não. **Só handoff:** o que fazer agora e o que a máquina precisa saber para
> julgar uma rodada. Referência estável mora em `docs/invariantes.md` e é citada por número.
> Teto de 6 KB imposto por `status_size.rs`; forma imposta por `status_handoff.rs`.

## Última iteração concluída

**0145** — o `Cargo.toml` da raiz nao declarava perfil nenhum, entao a suite inteira rodava em
`opt-level = 0`. Perfis `dev` e `test` passam a `opt-level = 1`, com `debug-assertions` e
`overflow-checks` explicitamente `true`: velocidade sem abrir mao das checagens.

## Próxima tarefa

**ROADMAP 10.70 — amostrar a sondagem de `testevent_descritor`.**

O laco de `crates/psx-core/tests/testevent_descritor.rs:97` roda a varredura inteira da tabela
EvCB **a cada passo da CPU**: ~66 leituras de barramento por passo contra 1-2 do `cpu.step`.
Sondar a cada N passos em vez de a cada passo. O laco ja sai cedo (`return`) ao achar; 90 M e
teto, nao trabalho feito.

Risco que a bateria TEM de cobrir: **amostrar pode cegar o teste.** O alvo fica dentro de
`crates/psx-core/src/`, entao `mutantes.ps1` roda sozinho; os mutantes atacam o mapeamento de
descritor de evento e o teste amostrado tem de continuar matando todos. Mutante que sobrevive
depois da amostragem e morria antes significa N grande demais — e isso e achado, nao ajuste
silencioso. Comecar em N=1024 e conferir pela bateria, nao pela intuicao.

Invariantes relevantes: 25, 29.

## Depois desta — ROADMAP 4.5

Rastrear **quem escreve** `BFC06FDC` em `mem[$v1+0x18]` entre o primeiro e o segundo boot.

Ja provado: (i) o trampolim `0x2C94..0x2DB8` carrega `$t0` de `mem[$v1+0x18]` e faz
`jalr $ra, $t0`; (ii) no primeiro boot o slot aponta para funcoes normais do kernel; (iii) aos
354 M contem `BFC06FDC`, que leva ao `SysInitMemory`. Medir: sonda de escrita (`sw`) com
endereco-alvo `$v1+0x18` pos-primeiro-boot, registrando PC e valor.

Armadilhas: (a) `$v1` e carregado antes do trampolim — ler `$v1` no STEP em `0x2DAC` e usar o
valor dinamico; (b) reconstruir release antes de medir; (c) sondas sao descartaveis, reverter
antes de commitar. Invariantes relevantes: 25, 27, 30, 31.

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

Workspace: **876** testes.

## Bloqueios

- **4.4 Boot de jogo**: o motor 4.4ad agora avança setores sequencialmente; a fronteira
  seguinte medida no Crash é VSync/IRQ0 pós-kernel. Imagens de disco ficam fora do
  repositório, em `.../Programacao com agentes/roms/extraido/`.
  **Nunca commitar imagem de disco.**
