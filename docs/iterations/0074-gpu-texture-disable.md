# 0074 — gpu-texture-disable

- **Data:** 2026-07-29
- **Item do roadmap:** 10.21
- **Objetivo:** gatear GPUSTAT bit 15 com GP1(09h).0, corrigindo as 3 falhas de `gpu/gp0-e1`.

## Revisão do PR anterior

Revisão do PR anterior (#87, iter 0073): sem achados. Os nove padrões conferidos:
1. Teste que não mede — o `assert_ne!` no `otc_ponteiro_guarda_24_bits` é segunda asserção depois de `assert_eq!`, não é o padrão proibido; m4 morre por um único teste (o novo), confirmado na bateria
2. Parâmetro não consumido — sem comandos GP0 novos
3. Regra de borda trocada — sem rasterização
4. Campo de bit lido errado — máscara MADR 0x00FF_FFFC (24 bits, alinhado), correta
5. Panic ou laço ilimitado — BCR=0 conta 0x10000 iterações, offset+4 <= ram.len() protege
6. Citação de spec — `confere-citacoes.ps1` verde
7. Escopo transbordado — `oc-loop.ps1` foi correção de infra necessária para o loop, documentada
8. Portão que não mede — bateria rodada e verificada pelo orquestrador
9. Manifesto arquivado — .resultado rastreado

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| psx-spx | GP1(09h) - Set VRAM size (v2) (L943) | docs/reference/03-gpu.md |
| psx-spx | GP0(E1h) - Draw Mode setting (L492) | docs/reference/03-gpu.md |
| psx-spx | GPUSTAT - GPU Status Register (L1002) | docs/reference/03-gpu.md |

A spec de GP1(09h) (L943-958) diz: bit 0 = 0 (default após reset) mascara todo Y para 9 bits;
bit 0 = 1 habilita a faixa Y 512-1023. O GP0(E1h) (L492-516) mapeia bit 11 → GPUSTAT.15, e a
tabela do GPUSTAT (L1003-1033) confirma que bit 15 = Texture page Y Base 2, "only for 2 MB
VRAM". O gate é GP1(09h).0: sem ele, o bit 11 do E1h não tem efeito visível no GPUSTAT.

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | flag-gate | Que GP1(09h) já estava implementado ou era irrelevante para os testes existentes | GP1(09h) é um comando separado que precisa ser tratado em write_gp1, e seu estado persiste | Três testes existentes quebravam: `gpu_status_gp0_gp1`, `gpu_textura_15bpp` e `gpu/gp0-e1` do hardware |
| 2 | escopo-teste | Que testar só a via GP0(E1h) bastava para fechar a bateria | O `apply_texpage_if_second` (polígonos texturizados) também escreve GPUSTAT.15 e precisava do mesmo gate | m5 sobreviveu na bateria com placar 5/6; reforçado com `poligono_texturizado_com_gate_fechado_nao_seta_gpustat_15` |
| 3 | manifesto | Que `@@PARA` vazio (deleção) era aceito pelo parser de manifesto | O parser exige texto no corpo do `@@PARA` | Meta-teste `mutation_anchors` reprovou m2 e m3; substituídos por blocos no-op (`let _ = bit;` e comentário) |

## Bateria de mutação

Placar da bateria: 6/6 mutantes mortos, 2/2 controles verdes, 0 equivalente — docs/mutantes/0074-gpu-texture-disable.mut

| Registro | Rótulo | Testes que pegaram, conforme o `.resultado` |
|---|---|---|
| m1 | GP1(09h) lê bit 1 em vez de bit 0 (gate invertido) | `e1h_com_gate_aberto_mantem_bits_0_10`, `gp1_09h_abre_o_gate_do_bit_15`, `poligono_texturizado_com_gate_aberto_seta_gpustat_15` |
| m2 | GP1(09h) nunca fecha o gate (não limpa stat.15) | `gp1_09h_bit0_zero_proibe_gpustat_15` |
| m3 | GP1(00h) esquece de resetar allow_upper_y | `gp1_00h_reset_fecha_o_gate_do_bit_15` |
| m4 | E1h sempre seta bit 15 (ignora o gate) | `apos_reset_gpustat_15_nao_reflete_e1h_bit11`, `comando_e1h_sozinho_nao_altera_gpustat_15`, `e1h_com_gate_fechado_ainda_escreve_bits_0_10`, `gp1_00h_reset_fecha_o_gate_do_bit_15` |
| m5 | apply_texpage_if_second sempre seta bit 15 (ignora o gate) | `poligono_texturizado_com_gate_fechado_nao_seta_gpustat_15` |
| m6 | GP1(09h) zera allow_upper_y em vez de setar | `e1h_com_gate_aberto_mantem_bits_0_10`, `gp1_09h_abre_o_gate_do_bit_15`, `poligono_texturizado_com_gate_aberto_seta_gpustat_15` |
| c1 | reordena campos da struct Gpu | sobreviveu |
| c2 | extrai condição do gate para variável local | sobreviveu |

O m5 merece destaque: ele morre **por um único teste** (`poligono_texturizado_com_gate_fechado_nao_seta_gpustat_15`), que foi acrescentado depois que a primeira rodada da bateria deu 5/6. Sem ele, o `apply_texpage_if_second` ignoraria o gate e o `gpu/gp0-e1` continuaria falhando em `testTexturedPolygons`.

## Placar antes → depois

Workspace: **558** → **560** testes (+2: `poligono_texturizado_com_gate_fechado_nao_seta_gpustat_15` e `poligono_texturizado_com_gate_aberto_seta_gpustat_15`).

**Hardware — `gpu/gp0-e1`:** esperado sair de 7p/3f para 10p/0f. O orquestrador medirá.

## Revisão cruzada (orquestrador)

<!-- Preenchido pelo orquestrador. -->

## Decisões e notas

1. O gate é aplicado na **escrita** do GPUSTAT, não na leitura. O bit 15 é limpo na máscara de
   bits do E1h e do `apply_texpage_if_second` quando `allow_upper_y` é false, e o GP1(09h).0=0
   também limpa o bit 15 ativamente. Não há latch separado para restaurar o valor quando o gate
   reabre — o software precisa reescrever E1h.
2. O manifesto 0050-display-regs.mut (K2) teve a âncora reparada: a adição do handler GP1(09h)
   entre GP1(08h) e o `_ => {}` quebrou o casamento de padrão de 4 linhas. Ambos os blocos
   `@@DE` e `@@PARA` receberam o novo bloco `0x09 => { ... }`.
3. A contagem de testes do workspace subiu de 558 para 560 (+2). O placar de hardware será
   medido pelo orquestrador.
