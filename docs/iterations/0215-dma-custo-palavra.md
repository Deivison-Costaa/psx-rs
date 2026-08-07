<!-- Custo, tokens e duração NÃO entram aqui: são medidos por scripts/oc-iter.ps1 em
     docs/metricas.csv, casados por head_antes/head_depois. -->

# 0215 — dma-custo-palavra

- **Data:** 2026-08-07
- **Item do roadmap:** 0215.1 — Degrau 8 da escada de timing de CPU/barramento
- **Objetivo:** `Dma::word_cost_per_256`/`transfer_cost` — tabela pura de custo de
  transferência por canal DMA, sem chamador ainda (o Degrau 9 liga isso no tick de verdade).

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| psx-spx | § DMA Transfer Rates (L217-227) | docs/reference/04-dma.md |
| psx-spx | § DRAM Hyper Page mode (L238-243), nota de cross-check | docs/reference/04-dma.md |

## Erros de primeira tentativa

Nenhum — os 5 valores (`110h`/`1800h`/`420h`/`1400h` ciclos por `100h` palavras) foram lidos
direto do bloco de texto da spec via `grep -n` e recalculados à mão (ex.: `0x1800/0x100=24`,
`0x420/0x100=33/8=4.125`), não copiados do "N clks/word" arredondado do cabeçalho da
tabela. O plano da escada já tinha essas frações certas (`17/16`, `24/1`, `33/8`, `20/1`),
mas recalculei do zero em vez de confiar na memória do plano — bateu exato.

## Bateria de mutação

Placar da bateria: 6/6 mutantes mortos, 2/2 controles verdes, 0 equivalente —
docs/mutantes/0215-dma-custo-palavra.mut

m1/m2 trocam o custo entre canais (CDROM↔SPU, SPU↔PIO); m3 quebra a fração 17/16 do
MDEC/GPU/OTC pra um clk/palavra plano; m4 usa divisor 255 em vez de 256 (100h); m5 remove a
divisão da fórmula (custo bruto, não normalizado por palavra); m6 faz canal inválido (7+)
custar como MDEC.IN em vez de zero — todos mortos pelos 13 testes de valor exato, incluindo
os dois cross-checks contra o texto da própria spec (`transfer_cost(MDEC_IN, 16) == 17`,
citado literalmente na nota de DRAM Hyper Page mode, e `transfer_cost(SPU, 8) == 33`).

## Placar antes → depois

Workspace: **1331 → 1344** testes (13 novos em `dma_custo_transferencia.rs`).

## Revisão cruzada (orquestrador)

Rodada de autoria do orquestrador (mesma exceção de executor da 0193/0209-0215). `cargo fmt
--all -- --check` e `cargo clippy --all-targets -- -D warnings` limpos. Zero risco de
regressão — função pura, sem chamador ainda, igual ao padrão do Degrau 4 (`Gte::
command_cycles`); nenhum teste existente de `dma.rs` foi tocado.

## Decisões e notas

Canal CDROM (3) fica fixo no padrão de 24 clks/palavra da BIOS (`24 => 1800h por 100h`); a
spec documenta que a maioria dos jogos reconfigura pra 40 clks/palavra via registrador de
memory control, mas não dá a fórmula que decide — modelar isso agora seria adivinhar (R1).
Fica achado aberto, não vira degrau ativo.

Tempo de decodificação do MDEC e tempo de desenho da GPU continuam de fora — a própria spec
declara os dois desconhecidos (`04-dma.md` L228-230, achado 10.116).

Degrau 9 (próximo): cobrar o custo de verdade em `Bus::tick_timers`, antes de drenar o
scheduler e antes de `Timers::tick` (se os timers não virem o mesmo delta, ficam
artificialmente lentos durante DMA). Depende do Degrau 6 (scheduler, já pronto — sem isso
um tick de 12288 ciclos de um setor de CD-ROM perderia eventos periódicos). Maior risco:
testes do Rayman com passo absoluto (`rayman_autoack.rs`/`rayman_exception_chain.rs`/
`rayman_tty_boot.rs`, achado 10.115) — converter pra condição-primeiro ANTES de tocar em
`bus.rs`, não depois.
