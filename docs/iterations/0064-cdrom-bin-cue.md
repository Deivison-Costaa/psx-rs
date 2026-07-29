# 0064 — cdrom-bin-cue

- **Data:** 2026-07-29
- **Item do roadmap:** 4.2b
- **Objetivo:** Parser de arquivo .cue para extrair caminho .bin, tabela de tracks (tipo, início, offset) e leitura de setores de dados (2048 bytes).

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| psx-spx | CDROM Disk Images CUE/BIN/CDT (Cdrwin) | docs/reference/16-cdrom-file-formats.md |

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | lint | Loop `for i in 0..2352` com indexação manual era aceitável | Clippy `needless_range_loop` rejeita | `cargo clippy -- -D warnings` |
| 2 | mut | `@@PARA` vazio em manifesto de mutação — assumi que era válido | Anchor validator exige `@@PARA` com pelo menos uma linha | Meta-teste `mutation_anchors` |

## Revisão do PR anterior (iter 0063)

1. **(TESTE QUE NÃO MEDE)** `setloc_rejeita_setor_bcd_invalido` só checava `hintsts & 0x7 == 5` sem verificar FIFO de resultado. Corrigido: adicionadas asserções de stat bit0=1 e error byte=0x10.
2. **(PARÂMETRO NÃO CONSUMIDO)** Teste `setloc_consome_tres_parametros_fifo_alinhado` cobre — OK sem achados.
3. **(REGRA DE BORDA)** CDROM não tem regra de borda gráfica — OK sem achados.
4. **(CAMPO DE BIT LIDO ERRADO)** Stat bits (6=seeking, 5=reading, 4=shell_open, 1=motor_on) conferidos — OK.
5. **(PANIC/LAÇO ILIMITADO)** Sem `unsafe`/`unwrap()` no código de cdrom — OK.
6. **(CITAÇÃO DE SPEC)** `confere-citacoes.ps1` verde — OK.
7. **(ESCOPO TRANSBORDADO)** Diff do PR #78 só contém o item 4.2a — OK.
8. **(ACHADO REAL)** `(ff & 0xF0) < 0x70` rejeitava setores BCD válidos 0x70-0x74; spec diz `asect < 75h`. Corrigido para `ff < 0x75`. Adicionado teste `setloc_aceita_setor_0x74_bcd_valido`. Validação de segundos agora inclui `(ss & 0x0F) < 0x0A` (spec pede "valid packed BCD").

## Bateria de mutação

Placar da bateria: 7/7 mutantes mortos, 2/2 controles verdes, 0 equivalente - ./docs/mutantes/0064-cdrom-bin-cue.mut

| Mutante | Rótulo | Resultado |
|---|---|---|
| m1 | Track type MODE2/2352 → Audio | MORREU |
| m2 | INDEX 01 MM ignorado → zero | MORREU |
| m3 | PREGAP descartado → None | MORREU |
| m4 | bin_offset usa 2048 em vez de 2352 | MORREU |
| m5 | read_data_sector extrai do offset 0 | MORREU |
| m6 | bin_path vazio → ignora FILE | MORREU |
| m7 | parse_cue perde última track | MORREU |
| K1 | parse_track_type em var local | verde |
| K2 | parse_mm_ss_ff em var local | verde |

## Placar antes → depois

Workspace: 541 testes (11 novos: cdrom_bin_cue).

## Revisão cruzada (orquestrador)

<!-- Preenchido pelo orquestrador -->

## Decisões e notas

- `DiscLayout` e `TrackInfo` são estruturas puras, sem acoplamento ao emulador (R3). O arquivo-alvo `crates/psx-core/src/cdrom_bin_cue.rs` é um módulo novo.
- `read_data_sector(&self, bin_data: &[u8], sector_index: u32) -> Vec<u8>` recebe o conteúdo do .bin como slice e extrai 2048 bytes do offset 0x10, mantendo psx-core puro.
- Offsets no .cue são endereços lógicos (MM:SS:FF × 2352 = offset no .bin). Conversão para endereço real (add 2 segundos + pregap acumulado) é dívida futura, quando o parser se acoplar ao emulador.
- PREGAP e INDEX 00 são armazenados como `Option<(u8, u8, u8)>` para permitir ausência.
- Track types suportados: AUDIO, MODE1/2048, MODE1/2352, MODE2/2336, MODE2/2352.
