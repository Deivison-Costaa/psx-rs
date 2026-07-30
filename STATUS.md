# STATUS

> Memória do projeto entre iterações. O contexto do agente é descartado a cada iteração;
> este arquivo não. **Só handoff:** o que fazer agora e o que a máquina precisa saber para
> julgar uma rodada. Referência estável mora em `docs/invariantes.md` e é citada por número.
> Teto de 6 KB imposto por `status_size.rs`; forma imposta por `status_handoff.rs`.

## Última iteração concluída

**0109** — referencia da tela real obtida; a conclusao da 0107 estava errada e foi corrigida; tres
hipoteses para o 2.2d medidas e descartadas (ROADMAP 2.2d).

## Próxima tarefa

**ROADMAP 2.2e — o texto do logo sai vermelho; deveria ser azul-escuro.**
Escolhido antes do 2.2d porque tem suspeito nomeado e e independente. Medido em 30/07 contra a
referencia oficial: fundo deveria ser branco e sai cinza (180,180,180); "SONY" deveria ser
azul-escuro e sai vermelho. A geometria do texto esta certa desde o item 2.2c — e so a cor.
Suspeito primario: **item 10.13**, `GP0(24h)` e modulacao e o bit 24 do comando (raw texture) nao
e lido. Os quads do logo sao `2Ch`, com bit 24 = 0, isto e, **modulados**.
Spec: `docs/reference/03-gpu.md` L1080 (tabela de Shaded Textures) e a secao de Render Polygon.
Arquivos-alvo: `crates/psx-core/src/gpu.rs`.
Critério de aceitação: no despejo da VRAM o texto "SONY" sai azul-escuro sobre fundo branco.
Invariantes relevantes: 21.

**Ja medido e descartado para o 2.2d — nao repetir:** (1) projecao do GTE — **zero** chamadas de
`rtps` em 85 M passos, o logo nao passa pelo GTE; (2) resolucao vertical mal reportada —
`GPUSTAT=0x1406260D`, 640x240 sem entrelacamento, correto; (3) vertice escrito direto pela CPU —
nenhum `sw` com o valor do vertice, a lista de display vai por **DMA**. O proximo passo do 2.2d e
interceptar o canal 2 do DMA e ler os pacotes na RAM.

**Cuidado registrado (invariante 21):** no despejo da VRAM, a regiao de texpage aparece rosa/azul
porque cada halfword vira um pixel de 15 bits; ali sao quatro indices de CLUT. Para julgar COR,
olhe so o que foi rasterizado.

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
