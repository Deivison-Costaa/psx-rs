# 0084 — gte-registers

- **Data:** 2026-07-30
- **Item do roadmap:** 5.1
- **Objetivo:** Implementar registradores GTE (cop2r0-63) e instrucoes MFC2/MTC2/CFC2/CTC2/LWC2/SWC2.

## Revisão do PR anterior

Revisão do PR #98 (iter 0083): achado defect de regressão em `find_in_index`.

Defect encontrado: busca por título curto ("Opcode/Parameter Encoding") casa indevidamente com o título mais longo ("Coprocessor Opcode/Parameter Encoding") em vez do exato, porque `find_in_index` privilegiava sempre o mais longo. Corrigido com preferência por casamento exato (`normalize_title(e.title) == s`) antes do desempate por comprimento. O mesmo guard foi adicionado a `index_match_ambiguity` para evitar reportar ambiguidade quando há casamento exato.

Nove padrões conferidos:
1. Teste que não mede — achado: teste `find_in_index_prefere_titulo_mais_longo_nao_substring` só exercita o termo mais longo; adicionado teste reverso `find_in_index_prefere_casamento_exato_sobre_mais_longo`
2. Parâmetro não consumido — sem novos comandos GPU
3. Regra de borda trocada — sem rasterização
4. Campo de bit lido errado — sem novos registradores
5. Panic ou laço ilimitado — sem unwrap/unsafe fora de teste
6. Citação de spec — `confere-citacoes.ps1` verde
7. Escopo transbordado — 0083 implementou só o item 10.16
8. Portão — âncora c2 do manifesto 0083 quebrou pela inserção de código; reparada em vez de arquivada
9. Manifesto arquivado — sem arquivamentos; c2 reparado

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| psx-spx | GTE Load Delay Slots (L101), GTE Command Encoding (L117), Data Register Summary cop2r0-31 (L137), Control Register Summary cop2r32-63 (L156), GTE Saturation (L341) | docs/reference/07-gte.md |
| psx-spx | Coprocessor Opcode/Parameter Encoding (L207), CPU Coprocessor Opcodes (L501), Coprocessor Instructions COP0..COP3 (L502) | docs/reference/02-cpu.md |

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | registrador-gte | rd=3 (cop2r35) seria standalone 16-bit | cop2r35 é packed (RT32+RT31), não standalone; standalone são rd=4(36), 12(44), 20(52), 27(59), 29(61), 30(62) | revendo o índice de registradores: a lista de standalone S16 são os últimos elementos de cada grupo de matriz (RT33, L33, LB3) + DQA + ZSF3 + ZSF4 |
| 2 | registrador-gte | rd=5 (ctrl r5 = TRX) seria 16-bit | TRX (cop2r37) é 32-bit (1bit sign, 31bit integer) — sem sign-extension | teste `ctc2_escreve_registro_de_controle_e_cfc2_le_de_volta` usava `assert_ne!` esperando sign-extension errada; corrigido para `assert_eq!` com valor integral |
| 3 | manifesto-mutacao | c1 (renomear parâmetro) seria cosmético | renomear `rd` → `reg` na assinatura quebra o corpo da função que ainda usa `rd` | `mutantes.ps1` reportou ERRO DE MANIFESTO; c1 alterado para adicionar comentário |

## Bateria de mutação

Placar da bateria: 5/5 mutantes mortos, 2/2 controles verdes, 0 equivalente — docs/mutantes/0084-gte-registers.mut

| # | Tipo | Rótulo | Resultado |
|---|---|---|---|
| m1 | mutante | read_data retorna 0 em vez do valor armazenado | MORREU |
| m2 | mutante | read_control não sign-extende registradores 16-bit | MORREU |
| m3 | mutante | write_data índice deslocado (+1) | MORREU |
| m4 | mutante | is_standalone_s16_control sempre false | MORREU |
| m5 | mutante | is_standalone_s16_control sempre true | MORREU |
| c1 | controle | adiciona comentário antes de read_data | verde |
| c2 | controle | adiciona comentário antes do return de read_control | verde |

## Placar antes → depois

Workspace: **589** → **599** testes (+10: gte_registers).

`confere-citacoes.ps1` permanece verde.

## Decisões e notas

1. **Load delay de MFC2/CFC2 reusa o mecanismo de load delay da CPU.** MFC2/CFC2 retornam `Some((rt, val))` que a CPU trata como load delay de 1 instrução (via `self.load_delay`), igual ao `mfc0`. Spec 07-gte.md L102: "Using CFC2/MFC2 has a delay of 1 instruction until the GPR is loaded."

2. **Sign-extension apenas para registradores standalone 16-bit.** Registradores packed (cop2r32-35, 40-43, 48-51) retornam o valor 32-bit integral. Registradores standalone (cop2r36, 44, 52, 59, 61, 62) sign-extendem o valor de 16 para 32 bits na leitura via CFC2. Spec 07-gte.md L190-191: "Reading the last elements (RT33,L33,LB3) returns the 16bit value sign-expanded to 32bit."

3. **MTC2/CTC2 não disparam saturação.** Spec 07-gte.md L379-381: escrita de 32 bits em registrador 16-bit não dispara flag nem satura. O FLAG permanece zerado após MTC2/CTC2.

4. **COP2 (0x12), LWC2 (0x32) e SWC2 (0x3A) removidos da lista de opcodes CpU** no teste `todos_primarios_cpu_geram_cpu`, pois agora são instruções válidas de COP2.
