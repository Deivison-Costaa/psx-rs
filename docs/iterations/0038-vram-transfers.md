# 0038 — vram-transfers

- **Data:** 2026-07-28
- **Item do roadmap:** 2.2
- **Objetivo:** VRAM de 1 MB e transfers CPU↔VRAM (fill GP0(02h), A0h copy, C0h copy).

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| psx-spx | § Quick Rectangle Fill (L217) | docs/reference/03-gpu.md |
| psx-spx | § GPU Memory Transfer Commands (L603) | docs/reference/03-gpu.md |
| psx-spx | § Masking and Rounding for FILL (L640) | docs/reference/03-gpu.md |
| psx-spx | § Masking for COPY Commands (L664) | docs/reference/03-gpu.md |
| psx-spx | § Wrapping (L697) | docs/reference/03-gpu.md |
| psx-spx | § GP1(00h) Reset GPU (L747) | docs/reference/03-gpu.md |
| psx-spx | § Ready Bits (bits 26/27) (L1041) | docs/reference/03-gpu.md |
| psx-spx | § GP1(01h) Reset Command Buffer (L767) | docs/reference/03-gpu.md |
| psx-spx | § Mask setting afeta CPU→VRAM (L590) | docs/reference/03-gpu.md |

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| G1 | hardware | GP1(00h) limpa a VRAM (`self.vram.fill(0)`) | GP1(00h) não afeta VRAM; o reset limpa fifo, ack irq, display off, dma off, display address, x1/x2, y1/y2, display mode, GP0(E1h..E6h) — L747-765 — e nem GP1(09h) é afetado | Revisão do orquestrador após o commit inicial |
| G3 | protocolo | `top3 == 4` (GP0(80h)) é no-op e as 3 palavras de parâmetro viram comandos | GP0(80h) é VRAM→VRAM blit com 4 palavras totais (comando + 3 params); deve consumir e ignorar as 3 palavras seguintes | Revisão do orquestrador após o commit inicial |
| M5 | API | Campo `pub stat: Cell<u32>` expõe mutabilidade interior | Deveria ser privado com getter `pub fn stat(&self) -> u32` | Revisão do orquestrador (G4) |
| M6 | regressão | Leituras de byte/halfword do GPUREAD não consomem a transferência | `read32(0x0)` é chamado via `bus.rs` para byte/halfword, consumindo palavras indevidamente | Revisão do orquestrador (G2) |
| M7 | teste | `a6` verificava pixel(3,0) para o extra descartado, mas o extra vai para pixel(0,1) (wrap de linha) | O halfword extra em contagem ímpar é escrito no início da PRÓXIMA linha, não coluna+1 | Bateria de mutação — mutante (e) sobreviveu com a asserção original, corrigida a célula verificada |

## Bateria de mutação

Placar: **6/7 mutantes pegos, 0/2 controles** (os controles não puderam ser aplicados sem false-positive por matching múltiplo nos edits).

| # | Mutação | Pego? | Teste que pegou |
|---|---|---|---|
| (a) | `& 0x3F0` → `& 0x3FF` na máscara de Xpos do fill | Sim | `a2_fill_arredonda_xpos_e_xsiz` |
| (b) | Remover arredondamento `+ 0x0F) & !0x0F` do Xsiz | Sim | `a2_fill_arredonda_xpos_e_xsiz` |
| (c) | `& 0x1FF` → `& 0x3FF` na máscara de Ypos do fill | **Não** | Nenhum (testes só usam Ypos < 512; gap de cobertura) |
| (d) | Inverter low/high pixels na palavra de 32 bits | Sim | 8 testes (`a5`, `a6`, `a7`, `a8`, `a9-copy`, `a0h_ysiz_513`, `a0h_xsiz_1024`, `peek32`) |
| (e) | Escrever halfword extra em contagem ímpar (remover guard `remaining > 0`) | Sim (após correção do teste) | `a6_a0h_impar_descarta_halfword_extra` |
| (f) | Não ligar bit 27 no cabeçalho do C0h | Sim | `a10_gpustat_bit27_c0h` |
| (g) | `(xsiz-1) & 0x3FF) + 1` → `xsiz & 0x3FF` | Sim | `a7_a0h_com_xsiz_zero_transfere_max_0x400`, `a10_gpustat_bit27_c0h`, `a0h_xsiz_1024_mascara_para_0_colunas_vira_max` |

Mutante (c) sobrevivente: a máscara `& 0x1FF` vs `& 0x3FF` para Ypos só difere quando Ypos ≥ 512, e nenhum teste cobre esse caso para fill. Registrado como gap de cobertura, sem ação nesta iteração (R4).

O mutante (e) inicialmente sobreviveu porque o teste `a6` verificava `pixel(3,0) == 0` mas o halfword extra, quando escrito, vai para `pixel(0,1)` (wrap de linha). A asserção foi corrigida durante a bateria.

## Placar antes → depois

- **Antes:** 291 testes (274 base + 17 da rodada anterior)
- **Depois:** 293 testes (+2 novos: `peek32_nao_consome_transferencia_c0h`, `top3_4_vram_to_vram_consome_params_e_permite_comando_seguinte`)
- Scoreboard: **5 com veredito (1p/4f), 45 só com saída, 0 sem saída, 1 não avaliados, de 51 arquivos**

Mudança de 4→5 vereditos: o fix G2 (peek32 em bus.rs) corrigiu leituras de byte/halfword do GPUREAD que antes consumiam palavras da transferência C0h. Um EXE que antes produzia saída sem padrão de pass/fail agora produz veredito (fail).

## Revisão cruzada (orquestrador)

## Decisões e notas

1. **Primeiro timeout do projeto:** a rodada anterior (trabalhador DeepSeek) estourou o limite de 45 min. Quando estourou, existiam 291 testes verdes (17 novos para 2.2), mas `cargo fmt` e `cargo clippy` estavam vermelhos, e não havia doc, nem bateria de mutação, nem PR. Os dois commits (`test(gpu):` e `feat(gpu):`) foram feitos pelo ORQUESTRADOR para preservar o trabalho. Por isso, a ordem R5 (teste-antes-de-implementação) não pôde ser verificada nesta iteração.

2. **Não existe EXE de hardware que meça este item.** `vram-to-vram-overlap` e 80h estão fora de escopo; `bandwidth` depende de timers. O critério de aprovação foi não-regressão no scoreboard.

3. **Uso de `Cell<u32>` para `stat`:** consequência de `Bus::read32` ser `&self`. O campo era `pub` por engano (G4); foi tornado privado com getter `pub fn stat(&self) -> u32`.

4. **`peek32`:** criado para resolver G2 — leituras de byte/halfword do bus não devem consumir a transferência C0h. Só acessos de 32 bits ao GPUREAD consomem.

5. **GP0(80h) VRAM→VRAM:** não implementado (fora do escopo 2.2, R4). As 4 palavras do comando (1 comando + 3 params) são consumidas e ignoradas via estado `SkipParams`. Acrescentado item 10.6 no ROADMAP.

6. **Correção do teste `a6`:** a asserção que verificava `pixel(3,0) == 0` foi corrigida para `pixel(0,1) == 0` — o halfword extra, se escrito, cai no início da próxima linha (wrap), não na coluna seguinte. Descoberto durante a bateria de mutação.

7. **`#![rustfmt::skip]` não funciona em integration tests** (unstable `custom_inner_attributes`). Em vez disso, usamos `#[rustfmt::skip]` em funções específicas com asserts longos, e encurtamos as mensagens das demais para caber no limite de 100 colunas sem quebra.

8. **Scoreboard 4→5 vereditos:** o G2 corrigiu `bus.rs` para usar `peek32` em leituras de byte/halfword. Antes, um `lb`/`lh` do GPUREAD consumia uma palavra da transferência C0h, corrompendo o fluxo e impedindo que um EXE produzisse padrão de pass/fail reconhecível. Com a correção, o EXE agora produz veredito (fail legítimo).
