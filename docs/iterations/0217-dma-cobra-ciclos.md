<!-- Custo, tokens e duração NÃO entram aqui: são medidos por scripts/oc-iter.ps1 em
     docs/metricas.csv, casados por head_antes/head_depois. -->

# 0217 — dma-cobra-ciclos

- **Data:** 2026-08-07
- **Item do roadmap:** 0217.1 — Degrau 9 (último) da escada de timing de CPU/barramento
- **Objetivo:** ligar `Dma::transfer_cost` (Degrau 8) na cobrança real de ciclos —
  transferências DMA passam a custar tempo de CPU de verdade, não mais zero.

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| psx-spx | § DMA Transfer Rates (L217-227), já usada no Degrau 8 | docs/reference/04-dma.md |

Nenhuma seção nova — este degrau liga a tabela já derivada e citada na 0215, não introduz
número novo da spec.

## Erros de primeira tentativa

Nenhum na aritmética (a tabela já vinha certa da 0215). O ponto que exigiu mais cuidado foi
arquitetural, não de spec: `try_execute_*`/`execute_*` (dma.rs) executam a transferência
INTEIRA de forma síncrona, dentro do `write32` que a CPU emite ao escrever CHCR/DPCR — não
existe "evento de DMA" no scheduler. Cobrar o custo, então, não podia ser "somar em
`tick_timers`" diretamente (`tick_timers` só é chamado pela CPU no FIM da instrução, e o
DMA já rodou por dentro dela) — tinha que ser um acumulador (`Bus::dma_extra_cycles`) que
o `write32` alimenta e que `tick_timers` dre na na primeira chamada seguinte, exatamente o
mesmo padrão que `Cpu::extra_cycles` já usa pros stalls de load/mult/GTE (Degraus 1, 3, 5).
Reconhecer esse paralelo ANTES de escrever código evitou uma tentativa errada (cobrar
direto em `total_cycles` de dentro do `write32`, que pularia a checagem de scheduler que só
`tick_timers` faz).

A mutação pegou uma lacuna real de teste, não de implementação: sem um teste que chamasse
`tick_timers` DUAS vezes depois de um único DMA, nada discriminava `self.dma_extra_cycles`
sendo lido sem `mem::take` (o custo vazaria pro tick seguinte também). Corrigido com
`custo_do_dma_e_drenado_no_tick_e_nao_se_repete_no_seguinte`.

## Implementação

Cada `try_execute_dmaN`/`try_execute_otc` (e os três helpers de `try_execute_dma2`:
`execute_burst`/`execute_block`/`execute_linked_list`) passa a devolver `usize` — as
palavras que realmente passaram pelo barramento nesta chamada (0 se o canal nem chegou a
rodar, por gate de DPCR/CHCR ausente). `Bus::charge_dma(channel, words)` converte isso em
ciclos via `Dma::transfer_cost` e soma em `dma_extra_cycles`; `Bus::tick_timers` drena esse
acumulador (`std::mem::take`) ANTES de avançar `total_cycles` — logo antes de drenar o
scheduler e antes de `Timers::tick`, como o plano exigia.

## Bateria de mutação

Placar da bateria: 5/5 mutantes mortos, 2/2 controles verdes, 0 equivalente —
docs/mutantes/0217-dma-cobra-ciclos.mut

m1 calcula o custo mas não acumula; m2 impede `tick_timers` de somar o acumulado; m3 usa
custo fixo por canal (ignora a contagem de palavras); m4 lê o acumulador sem drenar (custo
repete no tick seguinte); m5 troca `+=` por `=` (dois DMAs no mesmo tick perdem o primeiro).
Todos mortos pelos 5 testes de `dma_cobra_ciclos.rs`.

Sete manifestos antigos (`0057-dma-gpu`, `0066-dma-cdrom`, `0071-dma-dpcr-gate`,
`0117-dma-gpu-vram-para-ram`, `0184-mdec-cor-e-ack`, `0201-dpcr-retrigger`,
`0215-dma-custo-palavra`) tiveram âncoras reescritas (mudança de assinatura `()→usize` e
`self.charge_dma(...)` inserido após cada chamada — mesmo `@@DE`/`@@PARA` semântico, só a
forma da linha mudou) e rerodados: todos com o mesmo placar de antes, sem regressão.

## Placar antes → depois

Workspace: **1344 → 1349** testes (5 novos em `dma_cobra_ciclos.rs`).

## Revisão cruzada (orquestrador)

Rodada de autoria do orquestrador (mesma exceção de executor da 0193/0209-0216). Checado:
invariante 17 (`cpu_load_timing.rs::laco_de_espera_da_bios_cobre_um_frame`) sem mudança —
o laço da BIOS não usa DMA. Oráculo `gte_fuzz_hardware` sem mudança — DMA não toca no GTE.
As 14 suítes de DMA/CD-ROM/MDEC (`cdrom_dma`, `dma_burst_sync0[_chopping]`,
`dma_chain_looping`, `dma_custo_transferencia`, `dma_dicr_irq3`, `dma_dpcr_gate`,
`dma_dpcr_retrigger`, `dma_gpu[_vram_para_ram]`, `dma_lista_encadeada_longa`, `dma_otc`,
`mdec_registers_dma`) passam sem alteração — a mudança de assinatura (`()→usize`) não
alterou nenhum comportamento de transferência, só reporta o que já acontecia. `cargo fmt
--all -- --check` e `cargo clippy --all-targets -- -D warnings` limpos. `cargo nextest run
--workspace`: 1349/1349.

**Não pude rerodar os testes do Rayman convertidos na 0216 contra o disco real** — a
imagem ainda não está disponível nesta sessão/máquina (`../roms/extraido/`). Fica pendente
para quando a imagem estiver acessível; os testes compilam e a lógica foi revisada, mas a
janela (140M-220M passos) escolhida por inspeção não foi confirmada empiricamente.

## Decisões e notas

Lista encadeada (GPU, SyncMode=2) conta a palavra de cabeçalho de cada nó como transferida,
além dos dados — inferência declarada do Degrau 8 (a spec dá a tabela de custo por canal
mas não fala explicitamente do overhead do cabeçalho no modo de lista), agora efetivamente
em uso na cobrança.

**Último degrau da escada motivada pelo achado 0193.4.** Próximo passo natural (fora desta
rodada): remedir os 5 jogos do Degrau 7 (Tekken3/RE2/Tomb Raider especialmente, que
travavam perto de CD-ROM) contra a escada completa 1-9, e rerodar os 3 testes do Rayman
convertidos na 0216 contra o disco real assim que disponível.
