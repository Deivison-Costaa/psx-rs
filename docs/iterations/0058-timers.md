# 0058 — timers

- **Data:** 2026-07-29
- **Item do roadmap:** 3.4
- **Objetivo:** implementar registradores e contagem básica dos três timers do PS1 (CNT/MODE/TARGET), com wrap em FFFFh, reset por target (bit3), flags de IRQ e divisor de clock do timer 2.

## Revisão do PR anterior

Revisão do PR anterior (0057): achado 1 defeito.
- 1. Teste que não mede: `dma2_block_multiplas_palavras` só verificava `assert_ne!(Ready, 0)` sem checar pixels da VRAM. Corrigido nesta rodada — adicionados asserts de `vram_pixel` em 4 posições. OK após correção.
- 2. Parâmetro não consumido → FIFO dessincronizado: sync_mode, BS, BA, step, word_count, next_addr — todos consumidos. OK.
- 3. Regra de borda trocada: sem rasterização. OK.
- 4. Campo de bit lido errado: máscaras de next_addr (0x00FF_FFFF, 0x0080_0000), bit24 start, bit1 step — todas corretas. OK.
- 5. Panic ou laço ilimitado: guarda de 4096 nós no linked-list; bounds check antes de leitura de RAM. OK.
- 6. Citação de spec: confere-citacoes.ps1 verde. OK.
- 7. Escopo transbordado ou dívida não declarada: item 3.3 implementou exatamente DMA2 GPU block + linked-list. Simplificações registradas. OK.

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| psx-spx | Timers (L17) | docs/reference/05-timers.md |
| psx-spx | 1F801100h+N\*10h - Timer Current Counter (L18) | docs/reference/05-timers.md |
| psx-spx | 1F801104h+N\*10h - Timer Counter Mode (L30) | docs/reference/05-timers.md |
| psx-spx | 1F801108h+N\*10h - Timer Target Value (L70) | docs/reference/05-timers.md |
| psx-spx | Reset and Wrap (L88) | docs/reference/05-timers.md |

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | API-Rust | Que `read32` de MODE poderia limpar flags sem `&mut self` | Métodos com `&self` não podem modificar estado; GPU usa `Cell` para mutabilidade interior | erro de compilação `cannot borrow self.timers as mutable, as it is behind a & reference`; resolvido com `Cell<u32>` para o campo mode |
| 2 | ordem-de-operação | Que escrever CNT antes de MODE preservava o valor | Spec L24: "It gets forcefully reset to 0000h on any write to the Counter Mode register" — MODE sempre reseta CNT | testes falharam porque CNT era resetado ao escrever MODE; corrigido escrevendo MODE antes de CNT nos testes |
| 3 | flags-de-IRQ | Que flags bits 11 e 12 só seriam setados com IRQ enable (bits 4/5) | Bits 11 e 12 são flags de status independentes dos bits de enable; são setados sempre que o evento ocorre e limpos na leitura | bateria de mutação: m7 (bit11 sem reset_on_target) sobreviveu porque o teste `flag_ffff_alcancado` não verificava que bit11 ficava em 0; fortalecido com assert extra |
| 4 | borrowing | Que o `write32` do bus passava `&self` para `Timers::read32` | `region_read32` do bus usa `&self`; `read32` do Timers precisa limpar flags na leitura → requer mutabilidade | erro de compilação; resolvido com `Cell` no campo mode |

## Bateria de mutação

Placar da bateria: 7/7 mutantes mortos, 2/2 controles verdes, 0 equivalente - ./docs/mutantes/0058-timers.mut

| Mutante | Teste que o pegou |
|---|---|
| m1 (tick não incrementa contador) | `tick_incrementa_cnt_modo_system_clock`, `cnt_wrap_em_ffff_sem_target`, `cnt_reseta_no_target_com_bit3_setado` |
| m2 (MODE não reseta CNT) | `escrever_mode_reseta_cnt` |
| m3 (bit3 ignorado, sempre reseta em FFFFh) | `cnt_reseta_no_target_com_bit3_setado` |
| m4 (flags bits 11/12 nunca limpos na leitura) | `flag_target_alcancado_setado_e_limpo_na_leitura`, `flag_ffff_alcancado_setado_e_limpo_na_leitura` |
| m5 (timer 2 divisor ignorado) | `tick_respeita_divisor_de_clock_do_timer2` |
| m6 (sync enable não pausa timer 2) | `timer2_modo_0_sync_ativo_para_contador` |
| m7 (bit11 setado sem reset_on_target) | `flag_ffff_alcancado_setado_e_limpo_na_leitura` (assert de bit11=0) |

## Placar antes → depois

Workspace: **446** → **459** testes (446 existentes + 13 timers).

## Revisão cruzada (orquestrador)

<!-- Preenchido pelo Claude na revisão do PR -->

## Decisões e notas

1. **Ticking manual via `tick(base_addr, cycles)`.** Sem scheduler integrado, o avanço dos timers é feito por chamada explícita. Cada chamada processa `cycles` pulsos de system clock, respeitando o divisor do timer 2.
2. **Modos de sincronização simplificados.** Sem sinais de Hblank/Vblank, os modos de sync são tratados como: timer 0/1 modos 2 e 3 = pausado; timer 2 modos 0 e 3 = parado permanentemente. Os modos restantes operam como free run. Refinamento pendente de H/Vblank (item 3.4b).
3. **Fonte de clock simplificada.** Apenas system clock é suportada nesta iteração (bits 8-9 = 0 ou 2 para timer 0/1, 0 ou 1 para timer 2). Dotclock e Hblank como fonte serão implementados em iteração futura (item 3.4c).
4. **IRQ não conectada ao controlador.** Os flags bits 10-12 do MODE são mantidos corretamente (set no evento, clear na leitura), mas a geração de IRQ4/IRQ5/IRQ6 e a conexão com o `IrqController` ficam para item 3.4d.
5. **Acumulador de ciclos por timer.** Cada timer mantém um `cycle_acc` interno para suportar o divisor de clock do timer 2 (system clock/8). Ciclos não inteiros são acumulados entre chamadas de `tick`.
6. **Cell para mutabilidade interior.** O campo `mode` usa `Cell<u32>` (padrão da GPU) para permitir que `read32(&self)` limpe os flags bits 11-12 sem exigir `&mut self` no bus.
