# 0103 — ra-corrompido

- **Data:** 2026-07-30
- **Item do roadmap:** 4.4f
- **Objetivo:** achar e consertar a causa de o boot da BIOS morrer no passo 26 595 832 num
  `jr $ra` com `$ra = 3`, e fazê-lo passar desse ponto sem entrar em
  `A0(40h)` = `SystemErrorUnresolvedException`.

## Revisão do PR anterior

PR #118 (iter 0102), do próprio orquestrador: quatro checks verdes, `headRefOid` conferido, bateria
6/6 com uma âncora corrigida. Sem achados novos.

## Spec consultada

`docs/reference/02-cpu.md`:

- **L683** — `CAUSE` bit31 `BD`: *"Is set when EPC points to the branch instuction instead of the
  instruction in the branch delay slot, where the exception occurred."*
- **L682** — `CAUSE` bit30 `BT`: *"When BD is set, BT determines whether the branch is taken."*
- **L784-786** — o handler padrão da BIOS não funciona em branch delays, *"where BD gets set to
  indicate that EPC was modified"*. Confirma que recuar o EPC é o comportamento do hardware, e que
  o software conta com o BD para saber disso.

## A caça, em três medidas

Quatro rodadas de trabalhador falharam neste item antes desta iteração, todas atacando o dispatch
de eventos. A cadeia que resolveu foi:

1. **Watchpoint em `$31`.** A última escrita antes da morte é no passo 26 595 826, em
   `PC=8004A548`, e vem do `lw $ra, 0x2C($sp)` emitido em `8004A4F4` (delay slot de um `beq`
   incondicional). Ou seja: **o 3 vem da pilha**, não de um `jal`.
2. **Watchpoint no endereço lido** (`$sp + 0x2C = 0x801FFB84`): quem escreve ali é
   `sw $s0, 0x2C($sp)` em `8004A644` — um contador 0..15 da própria BIOS. Duas funções diferentes
   usando o mesmo slot só é possível com o `$sp` errado.
3. **Trilha completa do ciclo.** O `$sp` desce 0x60 por volta do laço. O passo exato:

```
26595737 PC=8004A708 instr=03E00008   jr $ra
26595738 PC=8004A70C instr=27BD0060   addiu $sp,$sp,0x60   <- delay slot
26595739 PC=80000080 instr=3C1A0000   interrupcao vetorada AQUI
26595740 PC=8004A4C8 ...  $sp=801FFB58 <- o addiu sumiu; o handler durou 1 instrucao
```

**Causa raiz:** o caminho de interrupção em `cpu.rs` era um `return` antecipado que ignorava
`delay_slot_pending` e `branch_target`. O salto pendente sobrevivia à vetoração e, na primeira
instrução do handler, sequestrava o PC para o destino do `jr`. Um único defeito explica os três
sintomas: o `addiu $sp` perdido (pilha desce 0x60 por volta), o handler abandonado (IRQ nunca
reconhecida) e, 16 voltas depois, o epílogo lendo `$ra` do slot do contador.

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que o comportamento real é | Como foi pego |
|---|---|---|---|---|
| 1 | hardware | Que o caminho de interrupção e o de exceção compartilhassem o tratamento de delay slot | São dois trechos independentes. O de exceção (`pending_exception`) sempre setou BD/BT, recuou o EPC e limpou `branch_target`; o de interrupção não fazia **nenhuma** das três coisas | Só apareceu na trilha instrução a instrução. Nenhum dos 707 testes cobria interrupção com salto pendente — a lacuna era exatamente onde o boot morria |
| 2 | teste | Que desarmar a IRQ antes do passo provasse "o handler não é sequestrado" | Desarmar a máscara **antes** do `step` faz a interrupção não ser tomada: o teste media outra coisa. Dois dos seis testes nasceram vermelhos pelo motivo errado | Falha com `left: 512` (o alvo do salto) contra `right: 0x80000080`. Corrigi a ordem: tomar a interrupção primeiro, desarmar depois |
| 3 | processo | Que `git checkout HEAD -- crates/psx-core/src/cpu.rs` restaurasse o arquivo ao estado em que eu estava | `HEAD` era o commit do **teste**; a correção ainda não estava commitada e foi apagada. Perdi o `fix` inteiro e só percebi porque a comparação com a `main` devolveu 0 byte | Refiz a correção e passei a **commitar antes** de qualquer experimento que troque o arquivo. É a mesma família do `cp backup gpu.rs` da 0038: restaurar por comando certo com referência errada destrói tanto quanto restaurar por comando errado |

## Bateria de mutação

Placar da bateria: 6/6 mutantes mortos, 2/2 controles verdes, 0 equivalente — docs/mutantes/0103-ra-corrompido.mut

| Registro | Rótulo | Testes que pegaram, conforme o `.resultado` |
|---|---|---|
| m1 | salto pendente sobrevive à vetoração | `handler_nao_e_sequestrado_pelo_salto_pendente`, `retorno_pelo_epc_refaz_o_salto_e_o_delay_slot` |
| m2 | EPC não recua para o branch | `interrupcao_no_delay_slot_aponta_epc_para_o_branch`, `retorno_pelo_epc_refaz_o_salto_e_o_delay_slot` |
| m3 | `CAUSE.BD` não é setado | `interrupcao_no_delay_slot_aponta_epc_para_o_branch` |
| m4 | `CAUSE.BT` não é setado | `bt_indica_branch_tomado_no_delay_slot` |
| m5 | recuo do EPC aplicado sempre | `interrupcao_fora_de_delay_slot_nao_marca_bd` |
| m6 | BD setado sempre | `interrupcao_fora_de_delay_slot_nao_marca_bd` |
| c1 | ordem das duas limpezas trocada | sobreviveu |
| c2 | vetor escrito em decimal | sobreviveu |

As atribuições foram lidas do `.resultado` gerado pela máquina. 6/6 na primeira execução.

## Placar antes → depois

Workspace: **707** → **713** testes (+6 em `cpu_interrupt_delay_slot`).

Efeito no boot da BIOS, medido com a BIOS real em 50 M passos:

| Medida | Antes (`main`) | Depois |
|---|---|---|
| Chamadas a `A0(40h)` | 1 071 429 | **0** |
| Buscas com `PC = 3` | contínuas a partir de 26 595 832 | **0** |
| Bytes de TTY | 557 | **2 029** |
| Última linha do TTY | `VSync: timeout (5:4)` | `VSync: timeout (55:54)`, após `ResetCallback: _96_remove ..` |

## Revisão cruzada (orquestrador)

Iteração inteira do orquestrador, durante a parada deliberada do loop. É o quinto ataque a este
item — os quatro anteriores foram do trabalhador e nenhum produziu PR mergeável; dois deles
(#114, #115) chegaram a alegar a correção com o TTY **byte a byte idêntico** ao da `main`.

## Decisões e notas

1. **O critério de aceitação foi medido, não inferido.** "O boot sobrevive" seria fácil de alegar
   olhando o TTY maior. Contei `A0(40h)` e buscas em `PC=3` com instrumentação descartável, e as
   duas foram a zero. A instrumentação foi apagada antes do commit.
2. **A duplicação de todo o texto do TTY é pré-existente.** Medi a `main` no mesmo binário: 557
   bytes com **2** linhas `System ROM Version`. Não foi introduzida aqui; virou o item 10.43.
3. **O item não fecha o boot, e o handoff diz isso.** As mensagens de `VSync: timeout` continuam,
   agora com o contador avançando até `(55:54)` — o kernel conta os vblanks mas quem espera não é
   acordado. Item 4.4g criado, com a hipótese a testar primeiro nomeada.
4. **Invariante 16 registrada:** qualquer caminho novo que vetore tem de descartar o salto pendente
   e recuar o EPC. O defeito existiu porque havia dois caminhos e só um obedecia à regra.
5. **O que este conserto NÃO cobre:** exceção *síncrona* dentro do handler, aninhamento de
   interrupções, e `CAUSE.BT` para branches condicionais não tomados — o teste só exercita o salto
   incondicional, que é o caso que o boot executa.
