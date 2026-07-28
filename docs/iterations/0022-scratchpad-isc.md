<!-- Custo, tokens e duração NÃO entram aqui: são medidos por scripts/oc-iter.ps1 em
     docs/metricas.csv, casados por head_antes/head_depois. -->

# 0022 — scratchpad-isc

- **Data:** 2026-07-28
- **Item do roadmap:** 1.9
- **Objetivo:** decodificação de região no Bus para Scratchpad (1KB), memory control (stubs), BCC (KSEG2), e isolamento de cache via SR.Isc.

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| psx-spx | Memory Map, KUSEG/KSEG0/KSEG1/KSEG2, Scratchpad, Memory Mirrors, Memory Exceptions | docs/reference/01-memory-map.md |
| psx-spx | Memory Control ports, RAM_SIZE, BCC | docs/reference/12-memory-control.md |
| psx-spx | cop0r12 - SR, bit 16 Isc | docs/reference/02-cpu.md |

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | Teste frágil | D3 (limite superior) sem testemunha RAM bastava | Um alias que passa o readback (escrita/leitura vão ao mesmo alias) não prova que o destino é scratchpad | Mutação de range (0x3FF→0x3FB) sobreviveu; D3 foi reforçado com testemunha RAM |

## Bateria de mutação

5/5 mutantes pegos, 1/1 controle verde.

| Mutação | Teste que pegou |
|---|---|
| Remove region_read32 do read32 → tudo cai na RAM | D1, D2, D3, D5, D6 |
| Remove exclusão KSEG1 do scratchpad | D2 (KSEG1 devolve dados) |
| Remove Isc check de sw | D4 (store passa com Isc=1) |
| Endereço do BCC trocado (FFFE0130→FFFE0134) | D6 (escrita vai pra RAM) |
| Range do scratchpad reduzido (0x3FF→0x3FB) | D3 (testemunha RAM revela alias) |
| **Controle:** renomear is_isc → cache_isolated | Todos verdes |

## Placar antes → depois

203 → 209 testes (+6 bus_scratchpad_isc).

## Revisão cruzada (orquestrador)

<!-- Preenchido pelo Claude na revisão do PR: achados no formato de docs/prompts/review.md, ou "sem achados". -->

## Decisões e notas

1. **Isc no CPU, não no Bus.** O Bus não conhece COP0 (regra R3 + armadilha 6 do STATUS). A checagem `(sr >> 16) & 1` está em `sw`/`sb`/`sh`/`swl`/`swr` no CPU, retornando cedo sem chamar o bus. Reads não são afetados (não há dado cacheado para servir).
2. **Scratchpad ignorado em KSEG1.** `region_read32`/`region_write32` checam `kseg == 0b101` e devolvem 0/ignoram. Comportamento ASSUMIDO até o 1.11 (Bus Error).
3. **MemCtrl e BCC são stubs.** Apenas guardam e retornam o valor escrito via `write32`/`read32`. Sub-word writes são ignorados. RAM_SIZE não afeta espelhos.
4. **KSEG2 não passa por `to_physical`.** Endereços em KSEG2 (0xC0000000+) mantêm o valor original via braço `_`, o que faz `0xFFFE_0130` chegar intacto ao region decode.
