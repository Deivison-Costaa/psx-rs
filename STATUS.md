# STATUS

> Memória do projeto entre iterações. O contexto do agente é descartado a cada iteração;
> este arquivo não. **Só handoff:** o que fazer agora e o que a máquina precisa saber para
> julgar uma rodada. Referência estável mora em `docs/invariantes.md` e é citada por número.
> Teto de 6 KB imposto por `status_size.rs`; forma imposta por `status_handoff.rs`.

## Última iteração concluída

**0103** — interrupção no delay slot descarta o salto pendente e recua o EPC; o boot da BIOS passa
do passo 26 595 832 (ROADMAP 4.4f).

## Próxima tarefa

**ROADMAP 4.4g — a BIOS ainda imprime `VSync: timeout`, agora com o contador avançando.**
Depois do 4.4f o boot não morre mais: sobrevive aos 50 M passos, com **0** chamadas a
`A0(40h)` (eram 1 071 429) e **0** buscas em PC=3. O TTY foi de **557** para **2 029** bytes e
agora chega a `ResetCallback: _96_remove ..`. O que sobra é a espera de VSync não ser satisfeita:
as mensagens seguem de `(2:1)` até `(55:54)` em 50 M passos, isto é, o kernel **conta** os vblanks
mas quem espera nunca é acordado. Comparar com o 4.4e: o handler agora roda inteiro (antes durava
uma instrução), então a hipótese a testar primeiro é o **acordar do evento**, não o dispatch.
Arquivos-alvo: `crates/psx-core/src/bus.rs`, `crates/psx-core/src/cpu.rs`.
Critério de aceitação: o TTY do boot passa de `ResetCallback` sem nova mensagem de `VSync: timeout`.
Invariantes relevantes: 16.

**Medido e descartado — não repetir:** dispatch de eventos do kernel (a trilha vai de `80000080`
ao kernel em RAM), base de tempo (89 vblanks em 50 M passos) e bit10 do modo do timer.
**TTY idêntico ao da `main` significa que não consertou nada** — foi assim que os PRs #114 e #115
caíram.

## Repositório

- `main` protegida a partir da iter 0004; 1 PR por item; merge commit (nunca squash);
  commits test→feat→docs; título de PR validado pela CI.
- **Escopo de commit é UM único identificador `[a-z0-9-]`.** `feat(bus,cpu)` reprova no
  `commit-lint`; quando a mudança toca dois módulos, escolha o principal e cite o outro no
  resumo. Custou uma reescrita de 4 mensagens no PR #36 (ver `0022-scratchpad-isc.md`).
- Iterações são cronológicas e nem sempre na ordem dos itens (0003↔item 0.5, p.ex.);
  o vínculo real está no título do PR e no doc da iteração.

## Placar de testes

Workspace: **720** testes.

## Bloqueios

- **4.4 Boot de jogo**: depende de imagem BIN/CUE fornecida pelo usuário. Não inventar,
  não baixar, não marcar. Quando a imagem estiver disponível, desbloquear.
