# 0201 — gpu-byte-index-mask

- **Data:** 2026-08-06
- **Item:** Achado 10.51 (indice de byte sem mascara na leitura de portas da gpu)
- **Objetivo:** impedir o shift invalido de `u32` em leituras de 16 bits no limite da palavra da gpu.

## O que entrou

- Teste de integracao que reseta o estado da gpu, fixa `0x1480_2000` como valor conhecido e le
  `0x1F80_1817`; antes do fix, a segunda chamada com `offset = 1` entrava em `>> 32`.
- Mascara final `& 3` no calculo de `byte_index` do unico braco de leitura da gpu.
- Nenhuma alteracao em `gpu.peek32` ou na semantica dos registradores.

## Spec consultada

| Fonte | Secao | Arquivo local |
|---|---|---|
| nenhuma | nao aplicavel: defeito aritmetico local, sem semantica de hardware nova | nao consultado |

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | enderecamento | A linha textual do indice identificava so o braco da gpu. | Nao houve consulta de spec; o arquivo tinha tres linhas identicas em bracos diferentes. | O `grep` apos o patch mostrou que a primeira ocorrencia alterada era a de `mem_ctrl`; o diff foi corrigido antes do commit. |
| 2 | processo | O PowerShell preservaria os zeros de `0201` no argumento da bateria. | Nao e assunto de spec; o script precisa receber o prefixo de quatro digitos como texto. | A primeira chamada recebeu `201` e nao encontrou o manifesto; a segunda usou `'0201'`. |
| 3 | processo | O resultado gerado pela bateria poderia ficar apenas na arvore de trabalho. | O meta-teste exige o `.resultado` rastreado para reconciliar clones limpos. | A primeira rodada de `cargo test --all` falhou em `mutation_battery`; o resultado foi adicionado ao Git. |

## Bateria de mutacao

Placar da bateria: **6/6 mutantes mortos, 2/2 controles verdes, 0 equivalente.**

- m1 sem mascara: panic de shift capturado por `gpu_read16_dobra_indice_de_byte_no_limite_da_palavra`.
- m2 com mascara `& 2`: byte incorreto capturado pelo mesmo teste.
- m3 com mascara `& 1`: byte incorreto capturado pelo mesmo teste.
- m4 com mascara `& 7`: shift invalido capturado pelo mesmo teste.
- m5 com mascara `& 0`: palavra incorreta capturada pelo mesmo teste.
- m6 mascara a parte fisica com `2`: ordem de bytes incorreta capturada pelo mesmo teste.
- c1 troca a ordem dos operandos da soma e permanece verde.
- c2 mascara o offset antes da soma e permanece verde.

## Placar antes -> depois

- Workspace: 1242 -> 1243 testes, com um novo teste de integracao em `bus_gpu_byte_index_mask.rs`.
- `cargo fmt --all -- --check`: verde.
- `cargo clippy --all-targets -- -D warnings`: verde.
- `cargo test --all`: verde.

## Revisao cruzada (orquestrador)

Pendente; este PR deve parar antes do merge para revisao adversarial.

## Decisoes e notas

- O endereco `0x1F80_1817` exercita `phys & 3 == 3` e o segundo acesso de `read16`; a mascara
  faz o indice voltar a zero dentro da palavra de 32 bits.
- O achado 10.51 foi removido de `docs/achados.md` e adicionado a `docs/ROADMAP-fechado.md`.
- `STATUS.md` nao foi alterado, conforme instrucao do lote do orquestrador.
