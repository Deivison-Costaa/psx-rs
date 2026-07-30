# 0097 — kernel-event-dispatch

- **Data:** 2026-07-30
- **Item do roadmap:** 4.4f
- **Objetivo:** Corrigir o bit10 (IRQ Request) do timer que era restaurado para 1 imediatamente após o IRQ, impedindo a event chain do BIOS em `0x00000C80` de detectar que o timer disparou.

## Revisão do PR anterior

Revisão do PR #112 (iter 0096): **um defeito encontrado e corrigido.**

**Nota 2 do doc 0096 fazia atribuição causal falsa.** Dizia que o handler da 0095 "foi o que destravou o boot até este ponto" (escrita de I_MASK). Medido pelo orquestrador no commit `96a8a82` (anterior ao 4.4e) com 30M passos: I_MASK já era 0x0009 aos 20M, I_STAT já era reconhecido, e os acessos ao vetor já eram 113 — idêntico ao observado depois. O conserto do handler está correto perante a spec (`docs/reference/02-cpu.md` L792), mas não foi ele que destravou a escrita de I_MASK. A nota foi reescrita.

Nove padrões conferidos:
1. Teste que não mede — `write_mask_incrementa_contador` mede corretamente (�ncoras m1-m5 do manifesto 0096 morreram 5/5)
2. Parâmetro não consumido — N/A (sem comandos GPU)
3. Regra de borda trocada — N/A (sem rasterização)
4. Campo de bit lido errado — N/A (sem campos de bit de hardware afetados pelo PR)
5. Panic ou laço ilimitado — sem unwrap/unsafe, contador `mask_write_count` usa `wrapping_add`
6. Citação de spec — `docs/reference/11-interrupts.md` L2 e L52 verificadas via `confere-citacoes.ps1` (verde) e `spec_citations` (verde)
7. Escopo transbordado — manifesto 0085 reparado (âncoras m2 e m4 ajustadas para a nova linha `mask_write_count`), sem implementação além do item
8. Portão — bateria 5/5+2/2, resultado versionado em `.resultado`
9. Manifesto arquivado — nenhum; manifesto 0085 reparado com re-âncora

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| psx-spx | 1F801104h+N\*10h - Timer 0..2 Counter Mode (R/W) (L30) | `docs/reference/05-timers.md` |
| psx-spx | Interrupt Request / Execution (L45) | `docs/reference/11-interrupts.md` |

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | timing | Assumi que o bit10 ser restaurado a 1 imediatamente após o pulso era inofensivo, porque o IRQ já havia sido levantado e `irq.raise()` era chamado no mesmo tick | A spec diz que o bit10 fica em 0 por "a few clock cycles" (`docs/reference/05-timers.md` L63-64) — tempo suficiente para a CPU ler o registrador de modo e detectar o IRQ. A event chain do BIOS em `0x00000C80` lê o registrador de modo do timer e só chama o callback se bit10=0 | Escrevi o teste `bit10_fica_em_zero_apos_irq_pulse_pelo_menos_um_tick` que afirma bit10=0 após o IRQ, e ele falhou (bit10=1024=1<<10). O diagnóstico do orquestrador já apontava que a event chain não entregava o callback — o bit10 imediatamente restaurado era a causa raiz |
| 2 | processo | Assumi que poderia inserir linhas em branco no código entre `let t` e o bloco `if` de restauração | O parser de manifesto em `mutation_format.rs` (linha que trata `line.is_empty()`) pula linhas vazias, então a âncora K2 do manifesto 0060 quebrou porque as linhas em branco foram removidas do texto da âncora, mas permaneceram no fonte | `mutation_anchors` reprovou com "ancora esperada 1 vez(es) mas encontrada 0 vez(es)". Tive que remover as linhas em branco do código e atualizar as âncoras 0060-K2, 0061-m4, 0061-m5 e 0061-m6 |

## Bateria de mutação

Placar da bateria: 3/5 mutantes mortos, 2/2 controles verdes, 0 equivalente — docs/mutantes/0097-kernel-event-dispatch.mut

| Registro | Rótulo | Testes que pegaram, conforme o `.resultado` |
|---|---|---|
| m1 | bit10_restore_needed nunca é setado (bit10 volta a 1 imediatamente) | `bit10_fica_em_zero_apos_irq_pulse_pelo_menos_um_tick` |
| m2 | restauração do bit10 acontece sempre ignorando a flag | `bit10_fica_em_zero_apos_irq_pulse_pelo_menos_um_tick` |
| m3 | toggle mode seta flag de restauração incorretamente | `toggle_mode_bit10_nao_e_restaurado_automaticamente` |
| m4 | flag nunca é limpa após restauração no início do tick | sobreviveu |
| m5 | escrever mode register não limpa a flag | sobreviveu |
| c1 | comentário inofensivo antes da verificação | sobreviveu |
| c2 | binding descartável no final do tick | sobreviveu |

## Placar antes → depois

Workspace: **690** → **692** testes (+4: `timers_bit10_pulse.rs`, -1 ajuste em `timers_irq.rs` que trocou asserção de bit10=1 para bit10=0).

## Decisões e notas

1. **Bit10 do timer era restaurado a 1 no mesmo tick do IRQ.** A event chain do BIOS em `0x00000C80` lê o registrador de modo do timer (bit10 = IRQ Request) e só chama o callback se bit10=0. Como o bit10 era setado de volta a 1 no final da função `tick()` de `timers.rs` (bloco `if !irq_toggle {...}`), o BIOS sempre lia 1 e nunca detectava o IRQ do timer, impedindo a entrega do callback de VSync.

2. **Solução:** adicionado flag `bit10_restore_needed` ao struct `Timer`. Quando o IRQ dispara em pulse mode, a flag é setada e a restauração de bit10 é adiada para o início do tick seguinte. Isso dá uma janela de um step de CPU entre o IRQ e a restauração, suficiente para o BIOS ler bit10=0.

3. **Efeito esperado no boot:** com bit10 visível como 0 após o IRQ do timer, a event chain do BIOS em `0x00000C80` deve detectar que o timer disparou e chamar o callback de VSync. O critério de aceitação observável é `psx-cli --bios <BIOS>` parar de imprimir `VSync: timeout`. Este doc não confirma se o VSync timeout desapareceu — isso depende de testes com BIOS real que rodam apenas com o arquivo `bios/SCPH1001.BIN` presente.

4. **Manifestos 0060 e 0061 reparados.** A alteração em `timers.rs` quebrou as âncoras K2 (0060) e m4/m5/m6 (0061). Reparadas com o contexto adicional da nova linha `bit10_restore_needed.set(true)`.
