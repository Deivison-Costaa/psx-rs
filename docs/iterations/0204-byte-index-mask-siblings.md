<!-- Custo, tokens e duração NÃO entram aqui: são medidos por scripts/oc-iter.ps1 em
     docs/metricas.csv, casados por head_antes/head_depois. -->

# 0204 — byte-index-mask-siblings

- **Data:** 2026-08-06
- **Item do roadmap:** 0203.4 (achado aberto na revisão adversarial do PR #215, achado 10.51)
- **Objetivo:** aplicar nos outros 4 braços de `region_read_byte` a mesma máscara final que o
  PR #215 já aplicou ao braço da GPU — impedir o shift além de `u32` quando um `read16` cai
  no último byte de uma palavra de 32 bits.

## Spec consultada

Nenhuma — mesmo caso do 10.51 original: defeito puramente aritmético (índice de byte sem
máscara), sem semântica de hardware nova em nenhum periférico (MEM_CTRL/BCC/DMA).

## Erros de primeira tentativa

Rodada do trabalhador (opencode) executou os passos 4-5 corretamente (teste vermelho → fix
verde, 4 casos, um por sítio, cada um checando o VALOR lido, não só ausência de panic) mas
travou no passo 6: o `git commit` do manifesto de mutação falhou porque o `workdir` passado
pela ferramenta de bash do trabalhador veio com um segmento do caminho faltando
("`Desktop\Programacao com agentes\...`" em vez de "`Desktop\faculdade\Programacao com
agentes\...`") — erro da ferramenta do trabalhador, não do conteúdo do commit. A rodada
terminou sem abrir PR, com o manifesto só no disco (não commitado). O orquestrador retomou a
partir daí: commitou o manifesto, rodou a bateria, e completou os passos 7-9.

## Bateria de mutação

Placar da bateria: 6/6 mutantes mortos, 2/2 controles verdes, 0 equivalente —
`docs/mutantes/0204-byte-index-mask-siblings.mut`.

- m1/m5 (MEM_CTRL sem máscara / máscara de 2 bits): mortos.
- m2 (espelho do MEM_CTRL sem máscara): morto.
- m3/m6 (BCC sem máscara / máscara de 1 bit): mortos.
- m4 (DMA sem máscara): morto.
- c1 (ordem da soma no MEM_CTRL): verde.
- c2 (literal hexadecimal equivalente no BCC): verde.

## Placar antes → depois

Workspace: **1242** → **1246** testes (4 novos em `bus_byte_index_mask_siblings.rs`, um por
sítio corrigido).

## Revisão cruzada (orquestrador)

Sem achados — o fix replica exatamente o padrão já usado nos braços de timers/SPU/GPU
(`& 3`/`& 1` final), sem tocar a lógica de leitura dos registradores em si. Os 4 testes
verificam o VALOR lido após o wrap, não só ausência de panic — não são teatrais.

## Decisões e notas

**1. Esta iteração fecha o achado 0203.4 inteiro** (os 4 sítios identificados na revisão do
PR #215) — não sobrou nenhum sítio irmão sem máscara em `region_read_byte`.

**2. `STATUS.md` não foi alterado**, conforme a política de lote do orquestrador em curso
(placar consolidado é atualizado uma vez, ao fechar o lote).
