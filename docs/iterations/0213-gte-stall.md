<!-- Custo, tokens e duração NÃO entram aqui: são medidos por scripts/oc-iter.ps1 em
     docs/metricas.csv, casados por head_antes/head_depois. -->

# 0213 — gte-stall

- **Data:** 2026-08-06
- **Item do roadmap:** 0213.1 — Degrau 5 da escada de timing de CPU/barramento
- **Objetivo:** ligar o stall do GTE na CPU — instrução que lê registrador GTE (MFC2/CFC2/
  SWC2) ou emite comando novo antes do comando anterior terminar trava a CPU pelo resto do
  custo em voo. MTC2/CTC2/LWC2 (escrita) não esperam.

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| psx-spx | § GTE Load Delay Slots (L101-115) | docs/reference/07-gte.md |

Texto literal (L112-114): "If an instruction that reads a GTE register or a GTE command is
executed before the current GTE command is finished, the CPU will hold until the
instruction has finished."

## Erros de primeira tentativa

A primeira versão da fórmula de `gte_busy_until` pro caso de comando novo (`co==0x10..=0x1F`)
só tinha `bus.total_cycles() + 1 + custo` — sem contar que, se o comando anterior ainda
estivesse em voo, o comando novo primeiro espera (via `gte_stall`, que empilha em
`extra_cycles`) e só DEPOIS começa de verdade. Peguei isso derivando à mão o caso
`comando_gte_espera_o_comando_anterior` antes de escrever código: a fórmula certa soma
`self.extra_cycles` (a espera que acabou de ser empilhada, ainda não cobrada em ciclos de
verdade) ao prazo do comando novo. Implementada certa de primeira com essa correção.

Mesmo assim a bateria de mutação pegou uma segunda lacuna: nenhum teste dependia de fato do
termo `+ self.extra_cycles` na fórmula composta, porque em todos os testes existentes
`extra_cycles` valia 0 no instante em que o comando novo era emitido (só um comando GTE por
vez estava em voo). O mutante m5 (que apaga esse termo) sobreviveu. Corrigido com um teste
de três instruções (`rtps` → `nclip` → `mfc2`) que só bate (26) se o prazo do `nclip` tiver
contado a espera de 15 ciclos do `rtps` — sem o termo, o `mfc2` final não esperaria nada (18).

## Bateria de mutação

Placar da bateria: 7/7 mutantes mortos, 2/2 controles verdes, 0 equivalente —
docs/mutantes/0213-gte-stall.mut

m1/m2/m3 removem a espera em MFC2/CFC2/SWC2 respectivamente; m4 remove a espera do comando
anterior antes de emitir um comando novo; m5/m6 quebram cada termo da fórmula composta
(`+ self.extra_cycles` e `+ 1`); m7 cruza `gte_busy_until` com `hilo_busy_until` (o campo do
Degrau 3) dentro de `gte_stall`. Todos mortos pelos 13 testes de `cpu_gte_stall.rs`.

## Placar antes → depois

Workspace: **1313 → 1326** testes (13 novos em `cpu_gte_stall.rs`).

## Revisão cruzada (orquestrador)

Rodada de autoria do orquestrador (mesma exceção de executor da 0193/0209-0212). Checado:
`gte_fuzz_hardware` (oráculo de hardware, 1100/1100) continua igual — ciclos não mudam
resultado numérico do GTE, só timing. `cpu_load_timing.rs::laco_de_espera_da_bios_cobre_um_
frame` (invariante 17) reexecutado sem mudança — o laço da BIOS não usa GTE. `gte_registers.
rs`, `cpu_lwc2_timing.rs` e `cpu_coprocessador_usavel.rs` reexecutados sem regressão.
`snapshot_estado.rs` atualizado (`VERSAO` 2→3, `TAMANHO_DO_ESTADO` +8 bytes pro campo novo
`gte_busy_until: u64`). `cargo fmt --all -- --check` e `cargo clippy --all-targets -- -D
warnings` limpos. `cargo nextest run --workspace`: 1326/1326.

## Decisões e notas

MTC2/CTC2/LWC2 deliberadamente não chamam `gte_stall`: `07-gte.md` L112-114 só documenta
espera pra leitura ("reads a GTE register or a GTE command"), não pra escrita — omissão
declarada, não medida, igual ao padrão já usado no Degrau 3 pra `mtlo`/`mthi`.

Checagem rápida pós-Degrau 4 (registrada na 0212): RE2 e Tekken 3 ainda travavam no mesmo
lugar contra os Degraus 1-3 (não usam mult/div/lwc2). Ainda não remedidos contra este degrau
— fica pro Degrau 7 (medir os 5 jogos), depois do Degrau 6 (scheduler), conforme o plano.

Degrau 6 (próximo): o scheduler para de perder/atrasar evento periódico sob tick grande
(`scheduler.rs`/`bus.rs`) — bug do próprio scheduler, não spec de hardware, pré-requisito
duro antes do Degrau 9 (DMA cobra ciclos de verdade), cujo tick de um setor de CD-ROM
(12288 ciclos) senão perderia amostras de SPU e hblanks.
