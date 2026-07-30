# 0083 — fix-spec-citations-substring

- **Data:** 2026-07-30
- **Item do roadmap:** 10.16
- **Objetivo:** Corrigir `find_in_index` para preferir título mais longo/específico em vez de substring match.

## Revisão do PR anterior

Revisão do PR #96 (iter 0081): sem achados.

Nove padrões conferidos:
1. Teste que não mede — t10 da 0081 foi corrigido e mede; t3 usa >= 2 (lower bound fraco mas funcional)
2. Parâmetro não consumido — sem novos comandos GPU na 0081 (iteração de revisão)
3. Regra de borda trocada — sem rasterização
4. Campo de bit lido errado — sem novos registradores
5. Panic ou laço ilimitado — `frame_cycles()` nunca retorna 0; sem unwrap/unsafe fora de teste
6. Citação de spec — `confere-citacoes.ps1` verde no main
7. Escopo transbordado — hblank declarado como dívida; sem implementação extra
8. Portão — `.resultado` rastreado, `mutation_anchors` verde
9. Manifesto arquivado — sem arquivamentos na 0081

## Spec consultada

Nenhuma — item 10.16 é correção de meta-teste (portão de citações).

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | manifesto-mutação | `ocorrencias:` depois de `@@FIM` valia para o registro atual | `ocorrencias:` precisa vir antes de `@@DE` (o script aplica `ocorrencias` ao PRÓXIMO registro) | `mutantes.ps1` reprovou m3 com "encontrada 2 vez(es), esperado 1" |
| 2 | manifesto-mutação | m2 e m4 tinham 2 ocorrências (find_in_index + index_match_ambiguity) | m2 e m4 só aparecem uma vez — `index_match_ambiguity` usa guardas diferentes (`<= 1` vs `is_empty`) | `mutantes.ps1` reprovou com contagem errada |
| 3 | controle-mutação | Renomear `let s` para `let busca` era cosmético | O código usa `s` em closures; renomear quebrou compilação | `mutantes.ps1` reportou ERRO DE MANIFESTO para c1 |

## Bateria de mutação

Placar da bateria: 5/5 mutantes mortos, 2/2 controles verdes, 0 equivalente — docs/mutantes/0083-fix-spec-citations-substring.mut

| # | Tipo | Rótulo | Resultado |
|---|---|---|---|
| m1 | mutante | retorna primeiro match em vez do mais longo (regressão) | MORREU |
| m2 | mutante | retorna None com múltiplos matches | MORREU |
| m3 | mutante | prefere título mais CURTO em vez do mais longo | MORREU |
| m4 | mutante | `longest.len() >= 1` aceita empate silencioso | MORREU |
| m5 | mutante | `index_match_ambiguity` inverte condição de retorno | MORREU |
| c1 | controle | adiciona comentário antes do filter | verde |
| c2 | controle | remove guard `if matches.len() <= 1` do início de `index_match_ambiguity` | verde |

## Placar antes → depois

Workspace: **586** → **589** testes (+3: spec_citation_index).

`confere-citacoes.ps1` permanece verde.

## Revisão cruzada (orquestrador)

<!-- Preenchido pelo orquestrador. -->

## Decisões e notas

1. **Casamento por título mais longo, não por substring.** A função `find_in_index` agora coleta todos os matches, filtra pelo mais longo e retorna None se houver empate (ambíguo). A `index_match_ambiguity` detecta e reporta empates com mensagem explícita.
2. **`index_match_ambiguity` integrada ao portão principal.** O `spec_citations.rs` agora chama `index_match_ambiguity` antes de `find_in_index` e reporta citações ambíguas como erro, em vez de escolher silenciosamente.
3. **`02-cpu.md` tem duas seções com título em relação prefixo-substring** (`Opcode/Parameter Encoding` e `Coprocessor Opcode/Parameter Encoding`). Com a correção, uma citação de `Coprocessor Opcode/Parameter Encoding (L207)` resolve corretamente para o índice L127 (real L207), não para o índice L99 (real L179).
4. **Itens registrados no ROADMAP:** 10.34 (meta-teste para `#[test]` sem asserção) e 10.35 (mismatch entre `mutantes.ps1` e `mutation_battery.rs` com nomes qualificados).
