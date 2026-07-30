# 0081 — revisao-0080-corrige-t10

- **Data:** 2026-07-30
- **Item do roadmap:** N/A (revisão adversarial do PR #95)
- **Objetivo:** Corrigir defeito achado na revisão do PR anterior (#95, iter 0080) e verificar o estado de 10.22.

## Revisão do PR anterior

Revisão do PR anterior (#95, iter 0080): **1 defeito encontrado e corrigido.**

**Padrão 1 — Teste que não mede** (t10 `gpu_vblank_irq_bios_boot_placeholder`):
apenas `eprintln!()` sem nenhuma asserção. Nenhuma mutação no código de vblank/IRQ0 faria
este teste falhar. Substituído por `t10_vblank_entra_e_sai_durante_dois_frames`, que verifica
que `vblank_active()` sai e re-entra durante dois frames completos.

Demais padrões conferidos:
1. Teste que não mede — t10 corrigido (acima); t3-t9 cobrem o comportamento com 7/7 mutantes mortos
2. Parâmetro não consumido — sem novos comandos GPU
3. Regra de borda trocada — sem rasterização
4. Campo de bit lido errado — sem novos registradores
5. Panic ou laço ilimitado — `frame_cycles()` nunca retorna 0; sem unwrap/expect/unsafe fora de teste
6. Citação de spec — `confere-citacoes.ps1` verde na 0080
7. Escopo transbordado — hblank declarado como dívida, sem implementação extra
8. Portão — `.resultado` rastreado, `mutation_anchors` verde
9. Manifesto arquivado — sem arquivamentos na 0080

## Spec consultada

Nenhuma — esta iteração é correção de defeito de teste, não implementação nova.

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| | nenhum | — | — | — |

## Bateria de mutação

Bateria de mutação: não se aplica — esta iteração corrige um teste que era no-op (substituição
de placeholder por teste real), sem implementar nova funcionalidade. O comportamento verificado
já era coberto pela bateria da 0080 (7/7 mutantes mortos, conferidos no `.resultado`).

## Verificação de 10.22

O STATUS.md da 0080 apontava 10.22 (`gpu/mask-bit`) como próximo item com "2 de 5 reprovando
no scoreboard". O scoreboard foi re-executado no commit `bd1a838` (HEAD após merge do #95):

```
ps1-tests/gpu/mask-bit,mask-bit.exe,pass,5p/0f
```

**10.22 está completo** — iter 0075 fechou os 2 subtestes que falhavam (3p/2f → 5p/0f).
O ROADMAP já marcava `[x] 10.22` corretamente. O STATUS.md da 0080 continha informação
incorreta (provavelmente oriunda de handoff anterior à 0075).

## Placar antes → depois

Workspace: **583** → **583** testes (t10 substituído, mesma quantidade; 1 teste de `gpu_vblank_irq`
que agora mede algo em vez de ser no-op).

Scoreboard no commit `bd1a838`: 5 suítes com veredito — 4 pass (cop, otc-test, gp0-e1, mask-bit),
1 fail (code-in-io). **mask-bit = 5p/0f.**

## Revisão cruzada (orquestrador)

<!-- Preenchido pelo orquestrador. -->

## Decisões e notas

1. **t10 substituído, não deletado.** O arquivo `gpu_vblank_irq.rs` mantém 10 testes; o novo t10
   verifica que vblank sai e re-entra durante dois frames. Isso mantém a cobertura numérica mas
   substitui um teste inútil por um teste que mede.
2. **STATUS.md corrigido.** A próxima tarefa era 10.22 (já completo). Atualizada para o primeiro
   item aberto do M5 (5.1 — GTE registradores + MFC2/MTC2), já que M4 está bloqueado (4.4
   precisa de imagem BIN/CUE).
3. **M4 bloqueado em 4.4.** O boot de jogo depende de imagem de disco que o usuário deve fornecer.
   Registrado em Bloqueios do STATUS.md.
