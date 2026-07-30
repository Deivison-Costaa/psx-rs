# 0076 — gpu-texture-disable-gate

- **Data:** 2026-07-29
- **Item do roadmap:** 10.32
- **Objetivo:** remover a limpeza ativa de GPUSTAT.15 no handler de GP1(09h).0=0 — fechar o gate nao limpa o latch.

## Revisão do PR anterior

Revisão do PR anterior (#89, iter 0075): sem achados. Os nove padrões conferidos:
1. Teste que não mede — T8-T12 com asserções contra valores concretos; bateria 5/5 confirma cobertura
2. Parâmetro não consumido — sem comandos GP0 novos; CPU→VRAM já existia
3. Regra de borda trocada — sem rasterização
4. Campo de bit lido errado — GPUSTAT.11 (force) e GPUSTAT.12 (check_mask), corretos per 03-gpu.md (L1010-1011)
5. Panic ou laço ilimitado — px/py mascarados a 10/9 bits; sem unwrap/expect/unsafe
6. Citação de spec — `confere-citacoes.ps1` verde
7. Escopo transbordado — apenas CPU→VRAM; VRAM→VRAM declarado como dívida
8. Portão que não mede — bateria 5/5 mortos, `.resultado` rastreado
9. Manifesto arquivado — 0038-vram-transfers.mut reparado (âncora atualizada com mask-bit)

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| psx-spx | GP1(09h) - Set VRAM size (v2) (L943) | docs/reference/03-gpu.md |
| psx-spx | GP0(E1h) - Draw Mode setting (L492) | docs/reference/03-gpu.md |
| psx-spx | GPUSTAT - GPU Status Register (L1002) | docs/reference/03-gpu.md |

A spec de GP1(09h) (L943-958) descreve o bit 0 como portão do decodificador de endereço de Y:
bit 0 = 0 mascara Y para 9 bits; bit 0 = 1 habilita a faixa 512-1023. **Não menciona limpeza
de GPUSTAT.15.** O GPUSTAT.15 é um latch escrito por GP0(E1h) e pelo texpage de polígono
texturizado, gateado na escrita. Fechar o portão não apaga o que já estava latcheado.

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | teste-que-certifica-o-defeito | Que GP1(09h).0=0 limpa GPUSTAT.15 ativamente (implementado na 0074) | A spec de GP1(09h) só descreve o bit 0 como gate do decodificador de Y; não menciona limpeza de GPUSTAT.15 | Medido pelo orquestrador em worktree isolado: `gpu/gp0-e1` 9p/1f com o `if`, 10p/0f sem ele. Confirmado localmente via scoreboard |
| 2 | manifesto | Que o m2 original da 0074 estava correto (GP1(09h) nunca fecha o gate como mutante a ser morto) | O comportamento do m2 original é exatamente o correto: fechar o gate não limpa o latch | m2 substituído por mutante novo (`bit == 0` em vez de `bit != 0`), pego por `gp1_09h_abre_o_gate_do_bit_15` |

**Categoria `teste-que-certifica-o-defeito`**: terceira ocorrência no projeto. A 0056 teve
asserções espelhadas no `dma_otc`, a 0074 teve este mesmo teste `gp1_09h_bit0_zero_proibe_gpustat_15`
que afirmava o comportamento errado, e agora a 0076 corrige.

## Bateria de mutação

### Manifesto 0076

Placar da bateria: 5/5 mutantes mortos, 2/2 controles verdes, 0 equivalente - docs/mutantes/0076-gpu-texture-disable-gate.mut

| Registro | Rótulo | Testes que pegaram, conforme o `.resultado` |
|---|---|---|
| m1 | GP1(09h) inverte o gate (bit == 0 em vez de bit != 0) | `gp1_09h_abre_o_gate_do_bit_15`, `gp1_09h_fecha_gate_reescrever_e1h_limpa_gpustat_15` |
| m2 | GP1(09h) sempre abre o gate (ignora bit, seta true) | `gp1_09h_fecha_gate_reescrever_e1h_limpa_gpustat_15` |
| m3 | GP1(09h) sempre fecha o gate (ignora bit, seta false) | `gp1_09h_abre_o_gate_do_bit_15`, `poligono_texturizado_com_gate_aberto_seta_gpustat_15` |
| m4 | GP1(09h) lê bit 1 em vez de bit 0 | `gp1_09h_abre_o_gate_do_bit_15`, `gp1_09h_fecha_gate_reescrever_e1h_limpa_gpustat_15` |
| m5 | GP1(00h) esquece de resetar allow_upper_y | `gp1_00h_reset_fecha_o_gate_do_bit_15` |
| c1 | renomeia bit para gate_bit (cosmético) | sobreviveu |
| c2 | extrai condição para variável local (cosmético) | sobreviveu |

### Manifesto 0074 (rerrodado após reparo de âncora)

Placar rerrodado: 6/6 mutantes mortos, 2/2 controles verdes, 0 equivalente — docs/mutantes/0074-gpu-texture-disable.mut

| Registro | Rótulo | Testes que pegaram, conforme o `.resultado` |
|---|---|---|
| m1 | GP1(09h) lê bit 1 em vez de bit 0 (gate invertido) | `e1h_com_gate_aberto_mantem_bits_0_10`, `gp1_09h_abre_o_gate_do_bit_15`, `poligono_texturizado_com_gate_aberto_seta_gpustat_15` |
| m2 | GP1(09h) inverte o gate (bit == 0 em vez de bit != 0) — **substituído na 0076** | `gp1_09h_abre_o_gate_do_bit_15`, `gp1_09h_fecha_gate_reescrever_e1h_limpa_gpustat_15` |
| m3 | GP1(00h) esquece de resetar allow_upper_y | `gp1_00h_reset_fecha_o_gate_do_bit_15` |
| m4 | E1h sempre seta bit 15 (ignora o gate) | `apos_reset_gpustat_15_nao_reflete_e1h_bit11`, `comando_e1h_sozinho_nao_altera_gpustat_15`, `e1h_com_gate_fechado_ainda_escreve_bits_0_10`, `gp1_00h_reset_fecha_o_gate_do_bit_15` |
| m5 | apply_texpage_if_second sempre seta bit 15 (ignora o gate) | `poligono_texturizado_com_gate_fechado_nao_seta_gpustat_15` |
| m6 | GP1(09h) zera allow_upper_y em vez de setar | `e1h_com_gate_aberto_mantem_bits_0_10`, `gp1_09h_abre_o_gate_do_bit_15`, `poligono_texturizado_com_gate_aberto_seta_gpustat_15` |
| c1 | reordena campos da struct Gpu | sobreviveu |
| c2 | extrai condição do gate para variável local | sobreviveu |

A bateria da 0074 foi rerrodada após a substituição do m2 (âncora reparada) e o `.resultado`
foi regerado. Placar mantido em 6/6.

### Bateria 0076 (manifesto próprio)

Placar da bateria: 5/5 mutantes mortos, 2/2 controles verdes, 0 equivalente - docs/mutantes/0076-gpu-texture-disable-gate.mut

| Registro | Rótulo | Testes que pegaram, conforme o `.resultado` |
|---|---|---|
| m1 | GP1(09h) inverte o gate (bit == 0 em vez de bit != 0) | `gp1_09h_abre_o_gate_do_bit_15`, `gp1_09h_fecha_gate_reescrever_e1h_limpa_gpustat_15` |
| m2 | GP1(09h) sempre abre o gate (ignora bit, seta true) | `gp1_09h_fecha_gate_reescrever_e1h_limpa_gpustat_15` |
| m3 | GP1(09h) sempre fecha o gate (ignora bit, seta false) | `gp1_09h_abre_o_gate_do_bit_15`, `poligono_texturizado_com_gate_aberto_seta_gpustat_15` |
| m4 | GP1(09h) lê bit 1 em vez de bit 0 | `gp1_09h_abre_o_gate_do_bit_15`, `gp1_09h_fecha_gate_reescrever_e1h_limpa_gpustat_15` |
| m5 | GP1(00h) esquece de resetar allow_upper_y | `gp1_00h_reset_fecha_o_gate_do_bit_15` |
| c1 | renomeia bit para gate_bit (cosmético) | sobreviveu |
| c2 | extrai condição para variável local (cosmético) | sobreviveu |

## Placar antes → depois

Workspace: **561** → **563** testes (+2: `gp1_09h_fecha_gate_reescrever_e1h_limpa_gpustat_15` e `poligono_abre_gate_fecha_e_latch_mantem_gpustat_15`). O teste `gp1_09h_bit0_zero_proibe_gpustat_15` foi renomeado para `gp1_09h_fecha_gate_nao_limpa_gpustat_15_latch` com asserção corrigida (de 0 para 1).

**Hardware — `gpu/gp0-e1`:** **10p/0f** (medido via scoreboard, commit d2aab49). Antes da correção: 9p/1f (PR #88). O subteste `testUnsetAllowTextureDisablePreservesBit` era a falha — esperava GPUSTAT.15=1 após fechar o gate, mas o `if bit == 0` limpava ativamente.

## Revisão cruzada (orquestrador)

<!-- Preenchido pelo orquestrador. -->

## Decisões e notas

1. O handler de GP1(09h) agora só armazena `allow_upper_y`: fecha o gate do decodificador de
   endereço de Y, sem tocar em GPUSTAT.15. O latch é limpo apenas pela máscara de E1h e
   `apply_texpage_if_second` na próxima escrita com o gate fechado.
2. O doc da 0074 foi corrigido: a nota 1 de "Decisões e notas" agora reflete o comportamento
   correto (fechar o gate não limpa o latch), e a linha do m2 na tabela da bateria aponta
   para o mutante novo.
3. O manifesto 0050-display-regs.mut (controle K2) teve a âncora reparada: o bloco `0x09 =>`
   foi encurtado (remoção do `if bit == 0`), e tanto `@@DE` quanto `@@PARA` refletem o novo
   código.
