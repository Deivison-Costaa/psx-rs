# 0104 — vsync-evento

- **Data:** 2026-07-30
- **Item do roadmap:** 4.4g
- **Objetivo:** a BIOS parava de morrer depois do 4.4f mas imprimia `VSync: timeout` para sempre.
  Achar por que a espera de VSync nunca é satisfeita e fazer o boot passar dela.

## Revisão do PR anterior

PR #119 (iter 0103), do próprio orquestrador: quatro checks verdes, `headRefOid` conferido, bateria
6/6. Duas âncoras alheias envelheceram com aquela mudança e foram tratadas lá (0055 reancorada e
rerodada, 0102 arquivada).

## Spec consultada

`docs/reference/02-cpu.md`, seção **Load Timing**:

- **L262-269** — *"the PSX has no data cache, so every load reads through to memory and halts the
  CPU until the data arrives. The number of CPU cycles per lw (including the 1-cycle issue …)
  depends on what is accessed (measured on hardware)"*, com a tabela: Scratchpad 1, On-die I/O 5,
  Main RAM 7, BIOS ROM 27..33.
- **L305-306** — store vai para a write-queue e executa em um único ciclo.
- **L281-296** — *Load Shadow*: parte do acesso lento se sobrepõe às instruções seguintes. **Não
  implementado aqui** (R4); virou o item 10.45.

## A caça

O sintoma dizia "evento não entregue", e a suspeita óbvia era o dispatch. Foi o contrário:

1. **Trilha antes do `printf`.** Capturei os 4 000 passos anteriores à terceira mensagem de
   timeout. O que gira ali é um laço de 12 instruções em `80059DC8..80059E10`:

```
loop:  lw   $v0, 0x1C($sp)      ; orçamento
       addiu $t9, $t8, -1
       bne  $v0, $zero, +8
       sw   $t9, 0x1C($sp)      ; orçamento--
check: lw   $t0, [0x80079D9C]   ; contador de vblank
       slt  $at, $t0, $a1       ; contador < alvo ?
       bne  $at, $zero, loop
```

2. **O contador funciona.** Watchpoint em `0x80079D9C`: ele incrementa **uma vez por vblank**, a
   cada 566 188 passos, exatamente como deve. 89 vblanks em 50 M passos, 40 incrementos a partir do
   momento em que a BIOS liga a contagem. O evento é entregue.
3. **O orçamento é o problema.** Medido lendo `0x1C($sp)` na entrada do laço: **32 768** iterações,
   constante fixa da BIOS. Com o nosso modelo de 1 ciclo por instrução, 32 768 × 12 = 393 216
   ciclos — **69% de um frame** (566 187). A BIOS desiste antes do vblank, sempre.

**Causa raiz:** não cobrávamos o custo de acesso à memória. A conta com os números da spec fecha:
9 instruções × 1 + 3 loads da RAM × 7 = **30 ciclos por iteração**, e 32 768 × 30 = 983 040 ciclos
= **1,74 frame**. O número saiu da spec, não de ajuste ao sintoma — é o que torna a explicação
falsificável.

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que o comportamento real é | Como foi pego |
|---|---|---|---|---|
| 1 | hardware | Que `VSync: timeout` significasse evento não entregue — o mesmo caminho que as 4 rodadas anteriores tentaram | O contador de vblank do kernel incrementa certo, uma vez por frame. O que falha é o **orçamento** do laço de espera, que é contado em iterações e depende do custo em ciclos de cada instrução | Watchpoint no endereço que o laço lê (`0x80079D9C`). Foi a medida que virou a investigação do avesso: parei de procurar defeito de lógica e fui medir tempo |
| 2 | teste | Que um controle da bateria pudesse escrever a mesma faixa com outra aritmética (`0x1FC0_0000..=0x1FC0_0000 + 0x7_FFFF`) | Rust não aceita expressão como limite de pattern. O "controle" não compilava, e o script corretamente recusou creditá-lo | `c1 (controle) ... ERRO DE MANIFESTO`. O `mutantes.ps1` distingue erro de compilação de teste falhando — se não distinguisse, um mutante que nem compila entraria como "pego" |
| 3 | processo | Que a bateria abortada deixasse a árvore limpa | Ficou a sentinela `logs/mutantes-em-andamento.txt` (item 10.17, **sexta** ocorrência). O `bus.rs` estava íntegro — conferi antes de apagar a sentinela, em vez de restaurar por reflexo | O próprio script recusou a segunda execução e imprimiu o comando de restauração |

## Bateria de mutação

Placar da bateria: 6/6 mutantes mortos, 2/2 controles verdes, 0 equivalente — docs/mutantes/0104-vsync-evento.mut

| Registro | Rótulo | Testes que pegaram, conforme o `.resultado` |
|---|---|---|
| m1 | RAM principal volta a custar 1 ciclo | `lw_da_ram_principal_custa_7_ciclos`, `laco_de_espera_da_bios_cobre_um_frame` |
| m2 | scratchpad custa como a RAM | `lw_do_scratchpad_custa_1_ciclo` |
| m3 | I/O on-die custa como a RAM | `lw_de_io_on_die_custa_5_ciclos` |
| m4 | ROM da BIOS custa como a RAM | `lw_da_bios_custa_27_ciclos` |
| m5 | faixa do I/O on-die encurtada | `lw_de_io_on_die_custa_5_ciclos` |
| m6 | classificação sobre o endereço virtual, sem traduzir KSEG | `lw_da_ram_principal_custa_7_ciclos`, `laco_de_espera_da_bios_cobre_um_frame` |
| c1 | parâmetro renomeado | sobreviveu |
| c2 | braços do match reordenados (faixas disjuntas) | sobreviveu |

## Placar antes → depois

Workspace: **713** → **720** testes (+7 em `cpu_load_timing`).

Boot da BIOS real:

| Medida | Antes (iter 0103) | Depois |
|---|---|---|
| Mensagens `VSync: timeout` | 55 em 50 M passos | **0** |
| Bytes de TTY | 2 029 (quase tudo timeout) | 389, e o boot segue calado como deve |
| Pixels não-zero na VRAM | — | **179 774**, 540 cores distintas |
| Display | — | x=0, y=241, range 608..3168 |

A VRAM despejada mostra o **logo da PlayStation sendo desenhado** e o sprite
`SONY COMPUTER ENTERTAINMENT` carregado.

## Revisão cruzada (orquestrador)

Iteração inteira do orquestrador.

## Decisões e notas

1. **A ponta baixa do intervalo da ROM (27) é escolha declarada.** A spec dá 27..33 e diz que varia
   entre consoles porque o atraso de barramento é programável. Um número escolhido e dito é melhor
   que um número sorteado; o teste cita a linha da spec e o motivo.
2. **O *load shadow* NÃO entrou (R4).** A spec descreve, em seção própria, que metade do acesso se
   esconde atrás de instruções independentes seguintes. Ignorá-lo torna os loads mais caros que o
   hardware; para este item isso é conservador (o laço passa com folga de 1,74 frame nos dois
   modelos). Item 10.45.
3. **LWC2/SWC2 e a busca de instrução continuam a 1 ciclo.** A tabela da spec fala de `lw`; o custo
   de fetch depende da I-cache, que não modelamos. Ambos ficam fora deste item, e isto está dito
   aqui para que a próxima medição de tempo saiba o que já é e o que não é modelado.
4. **O que este item NÃO conserta:** a tela sai errada. O losango do logo aparece só pela metade,
   "PlayStation" vira barra vermelha e o sprite da Sony nunca é composto — os três são
   `GP0(80h)`, o blit VRAM→VRAM, que hoje é consumido e ignorado. É o próximo item (2.2b), e o
   handoff já leva a medida e o critério visual.
