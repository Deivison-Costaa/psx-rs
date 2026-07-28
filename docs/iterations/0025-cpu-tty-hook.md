<!-- Custo, tokens e duração NÃO entram aqui: são medidos por scripts/oc-iter.ps1 em
     docs/metricas.csv, casados por head_antes/head_depois. -->

# 0025 — cpu-tty-hook

- **Data:** 2026-07-28
- **Item do roadmap:** 1.10
- **Objetivo:** hook de TTY (A0h/B0h) observando jal para putchar/puts e acumulando bytes no Bus.

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| psx-spx | A-Functions (§ L496), Parameters/Registers/Stack (§ L481) | docs/reference/13-kernel-bios.md |
| psx-spx | A(3Ch)/B(3Dh) putchar (§ L2776) | docs/reference/13-kernel-bios.md |
| psx-spx | A(3Eh)/B(3Fh) puts (§ L2742) | docs/reference/13-kernel-bios.md |

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | puts aceitava R9=3Eh via B0h e R9=3Fh via A0h | wildcard `(0x3E, _) \| (0x3F, _)` bastava | spec estrita: puts = A(3Eh) ou B(3Fh); cruzado (3Eh via B0h, 3Fh via A0h) são gets/printf | revisão adversarial F1 — teste `puts_b0h_com_numero_de_a0h_ignorado` pego pela ausência do mutante |
| 2 | hook lia registradores ignorando load delay pendente | `self.regs[4]`/`self.regs[9]` no topo do step | idioma padrão MIPS: lw no delay slot do jal → R4 assentado pelo load quando o hook lê | revisão adversarial F2 — teste `putchar_com_lw_no_delay_slot_do_jal` |

A implementação seguiu o handoff corrigido (iter 0024) — endereços de chamada, não códigos de
syscall. As armadilhas do STATUS estavam todas
cobertas: máscara de endereço físico, puts(0) = `<NULL>`, teto de 1 MiB no puts, e o byte cru
do putchar sem expansão TAB/LF.

## Bateria de mutação

6/6 mutantes pegos, 2/2 controles verdes.

| Mutação | Teste que pegou |
|---|---|
| M1: trocar `phys == 0xA0 \|\| phys == 0xB0` por só `phys == 0xB0` | D1 (putchar_por_a0h), D3 (puts_le_ate_zero), D4 (puts_null_emite_texto_null), D6 (espelho_kseg0_dispara_hook) |
| M2: usar `instr_pc` em vez de `phys` na condição e no match | D6 (espelho_kseg0_dispara_hook): 0x800000A0 ≠ 0xA0 |
| M3: ler `self.regs[8]` em vez de `self.regs[9]` | D1, D2a, D3, D4, D6 — R9 não lido |
| M4: remover `if src == 0` do puts | D4 (puts_null_emite_texto_null): leu byte 0x28 de RAM em vez de emitir `<NULL>` |
| M5: inverter puts `(0x3E, 0xB0) \| (0x3F, 0xA0)` | D3, D4: 0x3E via A0h não reconhecido |
| M6: wildcard `(0x3E, _) \| (0x3F, _)` em vez de estrito | F1 (puts_b0h_com_numero_de_a0h_ignorado): 3Eh via B0h seria gets mas wildcard trata como puts |
| **C1:** renomear `fn_idx` → `f` | Todos verdes |
| **C2:** reordenar match arms (puts antes de putchar) | Todos verdes |

## Placar antes → depois

Workspace: **212 → 221** testes (9 novos na 0025). A base de 209 no STATUS estava errada: `bus_scratchpad_isc` tinha 9 testes desde a 2ª rodada da 0022, mas o STATUS mostrava 6. Corrigido na revisão adversarial (F3).

## Decisões e notas

1. **putchar grava byte cru, sem expansão TAB/LF.** O putchar real da BIOS expande TAB → espaços
   e LF → CR+LF. Nosso hook observa sem substituir a função; o byte cru é o que o código gravou
   em R4. Decisão deliberada, registrada no STATUS (armadilha 5). Ponto de resolução: comparar
   com a saída de uma BIOS real quando o runner existir.

2. **puts(0) = `<NULL>` implementado conforme spec (L2746-2749).** Seis caracteres sem CR/LF.

3. **Teto de 1 MiB no puts.** Sem terminador 00h, o loop para após 1.048.576 bytes. A spec não
   especifica teto; adotamos 1 MiB por segurança contra loops infinitos.

4. **Correção pós-revisão — puts estrito + load delay.** O wildcard `(0x3E, _) | (0x3F, _)`
   foi substituído por `(0x3E, 0xA0) | (0x3F, 0xB0)` conforme a spec (F1). O hook passou a
   consultar `reg_with_pending` em vez de `regs[i]` para R4/R9, resolvendo o caso de `lw`
   no delay slot do `jal` (F2). Dois novos testes cobrem ambas as correções.
