# 0040 — manifesto-mutacao

- **Data:** 2026-07-28
- **Item do roadmap:** 0.10
- **Objetivo:** Formato de manifesto de mutação + meta-teste que impede falsificação de placar.

## Spec consultada

Nenhuma — item de ferramental, não de emulação. A gramática do formato `.mut` está em
`docs/mutantes/README.md`.

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| P1 | protocolo | O arquivo `mutation_manifest.rs` caberia em 500 linhas | `file_size.rs` impõe 500 linhas por arquivo de teste, e o parser + asserções somou 602 linhas | `wc -l` antes do commit |
| P2 | protocolo | `rec_ocorrencias` podia ser resetado na entrada de `@@DE` | Diretivas entre edições (ex.: `ocorrencias: 2` antes do terceiro `@@DE`) eram anuladas pelo reset no `@@DE`; o manifesto 0038 expôs o bug na asserção D (âncoras) | Teste `mutation_anchors` rodando contra o manifesto |

## Bateria de mutação

Bateria de mutação: não se aplica — item de ferramental que não toca em código de emulação
(0.10 é o formato e o meta-teste que viabilizam a bateria; o script que roda é o 0.11).

## Placar antes → depois

314 testes → 316 testes (+2 meta-testes: `mutation_manifest` + `mutation_anchors`).
Scoreboard inalterado.

## Decisões e notas

1. **Parser em módulo separado com `#[path]`.** O parser mora em
   `tests/support/mutation_format.rs` e é incluído via `#[path = "support/mutation_format.rs"]`
   apenas nos dois arquivos que o usam (`mutation_manifest.rs`, `mutation_anchors.rs`). NÃO
   foi adicionado ao `mod.rs` do `support/` para não ser compilado 28 vezes em cada build.
2. **Split parser vs. asserções.** O arquivo original tinha 602 linhas e foi dividido:
   `mutation_format.rs` (parser, 414 linhas), `mutation_manifest.rs` (forma, 71 linhas),
   `mutation_anchors.rs` (âncoras + existência, 137 linhas). Todos abaixo de 500.
3. **Manifesto retroativo da 0038.** `docs/mutantes/0038-vram-transfers.mut` serve de fixture
   com 6 mutantes, 1 equivalente e 2 controles — todos com âncoras verificadas contra
   `crates/psx-core/src/gpu.rs` via `grep -Fxc`.
4. **`PRIMEIRA_ITER_COM_MANIFESTO = 42`.** As iterações 0040 e 0041 são ferramental que
   constroem o portão; 0042 é o próximo item de hardware. Retrofit de 38 manifestos seria
   arqueologia, não medição — mesmo raciocínio do `MAX_LAG` em `metrics_freshness.rs`.
5. **Opt-out no doc, não no teste.** A asserência H exige manifesto OU linha
   `Bateria de mutação: não se aplica — <motivo de pelo menos 40 chars>` no doc da iteração.
   A lista de exceções fica ao lado da afirmação de cobertura, onde o revisor a falsifica.
6. **`ocorrencias:` é resetado por `@@FIM`, não por `@@DE`.** O bug P2 foi corrigido no parser:
   a entrada de `@@DE` não reseta `rec_ocorrencias` (quem reseta é `@@FIM` com `std::mem::take`).
   Isso permite que `ocorrencias: N` seja colocado entre edições de um mesmo registro.
7. **SKILL.md, passo 6, reescrito.** A bateria passa a ser `docs/mutantes/NNNN-slug.mut`, o
   placar sai do script (item 0.11), e fica proibido mutar arquivo de teste.

## Revisão cruzada (orquestrador)

<!-- Preenchido pelo Claude na revisão do PR -->
