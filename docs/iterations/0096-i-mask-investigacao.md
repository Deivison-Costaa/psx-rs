# 0096 — i-mask-investigacao

- **Data:** 2026-07-30
- **Item do roadmap:** 4.4d
- **Objetivo:** Investigar por que I_MASK permanece 0x0000 durante o boot da BIOS e verificar se a BIOS escreve I_MASK.

## Revisão do PR anterior

Revisão do PR #110 (iter 0095): sem achados.

Nove padrões conferidos:
1. Teste que não mede — `handler_retorna_ao_epc_apos_interrupcao` mede corretamente: sem mfc0+jr, handler não retorna ao EPC e t0≠9
2. Parâmetro não consumido — N/A (sem comandos GPU)
3. Regra de borda trocada — N/A (sem rasterização)
4. Campo de bit lido errado — N/A (sem campos de bit de hardware)
5. Panic ou laço ilimitado — guarda max_steps=100, sem unwrap/unsafe
6. Citação de spec — confere-citacoes.ps1 verde, spec_citations verde
7. Escopo transbordado — item 4.4e implementado, manifesto 0093 reparado
8. Portão — bateria 5/5+2/2, resultado versionado
9. Manifesto arquivado — nenhum; 0093 reparado com re-âncora

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| psx-spx | 1F801074h I_MASK - Interrupt mask register (L2) | `docs/reference/11-interrupts.md` |
| psx-spx | Interrupt Acknowledge (L52) | `docs/reference/11-interrupts.md` |

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | timing | Assumi que I_MASK nunca seria escrito pela BIOS, porque os testes com 15M passos mostravam máscara 0x0000 | `docs/reference/11-interrupts.md` L2: I_MASK é R/W e a BIOS o configura durante o boot | Rodei o teste com 50M passos e vi que I_MASK vira 0x0009 entre 15M e 20M passos |
| 2 | instrumentation | Adicionei Vec para logar valores e endereços de cada write_mask, poluindo a struct Irq | Instrumentação mínima basta: um contador público já resolve o diagnóstico | Removi os Vecs e deixei só mask_write_count, que é suficiente para o teste de aceitação |

## Bateria de mutação

Placar da bateria: 5/5 mutantes mortos, 2/2 controles verdes, 0 equivalente — docs/mutantes/0096-i-mask-investigacao.mut

| Registro | Rótulo | Testes que pegaram, conforme o `.resultado` |
|---|---|---|
| m1 | write_mask não incrementa contador | `write_mask_incrementa_contador` |
| m2 | write_mask_byte não incrementa contador | `write_mask_incrementa_contador` |
| m3 | write_mask_half não incrementa contador | `write_mask_incrementa_contador` |
| m4 | contador começa em 1 em vez de 0 | `write_mask_incrementa_contador` |
| m5 | write_mask incrementa em 2 em vez de 1 | `write_mask_incrementa_contador` |
| c1 | comentário inofensivo antes de write_mask | sobreviveu |
| c2 | espaço extra no final de write_mask_half | sobreviveu |

## Placar antes → depois

Workspace: **689** → **690** testes (+1: `write_mask_incrementa_contador`). O teste `bios_escreve_i_mask_durante_boot` é condicional ao arquivo de BIOS e não roda em CI.

## Decisões e notas

1. **I_MASK É escrito pela BIOS com valores não-zero.** A instrumentação revelou 4 escritas em I_MASK durante o boot: duas com valor 0 (inicialização e seção crítica), uma com valor 0x0001 (habilita VBlank) por volta de 19M passos, e uma com valor 0x0009 (VBlank + DMA). O STATUS.md anterior afirmava "I_MASK permanece 0x0000" porque o harness do orquestrador rodava menos passos (provavelmente 10M-15M). Com 30M passos, o critério de aceitação é atingido: I_MASK deixa de ser 0x0000.

2. **O handler de exceção da 0095 foi o que destravou o boot até este ponto.** Sem o mfc0+jr+rfe corrigido, a CPU não retornava ao EPC após interrupções, e a BIOS ficava presa antes de alcançar o código que escreve I_MASK. A 0095 corrigiu o handler e indiretamente permitiu que a BIOS chegasse ao ponto de habilitar interrupções.

3. **"VSync: timeout" persiste.** Mesmo com I_MASK != 0 e I_STAT sendo acknowledged (I_STAT=0x0000 após o handler), a BIOS imprime `VSync: timeout (2:1)`. Isso indica que o handler de interrupção em 0x80000080 despacha para 0x00000C80 e reconhece I_STAT, mas o dispatch de eventos do kernel (que entrega o callback de VSync) não está funcionando corretamente. Este é um item separado, posterior ao 4.4d.

4. **Campo `mask_write_count` adicionado a `Irq`.** Um contador público `u64` que incrementa em `write_mask`, `write_mask_byte` e `write_mask_half`. Útil para diagnóstico de escrita em I_MASK durante o boot. Não afeta o comportamento funcional.

5. **Manifesto 0085 reparado.** As âncoras m2 e m4 do manifesto 0085 quebravam porque `write_mask_half` e `write_mask_byte` ganharam a linha `self.mask_write_count = ...`. Reparadas com o contexto adicional.
