# STATUS

> Memória do projeto entre iterações. O contexto do agente é descartado a cada iteração;
> este arquivo não. **Só handoff:** o que fazer agora e o que a máquina precisa saber para
> julgar uma rodada. Referência estável mora em `docs/invariantes.md` e é citada por número.
> Teto de 6 KB imposto por `status_size.rs`; forma imposta por `status_handoff.rs`.

## Última iteração concluída

**0110** — 2.2e resolvido por medição, sem código: "SONY" vermelho e CLUT pisoteada pelo losango
(2.2d) e fundo cinza e o fade congelado pelo crash; o display nem chega a ligar (ROADMAP 2.2e).

## Próxima tarefa

**ROADMAP 4.4h — segundo crash do boot: `$ra` restaurado da pilha vale 4, passo 85 544 264.**
Promovido porque a 0110 mediu que ele **bloqueia toda a tela do logo**: a BIOS desenha o fade
inteiro com o display desligado (GPUSTAT.23=0 do inicio ao crash) e so ligaria o display — e
definiria o modo final de video — depois do ponto onde morre. Fundo branco, texto azul e modo
480i/240p sao incognosciveis ate ele cair (invariante 22).
O que ja se sabe: `$ra` volta da pilha valendo 4, logo alguem corrompeu o slot na RAM ou o load
veio do endereco errado. Caminho: harness `psx-estado/instrumentacao/vramshot.rs` (ou bootbios)
+ log condicional perto do passo 85,5 M; achar o `sw` que gravou 4 naquele endereco de pilha ou o
`lw` que leu de endereco errado.
Spec: `docs/reference/13-kernel-bios.md` (convencao de chamada/pilha) so se a medicao pedir.
Arquivos-alvo: `crates/psx-core/src/cpu.rs`; talvez `bus.rs`/`dma.rs` se a corrupcao vier de fora.
Critério de aceitação: boot passa do passo 85 544 264 sem `$ra=4`; bonus se o display ligar
(GPUSTAT.23=1) e o fade completar em `FFFFFF`.
Invariantes relevantes: 21, 22.

**Estado do 2.2d apos a 0110:** a lista da BIOS e uma cena de 480 linhas (losango 112..368,
centro 240 — proporcoes exatas da referencia) desenhada 2x por frame, offsets (0,1)/(0,241),
display start alternando 1/241, GP1(08h)=03 sempre; as duas metades da VRAM recebem a MESMA
metade superior da cena. Nao gastar iteracao tentando "consertar geometria" antes do 4.4h.

## Repositório

- `main` protegida a partir da iter 0004; 1 PR por item; merge commit (nunca squash);
  commits test→feat→docs; título de PR validado pela CI.
- **Escopo de commit é UM único identificador `[a-z0-9-]`.** `feat(bus,cpu)` reprova no
  `commit-lint`; quando a mudança toca dois módulos, escolha o principal e cite o outro no
  resumo. Custou uma reescrita de 4 mensagens no PR #36 (ver `0022-scratchpad-isc.md`).
- Iterações são cronológicas e nem sempre na ordem dos itens (0003↔item 0.5, p.ex.);
  o vínculo real está no título do PR e no doc da iteração.

## Placar de testes

Workspace: **735** testes.

## Bloqueios

- **4.4 Boot de jogo**: DESBLOQUEADO em 30/07 — o usuário forneceu as imagens. Ficam fora do
  repositório, em `C:\psx-roms\` (extraídas dos zips em `.../roms`). **Nunca commitar imagem de
  disco.** Depende agora do 2.2b.
