# 0193 — desktop-disco

- **Data:** 2026-08-03
- **Item do roadmap:** 9.1 (semente; o item continua aberto — falta a tela de biblioteca)
- **Objetivo:** primeira sessão de jogo real no app desktop; regularizar os patches da
  sessão e registrar o que ela mediu.

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| — | nenhuma (código de app, não de hardware) | — |

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | endereçamento | `inject_disc` bastaria para o boot ver o disco | o drive também precisa de `insert_disc()`; sem ele o GetID responde INT5 e a BIOS cai no shell | 132× `SECOND_IRQ2 intsts=0x05` no log; BIOS abriu o gerenciador de memory card |
| 2 | timing | jogo 2× rápido = falta de cap de FPS no app | o cap existia; a causa é 0193.4 (1 ciclo/instrução, GPU em 0 ciclos) — slider a 0,5× desacelera tudo e não conserta | usuário jogou a 0,5×: lógica mais lenta, áudio a 22 kHz, injogável |

## Bateria de mutação

Bateria de mutação: não se aplica — o PR só toca `psx-desktop` (fora da bateria por
10.33/10.58) e documentos; validação por sessão de jogo do usuário e pelos logs de boot
(INT1 de dados streamando após o `insert_disc`).

## Placar antes → depois

Workspace: 1137 → 1137 (nenhum teste novo; PR de app + registro).

## Revisão cruzada (orquestrador)

n/a — o orquestrador é o autor (ver exceção em `docs/orquestracao.md`, 2026-08-03).

## Decisões e notas

- App aceita `psx-desktop <BIOS> [jogo.cue] [cartao.mcd]`; carga de CUE copiada do
  `psx-cli` (`load_disc`), com `atribui_lbas_absolutos` por arquivo do cue.
- Pacing por `total_cycles` contra relógio de parede (33,8688 MHz), `dt` limitado a
  50 ms; slider 0,25–2,0× multiplicando o alvo — ferramenta de diagnóstico, não fix.
- Escala 4:3 com `fit_to_exact_size` + NEAREST; barra de status com Hz do áudio e
  resolução do modo de vídeo.
- A sessão abriu os achados 0193.1–0193.7 e a escada de correção (ver STATUS.md);
  exceção de executor documentada no diário.
