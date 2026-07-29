# 0043 — verificador-citacoes

- **Data:** 2026-07-29
- **Item do roadmap:** 0.12
- **Objetivo:** Verificador de citações de spec que detecta offsets do índice vs. linha real.

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| Tarefa | Descrição completa do item | ROADMAP 0.12 |

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| E1 | API-Rust | `text.find('§')` retorna posição em caracteres, não em bytes | `§` é caractere multibyte UTF-8 (2 bytes); `find()` retorna byte offset e o slice `[pos+1..]` quebra em posição inválida | Panic "start byte index is not a char boundary" ao rodar o teste |
| E2 | gramática | Títulos de seção só aparecem após `§` ou entre aspas | Também aparecem no padrão `seção Title (L<num>)` no STATUS.md e `Title (L<num>)` em prosa corrida | As duas citações erradas (`docs/reference/02-cpu.md` L805 e `docs/reference/03-gpu.md` L138) não eram detectadas porque `extract_section_title` retornava None |
| E3 | gramática | `seção` é palavra-chave global que se aplica a todos os refs da linha | `seções Texture Caching (L138), Texpage (L471)` — a palavra-chave está perto do primeiro ref mas longe do segundo; extraía o título errado para o segundo ref | title extraído com 80+ caracteres; corrigido com limite de 80 chars na keyword |
| E4 | gramática | `§ GP1(00h) Reset GPU (L747)` — o `find('(')` encontra o `(` antes do ref | O título contém parênteses: `GP1(00h)`. O primeiro `(` pertence ao título, não ao ref | Seção extraída como "GP1" em vez de "GP1(00h) Reset GPU"; corrigido usando posição do `(` final em `before` |
| E5 | mutação | `@@PARA` vazio funciona para deleção | O validador do manifesto (`mutation_format.rs`) rejeita `@@PARA` vazio (asserção D) | Meta-teste `mutation_anchors` reprovou o manifesto |
| E6 | mutação | `@@DE` com `#[test]` é conteúdo válido | Linhas começando com `#` são tratadas como comentários pelo parser de manifesto | `@@DE vazio`; corrigido usando conteúdo sem `#` |
| E7 | file_size | 643 linhas no teste > 500 | Tetro de 500 linhas para arquivo de teste (R8) | `file_size.rs` reprovou; movido código utilitário para `support/spec_citation_data.rs` |

## Bateria de mutação

Placar da bateria: 5/5 mutantes mortos, 2/2 controles verdes, 0 equivalente - docs/mutantes/0043-verificador-citacoes.mut

| # | Mutação | Pego? | Teste que pegou |
|---|---|---|---|
| c1 | Controle: acrescenta bind let inofensivo | Verde | — |
| c2 | Controle: acrescenta bind local inofensivo | Verde | — |
| m1 | Hardcoda offset em 115 (obrigatório) | Sim | L885 em 02-cpu.md calculado com offset 115 em vez de 80 |
| m2 | Inverte A2: todo número de linha ≤ total vira erro | Sim | L1349-L1557 e L471-L521 falham A2 |
| m3 | Reduz teto de A2 para 1: toda linha > 1 falha | Sim | Todas as citações do STATUS.md falham A2 |
| m4 | Zera offset: toda seção tem real = k + 0 | Sim | Todas as seções com título falham o diagnóstico de offset |
| m5 | Expande escopo para n >= 43 (inclui doc 0043) | Sim | Bare L-numbers no doc 0043 geram erros |

## Placar antes → depois

- **Antes:** 338 testes
- **Depois:** 339 testes (+1: `citacoes_de_spec_sao_validas`)
- Scoreboard: inalterado

## Revisão cruzada (orquestrador)

<!-- Preenchido pelo Claude na revisão do PR -->

## Decisões e notas

1. **Motor único em Rust, não duas implementações.** Toda a lógica de parsing e verificação mora em dois arquivos: `crates/psx-core/tests/spec_citations.rs` (470 linhas, abaixo do teto) e `crates/psx-core/tests/support/spec_citation_data.rs`. O script `scripts/confere-citacoes.ps1` é uma casca fina de 26 linhas que chama o teste e propaga o código de saída.

2. **Gramática das citações implementada por linha.** A resolução de arquivo segue as 5 regras do item: menções a `NN-nome.md` (com ou sem prefixo), refs `L<n>`, `L<n>-<m>`, e `:<n>`. A ligação por adjacência (`NN-nome.md:123`) usa o arquivo mais próximo dentro de 10 linhas.

3. **Check A4 (diagnóstico de offset) é o coração do verificador.** Para cada citação com título de seção extraível, o verificador encontra o índice (`- L<k>: <título>`) no arquivo de referência, computa `real = k + offset`, e compara com o número citado. Se o número cai no intervalo do índice → erro de offset. Se não cai nem no índice nem na seção real → erro de linha errada.

4. **Títulos extraídos por três caminhos:** (a) palavra-chave `seção`/`seções`/`section` até o `(` do ref (limite 80 caracteres); (b) símbolo `§` até o `(` do ref; (c) texto antes do `(L<num>)` delimitado por vírgula/ponto-e-vírgula.

5. **Offsets por arquivo (nunca hardcoded):** 01-memory-map (+23), 02-cpu (+80), 03-gpu (+115), 04-dma (+25), 05-timers (+16), 06-cdrom (+170), 07-gte (+72), 08-spu (+109), 09-mdec (+49), 10-controllers-memcards (+212), 11-interrupts (+19), 12-memory-control (+23), 13-kernel-bios (+320), 14-io-map (+35), 15-cdrom-format (+135), 16-cdrom-file-formats (+1000).

6. **Check C:** todo `docs/reference/*.md` (exceto README) tem exatamente uma linha `CORPO:`. Verificado — 16 arquivos, todos com 1 ocorrência.

7. **Dois erros vivos corrigidos no STATUS.md:**
   - L805 → L885 (cop0r16-r31 - Garbage, 02-cpu.md: offset +80, índice L805, real L885)
   - L138-L206 → L1349-L1557 (Texture Caching, 03-gpu.md: offset +115, índice L1234, real L1349; fim da seção em L1557, antes de "24bit RGB to 15bit RGB Dithering")

8. **Prova do offset constante +115 no doc 0038 (antes de 5bed036):** As 6 citações da tabela "Spec consultada" no commit `5bed036~1` usam os offsets do índice (L1041, L488, L525, L549, L582, L632) em vez das linhas reais (L1156, L603, L640, L664, L697, L747). O verificador produziria o diagnóstico agregado: "as 6 citações deste doc batem todas com offset constante +115 — o doc inteiro veio do índice". A correção foi feita no commit `5bed036` (orquestrador).

9. **Convenção de citação:** A partir desta iteração, a convenção passa a ser `§ Título (L<n>)` — o título sobrevive à regeneração da spec, o número não. Acrescentado ao SKILL.md.
