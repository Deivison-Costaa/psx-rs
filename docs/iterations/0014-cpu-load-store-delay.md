<!-- Custo, tokens e duração NÃO entram aqui: são medidos por scripts/oc-iter.ps1 em
     docs/metricas.csv, casados por head_antes/head_depois. -->

# 0014 — cpu-load-store-delay

- **Data:** 2026-07-27
- **Item do roadmap:** 1.4
- **Objetivo:** implementar LB/LBU/LH/LHU/LW (loads) e SB/SH (stores), mais o load delay slot.

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| psx-spx | Load instructions (L157), Caution - Load Delay (L171), Load Timing (L180), Load Shadow (L201), Store instructions (L299) | docs/reference/02-cpu.md |

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | teste | Endereço de setup no `sb_offset_negativo` era 0x2004 (fora da área alvo) | Deveria ser 0x2000 para SB escrever em 0x2000 | Teste falhou com left=0 — bytes não inicializados. Corrigido na primeira execução. |
| 2 | nenhum | — | — | Load/store semantics e load delay acertados de primeira. Nenhum erro de emulação. |

## Bateria de mutação

**Placar: 6/6 mutantes pegos, 3/3 controles verdes.**

| Mutação | Teste que pegou |
|---|---|
| LB sem sign-extend (`val as u32` em vez de `as i8 as u32`) | `lb_carrega_byte_signed` |
| LH sem sign-extend (`val as u32` em vez de `as i16 as u32`) | `lh_carrega_half_signed` |
| SB usa `write32` em vez de `write8` | `sb_nao_afeta_bytes_vizinhos` |
| SH usa `write32` em vez de `write16` | `sh_nao_afeta_halfword_alto` |
| Load delay ausente — LW escreve direto no registrador | `load_delay_basico` |
| Load delay nunca commitado — `load_delay.take()` removido | `lw_carrega_palavra` |
| Controle: remover guarda `reg != 0` no agendamento (R0 já protegido por set_reg) | verde |
| Controle: renomear variável local | verde |
| Controle: reordenar métodos | verde |

## Placar antes → depois

Workspace: **75 → 95** testes (8 meta + 8 bus_bios + 2 bios_flag + 1 version + 11 bus_scheduler + 8 cpu_fetch_decode + 26 cpu_alu + 14 cpu_shifts + **18 cpu_load_delay** + 3 psx-cli/desktop).

## Revisão cruzada (orquestrador)

<!-- Preenchido pelo Claude na revisão do PR: achados no formato de docs/prompts/review.md, ou "sem achados". -->

## Decisões e notas

- Load delay implementado com campo `load_delay: Option<(usize, u32)>`. A cada `step()`:
  1. Executa a instrução atual (loads retornam `(rt, valor)` sem escrever em regs)
  2. Comita o load delay pendente (escrita atrasada do passo anterior)
  3. Agenda novo load delay se a instrução foi um load
- Stores (SB/SH/SW) não têm delay — executam imediatamente.
- R0 é sempre ignorado: `set_reg` já o protege, e o agendamento também tem guarda `reg != 0`.
- As novas funções `read8`/`read16`/`write8`/`write16` foram adicionadas ao Bus com o mesmo padrão `MemoryOp`.
