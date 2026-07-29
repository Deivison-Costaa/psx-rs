# 0050 — display-regs

- **Data:** 2026-07-29
- **Item do roadmap:** 2.7a
- **Objetivo:** Implementar registradores de display GP1(05h-07h) com máscaras de bits.

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| psx-spx | § GP1(05h) - Start of Display area (L809-824) | docs/reference/03-gpu.md |
| psx-spx | § GP1(06h) - Horizontal Display range (L826-862) | docs/reference/03-gpu.md |
| psx-spx | § GP1(07h) - Vertical Display range (L864-883) | docs/reference/03-gpu.md |
| psx-spx | § GP1(00h) - Reset GPU (L747-765) | docs/reference/03-gpu.md |

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | API-Rust | K2 usava `_ => {}` como ancora, assumindo ocorrencia unica | `_ => {}` aparece 2x no arquivo (write32 e write_gp1) | mutantes.ps1: "edicao '@@DE' encontrada 2 vez(es), esperado 1" |
| 2 | nenhum | — | — | — |

## Bateria de mutação

Placar da bateria: 7/7 mutantes mortos, 2/2 controles verdes, 0 equivalente — docs/mutantes/0050-display-regs.mut

| Mutante | Rótulo | Morreu | Teste que matou |
|---|---|---|---|
| m1 | GP1(06h) inverte X1 e X2 | sim | t3 (esperava X1=0x258, recebeu X2=0xC38) |
| m2 | GP1(07h) inverte Y1 e Y2 | sim | t4 (esperava Y1=0x1E, recebeu Y2=0x12E) |
| m3 | GP1(05h) mascara X com 9 bits em vez de 10 | sim | t5 (X com lixo no bit9 nao foi mascarado) |
| m4 | GP1(00h) reset nao zera display_range_x1 | sim | t1 (x1 nao foi resetado para 0x200) |
| m5 | GP1(05h) Y usa mascara 8 bits em vez de 9 | sim | t5 (Y com lixo no bit8 nao foi mascarado) |
| m6 | GP1(07h) Y2 deslocamento 12 em vez de 10 | sim | t4 (Y2 leu do campo errado) |
| m7 | GP1(06h) armazena X1 no lugar de X2 | sim | t3 (X2 recebeu X1 em vez do campo correto) |
| K1 | inverte ordem Y1/Y2 em GP1(07h) | — | controle cosmético |
| K2 | variavel local nao usada _nop | — | controle cosmético |

## Placar antes → depois

Workspace: **377** → **384** testes (377 existentes + 7 novos).

## Revisão cruzada (orquestrador)

Revisão do PR anterior (#63 — iter 0049): sem achados
- Teste que não mede: conferido — 7 mutantes morreram, 2 controles sobreviveram
- Parâmetro não consumido: conferido — GP0(E6h) é single-word, sem risco FIFO
- Regra de borda trocada: conferido — mask bit não interage com regras de borda de rasterização
- Campo de bit lido errado: conferido — GPUSTAT.11-12 lidos com máscaras corretas, m1/m2 conferem
- Panic/laço: conferido — sem unwrap/unsafe, bounds seguros
- Citação de spec: pwsh scripts/confere-citacoes.ps1 verde
- Escopo transbordado: conferido — exatamente o que o item 2.6c pedia

## Decisões e notas

1. O item 2.7 do ROADMAP juntava 3 funcionalidades distintas: display registers, timing NTSC/PAL e vblank IRQ. Por R4, foi dividido em 2.7a (esta iteração — registers) e 2.7b (timing + vblank).
2. GP1(05h) X=10 bits (0-9), Y=9 bits (10-18) — default v0/v1 (sem bit 10 de Y via GP1(09h)).
3. GP1(06h) X1=12 bits (0-11), X2=12 bits (12-23).
4. GP1(07h) Y1=10 bits (0-9), Y2=10 bits (10-19).
5. Valores padrão de reset (GP1(00h)) conforme spec: X=0, Y=0 (05h); X1=0x200, X2=0xC00 (06h); Y1=0x10, Y2=0x100 (07h).
6. Esses registradores são write-only do ponto de vista da API da GPU; a leitura de volta (GP1(10h)) não está implementada e os testes usam acessores `pub fn` dedicados.
