# STATUS

> Memória do projeto entre iterações. O contexto do agente é descartado a cada iteração;
> este arquivo não. **Só handoff:** o que fazer agora e o que a máquina precisa saber para
> julgar uma rodada. Referência estável mora em `docs/invariantes.md` e é citada por número.
> Teto de 6 KB imposto por `status_size.rs`; forma imposta por `status_handoff.rs`.

## Última iteração concluída

**0106** — endereco do texel 4bpp/8bpp somava a linha duas vezes; o logo da BIOS passa a sair
legivel (ROADMAP 2.2c).

## Próxima tarefa

**ROADMAP 2.2d — a metade de baixo de cada triangulo do losango nunca e desenhada.**
Medido pelo orquestrador em 30/07, BIOS real, 400M passos, despejo dos poligonos nao texturizados.
O losango do logo e feito de triangulos gouraud como `(195,240),(320,115),(320,365)`: ordenados
por y, o topo e `(320,115)`, o meio e `(195,240)` e a base e `(320,365)`. Na VRAM aparece **so** o
trecho de y=115 a y=240 — a metade **acima** do vertice do meio. A metade de baixo some.
E o padrao classico de rasterizador que divide o triangulo em meia-superior e meia-inferior e
perde a segunda. Os testes de 2.3 nao pegam porque usam triangulos simples.
Spec: `docs/reference/03-gpu.md`, secao Render Polygon (offset +115 sobre o indice).
Arquivos-alvo: `crates/psx-core/src/gpu.rs`, funcao `render_triangle`.
Critério de aceitação: o losango do logo fecha nas quatro pontas no despejo da VRAM.
Invariantes relevantes: 13.

**Primeiro passo:** um teste com triangulo cujo vertice do meio esteja a ESQUERDA da aresta
longa e outro com ele a direita — os dois casos de divisao. Depois compare com o despejo real.

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
