# 0144 — kernel-2db8

- **Data:** 2026-08-01
- **Item do roadmap:** 4.5
- **Objetivo:** identificar que funcao do kernel mora em `0x2DB8` e quem a chama — o passo 4 do
  diagnostico que a 0142 iniciou e que faltava para decidir entre espurio ou legitimo.

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| psx-spx | § A-Functions (L176) | docs/reference/13-kernel-bios.md |
| psx-spx | § B-Functions (L365) | docs/reference/13-kernel-bios.md |
| psx-spx | § C-Functions (L468) | docs/reference/13-kernel-bios.md |
| psx-spx | § A(nnh) Jump Table (L188, RAM 0x0200) | docs/reference/13-kernel-bios.md |
| psx-spx | § B(nnh) Function Vector (L176, RAM 0x00B0) | docs/reference/13-kernel-bios.md |
| psx-spx | § C(nnh) Function Vector (L176, RAM 0x00C0) | docs/reference/13-kernel-bios.md |

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a medição mostrou | Como foi pego |
|---|---|---|---|---|
| 1 | enderecamento | Que `0x2DB8` era uma funcao de kernel — uma entrada de tabela A/B/C ou rotina exposta | Nenhuma das tres tabelas (A, B, C) contem `0x2DB8`. E zero; B e C idem. O endereco nao e uma funcao: e o DELAY SLOT (NOP) de `jalr $ra, $t0` em `0x2DB4` | Varredura completa das tabelas A(0x0200), B e C apos 5 M passos de boot; nenhum alvo cai em `0x2DB8` |
| 2 | enderecamento | Que `0x2DB8` era chamado via `jal` direto do jogo (0x800xxxxx) e que uma sonda de `jal` bastava para achar o chamador | Nenhuma instrucao `jal`, `jalr` nem `jr` tem `0x2DB4..0x2DB7` como alvo — o codigo e alcancado por execucao SEQUENCIAL (fall-through de `0x2DB0`). O `$ra=0x00002CDC` na entrada de `0x2DB4` mostra que a funcao CONTENDO o trampolim foi chamada de `0x80004120` — codigo do kernel, nao do jogo | Sondas descartaveis em `jal()`, `jalr()`, `jr()` e `step()` com mascara de alvo; 2300 hits no STEP com zero hits nos metodos de salto |

## Medição

### Varredura de tabelas (5 M passos, sem disco)

A-table (RAM `0x0000_0200`, 0x300 bytes, 8 bytes/entrada como `lui` + `j`/`jr`): 8
entradas parseaveis (0x62, 0x64, 0x66, 0x73, 0x74, 0x7C, 0x80, 0x8D), nenhuma aponta para
`0x2DB8`.

B-table (base dinamica, parseada do dispatch em RAM[0xB0/0xB4]): 63 entradas nao-nulas,
nenhuma = `0x2DB8`. Base em `0x00000874`.

C-table (base dinamica, parseada do dispatch em RAM[0xC0/0xC4]): 30 entradas nao-nulas,
nenhuma = `0x2DB8`. Base em `0x00000674`.

**Conclusao:** `0x2DB8` nao e funcao de kernel.

### Sondas de entrada (360 M passos, com disco Crash)

Sonda de `jal`/`jalr`/`jr` com alvo `0x2DB4..0x2DB7`: **zero hits em 360 M passos.**

Sonda de `step()` em `0x2DB4` e `0x2DB8`: **2300 hits**, todos com o mesmo padrao:

```
PROBE_STEP: pc=0x00002DB4  instr=0x0100F809  ra=0x00002CDC
PROBE_STEP: pc=0x00002DB8  instr=0x00000000  ra=0x00002DBC
```

`0x0100F809` decodifica como `jalr $ra, $t0` (opcode SPECIAL, rs=$t0=8, rd=$ra=31,
funct=JALR=0x09). `0x00000000` e NOP (delay slot). O `$ra=0x00002DBC` prova que e o mesmo
par instrucao/delay-slot medido pela 0142 (`de=00002DB8 → 1FC06FDC ra=00002DBC`).

Sonda de entrada da funcao que contem `0x2DB4` (no prologo em `0x2C94`):

```
PROBE_FUN_ENTRY: pc=0x00002C94  ra=0x00004124  sp=0x801FFBA0  a0=0x00000001
```

**Um unico valor de `$ra` para todos os hits:** `0x00004124` → a funcao sempre e chamada do
mesmo lugar: `0x80004120` (KSEG0, abaixo de `0x80010000` = codigo do kernel, nao do jogo).

### Cadeia de chamada

A funcao comeca em `0x2C94` (prologo: `addiu $sp, -0x28`), chama `C(16h) _cdevscan`
(`jal 0x3E80` em `0x2CD4`), e depois executa o trampolim:

```
0x2DAC: lw   $t0, 0x18($v1)     ; carrega ponteiro de funcao
0x2DB0: addiu $a1, $zero, 2     ; argumento
0x2DB4: jalr $ra, $t0           ; chama via ponteiro (linka $ra=0x2DBC)
0x2DB8: nop                     ; delay slot
```

O que muda entre as chamadas e o valor de `$t0`: aos 91k passos aponta para codigo do
kernel; aos 354 M (medido pela 0142) aponta para `BFC06FDC` no BIOS, que leva a
`SysInitMemory`.

## Bateria de mutação

Placar manual: 5/5 mutantes mortos, 2/2 controles verdes.  Manifesto em
`docs/mutantes/0144-kernel-2db8.mut`.  Alvo em `crates/psx-cli/src/main.rs`; o script
`mutantes.ps1` nao cobre psx-cli (invariante 29), bateria aplicada manualmente.

m1 (remove `ra($31)` do format): morto — teste `trace_pcs_inclui_ra_do_chamador` falha.
m2 (`cpu.regs[31]` → `cpu.regs[30]`): morto — saida tem valor errado para `$ra`.
m3 (label `ra($31)` → `ra($0)`): morto — assert procura `ra($31)` no stderr.
m4 (formato decimal): morto — `ra($31)={}` nao casa com `ra($31)=0x`.
m5 (troca ordem arg13/arg31): morto — valor de `$ra` troca com `$t5`.

## Placar antes → depois

Workspace: **870** → **872** testes (2 novos em `kernel_funcao_2db8`).

## Revisão cruzada (orquestrador)

<!-- Preenchido na revisão do PR. -->

## Decisões e notas

**1. `0x2DB8` nao e uma funcao, e um delay slot.** A 0142 mediu corretamente a transicao
`0x2DB8 → BFC06FDC` mas nao identificou que `0x2DB8` e o NOP de `jalr $ra, $t0` em `0x2DB4`.
A "funcao" cujo chamador procuravamos e um trampolim: `lw $t0, 0x18($v1); jalr $ra, $t0`.
O vies de nomenclatura ("funcao") atrasou o diagnostico em uma iteracao.

**2. O trampolim so e alcancado por fall-through.** Nenhuma instrucao de salto (jal, jalr,
j, jr) tem `0x2DB4..0x2DB7` como alvo. O codigo flui sequencialmente de `0x2DB0` para
`0x2DB4`. A sonda de jal com mascara larga (`phys & 0xFFFF_FFFC == 0x2DB4`) nao achou nada
em 360 M passos — a hipotese de que o jogo chama `0x2DB8` via `jal` esta refutada.

**3. Quem chama a funcao que CONTEM o trampolim e o kernel, nao o jogo.** `$ra=0x00004124`
(primeira instrucao em `0x2C94`) aponta para `0x80004120`, dentro da regiao de codigo do
kernel (< 0x80010000). O jogo nao esta envolvido diretamente nesta cadeia de chamada.

**4. O que falta para fechar o 4.5.** O defeito nao e "quem chama o trampolim" — e "o que
poe `0xBFC06FDC` em `mem[$v1+0x18]`". O trampolim e inocuo: chama o endereco que
`$v1+0x18` aponta. Na primeira execucao do boot, esse ponteiro leva a funcoes normais do
kernel; na segunda execucao (aos 354 M), leva a `BFC06FDC → SysInitMemory`. O passo 5 (proxima
iteracao) e rastrear **quem escreve** nesse slot da RAM entre os dois boots.

**5. O `--trace-pcs` agora inclui `ra($31)`.** Mudanca permanente em `main.rs` (linha 78):
o trace de PC diagnosticado passa a exibir o registrador `$31` (return address), essencial
para rastrear cadeias de chamada. O teste `trace_pcs_inclui_ra_do_chamador` em
`kernel_funcao_2db8.rs` valida o formato.
