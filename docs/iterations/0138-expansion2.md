<!-- Custo, tokens e duração NÃO entram aqui: são medidos por scripts/oc-iter.ps1 em
     docs/metricas.csv, casados por head_antes/head_depois. -->

# 0138 — expansion2

- **Data:** 2026-08-01
- **Item do roadmap:** conserto do defeito nº 3 da auditoria do plano de saída (Expansion
  Region 2 aliasava RAM)
- **Objetivo:** acessos a 0x1F802000-0x1F803FFF (I/O de expansão, inclui o POST em
  0x1F802041 que a BIOS escreve no boot) deixam de cair no fallback mascarado do bus e
  corromper RAM[0x2xxx] em silêncio.

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| psx-spx | linha L32 da tabela de regioes: "1F802000h 9F802000h BF802000h 8K Expansion Region 2 (I/O Ports)" | docs/reference/01-memory-map.md |

## Mecanismo

Golden do orquestrador primeiro (4 vermelhos): escrita 8/16/32 no POST não pode alterar
RAM física 0x2041; leitura não pode devolver conteúdo da RAM; espelho KSEG1 idem. Fix do
trabalhador (rodada única, 7 min): braço explícito 0x1F802000..=0x1F803FFF nos quatro
caminhos (read word/byte, write word/byte) — escrita é sink de I/O, leitura devolve
barramento aberto (0xFF por byte). Nenhuma outra região tocada.

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | nenhum | — | — | rodada única do trabalhador, verde de primeira; o defeito em si foi achado pela auditoria multi-agente (não por sintoma), terceira ocorrência do padrão porta-no-sumidouro (4.4j, 4.4m) |

## Bateria de mutação

Bateria de mutação: não se aplica — o fix é um braço de match de 4 linhas cuja remoção
qualquer um dos 4 goldens mata por construção (cada golden é o próprio mutante do braço
que cobre); bateria formal reservada para itens com lógica de estado.

## Placar antes → depois

857 → 861 (4 goldens novos em `bus_expansion2`).

## Revisão cruzada (orquestrador)

Diff mínimo conferido linha a linha: os 4 braços novos são idênticos em faixa
(0x1F802000..=0x1F803FFF), leitura word devolve 0xFFFFFFFF e byte 0xFF (barramento aberto,
sem registradores inventados), escrita silenciosa. Testes/clippy/fmt verdes na minha
execução, não só na do trabalhador.

## Decisões e notas

- O padrão sistêmico (região×tamanho caindo no sumidouro de RAM) agora tem TRÊS
  ocorrências consertadas (4.4j SIO, 4.4m timers, esta). A varredura completa
  região×tamanho fica como candidata a item de auditoria dedicado.
- Não medimos efeito no boot (a corrupção de RAM[0x2041] era silenciosa por natureza);
  o gate de regressão de sistema roda no PR seguinte junto do diagnóstico do VBlank.
