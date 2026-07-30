# 0096 — relatorio-m4-m6

- **Data:** 2026-07-30
- **Item do roadmap:** 11.1
- **Objetivo:** consolidar no `docs/relatorio.md` o período M4→M6, com os números lidos do
  `docs/metricas.csv` e os padrões de falha medidos nas 58 execuções de 29–30/07.

## Revisão do PR anterior

Revisão do PR #110 (iter 0095, item 4.4e): sem achados novos de código. O conserto do handler está
certo e a nota 1 cita a spec — `docs/reference/02-cpu.md` L792 diz que `RFE` não salta para o EPC, e
o handler anterior terminava em `rfe; nop` sem `mfc0 k0,epc; jr k0`.

Um achado de **escrituração**, registrado aqui porque engana quem lê o ROADMAP: a linha do item 4.4c
é `- [x] 4.4c BIOS nunca escreve I_MASK (iter 0085)`. O título do item é o **problema**, marcado como
resolvido, enquanto o problema segue aberto na linha seguinte como `- [ ] 4.4d I_MASK=0x0000 por
todo o boot`. O 4.4c entregou suporte a store de 16 bits nesses registradores, que é pré-requisito;
não entregou a escrita. Quem lê o ROADMAP conclui o contrário.

## Spec consultada

Nenhuma. Item de documentação: a fonte autoritativa é o `docs/metricas.csv` e os 99 docs de iteração.

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que os dados dizem | Como foi pego |
|---|---|---|---|---|
| 1 | processo | Que a melhora de `VSync: timeout` de 104 para 8, medida logo após o item 4.4c, tivesse como causa a escrita de I_MASK passar a funcionar | `I_MASK` é **0x0000 do primeiro ao último passo do boot**, e os acessos ao vetor de exceção seguem em 44, inalterados. A melhora é real e a causa que eu dei é falsa — e continua desconhecida | Harness instrumentado, relido no commit `a3e9cbb` só porque o doc da 0095 afirmava o contrário do que eu havia registrado |
| 2 | processo | Que "suíte de hardware silenciosa" fosse sintoma de defeito, e que `input/pad` em zero vereditos significasse pad quebrado | O `input/pad` é **interativo por design** (`while(1)` imprimindo botões, sem PASS/FAIL) e 45 das 51 suítes renderizam na VRAM em vez de imprimir veredito | A iteração 0093 foi ler o **fonte do teste**; eu havia inferido da ausência de saída. Método dela melhor que o meu |
| 3 | cobertura de teste | Que afirmar a presença de uma string no arquivo provasse comportamento — `check-runs` presente provaria o caminho da API, `PENDENTE` presente provaria a espera | `check-runs-old` **contém** `check-runs`; e `PENDENTE` continua no `jq` mesmo com a comparação removida. Os dois mutantes sobreviveram | Bateria de mutação em 2/5 no item 10.37, com o teste já escrito e `cargo test --all` verde |

## Bateria de mutação

Bateria de mutação: não se aplica — a iteração não altera nenhum arquivo sob `crates/*/src/`, e mutar documento não é exercício falsificável: qualquer edição de texto "sobrevive".

O que substitui a bateria aqui é a **procedência dos números**: todas as linhas quantitativas da
seção 3.5 foram computadas do `docs/metricas.csv` por agregação, não digitadas de memória, e as
seções 5 e 6 citam commit, arquivo ou linha para cada afirmação verificável.

## Placar antes → depois

`docs/relatorio.md`: **158** → **242** linhas. Nenhum teste de workspace envolvido.

## Decisões e notas

1. **A seção 3.5 diz o custo sem suavizar.** O período custou US$ 12,74 contra US$ 1,87 do M1
   inteiro — 6,8× — e seis das 58 execuções morreram no teto de 45 minutos, duas delas deixando
   trabalho pronto numa branch órfã que uma rodada seguinte refez do zero. Um relatório onde o custo
   aparece só quando é favorável não serve como registro empírico.
2. **A seção 5 ganhou os erros do próprio orquestrador**, com a mesma taxonomia dos do trabalhador:
   três blocos de instrução com portão que testava rótulo em vez da coisa, e uma atribuição causal
   falsa desfeita por segunda medição. O projeto mede o trabalhador desde a primeira iteração; medir
   quem orquestra é a metade que faltava.
3. **A linha do M4 na tabela de marcos registra o R2 pelo que ele foi**: declarado inviolável desde
   o dia 1 e ligado a nada por 79 iterações, escondido porque todo teste chamava `enter_vblank()` à
   mão e o verde da suíte não distinguia produção de teste.
4. **Iteração feita pelo orquestrador, não pelo trabalhador.** O material — quais medições foram
   feitas, quais premissas caíram e em que ordem — está no contexto de quem mediu. Delegar isso
   produziria uma recontagem de segunda mão dos mesmos arquivos.
