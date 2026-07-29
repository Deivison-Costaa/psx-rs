# 0057 — dma-gpu

- **Data:** 2026-07-29
- **Item do roadmap:** 3.3
- **Objetivo:** implementar DMA canal 2 GPU — linked-list (SyncMode=2) para listas de comandos GP0 e block (SyncMode=1) para transferência de VRAM.

## Revisão do PR anterior

Revisão do PR anterior (0056): sem achados.
- 1. Teste que não mede: 13 testes dma_otc + mutantes todos mortos. OK.
- 2. Parâmetro não consumido → FIFO dessincronizado: sem comandos GP0 novos. OK.
- 3. Regra de borda trocada: sem rasterização. OK.
- 4. Campo de bit lido errado: máscaras 0x00FF_FFFF, 0x5100_0000, 0x1F_FFFC verificadas. OK.
- 5. Panic ou laço ilimitado: bounds check antes de copy_from_slice; sem unwrap/unsafe. OK.
- 6. Citação de spec: confere-citacoes.ps1 verde. OK.
- 7. Escopo transbordado ou dívida não declarada: simplificações declaradas nas notas 6-8 (DPCR sem máscara, DICR sem IRQ, canais 0-5 CHCR irrestrito). OK.

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| psx-spx | DMA Register Summary (L27) | docs/reference/04-dma.md |
| psx-spx | D#\_MADR (L43) | docs/reference/04-dma.md |
| psx-spx | D#\_BCR (L61) | docs/reference/04-dma.md |
| psx-spx | D#\_CHCR | docs/reference/04-dma.md |
| psx-spx | Commonly used values (L184) | docs/reference/04-dma.md |
| psx-spx | Linked List DMA (L198) | docs/reference/04-dma.md |
| psx-spx | KSEG2 I/O region (L534) | docs/reference/02-cpu.md |

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | API-Rust | Que `read_ram32` auxiliar seria usada nos testes | A função não foi usada — warning de dead_code | warning do compilador na primeira compilação |
| 2 | endereçamento | Que `gpuread_word()` retornaria valor útil para verificar transfer block | `gpuread_word()` só retorna dados em estado VramToCpu; em Idle retorna 0 | teste `dma2_block_transfere_dados_para_vram` falhou na asserção; corrigido para verificar VRAM via `gpu.vram_pixel()` e GPUSTAT bit26 |
| 3 | timing | Que a bateria de mutação rodaria sem travamento com m2 (SyncMode ignorado) | Sem guarda anti-loop, `execute_linked_list` entra em laço infinito quando alimentada com dados de block mode (addr=0, zeros lidos, nunca atinge end-marker) | timeout de 10 min na bateria; adicionado guarda de 4096 nós |

## Bateria de mutação

Placar da bateria: 6/6 mutantes mortos, 2/2 controles verdes, 1 equivalente - ./docs/mutantes/0057-dma-gpu.mut

| Mutante | Teste que o pegou |
|---|---|
| m1 (end-marker 0x00FFFFFF → 0x00000000) | `dma2_linked_list_madr_contem_end_marker_apos_transferencia` |
| m2 (SyncMode ignorado, sempre linked-list) | `dma2_block_transfere_dados_para_vram`, `dma2_block_multiplas_palavras`, `dma2_chcr_gravavel_e_legivel` |
| m3 (BS=0 não tratado como 0x10000) | `dma2_block_bs_zero_equivale_a_10000h` |
| m4 (bit24 não limpo após linked-list) | `dma2_chcr_bit24_e_limpo_apos_linked_list` |
| m5 (step ignorado, sempre +4) | **Equivalente** — DMA2 GPU sempre usa step=+4 (bit1=0), ver justificativa no manifesto |
| m6 (bit23=0 tratado como end-marker) | `dma2_linked_list_madr_contem_end_marker_apos_transferencia`, `dma2_linked_list_multiplos_nos` |
| m7 (bit24 não limpo após block) | `dma2_chcr_bit24_e_limpo_apos_block` |

## Placar antes → depois

Workspace: **432** → **446** testes (432 existentes + 14 dma_gpu).

## Revisão cruzada (orquestrador)

<!-- Preenchido pelo Claude na revisão do PR -->

## Decisões e notas

1. **Linked-list executa imediatamente no write do CHCR.** Como não temos scheduler de eventos nem DREQ do GPU, a transferência linked-list e block acontece de forma síncrona durante o `write32` ao CHCR quando bit 24 está setado. Bit 24 é limpo ao final da transferência.
2. **Guarda anti-loop infinito de 4096 nós.** Se a RAM contiver dados que formam um ciclo (next_addr que nunca atinge end-marker), o laço é interrompido após 4096 iterações. Uma lista de comandos GPU real tem tipicamente < 100 nós.
3. **End-marker do linked-list é 0x00FFFFFF.** Conforme `04-dma.md` L208: "Address of the next node (or end marker)". Bit 23 setado indica fim. Compatível com revisões antigas (qualquer addr com bit 23 setado) e novas (todos os bits setados).
4. **BCR para SyncMode=2 não é usado.** Transferência termina apenas ao encontrar o end-marker. BCR deve ser 0.
5. **BCR para SyncMode=1 usa BS (16 bits baixos) e BA (16 bits altos).** BS=0 equivale a 0x10000 palavras, BA=0 equivale a 0x10000 blocos. Total = BS × BA palavras.
6. **MADR é atualizado ao final da transferência.** Para linked-list, MADR contém o end-marker. Para block, MADR contém o endereço após o último bloco.
7. **DMA2 GPU exige GPU configurado previamente.** Para block mode, o GPU precisa estar em estado CpuToVram (comando A0h enviado antes). Para linked-list, os comandos na lista são enviados diretamente para GP0 e processados pela máquina de estados da GPU.
8. **Conexão com IRQ3 pendente.** Os flags de IRQ do DICR (bits 24-30) e a lógica de interrupção DMA → IRQ3 serão conectados quando o scheduler de eventos estiver pronto (itens 3.4+).
