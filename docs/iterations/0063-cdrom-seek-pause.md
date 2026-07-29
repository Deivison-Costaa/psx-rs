# 0063 — cdrom-seek-pause

- **Data:** 2026-07-29
- **Item do roadmap:** 4.2a
- **Objetivo:** Setloc, SeekL e Pause com estado do drive — motor, seeking, BCD validation, INT3/INT2/INT5.

## Revisão do PR anterior (0062 — cdrom-regs)

1. **Teste que não mede** — encontrados dois gaps: (a) `getstat_20h_retorna_data_e_versao` e `getstat_21h_retorna_flags_dos_switches` não verificavam HINTSTS após comando Test(0x19) — remover `self.intsts.set(3)` do handler sobreviveria; (b) `init_command_dispara_int3_e_depois_int2` e `result_fifo_esvaziado_apos_acknowledge_e_leitura_da_segunda_resposta` liam o stat da INT3 do Init sem assert no valor. **Consertado:** adicionados asserts de HINTSTS nos testes GetStat e assert de stat bit1 nos testes Init.
2. **Parâmetro não consumido** — Setloc (3 params), SeekL (0 params), Pause (0 params) — cada comando novo conta as palavras por combinação de bits e FIFO alinhado após. Ok.
3. **Regra de borda trocada** — N/A (sem rasterização).
4. **Campo de bit lido errado** — BCD validation usa `(ss & 0xF0) < 0x60` e `(ff & 0xF0) < 0x70 && (ff & 0x0F) < 0x0A`. Confirmado contra spec: "ass < 60h and asect < 75h". Ok.
5. **Panic/laço ilimitado** — sem unwrap/unsafe; índices de param pop protegidos por `count == 0 → return 0`. Ok.
6. **Citação de spec** — `confere-citacoes.ps1` verde.
7. **Escopo transbordado** — item 4.2 original juntava parser BIN/CUE + 5 comandos + máquina de estados ReadN. Dividido em 4.2a (Setloc/SeekL/Pause), 4.2b (parser BIN/CUE), 4.2c (ReadN/ReadS). A subdivisão está no ROADMAP.

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| psx-spx | Setloc - Command 02h (L787) | docs/reference/06-cdrom.md |
| psx-spx | SeekL - Command 15h (L800) | docs/reference/06-cdrom.md |
| psx-spx | Pause - Command 09h (L750) | docs/reference/06-cdrom.md |
| psx-spx | Status code (stat) (L988) | docs/reference/06-cdrom.md |
| psx-spx | Command Summary (L546) | docs/reference/06-cdrom.md |

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | API-Rust | Que `cargo test --test` aceitava múltiplos targets separados por espaço (`cdrom_seek_pause cdrom_regs`) | O campo `teste` no manifesto alimenta `--test` que aceita UM target | Bateria de mutação deu erro em todos os 9 registros — corrigido para `teste: cdrom_seek_pause` |
| 2 | Teste | Que Init sempre retornava stat=0x02 (hardcoded) | Init retorna `stat_byte()` do drive; sem disco, motor está desligado e stat bit1=0 | Testes antigos `init_command_dispara_int3_e_depois_int2` e `result_fifo_esvaziado_apos_acknowledge_e_leitura_da_segunda_resposta` falharam com stat=0 — corrigidos para inserir disco stub e verificar bit1 em vez de valor exato |
| 3 | API-Rust | Que um controle de mutação renomeando `let mm` para `let minute` funcionaria sem renomear também `seek_min.set(mm)` | O compilador reprova variável não renomeada em todos os usos | Controle K1 quebrou na bateria — substituído por controle que extrai `disc_inserted` em variável local |
| 4 | API-Rust | Que comandos INT3-only (sem segunda resposta) limpavam BUSYSTS automaticamente | `busy` era setado em `send_command` mas só limpo em `deliver_second` — GetStat e default arm mantinham BUSYSTS=1 para sempre | Descoberto durante a implementação: adicionado `self.busy.set(false)` em todos os comandos sem `pending_second` |

## Bateria de mutação

Placar da bateria: 7/7 mutantes mortos, 2/2 controles verdes, 0 equivalente - ./docs/mutantes/0063-cdrom-seek-pause.mut

| Mutante | Teste que o pegou |
|---|---|
| m1 (BCD sec inválido aceito) | `setloc_rejeita_segundo_bcd_invalido` |
| m2 (SeekL sem disco retorna INT3) | `seek_l_sem_disco_retorna_int5` |
| m3 (SeekL não seta seeking) | `seek_l_com_disco_retorna_int3_depois_int2` |
| m4 (deliver_second não limpa seeking) | `seek_l_com_disco_retorna_int3_depois_int2` |
| m5 (Pause sem segunda resposta) | `pause_retorna_int3_depois_int2` |
| m6 (Setloc sem disco retorna INT3) | `setloc_sem_disco_retorna_int3_mas_stat_com_bit0` |
| m7 (stat_byte sem bit1 motor) | `stat_byte_reflete_motor_ligado_com_disco` |

## Placar antes → depois

Workspace: **506** → **516** testes (506 existentes + 10 cdrom_seek_pause).

## Revisão cruzada (orquestrador)

<!-- Preenchido pelo Claude na revisão do PR -->

## Decisões e notas

1. **Divisão do item 4.2:** o item original juntava parser BIN/CUE + Setloc/SeekL/Pause/ReadN/ReadS/Init. R4 exige uma micro-funcionalidade por iteração. Dividido em 4.2a (Setloc/SeekL/Pause + drive state), 4.2b (parser BIN/CUE) e 4.2c (ReadN/ReadS + INT1/DRQSTS).
2. **Estatística `stat_byte()`:** construída dinamicamente do estado do drive (seeking, reading, shell_open, motor_on). Substitui os valores hardcoded 0x02 usados na 0062.
3. **Disco stub:** `insert_disc()` seta `disc_inserted=true` e `motor_on=true`. Comandos que exigem disco (SeekL, Setloc) retornam INT5(stat|0x01, 80h) sem disco. Suficiente para o emulador avançar até o ponto em que a BIOS tenta bootar.
4. **BCD validation:** Setloc valida `(ss & 0xF0) < 0x60` (BCD segundos < 60) e `(ff & 0xF0) < 0x70 && (ff & 0x0F) < 0x0A` (BCD setores < 75). Inválido retorna INT5(stat|0x01, 10h).
5. **Init reentrante:** se `pending_second == 1` (Init já em andamento), novo Init é silenciosamente ignorado — spec diz "silently dropped with no response".
6. **BUSYSTS corrigido:** comandos sem segunda resposta (Setloc, GetStat, default arm) agora limpam `busy` ao final do handler. Na 0062, BUSYSTS ficava permanentemente 1 após GetStat.
