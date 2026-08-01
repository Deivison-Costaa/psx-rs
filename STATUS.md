# STATUS

> Memória do projeto entre iterações. O contexto do agente é descartado a cada iteração;
> este arquivo não. **Só handoff:** o que fazer agora e o que a máquina precisa saber para
> julgar uma rodada. Referência estável mora em `docs/invariantes.md` e é citada por número.
> Teto de 6 KB imposto por `status_size.rs`; forma imposta por `status_handoff.rs`.

## Última iteração concluída

**0134** — 4.4ac fechado: 2a resposta ENFILEIRADA; o ack (HCLRCTL) so marca, o bus agenda
CDROM_SECOND (+0x4A00) e o IRQ2 sobe na entrega — nunca no instante do ack (06-cdrom.md
L333-337). Bateria 5/5+2/2. **Boot 400M: flag 0x91C4=1, BIOS saiu do retry de Init, TTY
atravessa KERNEL SETUP → BOOTSTRAP LOADER → "boot file: cdrom:PSX.EXE;1"** — a tela passou
da licenca. Fecha 10.54. Executada pelo orquestrador (plano de saida; papeis invertidos
aprovados 01/08: goldens do orquestrador, implementacao do trabalhador).

## Próxima tarefa

**ROADMAP 4.4ad — Motor de respostas do CD-ROM (Fase B do plano de saida).** Ordem: (1)
diagnostico puro com R8 SUSPENSA por escrito so p/ este item: ler 06-cdrom.md INTEIRO e
commitar docs/cdrom-comandos.md — tabela opcode → 1a/2a resposta, ciclos, citacao com
linha conferida via grep -n, unknown marcado. (2) goldens do ORQUESTRADOR citando a
tabela: fila tipada; timing DISTINTO por comando (Pause != Init por construcao); AVANCO de
seek entre setores — hoje seek_min/sec/sect so sao escritos no Setloc e ReadN reentrega o
MESMO setor (setores N/N+1 com bytes DIFERENTES no .bin sintetico); rearm do ReadS;
Setmode; buffer 2340B. (3) worker implementa ate verde (R4 suspensa por escrito).
Evidencia de que o avanco de seek e o bloqueio atual: loader caiu no fallback PSX.EXE
(SYSTEM.CNF ilegivel) e PCs 367M-400M em laco de espera do kernel (0xA0/0x5C4-5DC).
Armadilhas: (a) invariante 31 (ordem IRQ no Cpu::step); (b) o caminho CDROM_RESPONSE do
bus.rs tambem entrega INTs — cobrir os dois; (c) rebuild release antes de medir (corolario
rlib); (d) passo primo na amostragem. Invariantes relevantes: 30, 31, 32, 33.

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

Workspace: **850** testes.

## Bloqueios

- **4.4 Boot de jogo**: fronteira atual é a leitura SEQUENCIAL de setores (4.4ad): o
  loader da BIOS chegou ao "boot file" mas ReadN reentrega o mesmo setor. Imagens de disco
  ficam fora do repositório, em `.../Programacao com agentes/roms/extraido/`.
  **Nunca commitar imagem de disco.**
