# 0156 — irq0-periodo-ciclos

- **Data:** 2026-08-02
- **Item do roadmap:** 10.82
- **Objetivo:** medir em ciclos o período real entre subidas consecutivas de IRQ0 e compará-lo ao frame NTSC.

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| psx-spx | § Vertical Video Timings (L1414) | docs/reference/03-gpu.md |
| psx-spx | § Vertical Refresh Rates (L1426) | docs/reference/03-gpu.md |
| psx-spx | § Vertical Timings (L1460) | docs/reference/03-gpu.md |

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | timing | Os offsets do índice da referência eram linhas físicas para `sed`. | Os offsets do índice são relativos ao corpo; a seção real aparece em `GPU Timings` na L1401. | A saída mostrou textura em vez de temporização; `grep -n` localizou o cabeçalho e a leitura foi repetida na faixa correta. |

## Bateria de mutação

Bateria de mutação: não se aplica — diagnóstico puro, sem alteração em `crates/*/src`; o teste verifica a taxa produzida pelo scheduler.

## Placar antes → depois

Workspace: **890 → 891** testes. `cargo fmt --all -- --check`, `cargo clippy --all-targets -- -D warnings` e `cargo test --all --no-fail-fast` verdes após atualizar o placar.

## Revisão cruzada (orquestrador)

**A medição está correta e refuta a suspeita do orquestrador.** Rodei o portão por conta
própria: 891 testes, verde. Conferi as citações abrindo as faixas: em `docs/reference/03-gpu.md`
a L1401 é `## GPU Timings` e a L1414 é `#### Vertical Video Timings` — apontam para o corpo.

**A suspeita era minha, e caí exatamente no erro contra o qual avisei no handoff.** Eu estranhei
"660 subidas de IRQ0 em 166 M passos" e escrevi que a taxa parecia o dobro do esperado. Mas
tratei passo de instrução como se fosse ciclo — a mesma confusão que o handoff proibia em
maiúsculas. A medição mostra ~2,24 ciclos por passo, o que explica inteiramente a discrepância
aparente. A taxa nunca esteve errada; a minha unidade é que estava.

**Resultado negativo, e ele vale.** O período mediano entre subidas de IRQ0 é **566 187
ciclos**, idêntico a `frame_cycles()`, com mínimo 566 187 e máximo 566 213 — a diferença de 26
ciclos é a quantização do instante de observação (fim de instrução), não um período diferente.
Razão medido/esperado: **1,000000**. A temporização de VBlank do emulador está certa, e essa
hipótese sai da mesa para o problema do Rayman.

Confere por outro caminho: 658 frames × 566 187 ciclos = 372,5 M ciclos, que a 33,87 MHz dão
11,0 s de tempo emulado — consistente com 658 frames a 60 Hz. Os dois lados da conta fecham.

**O teste permanente é do bom padrão da série.** Aciona o `Bus` real ciclo a ciclo por três
frames e afirma exatamente três subidas com 566 187 ciclos entre elas. Exercita o scheduler de
produção e falha se a taxa mudar; não declara constante para afirmá-la de volta.

**Nota de processo:** esta foi a primeira rodada desde a 0152 a completar o protocolo inteiro e
abrir o PR sozinha. A diferença em relação às três anteriores foi a proibição de caminho
absoluto — uma tentativa deste mesmo item morreu no passo 11 ao montar
`.../Área de trabalho/Programação com Agentes/...`, sem o componente `Faculdade/`, caindo fora
do projeto e disparando o pedido interativo de `external_directory`. Caminho relativo elimina a
classe inteira de falha, e a rodada seguinte foi até o fim.

## Decisões e notas

- A sonda temporária observou `Bus::total_cycles()` logo após cada aumento de `raise_count(0)` no runner do Rayman, até o passo `166378016`.
- Foram observadas 658 subidas e 657 deltas consecutivos. O delta mediano foi **566187 ciclos**, mínimo **566187**, máximo **566213**.
- O esperado é `frame_cycles() = 566187`; a razão mediana/esperado é **1.000000**. O máximo acrescenta apenas a quantização do instante observado ao fim de uma instrução, não um novo período do scheduler.
- A medição fecha o 10.82 como **não há defeito na taxa de VBlank/IRQ0**; não houve alteração de produção.
- O teste permanente `irq0_periodo_ntsc.rs` avança o `Bus` ciclo a ciclo e confirma por efeito dois deltas consecutivos de um frame NTSC.
