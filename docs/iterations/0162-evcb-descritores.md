<!-- Custo, tokens e duracao ficam em docs/metricas.csv, medidos pelo runner. -->

# 0162 — evcb-descritores

- **Data:** 2026-08-02
- **Item do roadmap:** 10.88
- **Objetivo:** conferir de quem sao os descritores de evento que o jogo fica esperando.
- **Fonte:** orquestrador.

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| psx-spx | § Table of Tables (L438-L455) | docs/reference/13-kernel-bios.md |
| psx-spx | § Event Classes (L1656-L1698) | docs/reference/13-kernel-bios.md |
| psx-spx | § C(08h) - SysInitMemory(addr,size) (L2551-L2554) | docs/reference/13-kernel-bios.md |

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | hipotese | Que os descritores `F1000001h`/`F1000004h` que o jogo consulta eram eventos de **memory card** — foi o que escrevi no doc de 0161 e no item 10.88. | § Table of Tables (L455) de `docs/reference/13-kernel-bios.md` da a formula `evcb=[120h]+(event AND FFFFh)*1Ch`: o descritor e um **indice de slot**, e o conteudo do slot muda com o tempo. | Lendo o EvCB no passo em que a espera COMECA (86.989.000) em vez de no fim da execucao: os slots 1 e 4 eram `F0000003h` spec `0020h` (*CDROM command completed*) e spec `8000h` (*CDROM error*). A leitura anterior era do fim, depois de a tabela ter sido refeita. **O item 10.88 nasceu de uma inferencia errada minha.** |
| 2 | hipotese | Que a espera de 454.122 `TestEvent` era um travamento unico. | Não é assunto de spec. | `F0000003h,0020h` foi entregue 27 vezes entre 87.010.147 e 163.809.635: sao muitas esperas curtas de CDROM que terminam, nao uma so que nunca termina. |

## Bateria de mutação

Bateria de mutação: não se aplica — a iteracao acrescenta somente um teste de integracao e documentacao; nenhum arquivo em `crates/*/src/` foi modificado, portanto nao ha producao para mutar. Conferi o oraculo trocando o passo esperado de `154_897_433` por `154_897_434`: o teste reprova, entao a assercao mede valor.

## Placar antes → depois

Workspace: **906 → 907** testes.

## Revisão cruzada (orquestrador)

Rodada de autoria do orquestrador; autorrevisão registrada como limite. O resultado principal e
uma **correcao de registro**: o doc de 0161 e o item 10.88 afirmavam que o jogo esperava eventos
de memory card. Nao esperava. A medicao nova esta no teste permanente
`rayman_evcb_descritores.rs`, que afirma o conteudo dos dois slots nos dois momentos.

## Decisões e notas

O mesmo par de descritores aponta para eventos diferentes conforme o momento:

| Momento | slot 1 (`F1000001h`) | slot 4 (`F1000004h`) |
|---|---|---|
| passo 86.989.000, quando a espera comeca | `F0000003h` spec `0020h` — CDROM *command completed* | `F0000003h` spec `8000h` — CDROM *error* |
| passo 165.000.000 | `F4000001h` spec `0004h` — card *done* | `F4000001h` spec `2000h` — card *err eject* |

O que troca o dono e uma reinicializacao: `C(08h) SysInitMemory` roda de novo no passo
**154.897.433**, com `$ra = 0xBFC06F4C` — ROM do BIOS, nao codigo do jogo.
§ C(08h) - SysInitMemory(addr,size) (L2553) de `docs/reference/13-kernel-bios.md` diz que ela
*"seems to deallocate any memory handles which may have been allocated via B(00h)"*, e o mapa em
§ Kernel Memory Map (L428) do mesmo arquivo diz que e exatamente ali que moram
*"ExCBs, EvCBs, and TCBs allocated via B(00h)"*. Depois dela o BIOS reabre os eventos de CDROM
(de `0xBFC071C8` em diante) e, mais tarde, o jogo abre os de card (de `0x8016A8A8`), caindo nos
slots que antes eram do CDROM.

Isso reforca, com numero, a "premissa refutada" que ja estava no STATUS desde 0147: o problema
esta no **encaixe temporal** entre `SysInitMemory` e o que o jogo tinha registrado antes, nao no
valor de um slot.

Duas consequencias praticas:

1. **O item 10.88 esta invalidado como estava escrito.** Nao ha nada a corrigir no caminho de
   card por causa dessa espera: com o slot vazio, `err busy` nao contradiz nada que o jogo
   consulte — o jogo nem estava olhando para card quando comecou a esperar.
2. A pergunta viva continua sendo a do contador de VSync (`[0x801CF2CC]`, item 10.85), agora com
   um vizinho novo: por que a ROM reinicializa a memoria do kernel no meio da execucao do jogo,
   no passo 154.897.433. Se essa reinicializacao nao devia acontecer, ela derruba junto tudo o
   que o jogo registrou antes dela.
