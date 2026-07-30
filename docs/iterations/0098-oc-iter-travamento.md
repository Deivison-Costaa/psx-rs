# 0098 — oc-iter-travamento

- **Data:** 2026-07-30
- **Item do roadmap:** 10.38
- **Objetivo:** o `oc-iter.ps1` distinguir rodada lenta de rodada travada, matando a travada em
  minutos em vez de segurar o loop pela parede inteira de 45 min, e rotulando as duas de formas
  diferentes na métrica.

## Revisão do PR anterior

Revisão do PR #112 (iter 0096) feita antes desta iteração, com medição própria e não por leitura do
doc. Dois resultados, ambos já registrados no estado do orquestrador:

1. **Confirmada** a afirmação central do item: a BIOS escreve `I_MASK` (0x0001 por volta de 19M
   passos, depois 0x0009), o `I_STAT` passa a ser reconhecido e a CPU vetora — 113 acertos no vetor
   de exceção em 30M passos.
2. **Refutada** a nota 2 do doc: medi o commit `96a8a82`, anterior ao item, com os mesmos 30M
   passos, e os valores são idênticos. O que o doc atribuiu ao item já valia antes dele.

Nenhum achado de código. O campo `mask_write_count` é instrumentação sem efeito funcional.

## Spec consultada

Nenhuma seção de spec de hardware. O item é ferramenta de orquestração; o comportamento
autoritativo é o do provedor, observado ao vivo nos JSONs das rodadas 2 e 4 da noite10.

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que o comportamento real é | Como foi pego |
|---|---|---|---|---|
| 1 | teste | Que exigir a presença de `$ultimoAvanco` depois da leitura do tamanho provasse que a marca de avanço é atualizada | O token continua aparecendo na comparação da janela mesmo quando a reatribuição some. Com a reatribuição removida, a marca congela na largada e o detector mata TODA rodada ao fim da janela — inclusive as que estão trabalhando | Bateria em 5/6: o m6 sobreviveu. Corrigido exigindo `$ultimoAvanco = Get-Date` na janela entre a leitura do tamanho e a avaliação do tempo |
| 2 | processo | Que eu pudesse provar o detector com um `Start-Process -ArgumentList` montando o corpo do filho por concatenação de string | É exatamente a armadilha de quoting que o comentário do próprio `oc-iter.ps1` documenta: a aspa dentro do argumento fecha a região citada e o resto vira token solto (`so: The term 'so' is not recognized`) | O caso de controle "processo que cresce" deu `falha:travamento` com o filho nunca tendo escrito nada — controle inválido que eu quase li como verde. Corrigido passando o corpo por `-File` |
| 3 | processo | Que o `.resultado` gerado pela máquina pudesse ficar solto na árvore entre duas execuções da bateria | `mutantes.ps1` recusa árvore suja na partida, e o `.resultado` não versionado é sujeira | `Die: arvore suja antes de comecar` na segunda execução. É o item 10.17, ainda aberto, se manifestando pela quarta vez |

## Bateria de mutação

Placar da bateria: 6/6 mutantes mortos, 2/2 controles verdes, 0 equivalente — docs/mutantes/0098-oc-iter-travamento.mut

| Registro | Rótulo | Testes que pegaram, conforme o `.resultado` |
|---|---|---|
| m1 | volta a esperar a parede inteira num `WaitForExit` cego | `espera_da_rodada_nao_e_um_waitforexit_cego_de_parede_inteira` |
| m2 | nunca lê o tamanho do JSON | `espera_observa_o_crescimento_do_json_da_rodada` |
| m3 | travamento cai no mesmo rótulo do timeout | `travamento_tem_rotulo_proprio_distinto_de_timeout` |
| m4 | janela igual à parede, o detector nunca dispara | `janela_de_travamento_e_menor_que_a_parede_da_rodada` |
| m5 | janela zero mata toda rodada na primeira volta | `janela_de_travamento_e_menor_que_a_parede_da_rodada` |
| m6 | marca de avanço nunca é atualizada | `espera_observa_o_crescimento_do_json_da_rodada` |
| c1 | intervalo de sondagem de 10 para 12 s | sobreviveu |
| c2 | sentinela inicial de tamanho de -1 para -2 | sobreviveu |

As atribuições foram lidas do `.resultado` gerado pela máquina, não preenchidas por inspeção.

## Placar antes → depois

Workspace: **691** → **695** testes (+4: `ci_oc_iter`). O 688 do doc da 0094 não vale mais como
base: as iterações 0095 e 0096 entraram no meio. Contado, não copiado.

Não há suíte de hardware envolvida. O efeito medido é operacional, e está na seção seguinte.

## Revisão cruzada (orquestrador)

Iteração **inteira do orquestrador**, como a 0094 — e pela mesma razão: o artefato é a ferramenta
que conduz o trabalhador, e ela estava consumindo as rodadas dele.

## Decisões e notas

1. **A medição que motivou o item.** Três rodadas seguidas da noite10 se perderam. Medindo o tempo
   de vida ativo de cada uma (primeiro ao último evento do JSON) contra a parede de 45 min:
   rodada 2, 5 steps, **89 s** de vida; rodada 3, 114 steps, 33 min de vida; rodada 4, 8 steps,
   **100 s** de vida. As três gastaram 45 min de parede. Só a rodada 3 estava trabalhando.
2. **A assinatura do travamento é o silêncio, não o erro.** Nos dois casos o último evento do JSON
   é um `step_start` e nada mais é emitido — nem erro, nem texto, nem tool call. O provedor aceita
   o pedido do próximo passo e nunca responde. Por isso o sinal usado é o tamanho do JSON parado, e
   não um código de saída: não há código de saída nenhum.
3. **Rótulo próprio (`falha:travamento`) em vez de reaproveitar `falha:timeout`.** Os dois modos
   pedem remédios opostos: rodada lenta argumenta por parede maior, rodada travada argumenta por
   morte rápida. Com um rótulo só, `docs/metricas.csv` mistura os dois e a série fica inútil para
   decidir qualquer um deles. Foi essa mistura que escondeu o problema até hoje.
4. **Janela de 5 min, e o teste exige que ela seja menor que a parede.** Cinco minutos é folgado
   contra o intervalo real entre eventos de uma rodada viva (segundos), e barato contra os ~43 min
   que cada travamento custava. Uma janela maior ou igual à parede nunca dispararia, e o conserto
   inteiro viraria decoração — daí o teste que compara os dois parâmetros.
5. **Verificação de comportamento, não só de texto.** Os quatro testes leem o script; isso prova
   forma, não função. Rodei o bloco de espera verbatim contra dois processos falsos: um mudo, que
   morreu com `falha:travamento` aos **70 s** em vez de esperar os 10 min configurados; e um que
   escrevia a cada 5 s, que **não** foi morto e terminou `ok` aos 100 s com a mesma janela de 1 min.
   O segundo é o caso que discrimina — sem ele, um detector que mata tudo passaria como sucesso.
6. **O loop precisa ser relançado para receber o conserto.** O `pwsh` vivo tem o script antigo em
   memória; consertar sem relançar não muda nada. Mesma nota da 0094, e a razão de esta iteração
   ter sido feita fora da árvore compartilhada, num worktree isolado, com o loop rodando.
