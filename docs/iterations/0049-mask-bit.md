# 0049 — mask-bit

- **Data:** 2026-07-29
- **Item do roadmap:** 2.6c
- **Objetivo:** implementar mask bit GP0(E6h) — force bit15 e write-protect no write_pixel.

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| psx-spx | § GP0(E6h) - Mask Bit Setting (L578-593) | docs/reference/03-gpu.md |
| psx-spx | § GPUSTAT bits 11-12 (L1010-1012) | docs/reference/03-gpu.md |

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | endereçamento | t4 esperava 0x001F (verde) para cor 0x00F80000 | R=0x00, G=0x00, B=0xF8 → B=31 (azul), resultado 0x7C00 | test t4 falhou com left: 31744 right: 31 |
| 2 | endereçamento | t8 passava 0xBEEF_0000 para CPU→VRAM e esperava 0xBEEF no pixel(0,0) | A0h escreve low halfword primeiro: 0x0000 vai para pixel(0,0), 0xBEEF para pixel(1,0) | test t8 falhou com left: 0 right: 48879 |
| 3 | API-Rust | refactor de `self.stat.get()` para `stat` em write_pixel quebrou âncora do manifesto 0047 | mutation_anchors verifica que toda âncora de manifesto existe no fonte | mutation_anchors falhou: ancora m5 do 0047 nao encontrada |
| 4 | mutação | m7 original (adicionar `let stat` no execute_fill) era cosmético, não mutante real | Mutante precisa alterar comportamento observável | m7 sobreviveu na bateria |
| 5 | mutação | K2 renomeava `stat` para `s` mas referências posteriores quebravam compilação | Controle cosmético precisa compilar | K2 deu erro de compilação |

## Bateria de mutação

Placar da bateria: 7/7 mutantes mortos, 2/2 controles verdes, 0 equivalente — docs/mutantes/0049-mask-bit.mut

| Mutante | Rótulo | Morreu | Teste que matou |
|---|---|---|---|
| m1 | write-protect le GPUSTAT.11 em vez de GPUSTAT.12 | sim | t3 (bit12=1, bit11=0 → write-protect falhou) |
| m2 | force bit15 le GPUSTAT.12 em vez de GPUSTAT.11 | sim | t1 (bit11=1, bit12=0 → force bit15 falhou) |
| m3 | write-protect verifica foreground (pixel) em vez de background (vram[idx]) | sim | t3 (foreground bit15=0, back bit15=1 → não protegeu) |
| m4 | force bit15 sempre desligado (if false) | sim | t1 (bit15 não foi ligado) |
| m5 | write-protect sempre desligado (if false) | sim | t3 (pixel protegido foi sobrescrito) |
| m6 | force bit15 aplica 0x4000 (bit14) em vez de 0x8000 (bit15) | sim | t1 (bit14 ligado em vez de bit15) |
| m7 | write-protect aplicado ao fill no execute_fill (spec diz que fill ignora) | sim | t7 (fill sobrescreveu pixel protegido) |
| K1 | inverte ordem de force bit15 e write-protect | — | controle cosmético |
| K2 | declara variável local não usada _cosmetic | — | controle cosmético |

## Placar antes → depois

Workspace: **369** → **377** testes (369 existentes + 8 novos).

## Revisão cruzada (orquestrador)

Revisão do PR anterior (#62 — iter 0048): sem achados
- Teste que não mede: conferido — 7 mutantes morreram, 2 controles sobreviveram
- Parâmetro não consumido: conferido — dither é estado (GPUSTAT.9), sem novos comandos
- Regra de borda trocada: conferido — render_triangle_dithered usa exclusivo (+1), render_single_line usa inclusivo (<=)
- Campo de bit lido errado: conferido — GPUSTAT.9 lido com (1 << 9), E1h bit 9 mapeia para GPUSTAT.9
- Panic/laço: conferido — bounds seguros, lerp_* protege divisão por zero, sem unwrap/unsafe
- Citação de spec: pwsh scripts/confere-citacoes.ps1 verde
- Escopo transbordado: conferido — !tex_active é consistente com limitação pré-existente do renderer

## Decisões e notas

1. O mask bit é implementado exclusivamente em `write_pixel`, que é o ponto central de escrita de todos os comandos de renderização (polígonos, linhas, retângulos). Fill (`execute_fill`) e transfers (CPU→VRAM, VRAM→CPU) escrevem diretamente na VRAM e não passam por `write_pixel`, portanto não são afetados — consistente com a simplificação do ROADMAP 10.7.
2. A verificação de write-protect (GPUSTAT.12) vem ANTES do force bit15 (GPUSTAT.11) e ANTES da semi-transparência. Se o pixel está protegido, nenhuma operação de escrita ocorre.
3. O force bit15 (GPUSTAT.11) modifica o pixel antes da semi-transparência — se a semi-transparência estiver ativa e o pixel tiver bit15=1, o blend será aplicado. Este é o comportamento esperado do hardware (o mask bit marca o pixel como semi-transparente).
4. O `self.stat.get()` foi lido uma única vez no início de `write_pixel` e armazenado na variável local `stat`, reutilizada nas verificações de mask bit e no cálculo do mode de semi-transparência. Isso exigiu corrigir a âncora m5 do manifesto 0047.
