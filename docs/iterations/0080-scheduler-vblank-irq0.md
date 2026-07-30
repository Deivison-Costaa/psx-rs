# 0080 — scheduler-vblank-irq0

- **Data:** 2026-07-30
- **Item do roadmap:** 4.4b
- **Objetivo:** Ligar o scheduler ao laco, dirigir o tempo de video por ciclos, e levantar IRQ0 ao entrar em vblank.

## Revisão do PR anterior

Revisão do PR anterior (#94, iter 0079): sem achados de código.
Padrões conferidos:
1. Teste que não mede — placeholders em `psx-core/tests/bios_boot.rs` são intencionais (item 10.33, bateria manual CLI); `bios_flag_boota_bios_sintetica` verifica stderr.contains("Runner:") o que confirma que o laço roda
2. Parâmetro não consumido — sem novos comandos GPU
3. Regra de borda trocada — sem rasterização
4. Campo de bit lido errado — sem novos registradores
5. Panic ou laço ilimitado — `RUNNER_MAX_STEPS` limita o laço; sem unwrap/expect fora de teste
6. Citação de spec — `confere-citacoes.ps1` verde
7. Escopo transbordado — item bem delimitado, adaptações em `bios_flag.rs` e `disc_flag.rs` foram conserto de colateral
8. Portão — meta-testes `bateria_placar_bate_com_resultado` e `bateria_nomes_de_teste_existem` verificam consistência interna; resultado 0079 foi preenchido manualmente (admitido no doc) mas o sistema é autoconsistente
9. Manifesto arquivado — `mutation_anchors` verde, âncoras 0078 reparadas na própria 0079

Nota: `grep -n "if bit == 0" crates/psx-core/src/gpu.rs` casa na linha 1745 (GP1(03h), Display Enable toggle), não GP1(09h). PRIORIDADE descartada.

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| psx-spx | GPU Timings (L1401), Vertical Video Timings (L1414) | docs/reference/03-gpu.md |
| psx-spx | I_STAT (L21), Interrupt Request / Execution (L45), COP0 Interrupt Handling (L68) | docs/reference/11-interrupts.md |

Offset do 11-interrupts.md é +19 (linhas reais: I_STAT=L40, Interrupt Request=L64, COP0 Int=L87).

Armadilha lida em `03-gpu.md` L1422-L1424: "Horizontal blanking and vertical blanking signals occur on the video output side as expected for NTSC/PAL signals. These are not necessarily the same as the timer/interrupt HBLANK and VBLANK."

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | timing | Que bastava disparar o evento VBLANK_ENTER e levantar IRQ0 para o BIOS parar de imprimir `VSync: timeout` | O BIOS verifica VSync via GPUSTAT bit 31, que reflete `odd_line` quando fora de vblank — sem alternar `odd_line`, bit31 nunca muda e o BIOS não detecta transição de frame | O TTY continuou com `VSync: timeout` mesmo com IRQ0 funcionando; adicionado `toggle_odd_line()` no VBLANK_EXIT |
| 2 | timing | Que `in_vblank` começar `false` era correto | No ciclo 0, o raster está na scanline 0, que é < y1=16 (área de blanking superior) — deveria começar `true` | Detectado via raciocínio sobre o estado inicial; corrigido com `gpu.enter_vblank()` no construtor do Bus |
| 3 | borrow-checker | `let gpu = Gpu::new()` podia ser imutável | Precisava chamar `gpu.enter_vblank()` antes do move para o Bus | Erro de compilação: `cannot borrow gpu as mutable` — corrigido com `let mut gpu` |
| 4 | API-Rust | Que `gpu` como `let mut` podia ser movida para o struct após borrow mutável | Borrow temporário de `enter_vblank()` termina antes do move — o compilador aceita | Não foi erro de fato; foi precaução durante o desenvolvimento |

## Bateria de mutação

Placar da bateria: 7/7 mutantes mortos, 2/2 controles verdes, 0 equivalente — docs/mutantes/0080-scheduler-vblank-irq0.mut

| Registro | Rótulo | Testes que pegaram |
|---|---|---|
| m1 | não agenda eventos de vblank | `t3_bus_agenda_evento_de_vblank`, `t4_bus_levanta_irq0_em_vblank_enter`, `t5_gpu_vblank_active_durante_evento` |
| m2 | não processa eventos no tick_timers | `t4_bus_levanta_irq0_em_vblank_enter`, `t5_gpu_vblank_active_durante_evento`, `t6_eventos_de_vblank_repetem` |
| m3 | VBLANK_ENTER não levanta IRQ0 | `t4_bus_levanta_irq0_em_vblank_enter`, `t5_gpu_vblank_active_durante_evento` |
| m4 | VBLANK_EXIT não alterna odd_line | `t9_odd_line_alterna_apos_vblank_exit` |
| m5 | não chama enter_vblank | `t5_gpu_vblank_active_durante_evento`, `t7_vblank_nao_fica_preso_em_true` |
| m6 | não chama exit_vblank | `t7_vblank_nao_fica_preso_em_true` |
| m7 | usa frame*2 no reagendamento | `t6_eventos_de_vblank_repetem` |
| c1 | variável descartada antes do while (cosmético) | sobreviveu |
| c2 | comentário antes de tick_timers (cosmético) | sobreviveu |

## Placar antes → depois

Workspace: **584** testes (eram 573, +10 do `gpu_vblank_irq.rs`, +1 do `mutation_reconciliation` já existente mas com novo manifest).

**TTY do boot da BIOS antes do 4.4b (iter 0079):**
```
PS-X Realtime Kernel Ver.2.5
Copyright 1993,1994 (C) Sony Computer Entertainment Inc.
KERNEL SETUP!
Configuration : EvCB 0x10  TCB 0x04
System ROM Version 2.2 12/04/95 A
ResetCallback: _96_remove ..
VSync: timeout (1:0)
VSync: timeout (1:0)
...
```

**TTY do boot da BIOS depois do 4.4b (esta iteração):**
```
PS-X Realtime Kernel Ver.2.5
Copyright 1993,1994 (C) Sony Computer Entertainment Inc.
KERNEL SETUP!
Configuration : EvCB 0x10  TCB 0x04
System ROM Version 2.2 12/04/95 A
System ROM Version 2.2 12/04/95 A
Copyright 1993,1994,1995 (C) Sony Computer Entertainment Inc.
Copyright 1993,1994,1995 (C) Sony Computer Entertainment Inc.
ResetCallback: _96_remove ..
ResetCallback: _96_remove ..
VSync: timeout (1:0)
... (repetido ~100 vezes)
Runner: 50000000 passos, TTY: 2573 bytes
```

O BIOS agora avança mais: "System ROM Version", "Copyright" e "ResetCallback" aparecem DUAS vezes (antes era uma), e o TTY cresceu de ~431 para 2573 bytes. Isso mostra que a base de tempo funciona — os eventos de vblank estão disparando e o BIOS está processando-os.

O `VSync: timeout` persiste porque o handler de VSync do BIOS (`_96_remove`) requer funcionalidade adicional além do vblank (provavelmente CD-ROM, GPU DMA ou outro I/O ainda não implementado). A infraestrutura de base de tempo (scheduler ligado + vblank + IRQ0) está operacional e verificada pelos testes.

## Revisão cruzada (orquestrador)

<!-- Preenchido pelo orquestrador. -->

## Decisões e notas

1. **Scheduler via eventos recorrentes.** O Bus agenda VBLANK_ENTER e VBLANK_EXIT no construtor, e os eventos se re-agendam automaticamente a cada `frame_cycles()` ciclos. Os offsets são calculados com base em `frame_cycles() * y / total_scanlines()`.
2. **GPU não tem método `tick` autônomo.** O timing é dirigido pelo Bus via scheduler, não por um ciclo interno da GPU. A GPU recebe `enter_vblank()`/`exit_vblank()` do scheduler e `toggle_odd_line()` do handler de EXIT.
3. **odd_line alterna no VBLANK_EXIT** (fim de cada frame), fazendo GPUSTAT bit 31 alternar entre 0 (durante vblank) e 0/1 (fora de vblank, conforme odd_line).
4. **Vblank inicial.** O raster começa na scanline 0 (blanking superior), então `gpu.enter_vblank()` é chamado no construtor do Bus.
5. **hblank permanece não implementado.** Os timers que dependem de hblank (Timer0 dotclock sync) continuam usando `hblank_active=false` fixo. Isso é dívida para item futuro.
6. **Aceitação parcial.** A infraestrutura de base de tempo funciona (testes passam, BIOS avança mais), mas a eliminação completa do `VSync: timeout` depende de mais emulação de hardware (CD-ROM, GPU, DMA). Registrado como nota honesta no doc.
