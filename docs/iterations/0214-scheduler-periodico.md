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

## Implementação

`Scheduler::advance_to` devolve `(u64, EventId)` em vez de só `EventId` — o prazo que
venceu, não só o id do evento. Os quatro reagendamentos periódicos em `Bus::tick_timers`
(HBLANK_ENTER/EXIT, VBLANK_ENTER/EXIT, SPU_TICK) passam a somar o período a esse `prazo`,
não a `self.total_cycles`. O laço `while let Some(...) = advance_to(...)` já fazia catch-up
corretamente — o bug era só a base errada do reagendamento, não a falta de laço.

## Bateria de mutação

Sem manifesto novo: nenhum mutante de comportamento numérico cabe aqui além do que a
mudança de tipo já força o compilador a checar (assinatura `(u64, EventId)` propagada em
todo call site). A cobertura de regressão veio de `bus_scheduler.rs` (teste novo
`scheduler_devolve_o_prazo_que_venceu_nao_o_instante_de_avanco`) e `bus_scheduler_
periodico.rs` (3 testes novos, valores derivados de `spu::CPU_CYCLES_PER_SAMPLE`).

O manifesto antigo `docs/mutantes/0203-hblank-agendado.mut` teve as âncoras m4/m5/c2
reescritas (mesmo `@@DE`/`@@PARA` semântico, só a forma da linha mudou pelo `rustfmt`) e a
bateria rerodada: 5/5 mortos, 2/2 controles verdes — sem regressão nos mutantes que já
cobriam este trecho.

## Placar antes → depois

Workspace: **1326 → 1330** testes (3 novos em `bus_scheduler_periodico.rs`, 1 novo em
`bus_scheduler.rs`).

## Revisão cruzada (orquestrador)

Rodada de autoria do orquestrador (mesma exceção de executor da 0193/0209-0213). Checado:
invariante 17 (`cpu_load_timing.rs::laco_de_espera_da_bios_cobre_um_frame`) sem mudança —
o laço da BIOS não passa por eventos periódicos. `irq0_periodo_ntsc.rs` sem mudança (tica 1
ciclo por vez, nunca exercitou o catch-up). As 19 suítes de CD-ROM/áudio apontadas como
risco pelo plano (152 testes) passam sem alteração. `cargo fmt --all -- --check` e
`cargo clippy --all-targets -- -D warnings` limpos. `cargo nextest run --workspace`:
1330/1330 (a única falha antes deste doc era o placar do `STATUS.md`, corrigido agora).

## Decisões e notas

Pré-requisito do Degrau 9 (DMA cobrando ciclos de verdade) cumprido: um setor de CD-ROM
custa 12288 ciclos (achado 0193.4), tick que não seria mais absorvido silenciosamente sem
perder ~15 amostras de SPU e ~5 hblanks por setor, como o plano projetava.

Degrau 7 (próximo, não é degrau de código): remedir os 5 jogos travados (FF7/Tekken3/RE2/
TombRaider/CTR) contra os degraus 1-6 com `--sample-pcs`/`--watch-mem`, decidindo se DMA
(degraus 8-9, os mais arriscados) ainda é necessário.
