# 0019 — 1.8 dividido em 1.8a/1.8b, com testes de aceitação derivados duas vezes

- **Data:** 2026-07-27
- **Item do roadmap:** 0.8 (orquestração; fora da escada do M1)
- **Objetivo:** Fechar o ciclo do 1.7 no registro e preparar o handoff do 1.8.
- **Autor:** orquestrador (Claude). Sem código de emulador.

## Por que dividir o 1.8

O item, como estava no ROADMAP, juntava cinco coisas: banco de registradores do COP0, três
opcodes de move, `RFE`, o mecanismo de entrada em exceção e cinco causas distintas
(overflow, syscall, break, AdEL, AdES) mais o bit BD.

A evidência de que isso era demais veio da 1.7: quatro opcodes de **uma família só** custaram
US$ 0,16, 23min e 106 steps, e ainda assim o PR chegou com bateria de mutação irreproduzível,
um teste de aceitação obrigatório ausente e uma lacuna de cobertura. R4 manda uma
micro-funcionalidade por iteração. Despachar o 1.8 inteiro seria repetir a aposta que acabou
de sair cara duas vezes seguidas.

- **1.8a** — registradores `r12`/`r13`/`r14`/`r8`/`r15`, `MFC0`, `MTC0`, `RFE`. Sem exceções.
- **1.8b** — mecanismo de exceção, as cinco causas, o bit BD, e o fechamento das dívidas 2 e 5
  das Notas do STATUS.

Decisão do orquestrador; o ROADMAP já previa sub-letras para trabalho fora da escada.

## Testes de aceitação do 1.8a (derivados duas vezes)

Regra da 0017e: todo literal que o orquestrador impõe é derivado por dois caminhos
independentes, e o handoff carrega a derivação para que o trabalhador possa reprovar o
orquestrador.

| Teste | Literal | Rota 1 | Rota 2 |
|---|---|---|---|
| A1 `rfe` | `SR = 0x34` → **`0x3D`** | bit a bit: b0←b2=1, b1←b3=0, b2←b4=1, b3←b5=1, b4-b5 inalterados = `111101` | por campos: bits 3:2=`01`→1:0; bits 5:4=`11`→3:2; 5:4 seguem `11`; `11 11 01` |
| A2 `mtc0` em CAUSE | `0x20` + `0xFFFF_FFFF` → **`0x320`** | só bits 8-9 são R/W (título da seção) | `(0x20 & !0x300) \| (0xFFFF_FFFF & 0x300)` |

O literal do A1 foi escolhido por ser **assimétrico**: os dois erros prováveis (shift do campo
de 6 bits; copiar 4-5 para 2-3 zerando 4-5) dão ambos `0x0D`, e inverter a ordem das cópias dá
`0x3F`. Nenhum passa por acidente — que era o problema do `0xAAFF_FFFF` da 0017, satisfeito
pelo modelo mental errado.

Literais do 1.8b já derivados e guardados no handoff futuro: `syscall` → `CAUSE = 0x20` e vetor
`0x8000_0080`; `break` → `CAUSE = 0x24` e vetor **`0x8000_0040`** (vetor próprio, é o erro
clássico do item); overflow → `CAUSE = 0x30` com `rd` intacto; `lw`/`sw` desalinhado →
`CAUSE = 0x10`/`0x14` com `BadVaddr` escrito; `syscall` em delay slot → `CAUSE = 0x8000_0020`
com `EPC` apontando para o **branch**.

## Armadilha na direção contrária

Nem toda armadilha é "faltou implementar". `MFC0` tem load delay de um opcode, mas `MTC0` em
COP0 **não** tem store delay — a spec é explícita ("one can read from a cop0 register
immediately after writing to it"), e derruba de passagem o boato de que o load delay de
coprocessador seriam dois opcodes. Sem isso nomeado, a simetria levaria o trabalhador a
implementar um delay que o hardware não tem. Handoff que só avisa do que falta implementa
metade do risco.

## Regra nova da bateria de mutação

**Helper compartilhado por N pontos de chamada rende N mutantes independentes; mutar a
definição testa 1 deles.** Origem empírica na 0018: `reg_with_pending` trocado por `reg`
**apenas dentro de `fn lwl`** não quebrava nenhum dos 25 testes, porque todo teste do idioma
dependia do LWR. A bateria original mutou o helper inteiro, pegou o mutante, e reportou
cobertura que não existia. Passa a valer para todo item com método auxiliar compartilhado.

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que era | Como foi pego |
|---|---|---|---|---|
| 1 | processo | Que a citação `L235` do meu doc da 0017 estava certa e a `L240` do PR #32 errada | `L235` é a seção-pai `Load/Store Alignment`; `L240` é `Unaligned Load/Store` | Abri o índice de `02-cpu.md` para preparar o handoff do 1.8 |
| 2 | processo | Que o mutante 5 da tabela do trabalhador escapava | O mutante descrito (helper inteiro) é pego; o que escapava era o meu, só no ponto de chamada do `lwl` | Reli a descrição da tabela ao redigir a revisão cruzada |

Ambos retratados no próprio PR #32, antes do merge.

## Bateria de mutação

Não se aplica (handoff, ROADMAP e docs de processo). O controle equivalente é a **dupla
derivação** de cada literal, tabelada acima.

## Placar antes → depois

178 testes → **178**.
