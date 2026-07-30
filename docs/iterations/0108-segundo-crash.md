# 0108 — segundo-crash

- **Data:** 2026-07-30
- **Item do roadmap:** 4.4h
- **Objetivo:** o boot da BIOS não avança depois do logo e nunca lê o disco. Descobrir por quê.
  **Resultado: diagnóstico completo, sem correção.** O conserto é o item 4.4h, que abre com este
  handoff pronto.

## Revisão do PR anterior

PR #123 (iter 0107), do próprio orquestrador: quatro checks verdes, `headRefOid` conferido,
iteração sem código e sem bateria, com a justificativa registrada.

## Spec consultada

Nenhuma seção nova. O opcode `0x1D` não constar da tabela de `docs/reference/02-cpu.md` é o que
torna correta a exceção RI que observamos — é a única consulta, e é negativa.

## O que foi medido

Partindo do sintoma "a BIOS chega ao logo e para":

1. **Nenhum comando de CD-ROM.** Contador em `Cdrom::send_command`: **zero** chamadas em 800 M
   passos. A BIOS nunca fala com o drive.
2. **Nenhum TTY novo.** Corrida de 3 bilhões de passos (4,38 bilhões de ciclos ≈ **129 s
   emulados**): nada depois de `ResetCallback: _96_remove ..`.
3. **Histograma de PC nos últimos 20 M passos:** 78 % em `0x00000xxx`, 21 % em `0xBFC0Dxxx`. Em
   detalhe: um laço fechado entre `0x000000A0` (vetor de chamada A0), `0x000005C4..0x5DC` (tabela
   de despacho) e `0xBFC0D8E0..0xBFC0D8E8`.
4. **A função chamada é sempre a mesma:** `A0(40h)`, 1 428 571 vezes em 20 M passos —
   `SystemErrorUnresolvedException`. É o mesmo poço do item 4.4f, só que 59 M passos depois.
5. **Primeira ocorrência: passo 85 544 586.** `CAUSE=0x28` → excode 10 (RI), `EPC=0x8005B6D0`.
   A palavra em `0x8005B6D0` é `0x77800000`: opcode primário `0x1D`, que **não existe no MIPS I**.
   Levantar RI está certo — o problema é o kernel estar executando ali.
6. **Como se chegou lá:** passo 85 544 264, `PC=0x8003FA18`, `8FBF002C` = `lw $ra, 0x2C($sp)` com
   `$sp=0x801FFDA0`, lendo de `0x801FFDCC` — e o valor lido é **4**. O `jr $ra` seguinte salta para
   `0x00000004`, que contém o stub `addiu $k0,$k1,0xC80 / jr $k0`, e daí a cadeia desemboca no
   erro fatal.
7. **A medida decisiva — watchpoint em `0x801FFDCC`:** **uma única escrita** em 85 milhões de
   passos, no passo 133 574, de `PC=0xBFC018FC` (`sw $v1, 0x1C($sp)`), valor 4. Isto é: o prólogo
   da função **nunca salvou `$ra` nesse slot**. O `$sp` do prólogo é diferente do `$sp` do epílogo.

**Conclusão:** é a mesma família do 4.4f — `$ra` restaurado de `0x2C($sp)` valendo um inteiro
pequeno porque o quadro de pilha se desalinhou. O conserto do 4.4f (interrupção em delay slot
descartando o salto pendente) **não cobre este caso**: existe um segundo mecanismo que perde um
ajuste de `$sp`.

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que o comportamento real é | Como foi pego |
|---|---|---|---|---|
| 1 | hipótese | Que o boot travado depois do logo fosse falta de suporte a CD-ROM, e que o item a atacar fosse o 4.4 | O CD-ROM nunca é acionado porque a CPU já morreu 85 M passos antes. Os dois sintomas que eu ia investigar (sem comando de disco, sem TTY) são **consequência**, não causa | Contador em `send_command` deu zero, o que não combina com "o M4 está 12/13 fechado". Fui procurar onde a CPU estava e caí no `A0(40h)` |
| 2 | processo | Que o 4.4f, fechado e medido, tivesse eliminado essa classe de defeito | Eliminou **um** mecanismo de desalinhamento de `$sp`, não a classe. O sintoma reapareceu idêntico — mesmo offset `0x2C($sp)`, mesmo tipo de valor — 59 M passos adiante | Só apareceu porque a corrida foi muito mais longa (800 M contra os 50 M do 4.4f). Medir por mais tempo é barato e teria mostrado isso antes |

## Bateria de mutação

Bateria de mutação: não se aplica — esta iteração não altera código de produção; o que ela entrega
é o diagnóstico do item 4.4h, com watchpoint e passo exato. A bateria vem no PR que corrigir.

## Placar antes → depois

Workspace: **735** → **735** testes. Nenhum código mudou.

## Revisão cruzada (orquestrador)

Iteração inteira do orquestrador.

## Decisões e notas

1. **O item 4.4 não é o próximo passo, e isso muda o plano.** Ele depende do 4.4h; atacar CD-ROM
   agora seria trabalhar num subsistema que a CPU nem alcança.
2. **A corrida longa é barata e passou a ser padrão.** 3 bilhões de passos custam minutos e cobrem
   129 s de console. Toda medição de boot daqui em diante usa esse horizonte, não 50 M.
3. **O que este diagnóstico NÃO resolve:** qual é o segundo mecanismo. As hipóteses óbvias — outro
   caminho que vetora sem limpar estado de branch, ou um ajuste de `$sp` perdido em exceção
   síncrona — ainda não foram medidas. O handoff diz por onde começar: watchpoint no próprio `$sp`.
