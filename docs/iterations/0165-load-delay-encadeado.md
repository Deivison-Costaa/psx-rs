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

Placar da bateria: **6/6 mutantes mortos, 2/2 controles verdes, 0 equivalente** — `docs/mutantes/0165-load-delay-encadeado.resultado`.
O sexto mutante entrou na revisão (ver abaixo): descarta o load recém-emitido junto com o pendente.

A bateria envelhecida de `0111-sp-desalinhado` foi reancorada e reexecutada: **5/5 mutantes mortos, 2/2 controles verdes, 0 equivalente**.

## Placar antes → depois

- Workspace: **915 -> 917 testes**.
- Amidog CPU: `Result: 00000109`, **588** ocorrencias `nop_.*_d value error` -> `Result: 00000109`, **0** ocorrencias.
- Portoes: `cargo fmt --all`, `cargo fmt --all -- --check`, `cargo clippy --all-targets -- -D warnings` e `cargo test --all --no-fail-fast`.

## Revisão cruzada (orquestrador)

Rodada do trabalhador (`openai/gpt-5.6-luna`), revisada pelo orquestrador antes do merge.

**Achado 1 — asserção incompleta.** A matriz de 49 pares afirmava só o que o observador lê
durante o delay (`INITIAL`). Uma implementação que jogasse fora **os dois** loads passaria no
teste: o valor final do destino nunca era conferido. É a mesma fraqueza da iteração 0160
(`ctrl_bit4_ack_limpa_stat_bit9` era vacuoso). O oráculo do Amidog cobria esse flanco — as 588
linhas só zeram se o valor final bate — mas o teste do repositório tem de sustentar sozinho.
Acrescentei `a_segunda_carga_encadeada_chega_ao_destino_quando_o_delay_dela_termina`, que fixa
`0x7654_3210` para o par `lw`/`lw`, e o mutante `m6`, que descarta o load recém-emitido. Sem a
asserção nova o `m6` sobrevive; com ela morre.

**Achado 2 — reancoragem do 0111 não afrouxou a bateria.** As três âncoras editadas apenas
absorveram o `&& !replaced_by_load` no texto casado; `m1` continua invertendo a guarda e `m2`
continua removendo-a. Reexecutada: 5/5 e 2/2.

**Conferido também:** manifesto com âncoras reais e pares `(de,para)` únicos; nenhum mutante
morto por erro de compilação; o `.resultado` está versionado; a citação de spec aponta a seção de
memória e não a homônima de coprocessador; CI verde nos quatro jobs.

## Decisões e notas

- A correcao nao cria uma fila geral: quando o novo load tem o mesmo destino, ele substitui o load pendente sem torna-lo visivel durante sua instrucao seguinte.
- Loads para registradores diferentes e escritas da ALU preservam a regra anterior de commit.
- A matriz local cobre todos os 49 pares de `lb`, `lbu`, `lh`, `lhu`, `lw`, `lwl` e `lwr` no mesmo destino.
- O item 10.92, de branches, nao foi iniciado.
