# STATUS

> Memória do projeto entre iterações. O contexto do agente é descartado a cada iteração;
> este arquivo não. **Só handoff:** o que fazer agora e o que a máquina precisa saber para
> julgar uma rodada. Referência estável mora em `docs/invariantes.md` e é citada por número.
> Teto de 6 KB imposto por `status_size.rs`; forma imposta por `status_handoff.rs`.

## Última iteração concluída

**0139** — Operacao migrada de Windows para Linux (Fedora 44). oc-iter/oc-loop acham o
binario do opencode nos dois SOs; flag de janela so no Windows (medido: `-WindowStyle`
LANCA no pwsh Linux); trabalhador vira opencode-go/gpt-5.6-luna com `--variant max`
(provider deepseek/ nao autentica nesta maquina). Bateria 7/7 + 2/2, MANUAL — mutantes.ps1
pula alvo fora de crates/psx-core/ (mutantes.ps1:366). Erro de 1a tentativa registrado: o
escape de aspas do prompt NAO e workaround so do Windows — o pwsh Linux tambem achata o
-ArgumentList, entao a mudanca planejada foi cortada. Contexto da 0137 (mecanismo do
congelamento, 8 hipoteses refutadas) em docs/iterations/0137-*.md.

## Próxima tarefa

**ROADMAP 10.61 — item fechado sai da escada mesmo em marco aberto.** 64 itens `- [x]`
ocupam 3906 dos 10000 bytes do ROADMAP (39% da escada e historico). A regra atual
(`roadmap_arquivo.rs`) so arquiva marco 100% FECHADO, entao fechado dentro de marco aberto
acumula para sempre — o M4 sozinho tem 37 fechados = 2126 bytes.
VERMELHO: baixar `MAX_BYTES` de `crates/psx-core/tests/roadmap_size.rs` de 10_000 para
7_000 (falha hoje: 9990). VERDE: mover TODOS os `- [x]` do ROADMAP.md para
`docs/ROADMAP-fechado.md`, VERBATIM, agrupados sob o cabecalho `## Mx` de origem (criar o
cabecalho la se faltar); atualizar o ponteiro no cabecalho do ROADMAP.
NAO tocar em item aberto. NAO apagar linha nenhuma — e movimentacao, nao poda.
Guardas que ja validam: `roadmap_arquivo.rs` (nenhum item nos dois arquivos; arquivo sem
item aberto; ponteiro no cabecalho) e `status_handoff.rs` (handoff pode citar item que
mora no arquivo de fechados). Marco que ficar sem NENHUM item perde o cabecalho tambem.
Bateria: use o opt-out formal — linha `Bateria de mutação: não se aplica — <motivo>` com
40+ chars de motivo no doc da iteracao (mutation_battery.rs), porque nao ha codigo de
producao no diff. Invariantes relevantes: nenhum.

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

Workspace: **868** testes.

## Bloqueios

- **4.4 Boot de jogo**: o motor 4.4ad agora avança setores sequencialmente; a fronteira
  seguinte medida no Crash é VSync/IRQ0 pós-kernel. Imagens de disco ficam fora do
  repositório, em `.../Programacao com agentes/roms/extraido/`.
  **Nunca commitar imagem de disco.**
