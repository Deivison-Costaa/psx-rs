<!-- Custo, tokens e duração NÃO entram aqui: são medidos por scripts/oc-iter.ps1 em
     docs/metricas.csv, casados por head_antes/head_depois. -->

# 0203 — cause-ce

- **Data:** 2026-08-05
- **Item do roadmap:** 10.4 (achado legado, sem iteração de origem registrada)
- **Objetivo:** CAUSE.CE tem que registrar o número do coprocessador quando a exceção é
  Coprocessor Unusable (0Bh); hoje fica sempre zero.

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| psx-spx | § cop0r13 - CAUSE, campo CE (L681) | docs/reference/02-cpu.md |

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a medição mostrou | Como foi pego |
|---|---|---|---|---|
| 1 | manutenção | Que mudar a assinatura de `enter_exception` (novo parâmetro `ce`) não afetaria nada fora do arquivo do item | Dois manifestos antigos (`0164-fetch-desalinhado.mut`, `0172-cpu-oraculo.mut`) ancoravam chamadas de 5 argumentos que deixaram de existir | `cargo test --test mutation_anchors` reprovou com "ancora esperada 1 vez(es) mas encontrada 0"; arquivados com `arquivada:` citando o motivo, seguindo o precedente de 0042/0047 |
| 2 | manutenção | Que o campo novo em `Cpu` não teria efeito visível fora do CPU | `Cpu` deriva `serde::Serialize` para save states; `TAMANHO_DO_ESTADO` (`snapshot_estado.rs`) é uma constante fixa que reprovou por +1 byte | `cargo test --test snapshot_estado`: `estado_codificado_tem_tamanho_fixo_conhecido` falhou com o tamanho exato — atualizada a constante |
| 3 | escopo | Que limpar CAUSE.CE em toda exceção seria "mais correto" e devia fazer parte do fix | A spec só define CE para CpU; para outros ExcCode o valor é implícito/não especificado. Zerar CE em toda exceção seria um comportamento NOVO, não coberto pelo achado nem por nenhum teste — ficou de fora por R4 (uma micro-funcionalidade) | Releitura do texto da spec antes de escrever o `match`: só limpa/escreve os bits 28-29 quando `ce` é `Some` |

## Bateria de mutação

Placar da bateria: 5/5 mutantes mortos, 2/2 controles verdes, 0 equivalente —
`docs/mutantes/0203-cause-ce.mut`.

- m1 (deslocamento errado, 26 em vez de 28): morto.
- m2 (`pending_ce` nunca setado): morto.
- m3 (máscara `0x1` perde o bit alto do coprocessador): morto — distingue cop1 (01) de cop3 (11).
- m4 (ramos `Some`/`None` do match invertidos): morto.
- m5 (CE sempre reporta 0, ignora qual coprocessador disparou): morto.
- c1 (ordem entre `pending_ce`/`raise_exception`, consumidos juntos depois): verde.
- c2 (renomeia `ce`→`coprocessador`, assinatura + uso no mesmo registro): verde.

## Placar antes → depois

Workspace: **1246** → **1247** testes (1 novo em `cpu_coprocessador_usavel.rs`).

## Revisão cruzada (orquestrador)

Sem achados — esta iteração foi conduzida pelo próprio orquestrador (exceção vigente em
`docs/orquestracao.md`; ver STATUS.md).

## Decisões e notas

**1. CE só é escrito para exceções CpU.** Para as demais (overflow, syscall, break, AdEL/AdES,
etc.) os bits 28-29 do CAUSE ficam como estavam antes — a spec não define o valor de CE fora
de Coprocessor Unusable, e não há teste (nem jogo conhecido) que dependa de um valor
específico nesse caso. Se isso vier a importar, é achado novo, não parte do 10.4.
