# 0050 — display-range

- **Data:** 2026-07-29
- **Item do roadmap:** 2.7a
- **Objetivo:** implementar GP1(05h,06h,07h) display range registers + reset defaults.

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| psx-spx | § GP1(05h) Start of Display area (L809-825) | docs/reference/03-gpu.md |
| psx-spx | § GP1(06h) Horizontal Display range (L826-862) | docs/reference/03-gpu.md |
| psx-spx | § GP1(07h) Vertical Display range (L864-884) | docs/reference/03-gpu.md |
| psx-spx | § GPUSTAT bits 13-23 (L1003-1021) | docs/reference/03-gpu.md |

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | endereçamento | T6: `0x1FFF << 12` cabia nos 24 bits do param GP1 | 0x1FFF << 12 = 0x01FF_F000, que em 32-bit OR com 0x06 << 24 = 0x07FF_FFFF, corrompendo o byte de comando para 0x07 | test t6 falhou com X1=0x260 (default) em vez de 0xFFF |

## Bateria de mutação

Placar da bateria: 7/7 mutantes mortos, 2/2 controles verdes, 0 equivalente — docs/mutantes/0050-display-range.mut

| Mutante | Rótulo | Morreu | Teste que matou |
|---|---|---|---|
| m1 | GP1(05h) troca máscaras de X e Y | sim | t2 (X=0x3FF capturado como 0x1FF) |
| m2 | GP1(06h) máscara X1 errada (0x3FF) | sim | t5 (X1=0xFFF capturado como 0x3FF) |
| m3 | GP1(07h) máscara Y1 errada (0x1FF) | sim | t8 (Y1=0x3FF capturado como 0x1FF) |
| m4 | GP1(06h) X2 shift errado (>> 10) | sim | t4 (X2=0xC60 capturado como outro valor) |
| m5 | GP1(07h) Y2 shift errado (>> 12) | sim | t7 (Y2=0xF0 capturado como outro valor) |
| m6 | reset preserva display_vram_x/y em valores não-zero | sim | t9 (display_vram_x/y ≠ 0 após reset) |
| m7 | reset usa defaults errados para X1/X2 | sim | t9 (X1/X2 wrong after reset) |
| K1 | reordena blocos GP1(05h) e GP1(06h) | — | controle cosmético |
| K2 | variável local não usada em write_gp1 | — | controle cosmético |

## Placar antes → depois

Workspace: **377** → **387** testes (377 existentes + 10 novos).

## Revisão cruzada (orquestrador)

(Preenchido pelo orquestrador na revisão do PR.)

## Decisões e notas

1. O item 2.7 do ROADMAP foi dividido (R4) porque juntava três funcionalidades independentes:
   - **2.7a** (esta iteração): GP1(05h,06h,07h) — registradores de display range
   - **2.7b** (item novo): vblank timing (NTSC/PAL) + IRQ via scheduler + GPUSTAT bit 13
   - **2.7c** (item novo): integração com pipeline de display (item 2.8)
2. Os valores de reset (GP1(00h)) seguem os defaults NTSC da spec: X1=0x260, X2=0x260+320*8 (=0xC60), Y1=0x88-120 (=0x10), Y2=0x88+120 (=0x100). display_vram_x/y resetam para 0.
3. Os registradores são puramente de armazenamento — não afetam GPUSTAT e não têm efeito colateral. Serão consumidos pelo pipeline de display no item 2.8.
4. GPUSTAT bits 16-22 (resolução, video mode) e bit 23 (display enable) já estavam implementados via GP1(08h) e GP1(03h) respectivamente.
