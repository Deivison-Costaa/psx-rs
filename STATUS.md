# STATUS

> Memória do projeto entre iterações. O contexto do agente é descartado a cada iteração;
> este arquivo não. **Só handoff:** o que fazer agora e o que a máquina precisa saber para
> julgar uma rodada. Referência estável mora em `docs/invariantes.md` e é citada por número.
> Teto de 6 KB imposto por `status_size.rs`; forma imposta por `status_handoff.rs`.

## Última iteração concluída

**0104** — custo em ciclos do load por regiao de memoria; a BIOS boota sem nenhum `VSync: timeout`
e desenha o logo da PlayStation (ROADMAP 4.4g).

## Próxima tarefa

**ROADMAP 2.2b — GP0(80h), o blit VRAM->VRAM, hoje consumido e ignorado.**
Com o 4.4g fechado a BIOS chega a desenhar: 179 774 pixels nao-zero na VRAM, 540 cores, display
configurado (x=0, y=241, range 608..3168). Mas a tela sai errada em tres pontos que apontam para o
mesmo comando: o losango do logo aparece so pela metade de baixo; onde deveria estar a palavra
"PlayStation" saem barras vermelhas horizontais; e o sprite `SONY COMPUTER ENTERTAINMENT` esta
carregado na VRAM (canto superior direito) mas nunca e composto na tela. Os tres sao copia de
retangulo dentro da VRAM.
Spec: `docs/reference/03-gpu.md`, secao GP0(80h) VRAM-to-VRAM (offset +115 sobre o indice).
Arquivos-alvo: `crates/psx-core/src/gpu.rs`.
Critério de aceitação: o losango do logo fica inteiro e o texto "PlayStation" deixa de ser barra.
Invariantes relevantes: 13, 17.

**Como medir sem chute:** rode `psx-cli --bios <BIOS>` por ~400M passos e despeje a VRAM
(1024x512, 16bpp) — a comparacao e visual e direta. O item 10.7 (mask setting aplicado ao
VRAM->VRAM) e vizinho e NAO entra aqui (R4).

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

- **4.4 Boot de jogo**: DESBLOQUEADO em 30/07 — o usuário forneceu as imagens. Ficam fora do
  repositório, em `C:\psx-roms\` (extraídas dos zips em `.../roms`). **Nunca commitar imagem de
  disco.** Depende agora do 2.2b.
