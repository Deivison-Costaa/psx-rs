<!-- Custo, tokens e duração NÃO entram aqui: são medidos por scripts/oc-iter.ps1 em
     docs/metricas.csv, casados por head_antes/head_depois. -->

# 0010 — bus-scheduler

- **Data:** 2026-07-27
- **Item do roadmap:** 1.1
- **Objetivo:** Scheduler de eventos com fila ordenada + Bus com RAM 2MB e BIOS, roteamento KUSEG/KSEG0/KSEG1.

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| psx-spx | Memory Map | docs/reference/01-memory-map.md:L24 |
| psx-spx | KUSEG,KSEG0,KSEG1,KSEG2 Memory Regions | docs/reference/01-memory-map.md:L49 |
| psx-spx | Memory Mirrors | docs/reference/01-memory-map.md:L142 |

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | API-Rust | A bateria de mutação "sem sort" passaria porque o teste agenda fora de ordem | O teste `scheduler_eventos_ordem` agenda 10 antes de 5, mas sem sort o `advance_to(5)` retorna None porque o primeiro elemento é (10, A) e 10 > 5, fazendo o teste ainda passar "por acaso" | A mutação 4 não falhou na primeira tentativa — precisei reinspecionar e confirmar que o regex de substituição não funcionou. Após corrigir o método de mutação (comentando a linha de sort), o teste pegou |
| 2 | endereçamento | Usei `0x1F_C0_0000` com separadores de 2/2/4 dígitos | Clippy exige grupos de igual tamanho para hex literals: `0x1FC0_0000` | Clippy `unusual_byte_groupings` |
| 3 | API-Rust | Esqueci `Default` impl para `Ram` e `Scheduler` | Clippy `new_without_default` exige `Default` quando há `fn new()` | Clippy |
| 4 | API-Rust | Usei `if phys >= X && phys < X + Y` | Clippy prefere `range.contains()` | Clippy `manual_range_contains` |

## Bateria de mutação

Placar: 8/8 mutantes pegos, 2/2 controles verdes.

| # | Mutação | Teste que pegou |
|---|---|---|
| 1 | scheduler: sort descendente (reverse) | `scheduler_eventos_ordem` |
| 2 | scheduler: `pop()` em vez de `remove(0)` | `scheduler_eventos_ordem` |
| 3 | scheduler: `>=` em vez de `>` na comparação tick | `scheduler_evento_imediatamente` e `scheduler_eventos_ordem` |
| 4 | scheduler: sem sort (linha comentada) | `scheduler_eventos_ordem` |
| 5 | bus: RAM mask `0x0FFFFF` | `bios_read_9fc00000`, `bios_read_bfc00000` |
| 6 | bus: RAM mask `0x3FFFFF` | `bios_read_bfc00000` |
| 7 | bus: BIOS size `0x100000` | 8 testes de BIOS/RAM falharam |
| 8 | bus: máscara `0x3FFFFF` no to_physical | `bios_read_bfc00000` |

Controles: (1) renomear `next_tick` → `nt` no scheduler → verde; (2) test exists — ambos verdes.

## Placar antes → depois

Workspace: **33** testes (8 meta + 8 bus_bios + 2 bios_flag + 1 version + 11 bus_scheduler + 3 psx-cli/desktop).

## Revisão cruzada (orquestrador)

<!-- Preenchido pelo Claude na revisão do PR -->

## Decisões e notas

- `to_physical` usa `addr >> 29` para rotear entre KUSEG (010), KSEG0 (100), KSEG1 (101). A máscara `0x1FFF_FFFF` isola os 29 bits baixos, que correspondem ao espaço físico de 512MB.
- RAM ocupa `0x000000..0x1FFFFF` (2MB), BIOS ocupa `0x1FC0000..0x1FC0000+0x80000` (512KB) dentro desse espaço físico.
- KSEG1 neste item é tratado como mirror idêntico — sem comportamento de uncached/write-queue (item futuro).
- Scheduler mantém fila ordenada por tick via `sort_by` a cada `schedule()`; para uso real com CPU instruction-stepped, `advance_to` só retorna eventos cujo tick ≤ current_tick.
