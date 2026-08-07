<!-- Custo, tokens e duração NÃO entram aqui: são medidos por scripts/oc-iter.ps1 em
     docs/metricas.csv, casados por head_antes/head_depois. -->

# 0214 — scheduler-periodico

- **Data:** 2026-08-06
- **Item do roadmap:** 0214.1 — Degrau 6 da escada de timing de CPU/barramento
- **Objetivo:** o scheduler para de perder/atrasar eventos periódicos (VBLANK/HBLANK/
  SPU_TICK) quando um único `tick_timers` cobre vários períodos de uma vez.

## Spec consultada

Não é spec de hardware — é bug do próprio scheduler (R2 do `CLAUDE.md`: "componentes
avançam por timestamps no scheduler"). A propriedade defendida é o teste já existente
`irq0_periodo_ntsc.rs::irq0_ntsc_mantem_periodo_de_um_frame_em_ciclos` (período de VBLANK
constante em 566.187 ciclos), que só nunca pegou este bug porque tica 1 ciclo por vez.

## Erros de primeira tentativa

Nenhum na implementação — o defeito e a correção já vinham identificados pelo plano da
escada (`scheduler.rs:46-56` descartava o prazo vencido; `bus.rs` reagendava a partir de
`self.total_cycles`). A surpresa foi na medição do risco: o plano previa "risco de
regressão em ~25 chamadas diretas a `bus.tick_timers(N)` grandes em `cdrom_*.rs`/
`audio_ring.rs`" que assumiriam "tick grande dispara o evento periódico no máximo uma vez".
Rodei as 19 suítes (`audio_ring.rs` + 18 arquivos `cdrom_*.rs`, 152 testes) depois do fix:
**nenhuma quebrou.** Ou os ticks usados nesses testes não cobrem mais de um período de
SPU_TICK/HBLANK/VBLANK, ou as asserções já checavam comportamento (IRQ subiu, resposta
chegou) em vez de contagem de disparo — o risco previsto não se concretizou.

Na bateria de mutação, sim: escrevi o manifesto confiando na herança de `teste:` do
cabeçalho pros registros m1/m2/m3/c1 (só declarei explicitamente nos que miram `bus.rs`).
STATUS.md já registrava esse gotcha há muito (10.71/0187: "`mutantes.ps1` herda o último
`teste:` visto"), e caí nele de novo: `m3` (que remove o elemento errado da fila do
scheduler — `self.events.remove(self.events.len() - 1)` em vez de `remove(0)`) acabou
rodando contra `bus_scheduler_periodico` em vez de `bus_scheduler`. Como esse alvo TEM
testes que chamam `bus.tick_timers` com um tick gigante, a mutação reintroduz exatamente o
cenário que ela quebra: o evento devido no topo da fila nunca é removido (o laço sempre
remove o do fim), então o `while let Some(...)` de `tick_timers` nunca esvazia — cada
evento indevidamente "disparado" reagenda outro, que vira o novo fim da fila, para sempre.
O processo ficou preso ~520s de CPU (confirmado via `Get-Process`) antes de eu matá-lo.
Corrigido declarando `teste: bus_scheduler` em TODO registro do manifesto, não só no
cabeçalho — com isso `m3` roda contra `bus_scheduler.rs` (sem `tick_timers` em loop) e
morre limpo em <1s.

## Implementação

`Scheduler::advance_to` devolve `(u64, EventId)` em vez de só `EventId` — o prazo que
venceu, não só o id do evento. Os quatro reagendamentos periódicos em `Bus::tick_timers`
(HBLANK_ENTER/EXIT, VBLANK_ENTER/EXIT, SPU_TICK) passam a somar o período a esse `prazo`,
não a `self.total_cycles`. O laço `while let Some(...) = advance_to(...)` já fazia catch-up
corretamente — o bug era só a base errada do reagendamento, não a falta de laço.

## Bateria de mutação

Placar da bateria: 5/5 mutantes mortos, 2/2 controles verdes, 0 equivalente —
docs/mutantes/0214-scheduler-periodico.mut

m1 reintroduz o bug original (`advance_to` devolve `ticks` em vez de `key.tick`); m2 quebra
a fronteira exata (`>` vira `>=`, evento no prazo exato deixa de disparar); m3 remove o
elemento errado da fila (incidente acima); m4/m5 reintroduzem o bug especificamente nos
braços SPU_TICK e VBLANK_ENTER de `bus.rs` (reagendar a partir de `total_cycles`). Todos
mortos por `bus_scheduler.rs`/`bus_scheduler_periodico.rs`.

O manifesto antigo `docs/mutantes/0203-hblank-agendado.mut` teve as âncoras m4/m5/c2
reescritas (mesmo `@@DE`/`@@PARA` semântico, só a forma da linha mudou pelo `rustfmt`) e a
bateria rerodada: 5/5 mortos, 2/2 controles verdes — sem regressão nos mutantes que já
cobriam este trecho.

## Placar antes → depois

Workspace: **1326 → 1331** testes (4 novos em `bus_scheduler_periodico.rs`, 1 novo em
`bus_scheduler.rs`).

## Revisão cruzada (orquestrador)

Rodada de autoria do orquestrador (mesma exceção de executor da 0193/0209-0213). Checado:
invariante 17 (`cpu_load_timing.rs::laco_de_espera_da_bios_cobre_um_frame`) sem mudança —
o laço da BIOS não passa por eventos periódicos. `irq0_periodo_ntsc.rs` sem mudança (tica 1
ciclo por vez, nunca exercitou o catch-up). As 19 suítes de CD-ROM/áudio apontadas como
risco pelo plano (152 testes) passam sem alteração. `cargo fmt --all -- --check` e
`cargo clippy --all-targets -- -D warnings` limpos. `cargo nextest run --workspace`:
1331/1331 (a única falha antes deste doc era o placar do `STATUS.md`, corrigido agora).

## Decisões e notas

Pré-requisito do Degrau 9 (DMA cobrando ciclos de verdade) cumprido: um setor de CD-ROM
custa 12288 ciclos (achado 0193.4), tick que não seria mais absorvido silenciosamente sem
perder ~15 amostras de SPU e ~5 hblanks por setor, como o plano projetava.

Degrau 7 (próximo, não é degrau de código): remedir os 5 jogos travados (FF7/Tekken3/RE2/
TombRaider/CTR) contra os degraus 1-6 com `--sample-pcs`/`--watch-mem`, decidindo se DMA
(degraus 8-9, os mais arriscados) ainda é necessário.
