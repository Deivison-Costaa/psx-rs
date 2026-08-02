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

Pendente, para preenchimento na revisão adversarial do PR.

## Decisões e notas

- A sonda temporária observou `Bus::total_cycles()` logo após cada aumento de `raise_count(0)` no runner do Rayman, até o passo `166378016`.
- Foram observadas 658 subidas e 657 deltas consecutivos. O delta mediano foi **566187 ciclos**, mínimo **566187**, máximo **566213**.
- O esperado é `frame_cycles() = 566187`; a razão mediana/esperado é **1.000000**. O máximo acrescenta apenas a quantização do instante observado ao fim de uma instrução, não um novo período do scheduler.
- A medição fecha o 10.82 como **não há defeito na taxa de VBlank/IRQ0**; não houve alteração de produção.
- O teste permanente `irq0_periodo_ntsc.rs` avança o `Bus` ciclo a ciclo e confirma por efeito dois deltas consecutivos de um frame NTSC.
