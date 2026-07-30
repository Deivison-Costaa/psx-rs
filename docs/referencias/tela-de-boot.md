# Tela de boot esperada — "SONY COMPUTER ENTERTAINMENT"

Referência fornecida pelo usuário em 2026-07-30, depois de o orquestrador ter concluído
(erradamente, na iteração 0107) que talvez não houvesse defeito por falta de referência.

**A imagem em si NÃO está versionada.** Ela foi colada no chat e o arquivo não foi salvo. Se
precisar dela de novo, peça ao usuário — a fonte era um render da tela oficial, encontrado em
`preview.redd.it` sob o nome `ps1-sony-computer-entertainment-boot-screen`. O host bloqueia
busca automática; não tente baixar.

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

Binário descartável em `crates/psx-cli/src/bin/` que boota com `--bios` + `--disc` e despeja a
VRAM (1024x512, 16bpp) como RGB; converter para PNG com `zlib` da stdlib do Python. **30 milhões
de passos bastam** — a tela já está completa aí, conferido despejando também em 50 M, 70 M, 85 M e
85,54 M passos: idêntica. Não use 400 M: o boot morre num defeito separado no passo 85 544 264
(item 4.4h) e tudo depois é máquina morta.

**Cuidado (invariante 21):** no despejo, a região da texpage aparece rosa/azul porque cada halfword
vira um pixel de 15 bits; ali cada halfword são quatro índices de CLUT. Para julgar **cor**, olhe
só o que foi rasterizado.
