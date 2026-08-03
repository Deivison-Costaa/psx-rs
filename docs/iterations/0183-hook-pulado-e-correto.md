<!-- Custo, tokens e duracao ficam em docs/metricas.csv, medidos pelo runner. -->

# 0183 — hook-pulado-e-correto

- **Data:** 2026-08-03
- **Item do roadmap:** 0182.2 (fechado como refutado).
- **Objetivo:** fechar o achado 0182.2 em vez de deixá-lo na lista.
- **Fonte:** orquestrador.

**Não era defeito.** O comportamento que a 0182 mediu é o que a spec descreve.

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| psx-spx | § B(19h) - HookEntryInt(addr) (L1467-1481) | docs/reference/13-kernel-bios.md |
| psx-spx | § Priority Chains (L1484-1502) | docs/reference/13-kernel-bios.md |

## O caminho, medido

Instrumentei o `sw` para imprimir o PC de toda escrita em `I_STAT` que apague o bit 0. Em 250 M
passos, **345 de 348** vêm de **`pc=0x00004A20`** — código da BIOS em RAM, o mesmo `0x4A1C` que o
achado 10.80 já tinha apontado. A ordem, com `raise` e `ack` interleaved com o rastro do
despachante do jogo, é sempre a mesma:

```
# RAISE0
# ACK0-w32   (pc=0x00004A20, BIOS)
trace pc=0x801B8E98   (despachante do jogo)
```

A BIOS apaga o bit 0 antes de o jogo olhar. Eu tinha registrado isso como defeito.

## Por que é correto

§ Priority Chains (L1484-1502) de docs/reference/13-kernel-bios.md lista `VblankIrq` na
**prioridade 1**, e diz que um handler que processou e confirmou a IRQ "may execute
ReturnFromException, which causes the handlers of lower priority to be skipped".

E § B(19h) - HookEntryInt (L1476-1479) do mesmo arquivo fecha:

> "The hook function is executed only if the ExceptionHandler has been fully executed (after
> processing an IRQ, many interrupt handlers are calling ReturnFromException to abort further
> exception handling, and thus **do skip the hook function**)."

O hook do jogo roda **por último** e é pulado justamente quando o `VblankIrq` da BIOS trata a
interrupção. Ver `I_STAT` bit 0 em 10 de 660 oportunidades é o esperado, não uma falha.

E o jogo recebe o VBlank pelo caminho certo: `DeliverEvent(F0000001h,1000h)` é chamado **1723
vezes** e o contador do jogo em `0x801CF2CC` chega a **1469** em 700 M passos.

Por fim, o `VSync: timeout` não é da BIOS: "VSync" aparece como **símbolo de biblioteca** no
formato de arquivo de símbolos (§ SYM file format (L1321) de docs/reference/16-cdrom-file-formats.md),
isto é, vem da PSY-Q linkada no jogo, esperando o contador dele — que no início ainda não está
sendo alimentado porque o callback não foi registrado.

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | hardware | Que o hook do jogo não ver `I_STAT` bit 0 fosse defeito de ordem na cadeia de handlers, e registrei como achado 0182.2. | § B(19h) - HookEntryInt (L1476-1479) de docs/reference/13-kernel-bios.md: o hook é **pulado** quando um handler chama `ReturnFromException`. | Fui ler a seção do hook em vez de continuar medindo. Duas medições excelentes e uma conclusão errada — de novo o padrão de parar de ler a spec cedo demais, o mesmo que reprovei no lote D. |

## A mudança

O achado 0182.2 sai de `docs/achados.md` e vai para `docs/ROADMAP-fechado.md` marcado como
**refutado**, com o motivo. E a asserção de `vsync_timeout_diag` que comparava contador com
número de VBlanks deixou de dizer "se passar, o defeito 0182.2 foi corrigido" — ela agora
declara o que realmente guarda: contagem em duplicidade.

## Bateria de mutação

Bateria de mutação: não se aplica — a rodada não toca `crates/*/src/`; corrige um comentário de
asserção e move um item de arquivo.

## Placar antes → depois

Workspace: **1024 → 1024** testes. Achados abertos: **um a menos**, e nenhum acrescentado.

## Revisão cruzada (orquestrador)

Rodada de autoria do orquestrador. A refutação não depende de julgamento: é a leitura literal de
duas seções da spec contra uma medição de PC que aponta para código da BIOS.

## Decisões e notas

O bloqueio do Rayman **não** é o VBlank. Ele está na alça `0x80132BF0`, esperando o byte de
completude de um descritor de 20 bytes na tabela em `0x801CF5E0` — o mesmo byte que `0x80132B50`
zera antes de despachar o pedido. É lá que a próxima rodada deve olhar.
