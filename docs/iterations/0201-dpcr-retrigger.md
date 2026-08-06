# 0201 — dpcr-retrigger

- **Data:** 2026-08-05
- **Item do roadmap:** Achado 10.30 (habilitar canal no DPCR nao dispara transferencia pendente)
- **Objetivo:** reexecutar os canais DMA armados quando uma escrita no DPCR habilitar o canal.

## Spec consultada

| Fonte | Secao | Arquivo local |
|---|---|---|
| psx-spx | § 1F8010F0h - DPCR - DMA Control Register (R/W) (L121) | docs/reference/04-dma.md |

## O que entrou

- `Bus::region_write32`, no arm de `0x1F80_10F0`, grava o DPCR e tenta os canais 0, 1, 2,
  3, 4 e 6. Cada `try_execute_dmaN` continua responsavel por seus proprios gates e por
  retornar sem efeito quando o canal nao esta armado ou ainda nao tem DREQ.
- A ordem confirma os enables da spec: DMA0=bit 3, DMA1=7, DMA2=11, DMA3=15, DMA4=19,
  DMA5=23 e DMA6/OTC=27. O canal 5 nao tem `try_execute` no modelo atual e permanece sem
  chamada.
- `service_dma_irq` e chamado depois das tentativas para propagar uma conclusao que tenha
  levantado a IRQ3.

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | processo | Que `-Iter 0201` preservaria o zero inicial no PowerShell | O runner exige o identificador textual de quatro digitos para encontrar o manifesto | A primeira chamada procurou a iteracao `201`; a segunda usou `-Iter '0201'` e executou 6/6 + 2/2 |
| 2 | processo | Que a suite completa poderia rodar antes de versionar a prova da bateria | O meta-teste reconcilia o placar contra um `.resultado` rastreado pelo git | `cargo test --all` reprovou somente por `0201-dpcr-retrigger.resultado` nao rastreado; ele foi commitado e a suite foi repetida |
| 3 | API-Rust | Que o import de `Bus` seria necessario por os testes chamarem `write32` | Os metodos sao usados pelo tipo inferido e so `BusRead` precisa estar no escopo | A primeira compilacao do teste acusou import nao usado; o import foi removido antes do commit vermelho |

## Bateria de mutacao

Placar da bateria: **6/6 mutantes mortos, 2/2 controles verdes, 0 equivalente.**

| Registro | Tipo | Alteracao | Teste que pegou |
|---|---|---|---|
| m1 | mutante | DPCR zerado antes do retrigger | `otc_armado_antes_do_dpcr_e_reexecutado_ao_habilitar_canal` |
| m2 | mutante | bit 27 do OTC removido da escrita | `otc_armado_antes_do_dpcr_e_reexecutado_ao_habilitar_canal` |
| m3 | mutante | retorno antes da tentativa do canal 0 | `otc_armado_antes_do_dpcr_e_reexecutado_ao_habilitar_canal` |
| m4 | mutante | retorno antes das tentativas dos canais 1 e 2 | `otc_armado_antes_do_dpcr_e_reexecutado_ao_habilitar_canal` |
| m5 | mutante | retorno antes da tentativa do canal 2 | `otc_armado_antes_do_dpcr_e_reexecutado_ao_habilitar_canal` |
| m6 | mutante | OTC recebe uma fatia de RAM sem espaco | `otc_armado_antes_do_dpcr_e_reexecutado_ao_habilitar_canal` |
| c1 | controle | OR por zero preserva o valor do DPCR | nenhum (sobreviveu) |
| c2 | controle | fatia completa preserva a RAM do OTC | nenhum (sobreviveu) |

## Placar antes -> depois

- Workspace: **1242 -> 1243** testes (+1 em `dma_dpcr_retrigger.rs`).
- `cargo fmt --all -- --check` e `cargo clippy --all-targets -- -D warnings` passaram.
- `cargo test --all` passou em todos os testes de codigo e meta-testes do item, mas o alvo
  `status_handoff` reprovou `placar_do_status_bate_com_a_contagem_de_testes`: o workspace tem
  1243 testes e `STATUS.md` ainda registra 1242. O arquivo foi deixado intacto por instrucao
  do orquestrador; esta pendencia deve ser resolvida na consolidacao do lote.

## Revisao cruzada (orquestrador)

Pendente — revisar no PR antes do merge.

## Decisoes e notas

- O fix ficou somente no barramento. `Dma::write_dpcr`, o formato do DPCR e todos os
  `try_execute_dmaN` permaneceram inalterados.
- Tentar todos os canais e seguro porque cada tentativa e idempotente para um canal nao
  armado ou ja concluido; isso evita duplicar no barramento a tabela de bits do DPCR.
- `STATUS.md` nao foi alterado: o orquestrador esta consolidando o lote e atualizara o
  handoff depois desta rodada.
