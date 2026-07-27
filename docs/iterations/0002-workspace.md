# 0002 — workspace Rust

- **Data:** 2026-07-27
- **Item do roadmap:** 0.2
- **Objetivo:** workspace com 3 crates e esqueleto de módulos do core.

## O que foi feito

Workspace edition 2024 (`rust-version 1.85`, resolver 3): `psx-core` (lib pura,
`#![forbid(unsafe_code)]`, 12 módulos vazios: bus, cdrom, cpu, dma, gpu, gte, irq, mdec,
scheduler, sio, spu, timers), `psx-cli` e `psx-desktop` (stubs). `psx-core` sem dependências —
o meta-teste `purity.rs` (iteração 0003) congela isso. fmt/clippy `-D warnings`/test verdes.

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que era | Como foi pego |
|---|---|---|---|---|
| — | nenhum | | | |

## Notas

Módulos nascem vazios de propósito: cada um ganha conteúdo na iteração do seu item do ROADMAP,
e `docs/mapa.md` (iteração 0005) aponta módulo → arquivo → responsabilidade.
