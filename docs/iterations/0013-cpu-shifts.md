<!-- Custo, tokens e duração NÃO entram aqui: são medidos por scripts/oc-iter.ps1 em
     docs/metricas.csv, casados por head_antes/head_depois. -->

# 0013 — cpu-shifts

- **Data:** 2026-07-27
- **Item do roadmap:** 1.3b
- **Objetivo:** implementar SLL/SRL/SRA (shift-imm com campo `sa`) e SLLV/SRLV/SRAV (shift-reg com quantidade em `rs & 0x1F`).

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| psx-spx | shifting instructions (L396), encoding (L184-185) | docs/reference/02-cpu.md |

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | nenhum | — | — | Acertou de primeira no código, mas o teste `opcode_desconhecido_especial_panics` (cpu_alu.rs) usava secondary=0x00 que agora é SLL válido e parou de panicar. |

## Bateria de mutação

**Placar: 3/3 mutantes pegos, 2/2 controles verdes.**

| Mutação | Teste que pegou |
|---|---|
| SRA usa `>>` lógico em vez de aritmético (`(self.reg(rt) >> sa)`) | `sra_aritmetico_propaga_sinal` |
| SRAV usa `>>` lógico em vez de aritmético | `srav_aritmetico_propaga_sinal` |
| Variante V sem máscara `& 0x1F` no shift | `sllv_shift_amount_mascara_0x1f` (panic de overflow) |
| Controle: renomear `shift` → `amt` | verde |
| Controle: reordenar casos do match | verde |

## Placar antes → depois

Workspace: **67 → 75** testes (8 meta + 8 bus_bios + 2 bios_flag + 1 version + 11 bus_scheduler + 8 cpu_fetch_decode + 26 cpu_alu + **14 cpu_shifts** + 3 psx-cli/desktop).

## Revisão cruzada (orquestrador)

**Os seis shifts estão corretos.** SRL é lógico e SRA é aritmético com a precedência certa
(`(x as i32) >> n`, porque `as` liga mais forte que `>>`); as variantes V mascaram a
quantidade com `& 0x1F`, e o teste usa `rs = 0x8000_0004` para provar que só os 5 bits
baixos contam — mutação boa, dessas que pegam o erro de quem copia `shift = rs`. O NOP
canônico (`sll $0,$0,0`) continua inócuo. Nenhum achado de emulação.

Os dois achados são de teste, ambos corrigidos nesta branch:

### Achado 1 — SEVERIDADE MÉDIA — teste declara que JR é opcode inválido

`cpu_alu.rs:opcode_desconhecido_especial_panics` foi reapontado de secondary `0x00` para
`0x08` porque 0x00 virou SLL nesta iteração. Mas **0x08 é JR** — instrução real, que chega
no item 1.5 (`docs/reference/02-cpu.md`, tabela `Secondary opcode field`, linha 169:
`08h=JR`). O teste passa hoje por acidente: JR ainda não existe. Quando o 1.5 o
implementar, o teste falha, e o risco não é a falha — é alguém "consertar" apagando a
asserção e o projeto perder a guarda de opcode desconhecido.

Trocado por `0x3F`, que a mesma tabela marca `N/A` e que nunca vira instrução no R3000A.
Escolher um slot reservado de verdade custa o mesmo e não expira.

### Achado 2 — SEVERIDADE BAIXA — nome de teste contradiz o que o teste faz

`sll_shift_32_vira_0` monta `sa = 0` e afirma `1 << 0 = 1`. Nada nele testa shift de 32 —
que aliás não é codificável, já que `sa` tem 5 bits. Nome errado em teste é pior que
comentário errado: é o que o próximo agente lê para decidir se o caso já está coberto.
Renomeado para `sll_com_sa_zero_e_identidade`.

### Achado 3 — SEVERIDADE BAIXA — na minha própria automação (iter 0011b)

A remediação de checkbox do `oc-loop.ps1` casava `\(ROADMAP (\d+\.\d+)\)`, que não aceita
item com sufixo de letra: para o título desta iteração, `(ROADMAP 1.3b)`, ela não casava e
pulava a remediação silenciosamente. Falha segura (não marcou nada errado) e sem efeito
aqui, porque o trabalhador marcou o checkbox sozinho — mas o ROADMAP prevê sufixos por
convenção. Regex corrigida para `(\d+\.\d+[a-z]?)`.

## Decisões e notas

- `sll $0,$0,0` (secondary=0x00, rd=0, rt=0, sa=0) é o NOP canônico — `set_reg` já ignora R0.
- O teste `opcode_desconhecido_especial_panics` da ALU precisou ser atualizado de secondary 0x00 → 0x08 porque 0x00 agora é SLL implementado.
