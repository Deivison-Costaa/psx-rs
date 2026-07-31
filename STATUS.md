# STATUS

> Memória do projeto entre iterações. O contexto do agente é descartado a cada iteração;
> este arquivo não. **Só handoff:** o que fazer agora e o que a máquina precisa saber para
> julgar uma rodada. Referência estável mora em `docs/invariantes.md` e é citada por número.
> Teto de 6 KB imposto por `status_size.rs`; forma imposta por `status_handoff.rs`.

## Última iteração concluída

**0112** — polaridade do GPUSTAT.23 corrigida em `framebuffer_for_display`; testes fossilizados
de 0053/0090 virados; baterias reancoradas e re-rodadas (ROADMAP 2.10).

## Próxima tarefa

**ROADMAP 2.11 — altura do display em 480i: `display_height` devolve y2-y1 cru.**
Spec: `docs/reference/03-gpu.md` § GP1(08h) - Display mode (L885) — vres 480 exige bit5
(interlace); "Interlace must be enabled to see all lines in 480-lines mode". Com GPUSTAT.19 e
GPUSTAT.22 ligados, as linhas exibidas sao **(y2-y1)*2**, lidas de linhas consecutivas da VRAM.
A BIOS pos-0111 liga exatamente esse modo (`GPUSTAT=0x144E220D`) — sem o item, o psx-desktop
mostra so a metade de cima da cena de 480 linhas.
Arquivos-alvo: `crates/psx-core/src/gpu.rs` (fns `display_height`/`framebuffer`).
Critério de aceitação: com a BIOS real a ~120 M passos, `framebuffer_for_display()` devolve
`Some` com altura 480 e o conteudo bate com o despejo da cena inteira; em 240p as alturas
existentes nao mudam (nao-regressao dos testes de framebuffer).
Invariantes relevantes: 22, 23.

**Medicao de referencia externa (30/07):** DuckStation rodando a MESMA BIOS confirma nosso fundo
(180,180,180) e as cores do losango (mesmos valores de 15 bits); sem "®" na tela real; captura
canonica em `psx-estado/referencias/tela-de-boot-duckstation.png`. Diferencas restantes: costuras
de gouraud no losango (candidato 10.14) e, apos o logo, a shell da BIOS (MAIN MENU) que ainda nao
alcancamos.

## Repositório

- `main` protegida a partir da iter 0004; 1 PR por item; merge commit (nunca squash);
  commits test→feat→docs; título de PR validado pela CI.
- **Escopo de commit é UM único identificador `[a-z0-9-]`.** `feat(bus,cpu)` reprova no
  `commit-lint`; quando a mudança toca dois módulos, escolha o principal e cite o outro no
  resumo. Custou uma reescrita de 4 mensagens no PR #36 (ver `0022-scratchpad-isc.md`).
- Iterações são cronológicas e nem sempre na ordem dos itens (0003↔item 0.5, p.ex.);
  o vínculo real está no título do PR e no doc da iteração.

## Placar de testes

Workspace: **745** testes.

## Bloqueios

- **4.4 Boot de jogo**: sem bloqueio conhecido desde a 0111 (o boot chega vivo ao laco de VSync
  pos-logo). Imagens de disco ficam fora do repositório, em
  `.../Programacao com agentes/roms/extraido/`. **Nunca commitar imagem de disco.**
