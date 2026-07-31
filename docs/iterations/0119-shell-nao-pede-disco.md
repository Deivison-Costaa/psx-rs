<!-- Custo, tokens e duração NÃO entram aqui: são medidos por scripts/oc-iter.ps1 em
     docs/metricas.csv, casados por head_antes/head_depois. -->

# 0119 — shell-nao-pede-disco

- **Data:** 2026-07-30
- **Item do roadmap:** 4.4n
- **Objetivo:** medir quem escreve `[0x80083C58]` e por que o driver de CD-ROM do kernel não
  conclui, como o handoff da 0118 mandou. **Iteração de diagnóstico: sem código de produção.**

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| psx-spx | § `0x1f801803` (write, bank 1): `HCLRCTL` | docs/reference/06-cdrom.md |
| psx-spx | § `0x1f801800` (read): `HSTS` | docs/reference/06-cdrom.md |
| psx-spx | § Controller Communication Sequence | docs/reference/10-controllers-memcards.md |

## O que foi medido

Harness `cdstate` (evolução do `cdshell` da 0118, como o handoff pedia — watch da variável,
janela de portos do CD, contagem do handler e bytes de endereço do SIO0).

**1. A troca do `GetStat` está completa e correta do nosso lado.** Janela de 87,4 M a 87,6 M:

```
  87464188  pc=0x80059C1C  W porta0 banco0 val=0x01   ; seleciona banco 1
  87464192  pc=0x80059C2C  W porta3 banco1 val=0x40   ; HCLRCTL: limpa FIFO de parametros
  87464249  pc=0x80057540  W porta0 banco1 val=0x00   ; volta ao banco 0
  87464254  pc=0x80057554  W porta1 banco0 val=0x01   ; comando GetStat
  87464412  pc=0x800584D8  R porta0 banco0            ; poll do HSTS
  87464419  pc=0x800584F4  R porta3 banco1            ; le HINTSTS
  87464449  pc=0x80058808  R porta1 banco1            ; le a resposta
  87464728  pc=0x800585C0  W porta3 banco1 val=0x07   ; HCLRCTL: ack INT1|INT2|INT3
  87464782  pc=0x80058614  W porta0 banco1 val=0x00   ; volta ao banco 0
```

Comando, poll, leitura do status de interrupção, leitura da resposta, ack. Nada falta. E depois
disso: **zero acesso aos portos do CD-ROM em 312 M passos**.

**2. `[0x80083C58]` não está travada — ela cicla.** As escritas, com o PC de cada uma:

```
  115686004  pc=0x8003D6CC  valor=2
  115909390  pc=0x8003D49C  valor=1     (223 k passos depois)
  115954485  pc=0x8003D6CC  valor=2
  116250262  pc=0x8003D49C  valor=1
  116576950  pc=0x8003D49C  valor=0     <-- concluiu uma vez
  123298863  pc=0x8003D6CC  valor=2     <-- e daqui em diante 2->1->2->1 para sempre
```

Decodificando à mão: `0x8003D49C` é `lw $t7,[0x80083C58] / addiu $t8,$t7,-1 / sw $t8,...` — um
**decremento**. `0x8003D6CC` escreve 2 no epílogo de uma função. E `0x8003D6D8` é a função de
espera: `if (a0 == 1 && [0x80083C58] >= 2) { gira }`. Ou seja, é um ciclo
**posta → expira → decrementa → repõe**, com cadência de ~um quadro. Não é um deadlock esperando
evento de CD que nunca chega, como o handoff supôs.

**3. As interrupções continuam correndo.** 1219 entradas no handler `0x80000080`, das quais 417
depois do `GetStat`.

## Hipóteses refutadas

| # | Hipótese | Como foi refutada |
|---|---|---|
| 1 | O driver de CD trava esperando o evento de conclusão do `GetStat` (o que o handoff da 0118 escreveu). | A troca do `GetStat` termina com ack completo, e a variável de estado **cicla** em vez de ficar presa. Nenhum comando novo chega ao porto — o bloqueio é a montante do driver, não dentro dele. |
| 2 | O shell está preso sondando **memory card** (item 6.3 aberto, sem modelo de cartão). | Contando os bytes de endereço escritos em `JOY_TX_DATA` depois de 110 M passos: **só `0x01`** (endereço do controle), 100 vezes — cerca de uma por quadro. Nenhum `0x81`. O shell não sonda cartão. |
| 3 | O bit de motor ligado no stat byte manda o shell por um caminho errado (`insert_disc()` liga `motor_on` sem `Init`). | Experimento: `insert_disc()` deixando o motor desligado. A sequência de comandos ficou **idêntica** (`Test(20h)` + `GetStat`, nada mais). Patch descartado, árvore restaurada com `git checkout --`. |
| 4 | É problema de sistema de arquivos / leitura de setor falhando (a leitura do `SYSTEM.CNF`). | Já refutada na 0118 e reconfirmada aqui: nenhum `Setloc`, nenhum `ReadN`, `HINTSTS==INT1` zero. Não há leitura para falhar. |

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a medição diz | Como foi pego |
|---|---|---|---|---|
| 1 | diagnóstico | Que `[0x80083C58]` era o estado do driver de CD travado, e que bastaria achar quem devia decrementá-lo. | É um contador de posta/expira/retenta com cadência de quadro, e chega a zero uma vez. O nome que eu dei a ele no handoff da 0118 ("estado do driver de CD-ROM") não está provado — o que está provado é o formato do ciclo. | O próprio watch pedido pelo handoff: as escritas mostram o padrão, não um valor preso. |
| 2 | processo | Que valia continuar instrumentando até achar o defeito nesta iteração. | Quatro hipóteses refutadas e nenhuma confirmada; o próximo passo barato não é mais instrumentação cega, é **comparar com um emulador de referência** rodando a MESMA BIOS e o MESMO disco, e diferenciar a sequência de comandos. | Custo: três reconstruções do harness e um experimento descartado. Registrado como invariante 27. |

## Bateria de mutação

Bateria de mutação: não se aplica — esta iteração não altera código de produção. Ela entrega a
troca do `GetStat` verificada porto a porto, o formato do ciclo de `[0x80083C58]`, quatro hipóteses
refutadas (incluindo duas minhas, do handoff anterior) e a redefinição do item como 4.4o.

## Placar antes → depois

Workspace: **790** testes, 0 falhas (inalterado — nenhuma linha de produção mudou).

## Revisão cruzada (orquestrador)

- **Árvore conferida.** O experimento da hipótese 3 foi revertido com `git checkout --` e o
  `git status --porcelain` saiu vazio antes do commit; `crates/psx-core/src/bin` removido.
- **O diff é só documentação.** Nenhum arquivo de `crates/` no PR.
- **Gates do projeto:** `roadmap_size`, `status_size`, `status_handoff`, `spec_citations` e
  `mutation_battery` verdes (o último não exige manifesto para iteração sem alvo).

## Decisões e notas

- **O que ficou provado.** O shell da BIOS, depois de receber a resposta do `GetStat`, **decide não
  perguntar mais nada ao disco**. Ele fica num laço por quadro: lê o controle (endereço `0x01`),
  cicla um contador interno com cadência de timeout, e não toca em drive nem em cartão. Nosso lado
  da conversa do `GetStat` está correto porto a porto.
- **O que NÃO ficou provado.** Qual bit da resposta, ou qual estado anterior, faz o shell decidir
  isso. Chamar `[0x80083C58]` de "estado do driver de CD" foi um chute meu na 0118 e não se
  sustenta.
- **Próximo passo, e por que ele é diferente.** Depois de quatro hipóteses refutadas por
  instrumentação, o discriminador mais barato deixou de ser outro harness: é rodar a **mesma BIOS
  com o mesmo disco** num emulador de referência (o DuckStation já está em
  `psx-estado/referencias/`) e comparar a sequência de comandos do CD-ROM. Se lá aparece um `GetID`
  depois do `GetStat`, o que falta é o que provoca esse `GetID`; se lá também não aparece, o shell
  está esperando outra coisa e o alvo muda de subsistema. É o item 4.4o.
