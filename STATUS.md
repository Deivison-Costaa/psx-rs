# STATUS

> Memória do projeto entre iterações. O contexto do agente é descartado a cada iteração;
> este arquivo não. **Só handoff:** o que fazer agora e o que a máquina precisa saber para
> julgar uma rodada. Referência estável mora em `docs/invariantes.md` e é citada por número.
> Teto de 6 KB imposto por `status_size.rs`; forma imposta por `status_handoff.rs`.

## Última iteração concluída

**0111** — 4.4h corrigido: escrita no delay slot de load agora cancela o load pendente (a BIOS
exige). Boot completa o logo, liga 480i, e 2.2d/e/f cairam junto (ROADMAP 4.4h).

## Próxima tarefa

**ROADMAP 2.10 — `framebuffer_for_display` le GPUSTAT.23 invertido.**
Spec: `docs/reference/03-gpu.md` § GPU Status Register (L1001) — bit23 e **0=Enabled, 1=Disabled**
(e § GP1(03h) - Display Enable (L779): param 0=On). `gpu.rs:446` devolve `None` quando bit23==0,
ou seja, esconde a imagem com o display LIGADO. Com o boot agora vivo (0111), a BIOS liga o
display e o psx-desktop mostraria "Display desligado" na tela do logo.
**Armadilha:** os testes d1/d2 da iteracao 0053 (`gpu_desktop_egui.rs`) codificam a polaridade
errada — precisam VIRAR junto, como o teste "assumido" de load delay virou na 0111. Conferir
tambem quem mais le o bit 23 (`GP1(03h)` em gpu.rs escreve certo: 1=set=disabled).
Arquivos-alvo: `crates/psx-core/src/gpu.rs` (fn `framebuffer_for_display`).
Critério de aceitação: com a BIOS real a 120 M passos, `framebuffer_for_display()` devolve
`Some(640x480)` e o despejo bate com a tela do logo; apos `GP1(03h)=1`, devolve `None`.
Invariantes relevantes: 23.

**Estado do boot apos a 0111:** para no laco de espera de VSync (`PC=0x80059DCC`) com a tela do
logo completa na VRAM e `GPUSTAT=0x144E220D` (640x480 entrelacado). O proximo passo funcional e
o M4/item 4.4 (boot de jogo via CD-ROM). Fundo do logo termina em `B4B4B4`; o render de
referencia mostra branco — diferenca anotada no 2.2e fechado, rejulgar se aparecer fonte melhor.

## Repositório

- `main` protegida a partir da iter 0004; 1 PR por item; merge commit (nunca squash);
  commits test→feat→docs; título de PR validado pela CI.
- **Escopo de commit é UM único identificador `[a-z0-9-]`.** `feat(bus,cpu)` reprova no
  `commit-lint`; quando a mudança toca dois módulos, escolha o principal e cite o outro no
  resumo. Custou uma reescrita de 4 mensagens no PR #36 (ver `0022-scratchpad-isc.md`).
- Iterações são cronológicas e nem sempre na ordem dos itens (0003↔item 0.5, p.ex.);
  o vínculo real está no título do PR e no doc da iteração.

## Placar de testes

Workspace: **741** testes.

## Bloqueios

- **4.4 Boot de jogo**: sem bloqueio conhecido desde a 0111 (o boot chega vivo ao laco de VSync
  pos-logo). Imagens de disco ficam fora do repositório, em
  `.../Programacao com agentes/roms/extraido/`. **Nunca commitar imagem de disco.**
