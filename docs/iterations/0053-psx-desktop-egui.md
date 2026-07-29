# 0053 — psx-desktop-egui

- **Data:** 2026-07-29
- **Item do roadmap:** 2.8b
- **Objetivo:** janela desktop com eframe/egui exibindo framebuffer da GPU; método `framebuffer_for_display()` que respeita GPUSTAT.23.

## Revisão do PR anterior

Revisão do PR anterior (0052): achado #1 — teste que não mede (y-mutation).
- 1. Teste que não mede: **ACHADO.** Mutação que substitui `(start_y + y)` por `start_y` em `framebuffer()` sobreviveu a todos os 5 testes (t1-t5). Adicionado t6 (`framebuffer_le_multiplas_linhas_da_vram`) que escreve cores diferentes nas linhas 10 e 11 da VRAM e verifica que o framebuffer as exibe em linhas distintas. Confirmado: t6 morre com a mutação, passa com o código correto.
- 2. Parâmetro não consumido → FIFO dessincronizado: nenhum comando GP0 novo na 0052, não aplicável.
- 3. Regra de borda trocada: não aplicável (sem rasterização nova).
- 4. Campo de bit lido errado: HR1/HR2 lidos dos bits corretos de GPUSTAT (16-18), verificado contra spec `docs/reference/03-gpu.md` (L885-892, L1015).
- 5. Panic ou laço ilimitado: sem unwrap/expect/unsafe/pânico no código novo.
- 6. Citação de spec: `confere-citacoes.ps1` passou (verde).
- 7. Escopo transbordado ou dívida não declarada: deferimento do desktop registrado como 2.8b no ROADMAP.

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| psx-spx | § GP1(03h) - Display Enable (L172-174) | docs/reference/03-gpu.md |
| psx-spx | § GP1(08h) - Display mode (L885-892) | docs/reference/03-gpu.md |
| psx-spx | § GPUSTAT bits 16-22 (L1015-1022) | docs/reference/03-gpu.md |

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | API-Rust | `from_rgba8_unmultiplied` existe em egui 0.32 | egui 0.32 tem `from_rgba_unmultiplied` (sem o `8`) | erro de compilação: método não encontrado |
| 2 | API-Rust | `ctx.tex_manager().write().alloc_managed(...)` é a API de textura do egui 0.32 | egui 0.32 usa `ctx.load_texture(name, image, options)` | erro de compilação: método `alloc_managed` não existe |
| 3 | API-Rust | `ImageSource::Texture(TextureId::Managed(id))` aceita TextureId | egui 0.32 espera `SizedTexture` | erro de compilação: tipo errado |
| 4 | GPU | Após `Gpu::new()`, GPUSTAT.23 = 0 (display desabilitado) | GPUSTAT.23 = 1 após reset (0x1480_2000), display habilitado por padrão | teste d1 falhou: esperava None, recebeu Some |

## Bateria de mutação

Placar da bateria: 5/5 mutantes mortos, 2/2 controles verdes, 0 equivalente — docs/mutantes/0053-psx-desktop-egui.mut

| Mutante | Rótulo | Morreu | Teste que matou |
|---|---|---|---|
| m1 | framebuffer_for_display sempre retorna None | sim | d2 (espera Some após habilitar) |
| m2 | framebuffer_for_display sempre retorna Some | sim | d1 (espera None após desabilitar) |
| m3 | checa bit errado (24 em vez de 23) | sim | d1 (desabilitar via GP1(03h)=0 não altera bit 24, display "permanece ligado") |
| m4 | inverte condição (retorna None quando display ON) | sim | d2 (habilitou mas retorna None) |
| m5 | retorna Framebuffer vazio (0x0) em vez de self.framebuffer() | sim | d3 (espera dimensões positivas e pixels corretos) |
| K1 | variável local cosmética em framebuffer_for_display | — | controle cosmético |
| K2 | reordena blocos do método (equivalente semântico) | — | controle cosmético |

## Placar antes → depois

Workspace: **398** → **402** testes (398 existentes + 1 t6 do review + 3 d1-d3 novos).

## Revisão cruzada (orquestrador)

<!-- Preenchido pelo Claude na revisão do PR -->

## Decisões e notas

1. Item 2.8b originalmente deferido da 0052 por incompatibilidade Rust 1.85 vs eframe 0.35. A toolchain foi atualizada para 1.97.1, e eframe 0.32.3 (rust-version 1.85) compila sem problemas.
2. `framebuffer_for_display()` é um wrapper sobre `framebuffer()` que verifica GPUSTAT.23 (display enable). O desktop usa este método para não tentar renderizar quando o display está desligado.
3. O loop do desktop atualmente mostra o framebuffer estático (VRAM inicial, toda preta). A integração com CPU/BIOS virá em iterações futuras (provavelmente após DMA e timers).
4. eframe foi adicionado como dependência do psx-desktop, não do psx-core — preservando R3 (psx-core puro, sem I/O).
5. A cada frame, a textura é reenviada ao egui via `ctx.load_texture()`. Para performance, uma iteração futura pode cachear o `TextureHandle` e usar `tex_manager().write().set()` para atualizar in-place.
