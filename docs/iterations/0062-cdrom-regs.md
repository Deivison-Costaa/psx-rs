# 0062 — cdrom-regs

- **Data:** 2026-07-29
- **Item do roadmap:** 4.1
- **Objetivo:** registradores INDEX0-3 do CDROM + comandos GetStat(19h,20h/21h), GetID(1Ah), Test/Init(0Ah) — suficiente para a BIOS detectar o drive.

## Revisão do PR anterior (0061 — timers-irq)

1. Teste que não mede — todos os testes têm valores específicos da spec; sem round-trip
2. Parâmetro não consumido — N/A (timers não têm FIFO GP0)
3. Regra de borda trocada — N/A (sem rasterização)
4. Campo de bit lido errado — máscaras de clock/sync/bit10 corretas; bits 11-12 limpos em read32
5. Panic/laço — sem unwrap/unsafe; índice protegido com `& 0x3`; `effective` bounded
6. Citação de spec — `confere-citacoes.ps1` verde
7. Escopo transbordado — **encontrado**: `tick()` retorna `Option<u32>` com o bit de IRQ, mas a conexão com `irq.raise(bit)` + I_STAT nunca foi feita no código de produção. Nenhum caller de produção chama `tick()`. **Consertado nesta rodada:** adicionado `bus.tick_timers(cycles)` que itera os três timers e propaga IRQs, e chamado ao final de `cpu.step()`.

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| psx-spx | INDEX0-3 (HSTS, COMMAND, PARAMETER, RESULT, HINTSTS, HINTMSK, HCLRCTL) | docs/reference/06-cdrom.md |
| psx-spx | GetStat 19h,20h/21h — data/versão e flags | docs/reference/06-cdrom.md |
| psx-spx | GetID 1Ah — primeira resposta INT3, segunda INT2/INT5 | docs/reference/06-cdrom.md |
| psx-spx | Init 0Ah — INT3(stat) → INT2(stat) | docs/reference/06-cdrom.md |

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | API-Rust | Que `read8` poderia ser `&self` mesmo consumindo o result FIFO | O result FIFO é consumido na leitura (pop), exigindo `&mut self` | Erro de compilação ao chamar `pop_front()` em `&self` — refatorado para `Cell` com interior mutability |
| 2 | API-Rust | Que `VecDeque` era compatível com interior mutability via `Cell` | `VecDeque` não implementa `Copy`; precisa de buffer `[u8; 16]` com índice de head/count | Erro de compilação — reimplementado com `Cell<[u8; 16]>` + `Cell<u8>` para head e count |
| 3 | teste | Que INDEX3 lido no bank 0 retornava HINTSTS | HINTSTS está nos banks 1/3; bank 0/2 retorna HINTMSK | Teste `init_command_dispara_int3_e_depois_int2` retornava 0 em vez de 3 — corrigido trocando para bank 1 |
| 4 | teste | Que `irq_pending` bastava testar com INTMSK setado | Mutante m7 removeu `& self.intmsk` e nenhum teste verificava INTMSK=0 com INTSTS≠0 | Teste `irq_nao_pendente_quando_intmsk_nao_cobre_intsts` adicionado — mutante m7 morreu |
| 5 | teste | Que o result FIFO ficava vazio após acknowledge do INT3 do Init | Init entrega INT2 com stat (0x02) no result FIFO, então RSLRRDY fica 1 | Teste `result_fifo_esvaziado_apos_acknowledge_da_int3` falhou — corrigido para ler a segunda resposta antes de verificar RSLRRDY=0 |

## Bateria de mutação

Placar da bateria: 7/7 mutantes mortos, 2/2 controles verdes, 0 equivalente - ./docs/mutantes/0062-cdrom-regs.mut

| Mutante | Teste que o pegou |
|---|---|
| m1 (PRMEMPT invertido) | `index0_read_retorna_status_inicial` |
| m2 (GetStat 20h retorna 0x01 em vez de 0x97) | `getstat_20h_retorna_data_e_versao` |
| m3 (Init sem segunda resposta) | `init_command_dispara_int3_e_depois_int2` |
| m4 (GetID retorna INT2 em vez de INT5) | `getid_sem_disco_retorna_int5` |
| m5 (result FIFO não avança head) | `result_fifo_leitura_retorna_padding_zero_apos_esvaziar` |
| m6 (CLRPRM não limpa param FIFO) | `escrever_mode_reseta_param_fifo` |
| m7 (IRQ pending ignora INTMSK) | `irq_nao_pendente_quando_intmsk_nao_cobre_intsts` |

## Placar antes → depois

Workspace: **493** → **506** testes (493 existentes + 12 cdrom_regs + 1 irq_nao_pendente).

## Revisão cruzada (orquestrador)

<!-- Preenchido pelo Claude na revisão do PR -->

## Decisões e notas

1. **Interior mutability com `Cell`:** `read8` consome o result FIFO (pop), mas é chamado de `region_read_byte` que recebe `&self`. Usamos `Cell<[u8; 16]>` para os buffers e `Cell<u8>` para head/count, seguindo o padrão do GPU e timers.
2. **Respostas do CDROM modeladas como enum interno:** `SecondResponse::Init` e `SecondResponse::GetId` controlam a entrega da segunda resposta (INT2/INT5) após acknowledge da primeira (INT3).
3. **GetStat(19h) consome o primeiro byte do param FIFO como sub-comando** (20h = data/versão, 21h = flags) e limpa o restante do FIFO.
4. **GetID sem disco:** primeira resposta INT3(stat=02h), segunda INT5(08h,40h,00h,00h,00h,00h,00h,00h). Disco licenciado (não implementado) retornaria INT2 com string "SCEx".
5. **Init (0Ah):** INT3(stat=02h) → acknowledge → INT2(stat=02h). Se um Init já está em andamento (segunda resposta pendente), um novo Init é aceito normalmente (spec diz "silently dropped" — dívida para precisão futura).
6. **HCLRCTL:** escrever `0x07` limpa os bits 0-2 do INTSTS e dispara a segunda resposta pendente. Escrever `0x40` (CLRPRM) limpa o param FIFO.
7. **HINTMSK:** defaults a 0; a BIOS escreve `0x1F` para habilitar todas as IRQs.
