# STATUS

> Memória do projeto entre iterações. O contexto do agente é descartado a cada iteração;
> este arquivo não. **Só handoff:** o que fazer agora e o que a máquina precisa saber para
> julgar uma rodada. Referência estável mora em `docs/invariantes.md` e é citada por número.
> Teto de 6 KB imposto por `status_size.rs`; forma imposta por `status_handoff.rs`.

## Última iteração concluída

**0129** — A premissa da 0128 caiu por medicao direta: `DeliverEvent(F0000003h, 20h)`
**OCORRE** (10x, steps 87,0M-89,6M; B-table[07h]=0x00001B44). O shell sai do loop de
TestEvent (ultimo poll 89 906 602), le ~86 setores (INT1, Pause pendente) e desenha
(SetGraphDebug + hankaku no TTY, que cresce 473→725 bytes). Depois: steps ~92M-200M sem
NENHUM comando novo ao drive, so VBlank/IRQ0. Invariante 32 criada.

## Próxima tarefa

**ROADMAP 4.4y — O shell desenhou O QUÊ, e o que ele espera?** Medido na 0129: apos consumir
os eventos do CD o shell inicializa graficos e entra em loop de VBlank sem pedir mais nada ao
drive (ate step 200 M). Descobrir se a tela desenhada ja e o logo/menu e o que destrava a
continuacao. Passos: (1) capturar a VRAM em ~120 M steps (harness tipo o `vramshot` da 0110,
NAO commitar; conferir LastWriteTime do exe antes de confiar nele) e comparar com
`psx-estado/referencias/tela-de-boot-duckstation.png`; (2) se a tela estiver certa, o
suspeito vira entrada (joypad) ou tempo — medir o que o shell le no loop de VBlank (trace de
PCs do loop, nao checkpoint). Armadilhas: (a) ordem de IRQ do `Cpu::step` CORRETA — nao
mexer (invariante 31); (b) o pipeline de eventos do CD esta eliminado — nao reabrir
(invariante 32); (c) medida negativa exige janela alem do horizonte (invariante 30).
Invariantes relevantes: 30, 31, 32.

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

Workspace: **840** testes.

## Bloqueios

- **4.4 Boot de jogo**: sem bloqueio conhecido; com disco montado o shell consome os eventos
  do CD, lê ~86 setores e desenha (0129) — fronteira atual é o 4.4y. Imagens de disco
  ficam fora do repositório, em `.../Programacao com agentes/roms/extraido/`.
  **Nunca commitar imagem de disco.**
