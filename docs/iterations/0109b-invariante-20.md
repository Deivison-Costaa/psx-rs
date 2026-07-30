# 0109b — invariante-20

- **Data:** 2026-07-30
- **Item do roadmap:** 2.2d
- **Objetivo:** a invariante 20, escrita na iteração 0109, apontava para uma hipótese que a própria
  0109 já tinha derrubado. Corrigir, e salvar a referência da tela num lugar durável.

## Revisão do PR anterior

PR #125 (iter 0109), do próprio orquestrador: quatro checks verdes, `headRefOid` conferido.
**E foi ele que introduziu o defeito corrigido aqui.**

## Spec consultada

Nenhuma. É correção de documento.

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que o comportamento real é | Como foi pego |
|---|---|---|---|---|
| 1 | processo | Que escrever a invariante 20 no começo da investigação e medir depois desse toda certo | Escrevi "a suspeita primária é a projeção (RTPS/RTPT, divisão UNR, OFX/OFY)" e **na mesma iteração** medi **zero** chamadas de `rtps`. O PR foi mergeado com a invariante mandando a próxima pessoa para o beco que eu acabara de fechar | O usuário perguntou se a referência estava salva. Ao conferir o que tinha ficado gravado, li a invariante e vi que ela contradizia o doc da própria iteração |
| 2 | processo | Que registrar a referência em prosa bastasse | A **imagem** não está em lugar nenhum: foi colada no chat e some no próximo `clear`. O que sobreviveu foi a minha descrição dela | Mesma pergunta do usuário. Criado `docs/referencias/tela-de-boot.md`, que descreve a tela e **diz explicitamente que o arquivo não está versionado** e que é preciso pedir de novo |

Os dois são a mesma falha de fundo, e é a que o `status_handoff.rs` foi criado para atacar na
iteração 0102: **documento escrito antes da medição e não reconferido depois.** Naquela vez foram
quatro iterações com o `STATUS.md` se contradizendo; aqui foi um PR. O portão pega forma, não
veracidade — isto continua sendo revisão humana.

## Bateria de mutação

Bateria de mutação: não se aplica — esta iteração não altera código de produção, apenas corrige um
documento que contradizia a medição da iteração anterior e cria um arquivo de referência.

## Placar antes → depois

Workspace: **735** → **735** testes.

## Revisão cruzada (orquestrador)

Iteração inteira do orquestrador.

## Decisões e notas

1. **A invariante agora lista as três hipóteses eliminadas**, em vez de propor uma. Invariante que
   propõe hipótese envelhece na primeira medição; invariante que registra o que já foi eliminado
   só melhora com o tempo.
2. **`docs/referencias/` é diretório novo** e existe para uma coisa só: guardar o que veio de fora
   do projeto e não pode ser remedido por nós. A imagem não entra no repositório (não é nossa), mas
   a descrição e o modo de reobtê-la entram.
