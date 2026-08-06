<!-- Custo, tokens e duração NÃO entram aqui: são medidos por scripts/oc-iter.ps1 em
     docs/metricas.csv, casados por head_antes/head_depois. -->

# 0210 — lwc2-timing

- **Data:** 2026-08-06
- **Item do roadmap:** 0210.1 — Degrau 2 da escada de timing de CPU/barramento
- **Objetivo:** LWC2 paga o custo de região do load como qualquer outro load (`lw`/`lh`/...).

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| psx-spx | § Load Timing (L260-279) | docs/reference/02-cpu.md |
| psx-spx | § Store instructions (L305-307) | docs/reference/02-cpu.md |

## Erros de primeira tentativa

Nenhum — a spec e o padrão a seguir já estavam totalmente resolvidos pelo Degrau 1 (0209) e
por `cpu_load_timing.rs`; a mudança é ampliar um predicado existente com o mesmo cálculo de
endereço já usado nos outros loads.

## Bateria de mutação

Placar da bateria: 5/5 mutantes mortos, 2/2 controles verdes, 0 equivalente —
docs/mutantes/0210-lwc2-timing.mut

m1 reverte a cobertura, m2 cobre SWC2 em vez de LWC2, m3 usa o opcode errado (0x30), m4
troca OR por AND (desliga o custo de load pra tudo), m5 vira uma faixa aberta que também
pega SWC2 — todos mortos pelos testes de região (RAM/scratchpad/I-O/BIOS) e pelo teste de
SWC2 continuar em 1 ciclo.

## Placar antes → depois

Workspace: **1269 → 1275** testes (6 novos em `cpu_lwc2_timing.rs`).

## Revisão cruzada (orquestrador)

Rodada de autoria do orquestrador (mesma exceção de executor da 0193/0209). Os 3 testes de
região (RAM/I-O/BIOS) falhavam contra o código antes do fix (todos mostrando 1 ciclo em vez
do custo real) e passam depois; o teste de SWC2 e o de CU2-desligado já passavam antes (o
segundo graças ao fix do Degrau 1) e continuam passando — confirmando que a mudança não
afeta o que já estava certo. `cpu_load_timing.rs` (invariante 17) reexecutado sem mudança.

## Decisões e notas

Degrau 3 (próximo): MULT/MULTU/DIV/DIVU passam a custar ciclos de HI/LO
(`docs/reference/02-cpu.md` L420-440).
