# Tela de boot esperada — "SONY COMPUTER ENTERTAINMENT"

Referência fornecida pelo usuário em 2026-07-30, depois de o orquestrador ter concluído
(erradamente, na iteração 0107) que talvez não houvesse defeito por falta de referência.

**A imagem em si NÃO está versionada** (não é nossa), mas desde a iteração 0110 existe em disco,
fora do repositório: o original do usuário em
`Programacao com agentes/ps1-sony-computer-entertainment-boot-screen-16k-v0-5e3diayxmyp71.webp`
e uma cópia em `psx-estado/referencias/tela-de-boot.webp`. Se ambas sumirem, peça ao usuário.

## O que a tela real mostra

- Fundo **branco**.
- **"SONY"** no topo, em **azul-escuro**, em fonte serifada larga.
- **Losango completo** ao centro, quatro pontas, gradiente **dourado/laranja**, com um **"S"
  vazado** no meio (o entalhe diagonal).
- **"COMPUTER ENTERTAINMENT"** embaixo, também **azul-escuro**, menor, com o símbolo ®.

## O que nós desenhamos (medido em 30/07, BIOS real + disco, 30 M passos)

| | Real | Nosso | Item |
|---|---|---|---|
| Fundo | branco | cinza RGB(180,180,180) | 2.2e |
| "SONY" | azul-escuro | **vermelho** | 2.2e |
| Losango | completo, centrado | **metade de baixo**, grande demais | 2.2d |
| "COMPUTER ENTERTAINMENT" | presente | **ausente** | 2.2f |

O "S" vazado e o gradiente do losango saem **corretos** — o que erra é escala, posição e cor.

## Como reproduzir o nosso lado

Harness descartável `psx-estado/instrumentacao/vramshot.rs` (copiar para
`crates/psx-cli/src/bin/`, nunca commitar): boota a BIOS e despeja a VRAM 1024x512 como PNG, mais
as três CLUTs da linha 480. **Use 85,5 M passos** — a 0110 mediu que o texto só é rasterizado
entre 50 M e 85,5 M (a alegação anterior de "30 M bastam, idêntico até 85 M" estava errada: aos
50 M havia zero quads texturizados). O boot morre no passo 85 544 264 (item 4.4h) e tudo depois é
máquina morta. **O display está desligado (GPUSTAT.23=0) até o crash** — esta tela é estado
intermediário que o hardware real nunca chega a exibir (invariante 22).

**Cuidado (invariante 21):** no despejo, a região da texpage aparece rosa/azul porque cada halfword
vira um pixel de 15 bits; ali cada halfword são quatro índices de CLUT. Para julgar **cor**, olhe
só o que foi rasterizado.
