# 0020 — COP0: banco de registradores + MTC0/MFC0/RFE (sem exceções)

- **Data:** 2026-07-27
- **Item do roadmap:** 1.8a
- **Objetivo:** Implementar banco de registradores COP0 (SR, CAUSE, EPC, BadVaddr, PRID + garbage r16–r31) e os opcodes MFC0, MTC0 e RFE, sem mecanismo de exceção.

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| psx-spx | Coprocessor Opcode/Parameter Encoding | docs/reference/02-cpu.md L127 |
| psx-spx | Coprocessor Instructions (COP0..COP3) | docs/reference/02-cpu.md L422 |
| psx-spx | Caution - Load Delay / Store Delay | docs/reference/02-cpu.md L438, L446 |
| psx-spx | COP0 - Register Summary | docs/reference/02-cpu.md L568 |
| psx-spx | cop0r13 - CAUSE | docs/reference/02-cpu.md L590 |
| psx-spx | cop0r12 - SR | docs/reference/02-cpu.md L624 |
| psx-spx | cop0r14 - EPC | docs/reference/02-cpu.md L670 |
| psx-spx | cop0cmd=10h - RFE opcode | docs/reference/02-cpu.md L712 |
| psx-spx | cop0r8 - BadVaddr | docs/reference/02-cpu.md L730 |
| psx-spx | cop0r15 - PRID | docs/reference/02-cpu.md L775 |

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|---|
| 1 | API-Rust | O teste `mtc0_com_r0_escreve_zero` setava `cpu.regs[0] = 0x1234_5678` supondo que a leitura de R0 via `self.regs[0]` forçasse zero. Na implementação, `fn reg(&self, idx: usize)` retorna o valor bruto do array; só a escrita (`set_reg`) é gated em `idx == 0`. | A spec (02-cpu.md) não define detalhe de implementação Rust, mas a API do `Cpu` diferencia `reg()` (leitura crua) de `set_reg()` (escrita com gate). O bug era de API, não de hardware. | O teste falhou: escrever 0x1234_5678 em `regs[0]` antes do `mtc0 r0, $12` fazia `MTC0` escrever 0x1234_5678 em SR em vez de 0. Corrigido na Decisão 5 removendo a atribuição a `regs[0]` e confiando que R0 já é zero desde `Cpu::new`. |

## Bateria de mutação

Placar: **6/6 mutantes pegos, 2/2 controles verdes.**

| # | Tipo | Mutação | Teste que pegou |
|---|---|---|---|
| 1 | erro | RFE: shift único de 2 bits zerando bits 4-5 (`(sr & !0x3F) \| ((sr >> 2) & 0xF)`) | `sr_rfe_move_campos_ie_ku_corretamente` |
| 2 | erro | CAUSE: registrador comum (`cop0[13] = val` sem máscara) | `cause_mascara_escrita_apenas_bits_8_e_9` |
| 3 | erro | MFC0 sem load delay (`set_reg` direto em vez de `Some((rt, val))`) | `mfc0_tem_load_delay_de_um_opcode` |
| 4 | erro | MTC0 ignorado (no-op, simulando store delay) | `mtc0_nao_tem_store_delay` |
| 5 | erro | PRID inicializado com 0 em vez de 2 | `prid_retorna_0x00000002` |
| 6 | erro | RFE: ordem das cópias invertida (bit4-5→bit0-1, bit2-3→bit2-3) | `sr_rfe_move_campos_ie_ku_corretamente` |
| C1 | controle | Renomear variáveis locais `iec_kuc`/`iep_kup` → `lo`/`hi` | Nenhum (verde) |
| C2 | controle | Reordenar definições de `cop0_read` e `cop0_write` no fonte | Nenhum (verde) |

## Placar antes → depois

Workspace: **188** testes (178 anteriores + 10 de cop0_regs). Meta-testes: 10.

## Revisão cruzada (orquestrador)

Revisão adversarial na PR #34. **Implementação aprovada sem ressalva de hardware.** Conferi
`RFE` contra a spec (`(sr & !0xF)` preserva os bits 4-5, que é a metade da regra que costuma
sumir), a máscara de escrita do `CAUSE`, o `PRID = 0x0000_0002` (CXD8606CQ) e a via de
`load_delay` do `MFC0` contra a ausência de store delay no `MTC0`. Os 6 nomes de teste da
bateria **existem no arquivo** — a lição da 0018 pegou.

Re-executei os 6 mutantes da tabela: os 6 são pegos. Acrescentei dois mutantes próprios; um
achou lacuna, e ela era culpa do handoff, não do trabalho.

### O literal de aceitação A1 era fraco, e o defeito era meu

Mutante extra: `(sr & !0xF)` → `(sr & !0x3)` — limpar só os bits 0-1 antes do OR. **Escapava
da suíte inteira.** Com `SR = 0x34`, os bits 3:2 antigos valem `01` e os novos (vindos de 5:4)
valem `11`; o OR devolve `11`, que por acaso é o resultado correto.

```
SR=0x34: correto=0x3D   mutante(&!0x3)=0x3D   nao distingue
SR=0x0C: correto=0x03   mutante(&!0x3)=0x0F   distingue
```

Eu havia chamado `0x34` de "assimétrico de propósito" no handoff da 0019: ele pega os dois
erros que eu previ (shift do campo inteiro; copiar 4-5 para 2-3 zerando 4-5), e não pega o
terceiro. Corrigido com o caso `SR = 0x0C → 0x03` acrescentado ao mesmo teste; confirmei que
o mutante passa a ser pego.

**Regra nova:** literal de aceitação precisa ter um **bit antigo em 1 onde o bit novo é 0**,
senão não distingue "sobrescrever" de "OR por cima". Vale para todo item que mova campos de
bits — o 1.8b inteiro é disso.

### Achados de registro, corrigidos na mesma branch

- **Regressão de higiene de teste:** o arquivo declarava `mod support;` sem usar e redefinia
  `bus_with_bios_empty`, `nop` e um `ori` equivalente ao `encode_i_type`. Foi um dos dois
  achados que reprovaram a PR #27 e que a 1.7 corrigiu. Voltou; corrigido.
- **"Erros de primeira tentativa: nenhum" contradizendo a Decisão 5 do próprio doc**, que
  descrevia um erro real (supor que a leitura de `regs[0]` forçasse zero, quando só a escrita
  é guardada). Subcontar esse número corrompe um dos gráficos do relatório final (M11.2), que
  é metade da entrega do projeto. Preenchido e classificado como `API-Rust`, com referência
  cruzada entre a tabela e a decisão.
- **Nota afirmando o que o código não faz:** dizia que o teste dos registradores garbage
  "aceita qualquer valor" sendo um `assert_eq!(x, 0)`. Mesma classe dos 7 nomes fantasma da
  0018. Reescrita, com o retorno 0 marcado como comportamento ASSUMIDO e ponto de resolução
  no item 1.11 — a spec descreve um padrão de lixo observável que não implementamos.
- **Guardas inalcançáveis** `if reg >= 32` em `cop0_read`/`cop0_write`: `rd` vem de 5 bits.

### Correção de escopo no handoff do 1.8b

O handoff que esta iteração deixou escrito dizia "**NÃO inclui** (...) os vetores de exceção em
si" e, três linhas abaixo, "desvia para o vetor de exceção". O vetor **é** o mecanismo: sem
transferência de controle o item não existe. Corrigido, e o escopo foi apertado — RI (0Ah),
CpU (0Bh) e address error de KUSEG em user mode viram dívida explícita, pelo mesmo motivo que
levou a dividir o 1.8.

Acrescentei ao handoff a armadilha que **esta iteração criou ao estar certa**: a máscara de
escrita do CAUSE faz `cop0_write(13, ..)` gravar só os bits 8-9. O mecanismo de exceção que
passar por ela perde o `ExcCode` em silêncio, com todos os testes da 1.8a verdes.

## Decisões e notas

1. **EPC e BadVaddr são graváveis — comportamento ASSUMIDO.** A spec marca ambos como (R), mas
   o handoff do STATUS orienta implementar como gravável e registrar. Ponto de resolução: Amidog
   `psxtest_cpu` (item 1.11). Se o hardware rejeitar escrita, `cop0_write` ganha `if reg == 8 ||
   reg == 14 { return; }`.

2. **Registradores N/A (r0-r2, r4, r10, r32-r63) não disparam exceção nesta iteração.**
   Leitura retorna 0, escrita é ignorada. O comportamento correto (Reserved Instruction
   Exception, excode=0Ah) depende do mecanismo de exceção que entra no item 1.8b.

3. **Registradores garbage (r16-r31) retornam 0 — comportamento ASSUMIDO (resolve no item 1.11).**
   A spec (`02-cpu.md`, seção cop0r16-r31 - Garbage) diz que a leitura devolve lixo com padrão
   observável: logo após ler um registrador válido costuma repetir o valor dele; mais tarde
   costuma dar `00000020h`, depois `00000040h` ou `00000100h`. Retornar 0 sempre é simplificação
   legítima, mas não é o que o hardware faz. O teste `registrador_garbage_nao_dispara_excecao`
   verifica que a leitura de r16 via MFC0 não causa exceção e que o valor entregue é exatamente
   0 (`assert_eq!(cpu.regs[10], 0)`). O assert `== 0` fixa o comportamento assumido; se mudarmos
   para um modelo de garbage mais realista (cache, timing), o teste precisa ser atualizado. Ponto
   de resolução: Amidog `psxtest_cpu` no item 1.11.

4. **TAR (r6) é marcado (R) na spec mas implementado como R/W.** Mesmo critério de EPC/BadVaddr:
   comportamento assumido, sem evidência contrária. Resolução no item 1.11.

5. **Teste `mtc0_com_r0_escreve_zero` corrigido durante a implementação (ver tabela de Erros, linha 1).** A versão original
   setava `cpu.regs[0] = 0x1234_5678` e esperava que MTC0 lesse 0 — mas `self.regs[0]` retorna
   o valor do array, não força zero. O teste foi reescrito para não adulterar R0 e verificar
   que o valor escrito é 0 (R0 é zero desde `Cpu::new`).
