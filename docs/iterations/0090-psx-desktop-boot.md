# 0090 — psx-desktop-boot

- **Data:** 2026-07-30
- **Item do roadmap:** 9.0
- **Objetivo:** psx-desktop carrega BIOS, cria Bus+Cpu, avanca CPU no update loop e exibe framebuffer.

## Revisão do PR anterior

PR #104 (iter 0089): **um achado**.

**Achado G1 — MVMVA cv==2 (Bugged FC) usa componentes errados do vetor.** O código usava `m12*vx + m13*vy` (VX, VY), mas `docs/reference/07-gte.md` L558-559 diz que a fórmula reduzida usa as duas últimas porções: `Mx12*Vx2 + Mx13*Vx3` (VY, VZ). Corrigido para `m12*vy + m13*vz`. Consertado nesta rodada.

Nove padrões conferidos:
1. Teste que não mede — testes com valores golden da spec; sem round-trip ou assert_ne como única asserção
2. Parâmetro não consumido — GTE não tem FIFO de parâmetro; comandos leem registradores fixos
3. Regra de borda trocada — N/A (GTE)
4. Campo de bit lido errado — **pego**: cv==2 usava VX,VY em vez de VY,VZ; corrigido conforme `docs/reference/07-gte.md` L558-559
5. Panic ou laço ilimitado — sem unwrap/expect/unsafe
6. Citação de spec — `confere-citacoes.ps1` verde; `docs/reference/07-gte.md` L373-375 confirma que `regs[63] = 0` é correto (todos os bits resetam no início do comando)
7. Escopo transbordado — MVMVA implementado conforme item 5.4a; sem funcionalidade extra
8. Portão — manifesto reparado (m6 duplicado → específico MVMVA); `.resultado` rastreado
9. Manifesto arquivado — sem arquivamentos

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| psx-spx | MVMVA Multiply matrix/vector/translation (L541) | `docs/reference/07-gte.md` |
| psx-spx | cop2r63 - FLAG (L336) | `docs/reference/07-gte.md` |
| psx-spx | MVMVA Multiply matrix/vector/translation (L541) | `docs/reference/07-gte.md` |

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | campo-de-bit | cv==2 (Bugged FC) usa VX,VY como componentes do vetor | A fórmula reduzida pula Vx1 e usa só Vx2+Vx3: MAC1=Mx12*Vx2+Mx13*Vx3 (`docs/reference/07-gte.md` L558-559) | Revisão adversarial: leitura da spec revelou que o código usava vx,vy em vez de vy,vz |
| 2 | nenhum | Teste `bios_vazia_nao_liga_display` assumia que display começa desligado | `Gpu::new()` inicializa `stat = 0x1480_2000` com bit 23 set (display ligado por padrão) | Teste falhou na primeira execução; corrigido para refletir estado real do hardware |

## Bateria de mutação

Placar da bateria: 5/5 mutantes mortos, 2/2 controles verdes, 0 equivalente — docs/mutantes/0090-psx-desktop-boot.mut

| # | Tipo | Rótulo | Resultado |
|---|---|---|---|
| m1 | mutante | Gpu::new com display desligado (bit 23 zerado) | MORREU |
| m2 | mutante | framebuffer_for_display sempre retorna None | MORREU |
| m3 | mutante | display_width retorna zero | MORREU |
| m4 | mutante | display_height retorna zero | MORREU |
| m5 | mutante | framebuffer retorna Vec vazio (data substituído por Vec::new) | MORREU |
| c1 | controle | adiciona `let _dummy = 0` no início de framebuffer | verde |
| c2 | controle | adiciona `let _x = 0` antes do stat.get em framebuffer_for_display | verde |

## Placar antes → depois

Workspace: **669** → **671** testes (+2: desktop_boot).

## Decisões e notas

1. **Nova prioridade do usuário: deixar jogável com controles.** A fila do STATUS.md (GTE 5.4b) foi sobreposta. Este é o item (1) da nova prioridade: psx-desktop boota BIOS.

2. **Teste movido para `psx-core/tests/`.** O teste exercita Bus/Cpu/GPU (core), não a GUI do eframe. O `mutantes.ps1` só roda `cargo test -p psx-core`, então o teste precisou ficar em psx-core (item 10.33).

3. **BIOS boot travado no VSync timeout.** O display liga (bit 23 set desde o init do GPU), mas a VRAM permanece zerada porque a BIOS nunca completa o boot — bloqueio conhecido do I_MASK (item 4.4c). O framebuffer aparece, mas é uma tela preta.

4. **Ciclo de CPU por frame.** `bus.gpu().frame_cycles()` retorna os ciclos de um frame NTSC (~564.480). A CPU é instruction-stepped com 1 ciclo por step (`bus.tick_timers(1)`). O orçamento de passos por frame é igual ao número de ciclos.

5. **CLI minimalista.** O caminho da BIOS é passado como argumento posicional (`args().nth(1)`), sem crate de parsing. A dependência extra não se justifica para um único argumento.

6. **Item 9.0 novo.** Registrado no ROADMAP M9 porque 2.8 já estava `[x]` e a implementação original (0052/0053) era uma casca sem CPU/BIOS.
