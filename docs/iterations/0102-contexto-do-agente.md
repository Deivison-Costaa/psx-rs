# 0102 — contexto-do-agente

- **Data:** 2026-07-30
- **Item do roadmap:** 10.41
- **Objetivo:** o `STATUS.md` volta a ser handoff puro. Invariante e nota saem para
  `docs/invariantes.md` e passam a ser citadas **por número**; um portão novo reprova handoff que
  aponta para item inexistente, que guarda referência no lugar do handoff, ou cujo placar mente.

## Revisão do PR anterior

PR #117 (iter 0101), do próprio orquestrador: quatro checks verdes, `headRefOid` conferido antes do
merge, bateria 5/5. Sem achados novos.

Nesta parada também foram **fechados sem merge os PRs #114 e #115**, ambos da sessão zumbi da 0099:
o TTY do boot com as mudanças deles é byte a byte idêntico ao da `main` (597 bytes, 8
`VSync: timeout` dos dois lados), a mudança de hardware não foi conferida contra a spec (R1), e o
item citado no título — `4.4f` — não existia no `ROADMAP.md`. O item foi criado nesta iteração.

## Spec consultada

Nenhuma seção de spec de hardware. O item é organização do contexto que o trabalhador lê.

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que o comportamento real é | Como foi pego |
|---|---|---|---|---|
| 1 | teste | Que a âncora do m2 (`Critério de aceitação:`) removeria a citação do item do handoff | A citação `ROADMAP 4.4f` mora **na linha do título** da tarefa; o m2 mutava uma linha que não contém `ROADMAP` nenhum — mutante cosmético disfarçado de mutante | Bateria em 5/6, m2 sobreviveu. É a quinta ocorrência da minha família de erro: afirmar/mutar **presença de string** em vez da propriedade (0094, 0098, 0100, 0101 e agora esta) |
| 2 | ferramenta | Que o manifesto pudesse ancorar em qualquer linha do alvo | `mutation_format.rs:175` descarta **toda** linha iniciada por `#`, inclusive dentro de `@@DE`/`@@PARA`. Com alvo `.md`, cabeçalho Markdown é inancorável — e também impossível de *produzir* no `@@PARA` | `registro 'm6', edicao 1: @@DE vazio`, com o `@@DE` visivelmente preenchido. Nunca apareceu antes porque todo manifesto anterior mirava `.rs` ou `.ps1`. Registrado como item 10.42 |
| 3 | processo | Que a seção `## Prioridade — boot da BIOS travado` fosse contexto útil | Ela afirmava que o bloqueio é o dispatch de eventos do kernel — a mesma hipótese que a `## Próxima tarefa`, duas telas acima, lista como **medida e descartada**. O arquivo se contradizia havia 4 iterações | Achado ao medir o que o trabalhador lê no passo 0, a pedido do usuário. Nenhum portão via isso, e nenhum vê hoje: o portão novo mede forma, não veracidade |

## Bateria de mutação

Placar da bateria: 6/6 mutantes mortos, 2/2 controles verdes, 0 equivalente — docs/mutantes/0102-contexto-do-agente.mut

| Registro | Rótulo | Testes que pegaram, conforme o `.resultado` |
|---|---|---|
| m1 | item citado vira fantasma (`4.4f` → `4.4z`) | `proxima_tarefa_cita_item_que_existe_no_roadmap` |
| m2 | handoff deixa de citar item nenhum | `proxima_tarefa_cita_item_que_existe_no_roadmap` |
| m3 | some a linha que aponta as invariantes | `handoff_aponta_invariantes_por_numero` |
| m4 | cita invariante que não existe (99) | `handoff_aponta_invariantes_por_numero` |
| m5 | placar volta a mentir (707 → 703) | `placar_do_status_bate_com_a_contagem_de_testes` |
| m6 | aponta invariantes por prosa, sem número | `handoff_aponta_invariantes_por_numero` |
| c1 | prosa da seção Repositório reescrita | sobreviveu |
| c2 | número da última iteração trocado | sobreviveu |

As atribuições foram lidas do `.resultado` gerado pela máquina.

**Um dos quatro testes não é coberto pela bateria, e isso é limitação da ferramenta, não do teste.**
`status_nao_guarda_invariante_no_lugar_do_handoff` exige uma linha `## Notas` no `STATUS.md`, que o
manifesto não consegue nem casar nem produzir (erro 2 acima). Verifiquei **à mão**: acrescentei
`## Notas` ao fim do `STATUS.md`, o teste reprovou com `Secoes encontradas: ["## Notas"]`, e
restaurei com `git checkout -- STATUS.md`. Está dito aqui em vez de ficar implícito num placar de
6/6 que não cobre o que parece cobrir.

## Placar antes → depois

Workspace: **703** → **707** testes (+4 em `status_handoff`).

O efeito medido é de contexto pago por rodada:

| Arquivo | Antes | Depois |
|---|---|---|
| `STATUS.md` | 9 740 bytes | **3 175 bytes** (−67%) |
| `docs/invariantes.md` | — | 7 396 bytes, lido só nas entradas citadas |
| teto do `status_size.rs` | 16 000 bytes | 6 000 bytes |

## Revisão cruzada (orquestrador)

Iteração inteira do orquestrador, durante a parada deliberada do loop pedida pelo usuário.

## Decisões e notas

1. **Mover, não apagar — e o teste afirma as duas coisas.** `status_nao_guarda_invariante_no_lugar_do_handoff`
   exige a ausência das seções no `STATUS.md` **e** um mínimo de entradas em `docs/invariantes.md`.
   Sem a segunda asserção, apagar as notas e não criar o arquivo deixaria o teste verde — é a
   mesma armadilha do m3 da bateria 0100.
2. **A numeração das notas foi preservada.** A única entrada da antiga seção `## Invariantes`
   (imediato sinalizado) virou a **15**, no fim; 1 a 14 continuam com o número que sempre tiveram.
   Renumerar invalidaria toda citação anterior, e o passo 8 do protocolo já proíbe.
3. **`Invariantes relevantes:` admite `nenhum`, e isso é deliberado.** Sem essa saída, o handoff de
   um item de infra citaria número por obrigação. Com ela, a escolha é explícita: o portão exige a
   linha, não exige que ela aponte para algo.
4. **Desvio de R4 assumido: a parede da rodada subiu de 45 para 75 min no mesmo PR.** Não depende
   do resto; entrou aqui porque é a mesma pergunta ("o que o pipeline gasta à toa") e é uma linha.
   Base medida: 3 das 25 rodadas de 29–30/07 morreram exatas em 2 700 023 ms com o JSON ainda
   crescendo, enquanto a rodada boa mediana leva 28 min e a travada de verdade agora cai em ~90 s
   pelo `$TravamentoMin` da 0098.
5. **O que este portão NÃO pega: veracidade.** Um `STATUS.md` de 3 KB com um diagnóstico errado
   custa mais que um de 10 KB certo. O erro 3 acima — quatro iterações com duas seções se
   contradizendo — continua invisível para a máquina. Isso é revisão humana e não tem substituto
   automático à vista.
6. **Também não pega inchaço por prosa dentro do teto**, nem impede que a `Próxima tarefa` cite um
   item que existe mas não é o item certo.
