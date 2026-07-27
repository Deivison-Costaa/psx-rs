# 0012 — cpu-alu

- **Data:** 2026-07-27
- **Item do roadmap:** 1.3
- **Objetivo:** Implementar as instruções aritméticas, lógicas e de comparação da ALU do R3000A, tanto no formato SPECIAL (registrador-registrador) quanto ALU-imediato.

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| psx-spx | § CPU ALU Opcodes: arithmetic, comparison, logical | `docs/reference/02-cpu.md` |
| psx-spx | § Opcode/Parameter Encoding: SPECIAL, alu-imm | `docs/reference/02-cpu.md` |
| psx-spx | § Primary opcode field / Secondary opcode field | `docs/reference/02-cpu.md` |

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | API-Rust | `encode_alu_imm` helper com rs/rt trocados no shift (`rs << 16`, `rt << 21`) | bits 25..21=rs, 20..16=rt | Teste falhou: addiu, addi, andi, xori, slti com resultado 0 |
| 2 | flags | SLTIU com imm=0x8000: esperava que 0 < 0xFFFF_8000 unsigned desse 0 (achei que imm negativo não coubesse na comparação unsigned) | SLTIU compara rs com imm **sign-extended** como unsigned: 0 < 0xFFFF_8000 → true (1) | Teste específico `sltiu_rs_menor_imm_unsigned` falhou |

## Bateria de mutação

Placar: **6/6** mutantes pegos, **2/2** controles verdes.

| Mutação | Teste que pegou | Status |
|---|---|---|
| SUBU: `wrapping_add` em vez de `wrapping_sub` | `subu_basico`, `subu_wraparound` | pego |
| NOR: `!(rs & rt)` em vez de `!(rs \| rt)` | `nor_basico`, `nor_nao_e_or` | pego |
| SLT: unsigned `<` em vez de signed `(i32)` | `slt_rs_menor_rt_signed`, `slt_rs_nao_menor_rt_signed` | pego |
| SLTU: signed em vez de unsigned | `sltu_rs_maior_rt_unsigned`, `sltu_rs_menor_rt_unsigned` | pego |
| `sign_extend_imm` zero-extende | `addiu_sign_extends_imm`, `addi_sign_extends_imm`, `slti_rs_menor_imm_signed`, `sltiu_rs_menor_imm_unsigned` | pego |
| ANDI: sign-extend em vez de zero-extend | `andi_zero_extends` | pego |
| Controle: renomear `val` → `resultado` em ADDU | (passa) | verde |
| Controle: reordenar `xori` antes de `andi` | (passa) | verde |

## Placar antes → depois

- Workspace: **41** testes → **67** testes (41 + 26 novos)
- Meta-testes: todos verdes

## Revisão cruzada (orquestrador)

**Nenhum erro de hardware no código entregue.** As 15 instruções implementadas conferem com
a spec, inclusive as duas armadilhas de sinal: SLTI compara com sinal, e SLTIU
sign-extende o imediato e SÓ ENTÃO compara sem sinal (o teste com `imm=0x8000` prova
que `0 < 0xFFFF_8000` dá 1 — é o caso que quase todo emulador iniciante erra). A bateria
de mutação é honesta: as 6 mutações existem, são plausíveis e cada uma cita o teste que a
pega. O `sign_extend_imm` extraído já nasceu obedecendo a invariante 1 do STATUS, criada
na revisão anterior — a correção da 0011 se propagou como pretendido.

Os dois achados são de **cobertura e registro**, não de emulação:

### Achado 1 — SEVERIDADE MÉDIA — item 1.3 marcado como concluído sem os shifts

O texto do item 1.3 no ROADMAP incluía `shifts`. Não há SLL (secondary 0x00), SRL (0x02),
SRA (0x03) nem as variantes por registrador SLLV/SRLV/SRAV (0x04/0x06/0x07) no `special()`
— caem todas no `unimplemented!`. A causa primária é minha: o handoff que escrevi no STATUS
da 0011 enumerou as instruções do item e **esqueceu os shifts**, e o trabalhador seguiu o
handoff (comportamento correto — o handoff é a fonte da tarefa).

O agravante é de processo: a auto-remediação de checkbox que entrou na iter 0011b marcou
1.3 como concluído porque **confia no título do PR**, que dizia `(ROADMAP 1.3)`. Automação
mecânica não sabe se o item foi cumprido — ela só corrige a omissão de marcar. Conferir
completude do item continua sendo trabalho do revisor, e este PR é a prova de que isso
não é formalidade.

Correção: item `1.3b` criado no ROADMAP com os seis opcodes, e o handoff da próxima tarefa
reapontado para ele (era 1.4). O 1.3 fica marcado com o texto do que de fato entregou.

### Achado 2 — SEVERIDADE BAIXA — nota de decisão contradiz o código

A nota 1 de "Decisões" afirmava que ADD/SUB foram implementados como ADDU/SUBU. Não foram:
não existem no match e dão `unimplemented!`. Quem ficou silenciosamente sem trap de
overflow foi o **ADDI**, que é idêntico ao ADDIU. A assimetria é ruim (ADD grita, ADDI
mente), mas fechá-la agora exigiria o mecanismo de exceção, que é o item 1.8 — então a
dívida virou a nota 2 do STATUS, com os três opcodes nomeados, em vez de ficar só numa
frase de doc que ninguém relê. Nota corrigida no lugar.

## Decisões e notas

1. ~~ADD/SUB (com overflow trap) implementados como ADDU/SUBU por enquanto~~ — **corrigido
   na revisão: isso não é o que o código faz.** ADD (0x20) e SUB (0x22) não estão no match
   e dão `unimplemented!`; quem ficou sem trap foi o ADDI. Dívida registrada na nota 2 do
   STATUS para fechar no item 1.8.
2. `sign_extend_imm` extraída como função auxiliar, reusada por addiu, addi, slti, sltiu e sw.
3. SPECIAL primário (0x00) decodifica pelo secondary opcode. Para já, a implementação cobre os 8 opcodes do item (ADDU, SUBU, AND, OR, XOR, NOR, SLT, SLTU); os demais dão `unimplemented!`.
