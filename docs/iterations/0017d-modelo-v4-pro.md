# 0017d — Trabalhador migra para deepseek-v4-pro

- **Data:** 2026-07-27
- **Item do roadmap:** 0.8 (orquestração; fora da escada do M1)
- **Objetivo:** Trocar o modelo padrão do trabalhador da geração anterior (`deepseek-chat`)
  para `deepseek-v4-pro`, e registrar a rodada abortada no meio da troca.
- **Autor:** orquestrador (Claude). Sem código de emulador.

## Motivação

O usuário perguntou se o trabalhador estava no "flash" quando deveria estar no "pro". O
orquestrador respondeu que esses nomes eram de outro fornecedor — **resposta errada**.
`opencode models` lista quatro modelos DeepSeek:

```
deepseek/deepseek-chat        <- default desde a iter 0009
deepseek/deepseek-reasoner
deepseek/deepseek-v4-flash
deepseek/deepseek-v4-pro
```

Ou seja: da iteração 0009 à 0017 o trabalhador rodou na geração anterior sem que ninguém
tivesse decidido isso — foi o default escrito no `oc-iter.ps1` no dia 0008 e nunca revisto.
O erro não é ter escolhido mal; é a escolha nunca ter sido uma escolha.

## Spec consultada

Não se aplica — item de orquestração, sem hardware envolvido.

## O que entrou

- `scripts/oc-iter.ps1` e `scripts/oc-loop.ps1`: `$Model` padrão passa a
  `deepseek/deepseek-v4-pro`.
- Comentário no `oc-iter.ps1` fixando a data da troca e o novo par de comparação.

## A rodada abortada (0018, primeira tentativa)

A segunda tentativa da 1.7 já estava em voo com `deepseek-chat` quando a troca foi decidida.
Estado no momento do kill: 18min36 de execução, commit `test(cpu)` feito (`f94858b`),
`cpu.rs` e o arquivo de teste modificados sem commit — ou seja, no meio do passo 5.

Foi morta. Três motivos, nessa ordem:

1. Rodava numa geração que o dono do projeto considera superada.
2. **18min36 contra ~5min das iterações anteriores** — 3,5×. O tempo não era progresso
   visível: o passo 5 é o mais curto das iterações anteriores.
3. O recurso escasso da noite não é a API (US$ 0,02/iteração), é o ciclo de revisão do
   orquestrador. Uma segunda reprovação na 1.7 custa mais caro do que recomeçar.

A branch foi preservada como `abandonada/0018-lwl-chat-v3` (local, não publicada) e a linha
`abortado:troca-de-modelo` entrou nas métricas com o custo em branco: o JSON do opencode não
chegou a ser fechado, então não há custo confiável para declarar. Duração registrada.

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que era | Como foi pego |
|---|---|---|---|---|
| 1 | nenhum (do projeto) — **erro de fato do orquestrador** | "flash/pro são nomes do Gemini, não existem no DeepSeek" | Existem: `deepseek-v4-flash` e `deepseek-v4-pro` | O usuário insistiu; `opencode models` decidiu em 1 comando |

Registrado porque é o tipo de erro que o projeto existe para medir: o orquestrador respondeu
de memória sobre o **ambiente**, que é verificável em um comando, exatamente o que a R1 proíbe
para hardware. A regra vale para o ferramental também.

## Bateria de mutação

**Não se aplica** — e a ausência é deliberada, não esquecimento. Um meta-teste que fixasse o
nome do modelo transformaria "trocar de modelo" em violação de processo, quando trocar de
modelo é justamente um dos experimentos do projeto. O controle correto já existe e é o
`docs/metricas.csv`: toda linha carrega a coluna `modelo`, então a deriva aparece no dado em
vez de precisar de guarda.

## Placar antes → depois

151 testes → **151** (nenhum teste novo; mudança de script).

## Decisões e notas

- **O plano de comparação muda de eixo.** Era `deepseek-chat` × `deepseek-reasoner` no item
  1.8. Passa a ser `deepseek-v4-pro` (padrão) × `deepseek-v4-flash` (barato), medido no
  `metricas.csv` por custo/iteração e por reprovações na revisão adversarial.
- **A 1.7 perde a comparação limpa que eu tinha planejado.** A ideia era repetir o item com o
  mesmo modelo para isolar o efeito do handoff corrigido. Com a troca, se a segunda tentativa
  passar, não dá para saber se foi o handoff ou o modelo. Perda real, registrada aqui: a
  decisão do dono do projeto sobre a ferramenta vale mais que a limpeza do meu experimento, e
  o handoff corrigido continua valendo para todos os itens seguintes de qualquer forma.
- O handoff da 1.7 no `STATUS.md` **não muda** — as duas armadilhas e o teste de aceitação
  obrigatório valem igual para qualquer modelo.
