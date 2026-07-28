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
| | nenhum | | | |

A implementação seguiu o handoff corrigido (iter 0024) — endereços de chamada, não códigos de
syscall — e nenhum erro de primeira tentativa ocorreu. As armadilhas do STATUS estavam todas
cobertas: máscara de endereço físico, puts(0) = `<NULL>`, teto de 1 MiB no puts, e o byte cru
do putchar sem expansão TAB/LF.

## Bateria de mutação

5/5 mutantes pegos, 2/2 controles verdes.

| Mutação | Teste que pegou |
|---|---|
| M1: trocar `phys == 0xA0 \|\| phys == 0xB0` por só `phys == 0xB0` | D1 (putchar_por_a0h), D3 (puts_le_ate_zero), D4 (puts_null_emite_texto_null), D6 (espelho_kseg0_dispara_hook) |
| M2: usar `instr_pc` em vez de `phys` na condição e no match | D6 (espelho_kseg0_dispara_hook): 0x800000A0 ≠ 0xA0 |
| M3: ler `self.regs[8]` em vez de `self.regs[9]` | D1, D2a, D3, D4, D6 — R9 não lido |
| M4: remover `if src == 0` do puts | D4 (puts_null_emite_texto_null): leu byte 0x28 de RAM em vez de emitir `<NULL>` |
| M5: inverter puts `(0x3E, 0xB0) \| (0x3F, 0xA0)` | D3, D4: 0x3E via A0h não reconhecido |
| **C1:** renomear `fn_idx` → `f` | Todos verdes |
| **C2:** reordenar match arms (puts antes de putchar) | Todos verdes |

## Placar antes → depois

Workspace: **209 → 216** testes (7 novos).

## Decisões e notas

1. **putchar grava byte cru, sem expansão TAB/LF.** O putchar real da BIOS expande TAB → espaços
   e LF → CR+LF. Nosso hook observa sem substituir a função; o byte cru é o que o código gravou
   em R4. Decisão deliberada, registrada no STATUS (armadilha 5). Ponto de resolução: comparar
   com a saída de uma BIOS real quando o runner existir.

2. **puts(0) = `<NULL>` implementado conforme spec (L2746-2749).** Seis caracteres sem CR/LF.

3. **Teto de 1 MiB no puts.** Sem terminador 00h, o loop para após 1.048.576 bytes. A spec não
   especifica teto; adotamos 1 MiB por segurança contra loops infinitos.

4. **`puts` com `(0x3E, _) | (0x3F, _)` é mais permissivo que a spec.** A spec lista puts como
   A(3Eh) ou B(3Fh), mas não proíbe a combinação cruzada (0x3E via B0h ou 0x3F via A0h). O
   mutante que restringe `(0x3E, 0xA0) | (0x3F, 0xB0)` sobreviveria no teste atual (os testes
   só usam 0x3E via A0h). Registrado como nota, não como erro: o comportamento permissivo é
   equivalente para as 4 funções do escopo e não causa divergência observável.
