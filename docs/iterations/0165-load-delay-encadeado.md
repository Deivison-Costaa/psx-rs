# 0165 — load-delay-encadeado

- **Data:** 2026-08-02
- **Item do roadmap:** 10.93
- **Objetivo:** corrigir a visibilidade do destino em dois loads consecutivos para o mesmo GPR.

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| psx-spx | § Caution - Load Delay (L251) — memoria | docs/reference/02-cpu.md |

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | delay-slot | Que o `Option` de um unico load podia ser commitado ao terminar o segundo load, mesmo com o mesmo destino. | § Caution - Load Delay (L251) de `docs/reference/02-cpu.md` (secao de memoria, nao a homonima de coprocessador): o destino nao e atualizado ate o opcode seguinte terminar e uma leitura no slot ve o valor antigo. | O teste vermelho com os 49 pares de loads reproduziu `Lb seguido de Lb`: o observador recebeu o primeiro resultado em vez de `0xCAFE_BABE`. |
| 2 | processo | Que o portao completo podia rodar antes de existir o doc pareado do manifesto. | O meta-teste de reconciliacao exige que cada manifesto tenha um doc e um `.resultado` versionado. | A primeira execucao de `cargo test --all --no-fail-fast` apontou a ausencia de `docs/iterations/0165-load-delay-encadeado.md`; a execucao tambem excedeu o timeout de 120 s nos testes Rayman. |
| 3 | processo | Que a mudanca local nao envelheceria ancoras de manifestos antigos. | Manifestos com ancora em `cpu.rs` precisam ser reancorados e sua bateria reexecutada quando o fonte muda. | `mutation_anchors` encontrou tres edicoes antigas de `0111`; reancorei e rodei `scripts/mutantes.ps1 -Iter 0111`, com `5/5` e `2/2`. |

## Bateria de mutação

Placar da bateria: **5/5 mutantes mortos, 2/2 controles verdes, 0 equivalente** — `docs/mutantes/0165-load-delay-encadeado.resultado`.

A bateria envelhecida de `0111-sp-desalinhado` foi reancorada e reexecutada: **5/5 mutantes mortos, 2/2 controles verdes, 0 equivalente**.

## Placar antes → depois

- Workspace: **915 -> 916 testes**.
- Amidog CPU: `Result: 00000109`, **588** ocorrencias `nop_.*_d value error` -> `Result: 00000109`, **0** ocorrencias.
- Portoes: `cargo fmt --all`, `cargo fmt --all -- --check`, `cargo clippy --all-targets -- -D warnings` e `cargo test --all --no-fail-fast`.

## Revisão cruzada (orquestrador)

Pendente: revisar adversarialmente o encadeamento de loads e o impacto em LWL/LWR.

## Decisões e notas

- A correcao nao cria uma fila geral: quando o novo load tem o mesmo destino, ele substitui o load pendente sem torna-lo visivel durante sua instrucao seguinte.
- Loads para registradores diferentes e escritas da ALU preservam a regra anterior de commit.
- A matriz local cobre todos os 49 pares de `lb`, `lbu`, `lh`, `lhu`, `lw`, `lwl` e `lwr` no mesmo destino.
- O item 10.92, de branches, nao foi iniciado.
