<!-- Custo, tokens e duracao ficam em docs/metricas.csv, medidos pelo runner. -->

# 0161 — rayman-autoack

- **Data:** 2026-08-02
- **Item do roadmap:** 10.87
- **Objetivo:** medir por que o VBlank e reconhecido antes do hook do jogo.
- **Fonte:** orquestrador.

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| psx-spx | § B(13h) - StartPAD2() (L1951-L1953) | docs/reference/13-kernel-bios.md |
| psx-spx | § patch_no_pad_card_auto_ack: (L3512-L3553) | docs/reference/13-kernel-bios.md |
| psx-spx | § Priority Chains (L1494-L1502) | docs/reference/13-kernel-bios.md |

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | hipotese | Que reconhecer VBlank no caminho de prioridade 2 antes do hook era defeito nosso. | § patch_no_pad_card_auto_ack: (L3513-L3514) de `docs/reference/13-kernel-bios.md` diz que o handler de Pad/Card reconhece IRQ0 automaticamente **por projeto**, e que o jeito de desligar isso e `B(5Bh) ChangeClearPAD(int)`. | Lendo a seção. O comportamento que eu ia "corrigir" e o comportamento documentado do BIOS. |
| 2 | hipotese | Que o jogo simplesmente nao desligava o auto-ack. | Não é assunto de spec: é medição. | A sonda achou `ChangeClearPAD(0)` no passo 164.110.587, vindo de `0x801B8BC0` — RAM do jogo, 747 passos ANTES da instalação do hook. O jogo desliga, sim. |
| 3 | API-Rust | Que dava para montar o teste só com o `setup()` do `rayman_exception_chain.rs`. | Não é assunto de spec. | Faltaram `table_base`/`table_entry`; a primeira compilação parou em `cannot find function table_entry`. |

## Bateria de mutação

Bateria de mutação: não se aplica — a iteracao acrescenta somente um teste de integracao e documentacao; nenhum arquivo em `crates/*/src/` foi modificado, portanto nao ha producao para mutar. Em lugar dela, conferi o oraculo trocando `0x801B8BC0` por `0x801B8BC4` numa copia do teste: ele reprova (`left: 2149288896, right: 2149288900`), entao a assercao mede valor e nao rotulo.

## Placar antes → depois

Workspace: **905 → 906** testes.

## Revisão cruzada (orquestrador)

Rodada de autoria do orquestrador; autorrevisão registrada como limite. A rodada **não** altera
produção e o seu resultado principal é ter derrubado a hipótese de defeito que o handoff anterior
carregava.

## Decisões e notas

A sequência medida, toda dentro de 13.264 passos:

| Passo | Chamada | Argumento | `$ra` | Quem |
|---:|---|---:|---|---|
| 164.110.587 | `B(5Bh) ChangeClearPAD` | 0 | `0x801B8BC0` | jogo |
| 164.111.334 | `B(19h) HookEntryInt` | — | `0x801B8FF4` | jogo |
| 164.113.220 | `B(12h) InitPAD2` | — | `0x801A78D4` | jogo |
| 164.123.374 | `B(13h) StartPAD2` | — | `0x801A7958` | jogo |
| 164.123.851 | `B(5Bh) ChangeClearPAD` | **1** | `0x00004BEC` | **kernel** |

O jogo desliga o auto-ack de IRQ0 e instala o próprio handler de VBlank — exatamente o roteiro
que § patch_no_pad_card_auto_ack: (L3548-L3550) de `docs/reference/13-kernel-bios.md` descreve
como *"probably desired for most games"*. Dez mil passos depois ele chama `StartPAD2`, e
§ B(13h) - StartPAD2() (L1952-L1953) do mesmo arquivo avisa, em nove palavras, o que acontece:
*"Enqueues the PadCardIrq handler, and does additionally initialize some flags"*. Uma dessas
flags é o auto-ack: a religada de `ChangeClearPAD(1)` vem de `0x00004BEC`, RAM do kernel, 477
passos depois do `StartPAD2`, e não do jogo.

Daí em diante o elemento de prioridade 2 reconhece IRQ0 antes do hook (0158), o hook sai em 26
instruções sem incrementar (0160), e `[0x801CF2CC]` fica parado em 1 enquanto o laço de
`0x801B9574` espera 2 (0159).

**O que isso fecha:** não há correção de produção a fazer no caminho da cadeia. Reconhecer IRQ0
no handler de Pad/Card com auto-ack ligado é o comportamento documentado do BIOS, e a religada
partiu do próprio kernel, não de código nosso.

**O que isso abre:** no hardware real esta mesma sequência levaria ao mesmo estado, então o jogo
depende de algo que ainda não acontece aqui. As duas pontas soltas, ambas medidas:

1. O jogo espera pelos eventos de memory card `F1000001h` (descritor do slot 1, `F4000001h,0004h`
   *card done*) e `F1000004h` (slot 4, `F4000001h,2000h` *card err eject*) — 454.122 chamadas de
   `TestEvent` entre 86.989.128 e 166.322.304. O BIOS entrega `F4000001h,0100h` (*err busy*), que
   não é nenhum dos dois. Sem card nenhum no slot, o desfecho esperado é *eject*, não *busy*.
2. A rotina do jogo em `0x801B8Bxx` que desliga o auto-ack rodou uma vez só. Se no hardware real
   ela roda de novo depois do `StartPAD2`, é porque o jogo chegou a um ponto do fluxo que aqui
   ele não alcança — e a ponta 1 é a candidata mais próxima.

O próximo item é a ponta 1: por que o caminho de card do BIOS conclui *busy* em vez de *eject*
com o slot vazio.
