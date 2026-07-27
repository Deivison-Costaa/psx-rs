<!-- Custo, tokens e duração NÃO entram aqui: são medidos por scripts/oc-iter.ps1 em
     docs/metricas.csv, casados por head_antes/head_depois. -->

# 0014 — cpu-load-store-delay

- **Data:** 2026-07-27
- **Item do roadmap:** 1.4
- **Objetivo:** implementar LB/LBU/LH/LHU/LW (loads) e SB/SH (stores), mais o load delay slot.

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| psx-spx | Load instructions (L157), Caution - Load Delay (L171), Load Timing (L180), Load Shadow (L201), Store instructions (L299) | docs/reference/02-cpu.md |

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | teste | Endereço de setup no `sb_offset_negativo` era 0x2004 (fora da área alvo) | Deveria ser 0x2000 para SB escrever em 0x2000 | Teste falhou com left=0 — bytes não inicializados. Corrigido na primeira execução. |
| 2 | nenhum | — | — | Load/store semantics e load delay acertados de primeira. Nenhum erro de emulação. |

## Bateria de mutação

**Placar: 6/6 mutantes pegos, 3/3 controles verdes.**

| Mutação | Teste que pegou |
|---|---|
| LB sem sign-extend (`val as u32` em vez de `as i8 as u32`) | `lb_carrega_byte_signed` |
| LH sem sign-extend (`val as u32` em vez de `as i16 as u32`) | `lh_carrega_half_signed` |
| SB usa `write32` em vez de `write8` | `sb_nao_afeta_bytes_vizinhos` |
| SH usa `write32` em vez de `write16` | `sh_nao_afeta_halfword_alto` |
| Load delay ausente — LW escreve direto no registrador | `load_delay_basico` |
| Load delay nunca commitado — `load_delay.take()` removido | `lw_carrega_palavra` |
| Controle: remover guarda `reg != 0` no agendamento (R0 já protegido por set_reg) | verde |
| Controle: renomear variável local | verde |
| Controle: reordenar métodos | verde |

## Placar antes → depois

Workspace: **75 → 95** testes (8 meta + 8 bus_bios + 2 bios_flag + 1 version + 11 bus_scheduler + 8 cpu_fetch_decode + 26 cpu_alu + 14 cpu_shifts + **18 cpu_load_delay** + 3 psx-cli/desktop).

## Revisão cruzada (orquestrador)

**O load delay em si está certo** — que era o risco desta iteração. `Cpu::step` executa a
instrução, só depois faz `load_delay.take()` e escreve o registrador, então a instrução
seguinte ao load lê o valor ANTIGO, exatamente como `02-cpu.md § Caution - Load Delay`
exige. Sign-extension de LB/LH e zero-extension de LBU/LHU corretas; SB/SH não sujam os
bytes vizinhos, com teste dedicado para isso. Primeira iteração com `deepseek-reasoner`
(comparação no fim).

### Achado 1 — SEVERIDADE ALTA — `bus.rs` — `read8`/`read16` ignoram a BIOS ROM

`read32` roteia para a BIOS quando o endereço físico cai em `0x1FC0_0000..+0x80000`, mas
`read8` e `read16`, adicionados nesta iteração, vão direto para `ram_offset()`, que faz
`phys & 0x1F_FFFF`. Resultado: `lb`/`lbu`/`lh`/`lhu` sobre a ROM devolvem lixo da RAM,
silenciosamente. Para `0xBFC0_0000` a máscara dá `0x000000` — lê o byte 0 da RAM.

Prova (`leitura_de_8_e_16_bits_da_bios_nao_cai_na_ram`, em `bus_scheduler.rs`): com `0xAA`
no byte 0 da BIOS e `0x11` no byte 0 da RAM, `read8(0xBFC0_0000)` devolvia `0x11`.

Isso quebraria o item 1.10 (hook de TTY) de um jeito difícil de diagnosticar: o BIOS lê
strings e tabelas da ROM byte a byte, e o sintoma seria texto corrompido, não um crash.
Corrigido extraindo `read_byte(addr)` — uma única rota física, usada por `read8` e
`read16`, alinhada com o que `read32` já fazia.

### Achado 2 — SEVERIDADE MÉDIA — caso não coberto: delay slot escreve o registrador do load

Nenhum teste cobre `lw r10,..` seguido de instrução que escreve `r10`. Nossa implementação
faz o load vencer (ele commita depois da execução). **Não corrigi**: a spec local não
decide a questão — o texto de `Caution - Load Delay` fala de leitura, não de precedência
de escrita — e inverter a ordem por raciocínio de pipeline é exatamente a intuição de MIPS
que a R1 manda desconfiar.

Encaminhamento: o teste `load_delay_vs_escrita_no_mesmo_registrador_comportamento_assumido`
fixa o comportamento atual e diz na própria asserção que é suposição não verificada; a nota
3 do STATUS nomeia o ponto de resolução (Amidog `psxtest_cpu`, item 1.11) e o que fazer se
ele reprovar. Incerteza declarada é melhor que comportamento acidental — no gb-rs esse tipo
de dúvida virava decisão silenciosa que ninguém revisitava.

### Experimento: `deepseek-reasoner` × `deepseek-chat`

| | reasoner (0014) | chat (0011–0013) |
|---|---|---|
| Custo | US$ 0,0402 | US$ 0,0173–0,0202 |
| Tokens in/out | 74.272 / 23.176 | ~36.000 / ~20.000 |
| Duração | 10 min | 4–7 min |
| Erros de 1ª tentativa | 1 (endereço de setup de teste) | 1–3 por iteração |
| Achado de emulação na revisão | 1 (bus, não CPU) | 1 em 3 iterações (SW sign-extend) |

Custa ~2× e demora ~2×. Acertou de primeira a parte conceitualmente difícil — o load delay,
que era justamente o alvo do teste — e o defeito que escapou foi de infraestrutura de
memória, não de semântica de instrução. Uma amostra não decide nada: repetir num item de
armadilha comparável (1.8 COP0/exceções, ou o GTE no M5) antes de concluir qualquer coisa.

## Decisões e notas

- Load delay implementado com campo `load_delay: Option<(usize, u32)>`. A cada `step()`:
  1. Executa a instrução atual (loads retornam `(rt, valor)` sem escrever em regs)
  2. Comita o load delay pendente (escrita atrasada do passo anterior)
  3. Agenda novo load delay se a instrução foi um load
- Stores (SB/SH/SW) não têm delay — executam imediatamente.
- R0 é sempre ignorado: `set_reg` já o protege, e o agendamento também tem guarda `reg != 0`.
- As novas funções `read8`/`read16`/`write8`/`write16` foram adicionadas ao Bus com o mesmo padrão `MemoryOp`.
