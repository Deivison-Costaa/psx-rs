# 0085 — bios-i-mask

- **Data:** 2026-07-30
- **Item do roadmap:** 4.4c
- **Objetivo:** Corrigir acesso por byte e halfword a I_STAT e I_MASK — write16/write8 descartados silenciosamente impediam BIOS de escrever I_MASK via SH.

## Revisão do PR anterior

Revisão do PR #99 (iter 0084): achado 1 defeito.

Nove padrões conferidos:
1. Teste que não mede — `cfc2_tem_load_delay_de_uma_instrucao` usava `assert_ne!(reg[9], 0)` (não afirmava valor); corrigido para `assert_eq!`
2. Parâmetro não consumido — sem novos comandos GPU na 0084
3. Regra de borda trocada — sem rasterização
4. Campo de bit lido errado — GTE registradores: sign-extension só para standalone S16 (4,12,20,27,29,30), conferido
5. Panic ou laço ilimitado — acesso a array limitado por encoding de instrução (5 bits); sem unwrap/unsafe fora de teste
6. Citação de spec — `confere-citacoes.ps1` verde no main
7. Escopo transbordado — spec_citation_index.rs era continuação do PR #98, necessário para CI
8. Portão — `.resultado` rastreado, mutation_anchors verde
9. Manifesto arquivado — sem arquivamentos

### Prioridade GP1(09h)

O guard `grep -n "if bit == 0" crates/psx-core/src/gpu.rs` acusou match na linha 1747, mas a linha está no braço **GP1(03h)** (Display Enable), não em GP1(09h). O handler de GP1(09h) (linha 1781) corretamente só seta `allow_upper_y` sem tocar GPUSTAT. O teste `gp1_09h_bit0_zero_proibe_gpustat_15` foi renomeado/corrigido na iter 0076. O defeito original (GP1(09h) manipulando GPUSTAT.15) já estava consertado. O guard de grep era mais largo que o defeito.

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| psx-spx | 1F801074h I_MASK — Interrupt mask register (L22), Interrupt Request / Execution (L45), COP0 Interrupt Handling (L68) | docs/reference/11-interrupts.md |

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | I/O-byte | BIOS usaria `sw` (32-bit) para escrever I_MASK | BIOS pode usar `sh` (16-bit), que era descartado silenciosamente por `region_write_byte` | revisão do código: write16 chamava region_write_byte que retornava true para 0x1F801074 mas não escrevia nada |
| 2 | valor-teste | Valor 0xFFFA tem bit 2=0, então limpa bit 2 que o teste esperava preservar | 0xFFFE preserva bit 2 (bit 2=1) | teste falhou: stat voltou 0 em vez de 4 |
| 3 | manifesto-mutacao | Mutantes m2-m5 seriam mortos pelos testes existentes | Testes não exercitavam offset != 0, valor previo em mask, byte alto do halfword | bateria de mutação: 4/5 sobreviveram na primeira rodada |

## Bateria de mutação

Placar da bateria: 5/5 mutantes mortos, 2/2 controles verdes, 0 equivalente — docs/mutantes/0085-bios-i-mask.mut

| # | Tipo | Rótulo | Resultado |
|---|---|---|---|
| m1 | mutante | write_stat_half substitui stat em vez de AND com mascara | MORREU |
| m2 | mutante | write_mask_half nao limpa bits antes de OR | MORREU |
| m3 | mutante | write_stat_byte shift dobrado | MORREU |
| m4 | mutante | write_mask_byte substitui mask inteira | MORREU |
| m5 | mutante | write_stat_half trunca val para u8 | MORREU |
| c1 | controle | comentario antes de write_stat_half | verde |
| c2 | controle | comentario antes de write_mask_half | verde |

## Placar antes → depois

Workspace: **608** → **613** testes (+5: irq_halfword — eram 9 mas o fix moveu de stub para 13, net +4; +1 gte_registers corrigido para assert_eq).

## Decisões e notas

1. **SH para I_MASK era descartado.** `write16` chamava `region_write_byte` que para 0x1F801024..0x1F801FFF retornava `true` sem escrever nada. A BIOS usa `sh` para setar I_MASK e o write era engolido — I_MASK permanecia 0x0000, IRQ0 nunca vetorava, boot parava em `VSync: timeout`.

2. **Correção adiciona dispatch por endereço em write16/write8/read16/read8.** Antes de cair no `region_write_byte` genérico, endereços de I_STAT (0x1F801070-0x1F801073) e I_MASK (0x1F801074-0x1F801077) são roteados para métodos específicos de Irq que aplicam a semântica correta (write 0=clear, write 1=preserve para I_STAT; write direto para I_MASK).

3. **Apenas I_STAT e I_MASK.** Outros registradores no range 0x1F801024..0x1F801FFF (timers, DMA, GPU, CDROM) continuam com o comportamento antigo para byte/halfword. A correção é incremental — o escopo desta iteração é destravar o boot da BIOS.

4. **Teste existente atualizado.** `bus_scratchpad_isc::io_catch_all_nao_corrompe_ram` esperava que read8/read16 de I_MASK retornassem 0 (comportamento de stub). Atualizado para os valores reais (byte baixo e halfword da mascara).
