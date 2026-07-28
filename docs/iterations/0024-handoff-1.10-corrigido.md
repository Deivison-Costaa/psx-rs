<!-- Custo, tokens e duração NÃO entram aqui: são medidos por scripts/oc-iter.ps1 em
     docs/metricas.csv, casados por head_antes/head_depois. -->

# 0024 — handoff-1.10-corrigido

- **Data:** 2026-07-28
- **Item do roadmap:** 1.10 (preparação; o item em si fica para a próxima iteração)
- **Objetivo:** corrigir o handoff do hook de TTY, que descrevia um mecanismo de hardware
  inexistente, antes de gastar uma rodada do trabalhador implementando-o.

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| psx-spx | § Parameters, Registers, Stack (L481) | docs/reference/13-kernel-bios.md |
| psx-spx | § A-Functions — Call 00A0h with function number in R9 Register (L496) | docs/reference/13-kernel-bios.md |
| psx-spx | § A(3Ch) or B(3Dh) - putchar(char) (L2776) | docs/reference/13-kernel-bios.md |
| psx-spx | § A(3Eh) or B(3Fh) - puts(src) (L2742) | docs/reference/13-kernel-bios.md |

## Erros de primeira tentativa

Os três erros abaixo estavam no **handoff** escrito na iteração 0022, não em código. Foram
pegos pelo orquestrador no passo 3 (spec primeiro) da iteração seguinte, antes de qualquer
implementação. É o caso que a regra R1 existe para produzir: a intuição de "MIPS tem syscall,
logo a BIOS usa syscall" é plausível, é errada, e teria custado uma rodada inteira.

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | endereçamento | `A0h`/`B0h` são códigos de serviço passados num registrador para a instrução `syscall` | São **endereços de chamada**: `jal 0x000000A0`/`0xB0`/`0xC0`. A tabela do `syscall` é outra (SYS-Functions, L504), com o número em R4 | Leitura de L496 antes de despachar o trabalhador |
| 2 | flags | O número da função viria em `$v0` (ou `$t4`/`$t5`, "a confirmar") | Vem em **R9** (`$t1`), sem ambiguidade nenhuma no título da seção | Mesma leitura; a dúvida registrada no STATUS simplesmente não existia na spec |
| 3 | flags | `A0h` = putchar e `B0h` = puts | `putchar` é A(3Ch) **ou** B(3Dh); `puts` é A(3Eh) **ou** B(3Fh). A mesma função existe nas duas tabelas com números diferentes | L2776 e L2742 |

Consequência de bônus: a "armadilha 1" do handoff antigo (precisaria de BIOS real ou de um
mini-handler em RAM para testar) era um problema derivado do mecanismo errado. Com o hook
em `jal 0xA0`, o teste não precisa de BIOS.

## Bateria de mutação

Não se aplica: iteração de processo, sem mudança em `crates/`.

## Placar antes → depois

212 → 212 testes (inalterado).

## Revisão cruzada (orquestrador)

Esta iteração *é* o resultado de uma revisão do orquestrador. O achado equivalente do lado do
processo: o handoff da 0022 afirmou comportamento de hardware ("o BIOS chama `syscall` com
`$a0 = 0x3D`") sem citar seção de spec, e ainda assim passou para o STATUS. As demais seções
de handoff do STATUS citam arquivo e seção; esta não citava — o campo **Spec** dizia
literalmente "seções de syscall/PUTCHAR no psx-spx (buscar)".

## Decisões e notas

1. **Handoff sem citação de seção é handoff suspeito.** O sinal de alerta desta iteração foi
   sintático, não semântico: o campo **Spec** dizia "(buscar)" em vez de apontar arquivo e
   linha. Regra adotada a partir daqui: um handoff cujo campo **Spec** não aponte
   `arquivo + seção` não deve ser despachado ao trabalhador; ele volta para o orquestrador.
2. **O hook observa, não substitui.** Ao disparar em `PC == A0h/B0h`, o hook grava a saída e
   deixa a execução seguir para o código da BIOS. A alternativa (emitir e saltar direto para
   RA) quebraria qualquer função A/B que não seja de TTY e mudaria o comportamento quando a
   BIOS real estiver carregada.
3. **Byte cru, sem expansão de TAB/LF — ASSUMIDO.** O `putchar` da BIOS real expande `09h`
   para espaços até o próximo múltiplo de 8 e `0Ah` para `0Dh,0Ah` (L2778-2780). Como o hook
   observa a chamada em vez de implementar a função, ele grava o byte como veio. Ponto de
   resolução: comparar com a saída de uma BIOS real quando o runner existir (1.11+).
4. **Comparar o endereço físico.** `A0h` pode chegar como `0x000000A0`, `0x800000A0` ou
   `0xA00000A0`; o hook mascara com `0x1FFF_FFFF` antes de comparar.
