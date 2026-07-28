<!-- Custo, tokens e duração NÃO entram aqui: são medidos por scripts/oc-iter.ps1 em
     docs/metricas.csv, casados por head_antes/head_depois. -->

# 0026 — spec-formato-psexe

- **Data:** 2026-07-28
- **Item do roadmap:** 1.11 (passo zero; o item em si fica para a próxima iteração)
- **Objetivo:** trazer para `docs/reference/` o capítulo que documenta o formato do
  executável PS-EXE, que faltava, e reescrever o handoff do 1.11 com offsets citados.

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| psx-spx | § PSX.EXE / FILENAME.EXE — header de 800h bytes (L1162-1184) | docs/reference/16-cdrom-file-formats.md (novo) |
| psx-spx | § memfill word-a-word, múltiplos de 4 (L1195) | docs/reference/16-cdrom-file-formats.md |
| psx-spx | § SP base=0 usa a pilha do chamador (L1188) | docs/reference/16-cdrom-file-formats.md |
| psx-spx | § parâmetros R4=1, R5=0 (L1200-1202) | docs/reference/16-cdrom-file-formats.md |

## Erros de primeira tentativa

O erro corrigido aqui é do handoff da 0025, não desta iteração. Registro completo na revisão
cruzada de `0025-cpu-tty-hook.md` (achado G1).

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | endereçamento | "O PC inicial do header é KSEG1 (`0xBFC0_xxxx`)" | `010h` Initial PC, "usually 80010000h" — KSEG0, em RAM. `BFC00000h` é o reset entrypoint da BIOS ROM (`14-io-map.md` L275) | Revisão do orquestrador no PR #39, antes de despachar o 1.11 |
| 2 | nenhuma (lacuna) | Que o layout do header estivesse em `docs/reference/` | `grep -r "PS-X EXE" docs/reference/` não devolvia nada: o capítulo `cdromfileformats.md` nunca tinha sido baixado | Mesma revisão |

## Bateria de mutação

Não se aplica: sem mudança em `crates/`.

## Placar antes → depois

221 → 221 testes (inalterado).

## Revisão cruzada (orquestrador)

Esta iteração é produto de uma revisão. Detalhe do achado em `0025-cpu-tty-hook.md` § G1.

## Decisões e notas

1. **O passo zero foi feito pelo orquestrador, não pelo trabalhador.** O handoff original
   mandava o trabalhador baixar o capítulo faltante. Preferi resolver antes de despachar:
   o item 1.11 tem catorze offsets de header, e a falha que a revisão do PR #39 acabara de
   pegar era justamente offset inventado. Com a spec local e as linhas citadas no handoff, a
   rodada do trabalhador começa com o dado na mão em vez de com uma tarefa de pesquisa.
2. **`fetch-reference-docs.ps1` é idempotente, e isso ficou comprovado.** Rodar com o
   capítulo 16 adicionado reescreveu os 16 arquivos, e o `git status` acusou apenas
   `16-cdrom-file-formats.md` (novo) e `README.md` (linha da tabela). Os 15 capítulos
   anteriores saíram byte a byte idênticos — o SHA pinado está fazendo o trabalho dele.
3. **Nomes dos campos: usar os da spec.** O handoff anterior falava em `t_addr`/`t_size`/
   `b_addr`/`b_size`, nomenclatura de outras ferramentas. O psx-spx chama de "Destination
   Address in RAM" (`018h`) e "Filesize" (`01Ch`), e reserva "Data section" para `020h`/`024h`
   — que é outro par de campos, tipicamente zero. Traduzir nome de campo entre dialetos é
   como offset errado nasce; o handoff agora usa a tabela da spec.
