<!-- Custo, tokens e duração NÃO entram aqui: são medidos por scripts/oc-iter.ps1 em
     docs/metricas.csv, casados por head_antes/head_depois. -->

# 0203e — hblank-agendado

- **Data:** 2026-08-06
- **Item do roadmap:** 10.117 (achado legado, iteração de origem 0176; parcial — ver Decisões)
- **Objetivo:** o sinal de Hblank tem que ser gerado por um evento real do scheduler, uma vez
  por scanline, não ficar sempre `false` porque nada nunca chama `gpu.set_hblank_active`.

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| psx-spx | § Vertical Timings — "The hblank signal is generated even during vertical blanking/retrace" (L1465-1467) | docs/reference/03-gpu.md |
| psx-spx | § GP1(06h) - Horizontal Display range (L826) | docs/reference/03-gpu.md |
| psx-spx | § Horizontal Timings (L1469) | docs/reference/03-gpu.md |

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a medição mostrou | Como foi pego |
|---|---|---|---|---|
| 1 | teste-teatral | Que "avançar o scheduler e ver hblank=true em algum momento e false em outro" provava que o evento estava agendado corretamente | A bateria de mutação matou só 1/5: o valor inicial `gpu.set_hblank_active(true)` do `Bus::new()` sozinho já bastava pra satisfazer "viu true", mascarando mutantes que trocavam X1↔X2 ou usavam o período de reagendamento errado (`frame` em vez de `cpu_cycles_per_scanline`) — o evento dispararia certo UMA vez (o agendamento inicial) e nunca mais, e o teste não notava | `scripts/mutantes.ps1 -Iter "0203"`: m1/m2/m4/m5 sobreviveram na primeira versão do teste |
| 2 | escopo | Que corrigir "hblank nunca agendado" resolveria também a outra metade do achado 10.117 ("System Clock" diverge ~13-70x do gabarito) | A iteração 0176 já tinha investigado isso e explicitamente não achou a causa raiz, mesmo depois de corrigir a propagação de timing da GPU pros timers (que também não moveu o número) — não há motivo pra esperar que agendar hblank sozinho mude uma métrica que a spec não liga a hblank | Reli `docs/iterations/0176-timers-gpu-gte-oraculo.md` antes de escrever o teste, não depois |

## Bateria de mutação

Placar da bateria: 5/5 mutantes mortos, 2/2 controles verdes, 0 equivalente —
`docs/mutantes/0203-hblank-agendado.mut`.

- m1 (X1/X2 trocados na conversão): morto — fração ativa medida cai pra ~25%, fora da faixa
  esperada (60-85%).
- m2 (HBLANK_ENTER liga `false` em vez de `true`): morto — 0 entradas em blanking observadas.
- m3 (HBLANK_EXIT desliga com `true` em vez de `false`): morto — fica preso em blanking pra
  sempre, fração ativa ~0%.
- m4 (HBLANK_ENTER reagenda a cada frame): morto — só 1 entrada em blanking em 3 scanlines,
  não 3.
- m5 (HBLANK_EXIT reagenda a cada frame): morto — mesmo padrão do m4, trava em blanking após
  a 1ª linha.
- c1 (ordem dos dois `schedule` iniciais): verde.
- c2 (ordem dos braços `HBLANK_ENTER`/`HBLANK_EXIT` no `match`): verde.

## Placar antes → depois

Workspace: **1250** → **1251** testes (1 novo em `timers_hblank_agendado.rs`).

## Revisão cruzada (orquestrador)

Sem achados — esta iteração foi conduzida pelo próprio orquestrador (exceção vigente em
`docs/orquestracao.md`; ver STATUS.md).

## Decisões e notas

**1. Este fix fecha só METADE do achado 10.117.** O achado original bundlava duas coisas: (a)
"hblank nunca agendado" (corrigido aqui) e (b) "'System Clock' diverge ~13-70x do gabarito,
causa raiz não encontrada" (iteração 0176, não corrigido lá nem aqui). Fechar 10.117 por
inteiro seria overclaim — reabri a parte (b) como achado novo `0203.3`, citando explicitamente
que a 0176 já tentou e não achou a causa, pra quem pegar esse achado depois não repetir o
mesmo caminho sem nova informação.

**2. Por que não medi o efeito no oráculo `timers` diretamente.** Não existe um
`oraculo_timers.rs` no `cargo test` — a comparação contra gabarito real (ps1-tests) que a 0176
usou parece ter sido um processo manual/script fora da suite normal. Medir isso exigiria
reproduzir aquele processo, o que ampliaria o escopo desta iteração (R4). O teste novo
(`timers_hblank_agendado`) prova a propriedade que dá pra provar sem essa infraestrutura:
hblank agora alterna de verdade, uma vez por linha, na proporção certa.

**3. Mesma simplificação do vblank: offset calculado uma vez.** `hblank_enter_offset`/
`hblank_exit_offset` são calculados a partir de X1/X2 (GP1 06h) uma única vez em `Bus::new()`
e reagendados sempre com o mesmo período (`cpu_cycles_per_scanline`) — um jogo que reescreve
GP1(06h) no meio da execução não realinha os eventos já agendados, igual o VBLANK já não
realinha com GP1(07h). Documentado inline no código, não é um achado novo — é a mesma dívida
técnica já aceita pro vblank.
