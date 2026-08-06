<!-- Custo, tokens e duração NÃO entram aqui: são medidos por scripts/oc-iter.ps1 em
     docs/metricas.csv, casados por head_antes/head_depois. -->

# 0209 — ciclos-excecao

- **Data:** 2026-08-06
- **Item do roadmap:** 0209.1 — Degrau 1 da escada de timing de CPU/barramento
- **Objetivo:** `enter_exception` zera `load_extra_cycles` junto do resto do estado de
  pipeline, pra um load que falta não deixar seu custo vazar pra próxima instrução.

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| psx-spx | § Load Timing (L260-279) | docs/reference/02-cpu.md |

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | timing | Que `load_extra_cycles` só podia vazar por um caminho (o load que falta). | Não é assunto de spec — é leitura do próprio código. | `enter_exception` já zerava `branch_target`/`delay_slot_pending`/`branch_taken`/`load_delay` mas não esse acumulador, apesar de ser exatamente o mesmo tipo de estado de pipeline "no meio de uma instrução que não vai completar". Achado numa investigação dedicada (workflow de 5 agentes, jogos comerciais travados) que apontou o mesmo padrão de bug em `timers.rs` (iter 0208) e motivou reler `cpu.rs` inteiro antes de mexer em qualquer coisa de timing. |

## Bateria de mutação

Placar da bateria: 5/5 mutantes mortos, 2/2 controles verdes, 0 equivalente —
docs/mutantes/0209-ciclos-excecao.mut

m1 vira no-op, m2 zera pro valor errado, m3 zera o campo errado (`written_gpr`), m4 remove o
`mem::take` no consumo (linha 218), m5 condiciona o reset a `exc_code == 0x09` (só BREAK) —
todos mortos pelos dois testes de exceção de load desalinhado (RAM e BIOS, que preferem
regiões com custo bem diferente de 1 pra deixar qualquer vazamento óbvio).

## Placar antes → depois

Workspace: **1266 → 1269** testes (3 novos em `cpu_ciclos_excecao.rs`).

## Revisão cruzada (orquestrador)

Rodada de autoria do orquestrador (exceção de executor vigente pra este tipo de trabalho,
`docs/orquestracao.md`, mesmo padrão da escada de velocidade da 0193) — a revisão adversarial
é do mesmo agente que escreveu, registrado em vez de escondido. Verificação independente do
julgamento de quem escreveu: os dois testes de vazamento falhavam contra o código antes do
fix (7 e 27 ciclos medidos, batendo exatamente com o custo da região que deveria ter sido
descartado) e passam depois; `cpu_load_timing.rs::laco_de_espera_da_bios_cobre_um_frame`
(invariante 17) foi reexecutado e continua com o mesmo valor — o laço da BIOS não passa por
exceção nenhuma, então não deveria mudar, e não mudou.

## Decisões e notas

Primeiro degrau da escada de timing de CPU/barramento (plano completo em
`docs/achados.md`/histórico de sessão — o achado motivador é 0193.4, "CPU cobra 1 ciclo/
instrução sem custo de RAM/ROM"). Investigação prévia (3 agentes de exploração + 1 de
planejamento) mostrou que o custo de load por região já estava implementado e testado
corretamente (`cpu_load_timing.rs`) — o que falta é MULT/DIV, GTE, DMA (todos ainda a
ciclo zero) e este vazamento pontual. Próximos degraus: LWC2 cobrindo a região de load
(mesma cobertura que este vazamento protege), depois MULT/DIV, GTE, scheduler, DMA — cada
um sua própria iteração.
