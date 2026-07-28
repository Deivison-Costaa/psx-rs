<!-- Custo, tokens e duração NÃO entram aqui: são medidos por scripts/oc-iter.ps1 em
     docs/metricas.csv, casados por head_antes/head_depois. -->

# 0028 — spec-printf

- **Data:** 2026-07-28
- **Item do roadmap:** 1.11b (passo zero; o item em si fica para a próxima iteração)
- **Objetivo:** medir por que o `psxtest_cpu` fecha o 1.11 com TTY vazio, e escrever o handoff
  do 1.11b com o dado na mão em vez de com uma hipótese.

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| psx-spx | § A(3Fh) Printf — argumentos, escape codes, prefixos (L2703-2740) | docs/reference/13-kernel-bios.md |
| psx-spx | § A(3Eh)/B(3Fh) puts — função distinta, não resolve `%` (L2742-2746) | docs/reference/13-kernel-bios.md |
| psx-spx | § GPUSTAT bit 26 "Ready to receive Cmd Word" (L1028) | docs/reference/03-gpu.md |
| psx-spx | § porta 1F801814h-Read = GPUSTAT (L147) | docs/reference/03-gpu.md |

## Erros de primeira tentativa

Nenhum erro novo: esta iteração é medição, não implementação. O erro que ela investiga está
registrado como nº 12 em `0027-sideload-psexe.md`.

## O que foi medido

Instrumentei o laço do runner com o `psxtest_cpu.exe` real (BIOS SCPH1001, 50M passos):

**1. Chamadas de BIOS ao longo da execução inteira:** duas, e só duas.

```
(A0h, r9=44h)  FlushCache   x1
(A0h, r9=3Fh)  printf       x1
```

**2. A única chamada de printf:**

```
printf #1 fmt="args: %d\n" a1=00000000 a2=00000000 a3=00000000
```

Basta `%d` para o placar sair de `fail`. O handoff mesmo assim pede um conjunto coerente
(`%%`, `%c`, `%s`, `%d`/`%i`, `%u`, `%x`/`%X`) em vez do caso único do teste — implementar só
o que o EXE de hoje exercita é como escrever o teste depois de olhar a resposta.

**3. Onde a execução para, e por quê.** Depois do printf o PC estaciona em `80014DF0`. As
instruções em volta, decodificadas à mão:

```
80014DE0: 3C041F80   lui  r4, 1F80h
80014DE4: 3C030400   lui  r3, 0400h
80014DE8: 8C821814   lw   r2, 1814h(r4)      ; GPUSTAT (1F801814h-Read)
80014DEC: 00000000   nop                     ; load delay slot
80014DF0: 00431024   and  r2, r2, r3         ; máscara 0400_0000h = bit 26
80014DF4: 1040FFFC   beq  r2, r0, -4         ; volta para 80014DF0
```

É espera por **GPUSTAT.26 — "Ready to receive Cmd Word"** (`03-gpu.md` L1028). O bus devolve
`0` para todo o range `1F801024h..1F801FFFh` — o catch-all de I/O introduzido na iteração 0022
—, então o bit nunca acende e o laço não sai nunca. Isso é o item **2.1** do ROADMAP
(GPUSTAT + decodificação GP0/GP1), não um defeito do 1.11 nem do 1.11b.

Consequência registrada no handoff: **o sucesso do 1.11b é o TTY conter `args: 0\n`**, não o
`psxtest_cpu` completar a suíte. Sem isso escrito, a próxima iteração persegue um verde que
depende de um item de outro marco.

## Bateria de mutação

Não se aplica: sem mudança em `crates/`.

## Placar antes → depois

230 → 230 testes (inalterado).

## Revisão cruzada (orquestrador)

Esta iteração é produto de uma medição do orquestrador; a revisão do trabalho medido está em
`0027-sideload-psexe.md`.

## Decisões e notas

1. **Ordem 1.11b antes de 1.12.** O 1.12 liga o job de scoreboard na CI e começa a publicar a
   série histórica. Publicar agora congelaria no histórico um placar em que todos os 51 EXEs
   são `fail` por falta de uma função de BIOS de meia dúzia de linhas. Com o printf antes, a
   primeira linha da série já tem sinal.
2. **O passo zero de novo com o orquestrador, e de novo pagando.** Mesma decisão da 0026: o
   item depende de constantes (número da função, endereço da porta, número do bit) e de um
   comportamento que só a execução revela. Medir custou uma instrumentação descartável; supor
   teria custado uma rodada inteira do trabalhador perseguindo o laço da GPU achando que era
   bug do printf.
3. **O escopo do 1.11b exclui `%o`, `%n`, `%p`, larguras e precisão**, com a regra de que o
   especificador não suportado sai literal e a limitação vai escrita no doc. Emulador que
   engole `%` desconhecido em silêncio esconde o defeito na saída de outro teste, depois.
