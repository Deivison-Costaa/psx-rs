# 0056 — dma-otc

- **Data:** 2026-07-29
- **Item do roadmap:** 3.2
- **Objetivo:** implementar registradores DMA no bus (MADR, BCR, CHCR para canais 0-6, DPCR, DICR) e canal 6 OTC (preenchimento de RAM com linked-list de fim de tabela).

## Revisão do PR anterior

Revisão do PR anterior (0055): sem achados.
- 1. Teste que não mede: 11 testes cpu_irq + mutantes todos mortos. OK.
- 2. Parâmetro não consumido → FIFO dessincronizado: sem comandos GP0 novos. OK.
- 3. Regra de borda trocada: sem rasterização. OK.
- 4. Campo de bit lido errado: máscaras 0x7FF, 0x1, 1<<10, 0xC000_007C verificadas. OK.
- 5. Panic ou laço ilimitado: sem unwrap()/unsafe. OK.
- 6. Citação de spec: confere-citacoes.ps1 verde. OK.
- 7. Escopo transbordado ou dívida não declarada: IRQ sem vblank declarado como dívida. OK.

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| psx-spx | DMA Register Summary (L27) | docs/reference/04-dma.md |
| psx-spx | D#\_MADR (L43) | docs/reference/04-dma.md |
| psx-spx | D#\_BCR (L61) | docs/reference/04-dma.md |
| psx-spx | D#\_CHCR | docs/reference/04-dma.md |
| psx-spx | DPCR (L121) | docs/reference/04-dma.md |
| psx-spx | DICR | docs/reference/04-dma.md |
| psx-spx | Commonly used values (L184) | docs/reference/04-dma.md |
| psx-spx | KSEG2 I/O region (L534) | docs/reference/02-cpu.md |

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | API-Rust | Que podia usar um segundo bus (`bus_read`) para ler após escrita sem problemas de borrow | Rust permite `write32` (mut) e depois `read32` (shared) no mesmo bus — a segunda variável era desnecessária e o compilador avisou com `unused_mut` | warning do compilador na primeira rodada de testes |
| 2 | endereçamento | Que o byte-write para DICR era tratado pelo handler de região | `region_write_byte` tem catch-all para `0x1F80_1061..=0x1F80_1FFF` que cobre os registradores DMA — byte-writes são silenciosamente engolidos | análise do código do bus; o teste `dicr_gravavel_e_legivel` usa `write32`, que funciona |
| 3 | timing | Que o OTC executava antes do read32 seguinte — é imediato em burst mode | Spec confirma: SyncMode=0 (burst) executa tudo de uma vez, bit 28 limpo no BEGIN, bit 24 no COMPLETION | teste `dma6_chcr_bit24_e_limpo_apos_otc` passou de primeira |

## Bateria de mutação

Bateria de mutação: 5/5 mutantes mortos, 2/2 controles verdes, 0 equivalente — docs/mutantes/0056-dma-otc.mut

Placar da bateria: 5/5 mutantes mortos, 2/2 controles verdes, 0 equivalente - ./docs/mutantes/0056-dma-otc.mut

| Mutante | Teste que o pegou |
|---|---|
| m1 (end marker 0x00000000 em vez de 0x00FFFFFF) | `dma6_otc_preenche_ram_com_linked_list` |
| m2 (BC=0 não tratado como 0x10000) | `dma6_otc_bcr_zero_equivale_a_10000h` |
| m3 (bits 24+28 não limpos após OTC) | `dma6_chcr_bit24_e_limpo_apos_otc`, `dma6_chcr_bit28_e_limpo_apos_otc` |
| m4 (incremento +4 em vez de -4) | `dma6_otc_preenche_ram_com_linked_list` |
| m5 (CHCR canal 6 sem restrição de bits) | `dma6_chcr_apenas_bits_24_28_30_e_1_sao_gravaveis` |

## Placar antes → depois

Workspace: **419** → **430** testes (419 existentes + 11 dma_otc).

## Revisão cruzada (orquestrador)

<!-- Preenchido pelo Claude na revisão do PR -->

## Decisões e notas

1. **OTC executa imediatamente no write do CHCR.** Como OTC usa SyncMode=0 (burst) com force-start (bit 28), a transferência acontece de forma síncrona durante o `write32` ao CHCR. Bits 24 e 28 são limpos ao final.
2. **CHCR do canal 6 tem bits restritos.** Apenas bits 24 (start), 28 (force-start) e 30 (snooping) são writable. Bit 1 é sempre 1 (increment=-4). Todos os outros bits são 0 e read-only.
3. **BC=0 equivale a 0x10000 palavras.** Conforme `docs/reference/04-dma.md` L76: "BC can be in range 0001h..FFFFh (or 0=10000h)".
4. **MADR é 24-bit com bits 0-1 forçados a 0 no endereçamento.** O valor armazenado preserva bits 0-1 (writable), mas `try_execute_otc` usa `madr & 0x00FF_FFFC` para word-alignment.
5. **End marker do OTC é 0x00FFFFFF.** Cada entrada anterior aponta para o endereço da entrada seguinte, mascarado com `0x1F_FFFC` (bits 2-20, word-aligned, dentro dos 2 MB de RAM).
6. **DPCR reset value = 0x07654321** — prioridades padrão conforme `docs/reference/04-dma.md` L140.
7. **DICR implementado como storage simples** — flags de interrupção e lógica de IRQ3 serão conectadas quando o scheduler de eventos e a conexão IRQ estiverem prontos (itens 3.3 e 3.4).
8. **Canais 0-5 têm CHCR sem restrição de bits.** Apenas o canal 6 (OTC) tem bits fixos. Os outros canais aceitam qualquer valor escrito (comportamento será refinado quando cada canal for implementado).
9. **Endereçamento descendente com wrap-around.** O decremento `addr.wrapping_sub(4)` faz wrap natural de u32. Endereços após wrap (> 0xFFFFFF) são truncados pelo `addr & 0x1F_FF_FF` e descartados se fora dos 2 MB de RAM.
