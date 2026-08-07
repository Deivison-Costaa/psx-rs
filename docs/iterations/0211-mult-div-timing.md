<!-- Custo, tokens e duração NÃO entram aqui: são medidos por scripts/oc-iter.ps1 em
     docs/metricas.csv, casados por head_antes/head_depois. -->

# 0211 — mult-div-timing

- **Data:** 2026-08-06
- **Item do roadmap:** 0211.1 — Degrau 3 da escada de timing de CPU/barramento
- **Objetivo:** MULT/MULTU/DIV/DIVU custam ciclos de HI/LO — ler o resultado antes do cálculo
  terminar trava a CPU pelo resto do custo.

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| psx-spx | § MULT/DIV timing (L420-441) | docs/reference/02-cpu.md |

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | spec | Que as faixas Fast/Med/Slow do `mult` (com sinal) não se sobrepunham. | `02-cpu.md` L429-430: a faixa Fast negativa é `FFFFF800h..FFFFFFFFh` e a faixa Med negativa é impressa como `FFF00000h..FFFFF801h` — os valores `FFFFF800h` e `FFFFF801h` satisfazem as duas faixas ao mesmo tempo. | Notado ao desenhar os testes de fronteira: escolhi valores de teste bem dentro de cada faixa (não nos limites) e resolvi a ambiguidade a favor da faixa Fast (mais apertada), documentado no comentário do código e no teste. Não é erro de implementação — é uma imprecisão da própria spec (provável typo de 1 dígito, `801h` em vez de `7FFh`). |
| 2 | teste | Que os valores de `rs` que eu já usava para `multu` (0x100/0x800/0x100000) seriam suficientes para provar que `multu` usa a tabela SEM sinal e `mult` usa a tabela COM sinal. | Não é assunto de spec — é cobertura de teste. | `scripts/mutantes.ps1 -Iter 0211`: os mutantes m2/m3 (trocar `mult_cost` por `multu_cost` e vice-versa) só morreram depois que acrescentei um teste com `rs=0xFFFFFF00` passado pra `multu` — as faixas positivas das duas tabelas são idênticas, só a extensão negativa do `mult` as distingue, e nenhum valor de teste anterior tocava nela. |

## Bateria de mutação

Placar da bateria: 6/6 mutantes mortos, 2/2 controles verdes, 0 equivalente —
docs/mutantes/0211-mult-div-timing.mut

m1 troca `saturating_sub` por `wrapping_sub` (o atraso vira lixo quando não há mult/div
pendente), m2/m3 trocam a tabela de custo do `mult` pela do `multu` e vice-versa, m4 muda o
custo fixo de div/divu (36→35, nas duas ocorrências), m5/m6 removem a chamada do stall em
`mfhi`/`mflo` — todos mortos pelos testes de faixa e pelo controle de valor grande sem sinal.

## Placar antes → depois

Workspace: **1275 → 1289** testes (14 novos em `cpu_mult_div_timing.rs`).

## Revisão cruzada (orquestrador)

Rodada de autoria do orquestrador (mesma exceção de executor da 0193/0209/0210). Os 8 testes
de faixa/custo falhavam contra o código antes do fix (todos mostrando o total sem nenhum
stall) e passam depois; os 3 controles (mfhi sem mult pendente, instrução que não lê HI/LO,
mthi) já passavam antes e continuam passando — confirmando que a mudança não afeta o que já
estava correto. `cpu_load_timing.rs` (invariante 17) reexecutado sem mudança — o laço da BIOS
não usa mult/div.

## Decisões e notas

`load_extra_cycles` renomeado para `extra_cycles`: deixou de ser só sobre custo de load,
agora acumula qualquer estouro além do 1 ciclo de emissão de QUALQUER instrução (load, e
agora mult/div; GTE no próximo degrau vai reusar o mesmo mecanismo). `Cpu` ganhou
`hilo_busy_until: u64` → `snapshot::VERSAO` 1→2, `TAMANHO_DO_ESTADO` +8 bytes.

Modelo de stall (mesmo que o Degrau 5, GTE, vai reusar): `busy_until = ciclo_de_emissão + 1 +
custo`, ancorado no exemplo literal da spec ("seis opcodes ALU cabem de graça entre multu e
mflo" — com `emissão + custo`, sem o `+1`, caberiam só cinco, não seis).

Degrau 4 (próximo): tabela pura de custo por comando GTE (`docs/reference/07-gte.md`), sem
chamador ainda — zero risco de regressão, isola o risco real pro Degrau 5 (o stall ligado na
CPU).
