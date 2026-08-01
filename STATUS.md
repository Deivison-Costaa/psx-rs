# STATUS

> Memória do projeto entre iterações. O contexto do agente é descartado a cada iteração;
> este arquivo não. **Só handoff:** o que fazer agora e o que a máquina precisa saber para
> julgar uma rodada. Referência estável mora em `docs/invariantes.md` e é citada por número.
> Teto de 6 KB imposto por `status_size.rs`; forma imposta por `status_handoff.rs`.

## Última iteração concluída

**0135** — Diagnostico puro (R8 suspensa por escrito): spec do CD-ROM lida INTEIRA e
**docs/cdrom-comandos.md** commitado — design doc do motor 4.4ad com citacao de linha
verificada, tabela por comando, unknowns explicitos e escopo dentro/fora decidido. Achado
que muda o desenho: o hardware NAO tem fila — 2 flags (INT2/INT1 pendentes; INT3 imediato)
e no maximo 1 INT1 nao entregue (06-cdrom.md L1969-1982). Spike de sideload registrado em
docs/spikes/sideload-crash.md: o jogo RODA pos-injecao e trava em VSync timeout + 100% dos
PCs no vetor de excecao — **GTE ainda nao e o muro; o proximo item-pai e VSync/IRQ do jogo**.

## Próxima tarefa

**ROADMAP 4.4ad — passos 2 e 3: goldens do orquestrador + implementacao do worker.**
(2) Orquestrador escreve `crates/psx-core/tests/cdrom_motor.rs` citando
docs/cdrom-comandos.md: modelo de 2 flags; gate de comando com INT pendente (06-cdrom.md
L1984-2000, fecha 10.53); timing DISTINTO por comando via § Second Response (L2064-2076;
Pause != Init por construcao); AVANCO de seek entre setores (setores N/N+1 com bytes
DIFERENTES num .bin sintetico — hoje reentrega o mesmo setor); rearm do ReadS; Setmode
bit5 → buffer 800h/924h; 2a resposta so nos 10 comandos de L2004-2014; INT5 na 1a suprime
a 2a (L2022-2026). (3) Worker implementa ate verde com R4 suspensa POR ESCRITO no doc da
0136; escopo fechado = "Decisoes de escopo do motor" do cdrom-comandos.md — o que esta
FORA e divida aceita, nao implementar. Armadilhas: (a) invariante 31 (ordem IRQ no
Cpu::step); (b) caminho CDROM_RESPONSE do bus.rs tambem entrega INTs; (c) nao quebrar
cdrom_fila_int (goldens do 4.4ac); (d) rebuild release antes de medir; (e) passo primo.
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

Workspace: **857** testes.

## Bloqueios

- **4.4 Boot de jogo**: fronteira atual é a leitura SEQUENCIAL de setores (4.4ad): o
  loader da BIOS chegou ao "boot file" mas ReadN reentrega o mesmo setor. Imagens de disco
  ficam fora do repositório, em `.../Programacao com agentes/roms/extraido/`.
  **Nunca commitar imagem de disco.**
