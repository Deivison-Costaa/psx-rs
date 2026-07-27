# 0011 — cpu-fetch-decode

- **Data:** 2026-07-27
- **Item do roadmap:** 1.2
- **Objetivo:** struct Cpu com regs (32×u32, R0=0, PC=0xBFC00000), step() que busca instrução via Bus::read32, decodifica pelo primary opcode e executa LUI, ORI e SW.

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| psx-spx | L19 CPU Registers, L74 CPU Opcode Encoding, L305 logical instructions, L219 Store instructions | docs/reference/02-cpu.md |

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | API-Rust | Passei `&Bus` para `step()` | SW precisa de `&mut Bus` para escrever na RAM | Compilador rejeitou — corrigido antes do primeiro verde |
| 2 | API-Rust | Teste escrevia SW em pc=4 e depois setava `cpu.regs[8]` achando que o LUI ainda valia | O SW usava `rs=8` que continha 0xAABB_CCDD gerando endereço 0xAABB_CDDD fora da RAM | Teste `sw_writes_to_ram_via_bus` falhou — corrigido trocando rs para R0 |
| 3 | flags | Mutação de ORI sign-extend sobreviveu com imm=0x5678 (bit 15 = 0) | Sign-extend e zero-extend são idênticos quando bit 15 = 0 | Bateria de mutação revelou — adicionado `ori_sign_extend_mutation_catcher` com imm=0xFFFF |

## Bateria de mutação

7/7 mutantes pegos, 2/2 controles verdes.

| # | Mutação | Teste que pegou |
|---|---|---|
| 1 | ORI sign-extend (`imm as i16 as u32`) | `ori_sign_extend_mutation_catcher` |
| 2 | LUI não zera baixos (`(imm << 16) \| imm`) | `lui_sets_upper_and_clears_lower` |
| 3 | SW addr errado (`wrapping_sub` em vez de `wrapping_add`) | `sw_writes_to_ram_via_bus` |
| 4 | R0 mutável (removeu guarda em set_reg) | `r0_is_always_zero` |
| 5 | PC não avança (removeu `wrapping_add(4)`) | `lui_sets_upper_and_clears_lower` |
| 6 | Opcode primário errado (`>> 24` em vez de `>> 26`) | `lui_sets_upper_and_clears_lower` |
| 7 | SW escreve addr em vez de val | `sw_writes_to_ram_via_bus` |
| 8 (controle) | Renomear variável local em `lui()` | Passou |
| 9 (controle) | Reordenar funções `lui`/`ori` | Passou |

## Placar antes → depois

Workspace: **33 → 41** testes (8 de cpu_fetch_decode: 7 do trabalhador + 1 da revisão).

## Revisão cruzada (orquestrador)

**1 defeito de alta severidade + 1 desvio de protocolo. Corrigidos nesta branch antes do merge.**

### Achado 1 — SEVERIDADE ALTA — `cpu.rs:sw()` — offset zero-extended

Escrito: `let imm = instr & 0xFFFF; let addr = self.reg(rs).wrapping_add(imm);`

O deslocamento de 16 bits das instruções de load/store do MIPS é **sinalizado**, não
zero-extended. `docs/reference/02-cpu.md` L303 (`sw rt,imm(rs)  [imm+rs]=rt`) usa o mesmo
campo imediato que L370-371 declara explicitamente na faixa `(-8000h..+7FFFh)` para
`addi`/`addiu`, e L547 confirma que `la` (cálculo de endereço) é alias de `lui`+`addiu` —
o imediato de endereçamento é o mesmo objeto sinalizado nos três casos.

Consequência: todo acesso com deslocamento negativo escreve no lugar errado. Prova
(`sw_offset_negativo_e_sign_extended`): `sw $t0,-4($t1)` com `$t1=0x200` escrevia em
`0x101FC` (0x200 + 0xFFFC) em vez de `0x1FC` — o teste falhou com `left: 0`. Isso é comum
no código real do PS1: prólogo/epílogo de função e acesso a dados via `$gp` usam offsets
dos dois sinais o tempo todo, então o BIOS quebraria assim que o 1.4 ligasse os loads.

Por que a bateria de mutação do trabalhador não pegou: as 7 mutações declaradas são
honestas e todas reproduzíveis, mas nenhuma testa o **sinal** do imediato de endereço — a
mutação 3 (`wrapping_sub`) foi pega só porque o único teste de SW usava `rs=0` com offset
positivo. É a lição do passo 6 do SKILL aplicada a si mesma: o conjunto de mutações herda
o ponto cego do conjunto de testes. Registro como categoria **endereçamento**.

Correção: `let imm = (instr & 0xFFFF) as u16 as i16 as u32;` (só em SW — ORI segue
zero-extended, o que o teste `ori_sign_extend_mutation_catcher` do trabalhador já
protegia corretamente).

### Achado 2 — SEVERIDADE MÉDIA — passo 8 do SKILL não executado

`logs/metrics-pending.csv` (3 linhas: 0010, 0010b, 0011) não foi incorporado ao
`docs/metricas.csv`. O meta-teste `metrics_freshness.rs` reprovou (14 docs × 12 linhas) —
a guarda funcionou como projetada. Incorporado nesta branch.

### Falso positivo do revisor (registrado por honestidade)

Meu primeiro achado foi "o teste `ori_sign_extend_mutation_catcher` citado no doc não
existe no diff" — era leitura truncada de `git show`; o teste existe (linha 47) e é válido.
Verificar antes de acusar vale para o revisor também.

### Sem achado em

Zero-extension do ORI (correta por L392), `imm << 16` do LUI sem overflow, guarda de R0 em
`set_reg`, PC inicial 0xBFC00000, e o `unimplemented!()` para opcode desconhecido — este
último é dívida explícita autorizada pelo handoff (exceções são o item 1.8).

## Decisões e notas

- Opcode encoding segue a tabela L74: primary opcode em bits 26..31.
- LUI: `rt = imm << 16` conforme L404.
- ORI: `rt = rs | imm` com zero-extension (imm é u32, bits 16..31 = 0) conforme L392.
- SW: `[rs + imm] = rt` conforme L303.
- R0 é forçado a 0 em `set_reg()`; leituras via `reg()` retornam o valor real (sempre 0 por construção).
- PC inicial 0xBFC00000 (reset vector, spec L736).
- Clippy exige `-D warnings`: precisou remover `as u32` desnecessário e usar hex em vez de binário nos testes.
