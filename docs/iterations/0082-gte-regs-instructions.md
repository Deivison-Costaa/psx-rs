# 0082 — gte-regs-instructions

- **Data:** 2026-07-30
- **Item do roadmap:** 5.1
- **Objetivo:** Implementar registradores GTE (cop2r0-63) e instrucoes MFC2/MTC2/CFC2/CTC2/LWC2/SWC2.

## Revisão do PR anterior

**PR #96 (iter 0081):**

1. **Teste que não mede** — t10 antigo era no-op (eprintln!), corrigido na propria 0081. OK.
2. Parâmetro não consumido — sem novos comandos GPU. OK.
3. Regra de borda — sem rasterizacao. OK.
4. Campo de bit — sem novos registradores. OK.
5. Panic/laço — `frame_cycles()` nunca retorna 0; sem unwrap/unsafe. OK.
6. Citação de spec — `confere-citacoes.ps1` verde. OK.
7. Escopo transbordado — apenas correcao de teste + doc. OK.
8. Portão — sem manifesto 0081 (iteracao foi so correcao de teste). OK.
9. Manifesto arquivado — sem .mut. OK.

**Achado menor:** O t10 nao verifica que vblank comeca ativo em `Bus::new()` (se remover
`enter_vblank()`, o teste ainda passa porque `!vblank_active()` e true no ciclo 0 e o
VBLANK_ENTER do scheduler acende depois). Nao e um defeito bloqueante.

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| psx-spx | GTE Load Delay Slots (L101) | docs/reference/07-gte.md |
| psx-spx | GTE Command Encoding (L117) | docs/reference/07-gte.md |
| psx-spx | Data Register Summary cop2r0-31 (L137) | docs/reference/07-gte.md |
| psx-spx | Control Register Summary cop2r32-63 (L156) | docs/reference/07-gte.md |
| psx-spx | Writing 32bit to 16bit GTE regs (L379) | docs/reference/07-gte.md |
| psx-spx | Coprocessor Opcode/Parameter Encoding (L207) | docs/reference/02-cpu.md |
| psx-spx | Coprocessor Instructions COP0..COP3 (L502) | docs/reference/02-cpu.md |

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | teste-falso | Os testes t6 e t7 de load delay usavam o mesmo registrador como fonte e destino do MFC2/CFC2, e o valor inicial coincidia com o resultado da leitura. O `addiu r9, r8, 0` via r8=0x42 tanto no delay slot quanto depois, mascarando a falta de load delay. | O load delay do MFC2/CFC2 e de 1 instrucao (07-gte.md L101). | Teste falhou com r9=66=0x42 em vez de r9=0. Corrigido separando registrador fonte (r9) do destino (r8) e inicializando r8=0. |
| 2 | manifesto | O formato do manifesto de mutacao foi escrito como comentarios livres, sem o header `formato: 1` e sem os marcadores `@@DE`/`@@PARA`/`@@FIM` com indentacao exata. | — | `mutantes.ps1` rejeitou com "chave desconhecida". Reescrevi no formato canônico. |
| 3 | manifesto | Controle c1 (renomear `co` para `cop_code`) usava ancoras que casavam em cop0_op tambem (2 ocorrencias para `let co =`). | — | `mutantes.ps1` reportou "encontrada 2 vez(es), esperado 1". Corrigido com ancoras unicas a cop2_op. |
| 4 | mutante-sobrevivente | m6 (LWC2 com zero-extend do imediato) sobreviveu porque o teste t9 usava offset=0, indistinguivel entre sign-extend e zero-extend. | LWCn usa imediato sign-extendido de 16 bits (02-cpu.md L207). | m6 sobreviveu na bateria. Adicionado t9b com offset=-8, que pega a diferenca. |

## Bateria de mutação

Placar da bateria: 7/7 mutantes mortos, 2/2 controles verdes, 0 equivalente - ./docs/mutantes/0082-gte-regs-instructions.mut

| Mutante | Descrição | Teste que pegou |
|---|---|---|
| m1 | MFC2 le de ctrl em vez de data | t2 (le valor errado apos MTC2) |
| m2 | CFC2 le de data em vez de ctrl | t3 (le valor errado apos CTC2) |
| m3 | MFC2 sem load delay (None) | t6 (delay slot ve valor novo em vez de antigo) |
| m4 | COP2 enable invertido | t11 (acesso deveria disparar CpU mas nao dispara) |
| m5 | MTC2 dispara FLAG bit 31 | t8 (FLAG deveria ser 0 mas tem bit 31 setado) |
| m6 | LWC2 nao sign-extende imediato | t9b (offset -8 le de endereco errado) |
| m7 | SWC2 le de ctrl em vez de data | t10 (armazena valor errado na memoria) |
| c1 | Binding descartavel antes do check | verde (cosmetico) |
| c2 | Binding descartavel depois do check | verde (cosmetico) |

## Placar antes → depois

Workspace: **~598** testes (570 medidos + ~28 filtrados por bios/feature).

Novo arquivo: `gte_regs_instructions.rs` com 15 testes (t1–t9, t9b, t10–t14).

## Revisão cruzada (orquestrador)

<!-- Preenchido pelo orquestrador. -->

## Decisões e notas

1. **COP2 enable via SR bit 30.** O acesso a COP2 so e permitido com `cop0[12]` bit 30 = 1.
   Quando desabilitado, dispara CpU (exccode=0x0B). O atraso de 2 ciclos documentado na spec
   (02-cpu.md L530-531) nao foi implementado — so afeta o momento em que COP2 fica disponivel
   apos escrever no bit, e sera tratado se necessario.

2. **COP2 commands (co=0x10..=0x1F) sao no-op.** O dispatch de comandos GTE (RTPS, NCLIP, etc.)
   pertence ao item 5.2 e seguintes. Por enquanto, COP2 imm25 e aceito sem excecao e sem efeito
   colateral (t13 confirma).

3. **Load delay de MFC2/CFC2 usa o mesmo mecanismo de load delay da CPU.** Retornar
   `Some((rt, val))` do execute aciona o `load_delay` existente, com 1 instrucao de atraso.
   O LWC2 tambem escreve no GTE sem load delay visivel na CPU (o delay de 2 instrucoes
   documentado na spec e para operacoes GTE internas, relevante a partir do 5.2).

4. **Escrita por software em registrador de 16 bits nao satura nem dispara flag.**
   Conforme 07-gte.md L379-381, MTC2/CTC2 armazenam o valor raw de 32 bits. A saturacao
   so ocorre via comandos GTE (5.2+). Teste t8 verifica que FLAG permanece 0 apos MTC2
   com valor que excede 16 bits.
