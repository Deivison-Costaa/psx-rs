# 0015 — Branches/jumps + branch delay slot

- **Data:** 2026-07-27
- **Item do roadmap:** 1.5
- **Objetivo:** Implementar todos os jumps (J, JAL, JR, JALR) e branches (BEQ, BNE, BLEZ, BGTZ, BLTZ, BGEZ, BLTZAL, BGEZAL) com branch delay slot.

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| psx-spx | CPU Jump Opcodes / jumps and branches + JALR cautions | docs/reference/02-cpu.md L459-L487 |

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | API-Rust | Que o branch target se aplica no mesmo step do branch (testes com 1 step) | O delay slot executa no step SEGUINTE ao branch; o redirecionamento só acontece após o delay slot | Testes falharam com PC=4 em vez do target — corrigi para 2 steps por branch |
| 2 | nenhum | — | — | Nenhum erro de emulação na primeira implementação |
| 3 | nenhum | — | — | Todos os 29 testes passaram de primeira após correção do número de steps |

## Bateria de mutação

| # | Mutação | Teste que pegou | Resultado |
|---|---|---|---|
| 1 | Delay slot ignorado: branch redireciona imediatamente no mesmo step | Quase todos os 18 testes de branch tomado — PC pula 4 a mais que o esperado | Pego |
| 2 | J/JAL sem preservar 4 bits altos do PC | `j_preserva_4_bits_altos_do_pc` — PC = 4 em vez de 0x8000_0004 | Pego |
| 3 | BLEZ sem signed: `self.reg(rs) <= 0` em vez de `(self.reg(rs) as i32) <= 0` | `blez_tomado_negativo` — 0xFFFF_FFFF como u32 é > 0 | Pego |
| 4 | BLTZAL/BGEZAL link só quando tomado (em vez de SEMPRE) | `bltzal_nao_tomado_mas_linka`, `bgezal_nao_tomado_mas_linka`, `bgezal_com_rs_ra_compara_valor_antes_do_link` — $ra mantém valor anterior | Pego |
| 5 | JALR escreve rd antes de ler rs (perde target quando rs=rd) | `jalr_mesmo_reg_rs_rd` — PC = 8 em vez de 0x3000 | Pego |
| 6 | Branch offset sem sign-extend (imm sem sinal) | `bne_tomado` — offset 0xFFFC vira 0x0000FFFC em vez de -4 | Pego |

Controles:
| # | Controle | Resultado |
|---|---|---|
| 1 | Renomear variável local em `jr` (rs → reg_idx) | Verde |
| 2 | Reordenar funções `beq` e `bne` | Verde |

Placar: **6/6 mutantes pegos, 2/2 controles verdes**.

## Placar antes → depois

- Workspace: 98 → **129** testes (29 novos de branch/delay + 2 da revisão cruzada)

## Revisão cruzada (orquestrador)

**A semântica de desvio está certa, incluindo as duas armadilhas que a spec destaca.** O
alvo é `$+4+imm*4`: como `step()` já avançou o PC antes de `execute`, `self.pc` vale o
endereço do delay slot e `branch_taken` soma `imm<<2` a ele — exatamente a fórmula de
`02-cpu.md § jumps and branches`. O link é `$+8`. E os dois avisos explícitos da spec
foram respeitados, cada um com teste: `bltzal/bgezal` gravam `$ra` **mesmo quando o
desvio não é tomado** (`bgezal_nao_tomado_mas_linka`), e quando `rs` é o próprio `$ra` a
comparação usa o valor **anterior** ao link (`bgezal_com_rs_ra_compara_valor_antes_do_link`);
`jalr` guarda o alvo antes de escrever `rd`, então `jalr r31,r31` funciona
(`jalr_mesmo_reg_rs_rd`) — a caution do `§ JALR cautions`. Ordem correta de primeira em
tudo isso, que era o risco do item.

### Achado 1 — SEVERIDADE MÉDIA — `cpu.rs` — pânico de overflow no cálculo do endereço de link

`jal`, `jalr` e `bcondz` calculavam o endereço de retorno com `self.pc + 4`, soma
checada. O resto do arquivo usa `wrapping_add`. Com o PC no fim do espaço de endereços a
soma estoura e o processo **entra em pânico** em build de debug — código convidado
derrubando o host, que é a classe de falha que um emulador não pode ter (o item 1.11 vai
rodar EXEs de teste que sondam endereços de propósito, e o Amidog testa justamente erros
de endereço).

Prova (`jal_no_fim_do_espaco_de_enderecos_nao_estoura`): `JAL` em `0xFFFF_FFF8` abortava
com `attempt to add with overflow` em `cpu.rs:210`. O PC do R3000A é aritmética de 32
bits com wrap — `ra` tem de valer `0x0000_0000`. Corrigido trocando as três somas por
`wrapping_add`; nenhuma mudança de comportamento fora da borda.

Vale notar o contraste: `branch_taken` já usava `wrapping_add` e tem teste que cruza a
borda (`bne` com offset −4 a partir do PC 0, chegando em `0xFFFF_FFF4`). O trabalhador
acertou o wrap onde escreveu um teste para ele e errou onde não escreveu — o mesmo
padrão da 0014 (o load delay, que era o alvo do teste, saiu certo; o defeito escapou no
`bus`, que não era).

### Achado 2 — SEVERIDADE BAIXA — `bcondz` — `rt` fora da tabela vira no-op silencioso

O `match` trata `rt` = 00h/01h/10h/11h e faz `_ => return` para o resto: nem desvia nem
linka. A spec local (`§ Opcode/Parameter Encoding`) só tabela esses quatro valores e não
diz o que os outros fazem, então **não mudei** — inventar o critério de decodificação
por memória de MIPS é exatamente o que a R1 proíbe. Encaminhamento igual ao da nota 3:
teste `bcondz_rt_fora_da_tabela_comportamento_assumido` fixa o que fazemos e declara na
asserção que é suposição; nota 4 do STATUS nomeia o ponto de resolução (item 1.11) e o
critério alternativo a testar primeiro se o psxtest_cpu reprovar.

### Achado 3 — SEVERIDADE MÉDIA — handoff pulou 1.6–1.12 e apontou para a GPU

O STATUS entregue mandava a próxima iteração fazer o item **2.1 (GPU)** com os sete itens
restantes do M1 em aberto — sem MULT/DIV, sem exceções, sem TTY, sem o psxtest_cpu que
resolve as notas 3 e 4. É a terceira reincidência de handoff fora de escopo (a 0010 fundiu
1.2–1.5). Reescrito para o 1.6.

Esse defeito é o que sobrou de **semântico** depois que o loop passou a remediar sozinho
checkbox, métricas e lint: as três remediações mecânicas dispararam nesta iteração
(commits `965257a`, `bed12ed`, `7cc6639`) e nenhuma delas olha para o handoff. Automatizar
o handoff exigiria o script saber qual item vem depois — dá para fazer, lendo o primeiro
`- [ ]` do ROADMAP, e é a próxima remediação candidata.

### Achado 4 — dívida estrutural registrada, não corrigida aqui

`cpu.rs` está em 440 das 500 linhas do teto de `file_size.rs`. MULT/DIV + HI/LO não cabem;
o 1.6 começa fatiando o módulo. Está no handoff, com a divisão sugerida. E `branch_target:
Option<u32>` não guarda "estou num delay slot", que o 1.8 precisa para `CAUSE.BD`/`EPC` —
nota 5 do STATUS.

## Decisões e notas

- O branch delay slot foi implementado via um campo `branch_target: Option<u32>` na CPU.
  O `step()` lê a instrução, aplica `branch_target` se houver (redirecionando o PC),
  executa a instrução, e then comita load delays. Isso garante que a instrução no delay
  slot (lida no PC do step anterior) executa antes do redirecionamento.
- JAL/JALR/BLTZAL/BGEZAL salvam `self.pc.wrapping_add(4)` como link address. Como `self.pc` já foi
  incrementado no início do step, isso resulta em PC_original + 8, que é o endereço de
  retorno após o delay slot (correto).
- Nota 3 do STATUS (load delay vs escrita no mesmo reg) permanece inalterada.
