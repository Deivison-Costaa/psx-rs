# 0052 — psx-desktop

- **Data:** 2026-07-29
- **Item do roadmap:** 2.8
- **Objetivo:** método `framebuffer()` na GPU lendo VRAM via display registers (GP1(05h-07h)) e convertendo 15bpp→RGBA8. Deferida a janela desktop com eframe/egui por incompatibilidade de versão do compilador.

## Revisão do PR anterior

Revisão do PR anterior (0051): sem achados.
- 1. Teste que não mede: todas as asserções são diretas e específicas, confirmado pela bateria 6/6
- 2. Parâmetro não consumido → FIFO dessincronizado: nenhum comando GP0 novo na 0051
- 3. Regra de borda trocada: não aplicável (sem rasterização nova)
- 4. Campo de bit lido errado: GP1(08h).3 → GPUSTAT.20 verificado correto na spec (03-gpu.md L889, L1018)
- 5. Panic ou laço ilimitado: sem unwrap/expect/unsafe/pânico no código novo
- 6. Citação de spec: `confere-citacoes.ps1` passou (verde)
- 7. Escopo transbordado ou dívida não declarada: item dividido em 2.7a/2.7b, declarado no doc

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| psx-spx | § GP1(06h) - Horizontal Display range (L826-846) | docs/reference/03-gpu.md |
| psx-spx | § GP1(07h) - Vertical Display range (L864-883) | docs/reference/03-gpu.md |
| psx-spx | § GPUSTAT bits 16-22 (L1015-1022) | docs/reference/03-gpu.md |
| psx-spx | § Dots per scanline table (L1483-1489) | docs/reference/03-gpu.md |

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | timing | Largura padrão do display é 320 pixels (HR1=1) | GPUSTAT após reset tem todos os bits de display zerados → HR1=0 (256 pixels) | t1 falhou: esperava 320, recebeu 256 |
| 2 | API-Rust | eframe 0.35 seria compatível com Rust 1.85 | eframe 0.35 requer Rust 1.92; toolchain do projeto está pinado em 1.85 | erro de compilação: trait `App` mudou API e o resolvedor de dependências travou na versão 1.92 |
| 3 | nenhum | — | — | — |

## Bateria de mutação

Placar da bateria: 6/6 mutantes mortos, 2/2 controles verdes, 0 equivalente — docs/mutantes/0052-psx-desktop.mut

| Mutante | Rótulo | Morreu | Teste que matou |
|---|---|---|---|
| m1 | cycles_per_pix sempre retorna 10 (ignora HR1/HR2) | sim | t2 (320px espera 8 ciclos/px) |
| m2 | display_width sem truncamento (& !3) | sim | t1 (largura padrão não arredondada) |
| m3 | framebuffer preenche com zeros | sim | t3 (pixels devem ser convertidos) |
| m4 | framebuffer ignora display_start | sim | t4 (offset não lido) |
| m5 | ordem de canais trocada (B,G,R em vez de R,G,B) | sim | t3 (vermelho=0xF8,0x00,0x00 fica 0x00,0x00,0xF8) |
| m6 | display_height usa wrapping_add em vez de wrapping_sub | sim | t1 (altura calculada errada) |
| K1 | variável local `_cosmetico` não usada em framebuffer | — | controle cosmético |
| K2 | inverte ordem dos campos do struct Framebuffer | — | controle cosmético |

## Placar antes → depois

Workspace: **393** → **398** testes (393 existentes + 5 novos).

## Revisão cruzada (orquestrador)

<!-- Preenchido pelo Claude na revisão do PR -->

## Decisões e notas

1. O item original 2.8 descrevia três funcionalidades: método `framebuffer()`, janela desktop com eframe/egui, e loop de render consumindo VRAM a cada frame. Por R4, esta iteração implementa apenas o `framebuffer()` — a janela desktop é deferida para iteração futura.
2. `cycles_per_pix()` deriva de GPUSTAT bits 16-18 (horizontal resolution), valores da spec: 256→10, 320→8, 512→5, 640→4, 368→7.
3. `display_width()` aplica a fórmula `((X2-X1)/cycles_per_pix + 2) AND NOT 3` conforme a spec (`docs/reference/03-gpu.md` L834-836).
4. `display_height()` = Y2 − Y1, sem arredondamento (`docs/reference/03-gpu.md` L871).
5. A conversão 15bpp→RGBA8 faz escalonamento linear: cada componente de 5 bits é deslocado 3 bits para esquerda (multiplicado por 8), resultando no valor máximo 0xF8 (248), compatível com o formato esperado pelo egui Color32.
6. O Framebuffer faz wrap de coordenadas da VRAM (X mod 1024, Y mod 512) conforme comportamento do hardware (`docs/reference/03-gpu.md` L822-824).
7. O deferimento do desktop foi registrado como item 2.8b no ROADMAP.
