# STATUS

> Memória do projeto entre iterações. O contexto do agente é descartado a cada iteração;
> este arquivo não. **Só handoff:** o que fazer agora e o que a máquina precisa saber para
> julgar uma rodada. Referência estável mora em `docs/invariantes.md` e é citada por número.
> Teto de 6 KB imposto por `status_size.rs`; forma imposta por `status_handoff.rs`.

## Última iteração concluída

**0130** — `--dump-vram` no psx-cli (VRAM inteira, 15bpp LE cru). Medido: a tela SCE esta
DESENHADA e igual a referencia do DuckStation (losango, SONY, fundo cinza, sem "®");
**0 pixels mudam entre os dumps de 120M e 200M steps** — o shell congela na tela SCE
(~14s emulados; em hardware ela dura ~3s). Combinado com a 0129: ele espera algo que nunca
chega.

## Próxima tarefa

**ROADMAP 4.4z — O que destrava a saída da tela SCE?** O shell desenhou a tela SCE
(confirmado na 0130) e ficou em loop de VBlank sem falar com o CD (0129). Descobrir o que o
loop espera. Candidatos, por ordem de suspeita: (1) SPU — o jingle de boot toca nessa tela e
o shell pode esperar fim de voz/transferencia que nosso SPU stub nunca sinaliza; (2)
contador de frames via VBlank — menos provavel, VBlank esta vivo (F2000003h/2 a cada frame,
0129); (3) joypad no shell. Discriminador: trace de PCs do loop de VBlank do shell (janela
100M-102M, deteccao continua) para achar o que ele le/testa a cada frame — enderecos de SPU
(0x1F801Cxx), contadores em RAM, ou SIO. `--dump-vram` e `--dump-mem` ja existem no psx-cli.
Armadilhas: (a) ordem de IRQ do `Cpu::step` CORRETA — nao mexer (invariante 31); (b) pipeline
de eventos do CD eliminado — nao reabrir (invariante 32); (c) medida negativa exige janela
alem do horizonte (invariante 30).
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

Workspace: **842** testes.

## Bloqueios

- **4.4 Boot de jogo**: sem bloqueio conhecido; com disco montado o shell consome os eventos
  do CD, lê ~86 setores e desenha (0129) — fronteira atual é o 4.4y. Imagens de disco
  ficam fora do repositório, em `.../Programacao com agentes/roms/extraido/`.
  **Nunca commitar imagem de disco.**
