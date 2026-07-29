# 0061 — timers-irq

- **Data:** 2026-07-29
- **Item do roadmap:** 3.4d
- **Objetivo:** conectar as saídas de IRQ dos timers (bit10 do MODE + flags de target/FFFF) ao interrupt controller — pulse/toggle, one-shot/repeat, IRQ4(timer0)/IRQ5(timer1)/IRQ6(timer2).

## Revisão do PR anterior (0060)

Revisão do PR anterior: sem achados

1. Teste que não mede — todos os testes têm valores específicos; único `assert!(cnt > 0)` é precedido de `assert_eq!(cnt, 0)`
2. Parâmetro não consumido — N/A (timers não têm FIFO GP0)
3. Regra de borda trocada — N/A (sem rasterização)
4. Campo de bit lido errado — máscaras de clock/sync/bit10 corretas; bits 11-12 limpos em `read32`
5. Panic/laço — sem unwrap/unsafe; índice protegido com `& 0x3`; laço com `effective` limitado
6. Citação de spec — `confere-citacoes.ps1` verde
7. Escopo transbordado — mudanças restritas a 3.4c (dotclock/Hblank)

Achado pré-existente (não do 0060): flag target (bit11) só setado dentro de `if reset_on_target`, mas spec diz que deve ser independente. Corrigido nesta iteração.

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| psx-spx | Counter Mode bits 4-7,10-12 | docs/reference/05-timers.md |
| psx-spx | I_STAT/I_MASK, edge-triggered | docs/reference/11-interrupts.md |

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | flag | Que o flag target (bit11) podia continuar dependente de `reset_on_target` | O flag é independente de bit3 — sempre setado quando counter==target | Teste `flag_ffff_alcancado_setado_e_limpo_na_leitura` falhou porque target=0 disparava bit11 no wrap, revelando que o flag agora é incondicional |
| 2 | teste | Que o teste `flag_ffff` não precisava de target explícito (já salvo pelo `reset_on_target` antigo) | Com o flag incondicional, target padrão 0 colide com wrap FFFF→0 | Teste falhou — corrigido adicionando `T0_TARGET = 0x0007` explícito |
| 3 | teste | Que os testes de toggle podiam usar one-shot (0x0090) e esperar múltiplos toggles | Em one-shot toggle, bit10 "remains zero after the IRQ" — não volta a 1 | Teste `irq_toggle_inverte_bit10` falhou — corrigido para repeat mode (0x00D0) |
| 4 | teste | Que `irq4_timer0...irq6_timer2` funcionava escrevendo CNT antes de MODE | Escrever MODE reseta o contador a 0, destruindo o valor escrito antes | Teste reescrito: define target e usa tick suficiente via clock/8 |
| 5 | API-Rust | Que `tick()` retornando `Option<u32>` não quebraria ancoras | Os `return;` no corpo precisam virar `return None;` e a assinatura muda | Erro de compilação + âncora K2 do manifesto 0060 quebrou (`cargo fmt` multi-linha) |

## Bateria de mutação

Placar da bateria: 7/7 mutantes mortos, 2/2 controles verdes, 0 equivalente - ./docs/mutantes/0061-timers-irq.mut

| Mutante | Teste que o pegou |
|---|---|
| m1 (IRQ target enable ignorado) | `irq_target_pulse_retorna_irq_bit4` |
| m2 (IRQ FFFF enable ignorado) | `irq_ffff_pulse_retorna_irq_bit4` |
| m3 (one-shot não suprime segunda IRQ) | `irq_oneshot_nao_dispara_segunda_vez` |
| m4 (pulse mode não restaura bit10) | `irq_target_pulse_retorna_irq_bit4` |
| m5 (toggle mode restaura bit10 como pulse) | `irq_toggle_inverte_bit10_e_so_retorna_irq_na_descida` |
| m6 (flag target regride a reset_on_target) | `irq_target_pulse_retorna_irq_bit4` |
| m7 (IRQ retorna bit fixo 4) | `irq4_timer0_irq5_timer1_irq6_timer2` |

## Placar antes → depois

Workspace: **479** → **493** testes (479 existentes + 14 timers_irq).

## Revisão cruzada (orquestrador)

<!-- Preenchido pelo Claude na revisão do PR -->

## Decisões e notas

1. **`tick()` agora retorna `Option<u32>`:** `Some(4+idx)` quando ocorre borda 1→0 em bit10, `None` caso contrário. O caller deve propagar para `irq.raise(bit)`. O acoplamento é mínimo — o timer não conhece o IRQ controller.
2. **Flag target (bit11) independente de reset_on_target:** removida a guarda `if reset_on_target` do flag. Agora `bit11` é setado sempre que counter==target, conforme `05-timers.md` L44 ("IRQ when Counter=Target (0=Disable, 1=Enable)"). O `reset_on_target` (bit3) controla apenas o reset do contador.
3. **Restauração de bit10 em pulse mode:** ao final de `tick()`, se `!irq_toggle`, bit10 é restaurado a 1. Em toggle mode, bit10 mantém o último valor togglado. Conforme `05-timers.md` L63-66.
4. **One-shot toggle permanece em 0:** `05-timers.md` L66: "in one-shot mode, it remains zero after the IRQ". Confirmado pelo teste `irq_oneshot_toggle_nao_dispara_segunda_vez`.
5. **Ordem target-depois-FFFF:** o loop verifica target primeiro, depois FFFF. Se counter==0 e target==0 após wrap, ambos disparam — em one-shot, o target suprime o FFFF.
6. **Âncora K2 do manifesto 0060 reparada:** `cargo fmt` reformatou a assinatura de `tick()` para multi-linha, quebrando a âncora. Corrigida para o formato atual.
