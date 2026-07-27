<!-- Custo, tokens e duração NÃO entram aqui: são medidos por scripts/oc-iter.ps1 em
     docs/metricas.csv, casados por head_antes/head_depois. -->

# 0013 — cpu-shifts

- **Data:** 2026-07-27
- **Item do roadmap:** 1.3b
- **Objetivo:** implementar SLL/SRL/SRA (shift-imm com campo `sa`) e SLLV/SRLV/SRAV (shift-reg com quantidade em `rs & 0x1F`).

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| psx-spx | shifting instructions (L396), encoding (L184-185) | docs/reference/02-cpu.md |

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | nenhum | — | — | Acertou de primeira no código, mas o teste `opcode_desconhecido_especial_panics` (cpu_alu.rs) usava secondary=0x00 que agora é SLL válido e parou de panicar. |

## Bateria de mutação

**Placar: 3/3 mutantes pegos, 2/2 controles verdes.**

| Mutação | Teste que pegou |
|---|---|
| SRA usa `>>` lógico em vez de aritmético (`(self.reg(rt) >> sa)`) | `sra_aritmetico_propaga_sinal` |
| SRAV usa `>>` lógico em vez de aritmético | `srav_aritmetico_propaga_sinal` |
| Variante V sem máscara `& 0x1F` no shift | `sllv_shift_amount_mascara_0x1f` (panic de overflow) |
| Controle: renomear `shift` → `amt` | verde |
| Controle: reordenar casos do match | verde |

## Placar antes → depois

Workspace: **67 → 75** testes (8 meta + 8 bus_bios + 2 bios_flag + 1 version + 11 bus_scheduler + 8 cpu_fetch_decode + 26 cpu_alu + **14 cpu_shifts** + 3 psx-cli/desktop).

## Revisão cruzada (orquestrador)

<!-- Preenchido pelo Claude na revisão do PR -->

## Decisões e notas

- `sll $0,$0,0` (secondary=0x00, rd=0, rt=0, sa=0) é o NOP canônico — `set_reg` já ignora R0.
- O teste `opcode_desconhecido_especial_panics` da ALU precisou ser atualizado de secondary 0x00 → 0x08 porque 0x00 agora é SLL implementado.
