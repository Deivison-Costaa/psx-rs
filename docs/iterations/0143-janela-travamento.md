# 0143 — janela-travamento

- **Data:** 2026-08-01
- **Item do roadmap:** 10.62
- **Objetivo:** destravar o trabalhador — a janela de travamento de 5 min matava toda rodada que
  executasse o portão do passo 7 do protocolo.

## Revisão do PR anterior

PR #159 (iter 0142). Sem revisão cruzada independente (dívida 10.65, revisor sem acesso ao repo).

## Spec consultada

Nenhuma seção de spec de hardware. O item é ferramental de orquestração; o comportamento
autoritativo é a duração medida do próprio portão.

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a medição mostrou | Como foi pego |
|---|---|---|---|---|
| 1 | processo | Que a bateria registrada da 0098 continuasse válida, e que reparar as âncoras dela fosse trabalho mecânico | O `.resultado` commitado dizia `m4 → morreu`; rodando a bateria hoje, **`m4 sobrevive`**. O placar apodreceu e ficou dois dias mentindo | Rodei a bateria da 0098 **antes** de tocar em qualquer coisa, só para ter a linha de base |

## O placar da 0098 estava mentindo, e o mecanismo é geral

O mutante `m4` da 0098 troca `TravamentoMin` de 5 para **45**. Ele morria porque o teste
`janela_de_travamento_e_menor_que_a_parede_da_rodada` exige `travamento < parede`, e **a parede era
45** quando a 0098 rodou (30/07). Depois a parede subiu para **75** (comentário no próprio script:
"Era 45 min e matava rodada VIVA"), e `45 < 75` passou a ser verdade: o mutante virou **equivalente**
e o teste deixou de matá-lo.

```
placar registrado (30/07):  m4,mutante,morreu,morreu,0.1,janela_de_travamento_...
placar real      (01/08):   m4  mutante  sobreviveu
```

**Nada no projeto detecta isso.** O `.resultado` é um arquivo commitado; `mutation_battery.rs` e
`mutation_reconciliation.rs` conferem apenas a **consistência interna** (o placar do doc bate com o
`.resultado`, os nomes de teste existem). Nenhum meta-teste re-executa a bateria, então um placar
verde sobrevive à mudança de qualquer constante da qual ele dependia — e some a cobertura sem aviso.
É primo do item 10.44 ("manifesto com alvo em documento vivo envelhece na iteração seguinte"), mas
pior: ali o envelhecimento **quebra a âncora** e reprova; aqui a âncora continua casando e o placar
continua verde.

Reparo aplicado: a âncora de `m4`/`m5` acompanhou o valor novo do parâmetro (`5,` → `25,`), e o
`@@PARA` de `m4` passou de `45,` para **`75,`**, restaurando a *intenção* do mutante ("janela igual
à parede") em vez de preservar o número velho. Bateria 0098 regenerada: **6/6 mutantes mortos,
2/2 controles verdes**.

## A correção do item

`[int]$TravamentoMin` de **5 → 25** min, e o teste
`janela_de_travamento_e_menor_que_a_parede_da_rodada` ganhou uma terceira afirmação que trava o
piso em `MINUTOS_DO_PORTAO = 15`.

O valor de 5 min não era arbitrário — foi medido na 0098 e estava certo **para aquela suíte**.
Ficou obsoleto no instante em que a BIOS e o disco chegaram (0139): os testes de emulação deixaram
de pular por ausência de arquivo, passaram a executar de verdade, e `cargo test --all` foi de
segundos para **842 s**. Durante o portão o trabalhador não emite evento nenhum — está esperando o
`cargo` — então o JSON para de crescer e o detector lê silêncio legítimo como provedor mudo.

Medido na 0140: as **duas** rodadas do trabalhador morreram exatamente ali, uma no primeiro
`cargo test --all` e outra no terceiro (US$ 0,136 no total). O trabalhador foi morto por obedecer o
protocolo.

A terceira afirmação existe para que a próxima mudança na duração do portão **quebre um teste** em
vez de matar rodadas em silêncio. Se o portão encolher (por exemplo adotando `nextest`, 449 s), o
piso pode cair junto — mas por decisão registrada, não por acidente.

## Bateria de mutação

Placar da bateria: 5/5 mutantes mortos, 2/2 controles verdes, 0 equivalente — docs/mutantes/0143-janela-travamento.mut

| Registro | Rótulo | Teste que pegou |
|---|---|---|
| m1 | janela volta a 5, o valor que matou as rodadas da 0140 | `janela_de_travamento_e_menor_que_a_parede_da_rodada` |
| m2 | janela em 14, um minuto abaixo do portão | idem |
| m3 | parede encolhe para 20, abaixo da janela | idem |
| m4 | parede igual à janela (25) | idem |
| m5 | parede zerada | idem |
| c1 | porta do daemon 4096 → 4097 | — (sobreviveu, como esperado) |
| c2 | sondagem do laço 10 → 11 s | — (sobreviveu, como esperado) |

Os cinco mutantes atacam os **dois lados** da mesma relação: encolher a janela e encolher a parede
degeneram o detector pelos extremos opostos.

**Bateria MANUAL** (invariante 29): `mutantes.ps1:366` pula alvo fora de `crates/psx-core/`.
Aplicada por runner descartável que casa por linha inteira, roda `cargo test -p psx-core --test
ci_oc_iter` e restaura o arquivo num `finally`; `git diff scripts/` vazio ao fim. Vale para as duas
baterias desta iteração (0143 e a 0098 regenerada).

## Placar antes → depois

Workspace: **870** → **870** testes. A afirmação nova entrou num teste que já existia.

## Revisão cruzada (orquestrador)

<!-- Preenchido na revisão do PR. -->

## Decisões e notas

**1. Por que 25 e não 15.** O piso do teste é 15 (o portão medido, arredondado para cima). O valor
adotado é 25, deixando ~10 min de folga para a cauda: a suíte pode crescer, e o trabalhador chega a
rodar o portão mais de uma vez por rodada (três vezes, na 0140). Ainda fica bem abaixo da parede de
75, então o detector continua distinguindo rodada travada de rodada lenta — que é a razão de ele
existir (0098).

**2. Dívida nova, e ela é maior que este item.** Nenhum meta-teste re-executa bateria; um
`.resultado` verde pode estar mentindo desde a última vez que rodou. Encontrei este por acaso, ao
pedir a linha de base antes de mexer. Não há como saber quantos dos outros ~70 `.resultado` do
projeto estão na mesma situação sem re-rodar todos. Fica registrado como item novo.

**3. O que isto destrava.** Com a janela corrigida, o trabalhador volta a conseguir executar o passo
7 sem ser morto — e o projeto volta a ter iteração de emulação a ~US$ 0,03 em vez de sair toda do
orquestrador. As quatro iterações anteriores (0139–0142) saíram do orquestrador por causa deste
bloqueio.
