# 0075 — gpu-mask-bit-transfer

- **Data:** 2026-07-29
- **Item do roadmap:** 10.22 (sobreposto a 10.7)
- **Objetivo:** aplicar mask-bit (force-bit15 e write-protect) a transferências CPU→VRAM, fechando as 2 falhas de `gpu/mask-bit`.

## Revisão do PR anterior

Revisão do PR anterior (#88, iter 0074): sem achados. Os nove padrões conferidos:
1. Teste que não mede — bateria 6/6 confirma cobertura; testes com asserções contra valores concretos
2. Parâmetro não consumido — GP1(09h) é register write, sem FIFO; E1h já existia
3. Regra de borda trocada — sem rasterização nova
4. Campo de bit lido errado — E1h bit11 → GPUSTAT.15 mapeamento correto; GP1(09h) bit0 correto
5. Panic ou laço ilimitado — sem arrays, loops, unwrap/expect/unsafe
6. Citação de spec — `confere-citacoes.ps1` verde
7. Escopo transbordado — `apply_texpage_if_second` corrigido (necessário para as 3 falhas); GP1(00h) reset incluído
8. Portão que não mede — bateria rodada com 6/6 mortos, `.resultado` rastreado
9. Manifesto arquivado — 0050-display-regs.mut reparado (âncora atualizada com GP1(09h))

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| psx-spx | GP0(E6h) - Mask Bit Setting (L578) | docs/reference/03-gpu.md |
| psx-spx | GPUSTAT bits 11-12 (L1010-1011) | docs/reference/03-gpu.md |

A spec de GP0(E6h) (L578-593) diz: bit0 força bit15=1, bit1 protege pixels com bit15=1. Aplica-se a
"all rendering commands, as well as CPU-to-VRAM and VRAM-to-VRAM transfer commands (where it
acts on the separate halfwords, ie. as for 15bit textures)". Mask NÃO afeta Fill-VRAM.

A implementação original (iter 0049, item 2.6c) aplicava mask-bit apenas em `write_pixel`
(renderização de polígonos/linhas/retângulos). As transferências CPU→VRAM escreviam diretamente
na VRAM sem passar por `write_pixel`, ignorando o mask-bit.

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | cobertura-bateria | Que os testes T8-T10 originais bastavam para fechar a bateria | Dois mutantes sobreviveram: m1 (bit trocado no check_mask) e m5 (write-protect removido do hw2) | Bateria deu 3/6; T11 e T12 adicionados fecharam os buracos |
| 2 | ordem-operacoes | Que a ordem de force-bit15 vs write-protect importava | São operações independentes: check lê pixel antigo, force modifica pixel novo | m6 sobreviveu como esperado; reclassificado como equivalente |

## Bateria de mutação

Placar da bateria: 5/5 mutantes mortos, 2/2 controles verdes, 1 equivalente — docs/mutantes/0075-gpu-mask-bit-transfer.mut

| Registro | Rótulo | Testes que pegaram, conforme o `.resultado` |
|---|---|---|
| m1 | write-protect verifica GPUSTAT.11 em vez de GPUSTAT.12 (bit trocado) | `t11_force_bit15_sobrescreve_pixel_protegido_sem_write_protect` |
| m2 | force-bit15 usa GPUSTAT.12 em vez de GPUSTAT.11 (bit trocado) | `t9_cpu_para_vram_force_bit15_seta_bit15` |
| m3 | CPU→VRAM nunca aplica write-protect (remove guarda) | `t8_cpu_para_vram_respeita_write_protect`, `t12_write_protect_protege_segundo_halfword_individualmente` |
| m4 | CPU→VRAM nunca aplica force-bit15 (remove OR 0x8000) | `t9_cpu_para_vram_force_bit15_seta_bit15`, `t11_force_bit15_sobrescreve_pixel_protegido_sem_write_protect` |
| m5 | write-protect aplica a hw1 mas nao a hw2 (remove guarda só do hw2) | `t12_write_protect_protege_segundo_halfword_individualmente` |
| m6 | mask-bit ANTES do write-protect (equivalente) | sobreviveu — ordem não afeta resultado |
| c1 | renomeia check_mask → mask_enabled (cosmético) | sobreviveu |
| c2 | inverte condição do if (De Morgan, cosmético) | sobreviveu |

## Placar antes → depois

Workspace: **559** → **561** testes (+2: T11 e T12). T8 renomeado e com asserção corrigida.

**Hardware — `gpu/mask-bit`:** saiu de **3p/2f** para **5p/0f**. Os dois subtestes que falhavam:
- `testSetBit`: force-bit15 não aplicado a CPU→VRAM
- `testCheckMaskBit`: write-protect não aplicado a CPU→VRAM

## Revisão cruzada (orquestrador)

<!-- Preenchido pelo orquestrador. -->

## Decisões e notas

1. O mask-bit em transferências é aplicado **por halfword individualmente**, conforme a spec:
   cada halfword de 16 bits dentro da palavra de 32 bits é tratada independentemente.
2. A ordem das operações é: **check-protect (lê VRAM) → force-bit15 (modifica pixel local) → write**.
   Inverter a ordem não afeta o resultado porque são operações independentes — o check lê o estado
   atual da VRAM, não o pixel modificado.
3. O manifesto 0038-vram-transfers.mut (mutante `e`) teve a âncora reparada: a adição do
   mask-bit expandiu o bloco `if remaining > 0`, quebrando o casamento do padrão `@@DE`/`@@PARA`.
4. VRAM→VRAM copy (top3=4) permanece um no-op (`SkipParams`); implementá-lo com mask-bit é o
   item 10.7 propriamente dito, enquanto 10.22 fechou apenas o impacto prático (os 2 subtestes).
5. O arquivo de teste `gpu_mask_bit.rs` foi de 196 para 284 linhas (ainda dentro do teto de 500).
