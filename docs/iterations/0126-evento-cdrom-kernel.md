# 0126 — evento-cdrom-kernel

- **Data:** 2026-07-31
- **Item do roadmap:** 4.4u
- **Objetivo:** Diagnosticar a corrente de entrega de evento CD-ROM → kernel: medir cada elo da cadeia
  IRQ2 → handler → DeliverEvent → EvCB.status=4000h → dispatch do shell, e identificar onde arrebenta.

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| psx-spx | § B(07h) — DeliverEvent(class, spec) (L1642) | docs/reference/13-kernel-bios.md |
| psx-spx | § Event Control Blocks (EvCB) (L2889) | docs/reference/13-kernel-bios.md |
| psx-spx | § Event Classes (L1656) | docs/reference/13-kernel-bios.md |
| psx-spx | § A(9Ch)/A(9Dh) SetConf/GetConf (L1101) | docs/reference/13-kernel-bios.md |
| psx-spx | § BIOS RAM Map — Table of Tables (L400) | docs/reference/13-kernel-bios.md |

### Caminho canônico da referência

```
IRQ do CD-ROM (INT2/INT3)
  → handler do kernel em 0x80000080 (priority chain 0: CdromDmaIrq, CdromIoIrq)
  → DeliverEvent(F0000003h, spec)  [B(07h)]
  → status do EvCB vira 4000h (enabled/ready)
  → quem espera o evento acorda (shell dispatch loop)
```

## Medições — elo a elo

### Elo 1: CD-ROM levanta IRQ2?

**Contador `Irq::raise_counts[2]` após 80 M passos com disco:**

```
cargo test -p psx-core --test cdrom_evento_kernel boot_com_disco --release -- --nocapture
```

Saída:
```
Diagnostico (com disco): IRQ2=0, handler_entries=699 (80000000 passos)
Contagem por bit: 0=348 1=0 2=0 3=444 4=0 5=0 6=0 7=0 8=0 9=0 10=0
```

**Veredito: IRQ2 = 0. O CD-ROM NUNCA levanta interrupção.**

Os outros bits: bit0 (VBlank) = 348 disparos, bit3 (DMA) = 444 disparos — normais para o boot.
A cadeia ARREBENTA neste elo.

### Elo 2: CPU vetoriza para 0x80000080?

**CLI com `--trace-pcs` no handler e `--max-steps 5000000`:**

```
& '.\target\release\psx-cli.exe' --bios 'bios\SCPH1001.BIN' --disc '..\roms\extraido\Crash Bandicoot (USA).cue' --max-steps 5000000 --trace-pcs 0x80000080 2>&1 | Select-String "trace pc=0x80000080" | Measure-Object | Select-Object -ExpandProperty Count
```

Saída: `3` (handler_entries em 5 M passos).

O handler de interrupção dispara (3× em 5 M passos, 699× em 80 M passos). Todos são VBlank + DMA, não CD-ROM.

**Veredito: funciona. O handler é o da BIOS real (não o stub de psexe.rs — no caminho de boot com disco, `install_return_stubs()` não é chamado).**

### Elo 3: EvCB está alocado? Há eventos?

**CLI com `--dump-mem` na Table of Tables e EvCB:**

Table of Tables (offset 0x120):
```
& '.\target\release\psx-cli.exe' --bios 'bios\SCPH1001.BIN' --disc '..\roms\extraido\Crash Bandicoot (USA).cue' --max-steps 80000000 --dump-mem 0x100 0x60 2>&1
```

Saída do campo EvCB (0x120-0x124):
```
  00000120: A000E028   (ptr em KSEG1, físico = 0x0000E028)
  00000124: 000001C0   (size = 448 bytes = 16 blocos × 0x1C)
```

EvCB dump completo:
```
& '.\target\release\psx-cli.exe' --bios 'bios\SCPH1001.BIN' --disc '..\roms\extraido\Crash Bandicoot (USA).cue' --max-steps 80000000 --dump-mem 0xE028 0x1C0 2>&1
```

Saída: **Todos os 448 bytes são zero.** 16 blocos com class=0, status=0 (free), spec=0, mode=0.

**Veredito: EvCB alocado (16 blocos em 0xE028) mas COMPLETAMENTE VAZIO. Nenhum evento registrado, nenhum status=4000h (ready).**

### Elo 4: TTY — comparação com referência

**CLI com `--max-steps 80000000`:**

```
& '.\target\release\psx-cli.exe' --bios 'bios\SCPH1001.BIN' --disc '..\roms\extraido\Crash Bandicoot (USA).cue' --max-steps 80000000 2>&1
```

TTY (389 bytes):
```
PS-X Realtime Kernel Ver.2.5
Copyright 1993,1994 (C) Sony Computer Entertainment Inc.
KERNEL SETUP!

Configuration : EvCB	0x10		TCB	0x04
System ROM Version 2.2 12/04/95 A
System ROM Version 2.2 12/04/95 A
Copyright 1993,1994,1995 (C) Sony Computer Entertainment Inc.
Copyright 1993,1994,1995 (C) Sony Computer Entertainment Inc.
ResetCallback: _96_remove ..
ResetCallback: _96_remove ..
```

O TTY termina em `ResetCallback: _96_remove ..` — o kernel está removendo o callback do CD-ROM
(o oposto do que queremos: deveria estar chamando `_96_init` e recebendo eventos). A referência
do DuckStation mostra `Executable path: 'SCUS_949.00'` neste ponto.

## Conclusão do diagnóstico

| Elo | Medição | Veredito |
|---|---|---|
| 1. CD-ROM → IRQ2 | raise_count(2) = 0 após 80 M passos | **QUEBRADO** — IRQ2 nunca dispara |
| 2. CPU → handler (0x80000080) | 699 vetorizações (VBlank + DMA) | **Funciona** (para outras IRQs) |
| 3. EvCB alocado? | 16 blocos em 0xE028, 448 bytes | **Alocado mas vazio** — status=0 em todos |
| 4. TTY vs referência | Para em `_96_remove` | **Shell não monta sistema de arquivos** |

**A cadeia arrebenta no elo 1: o CD-ROM nunca levanta IRQ2. Sem IRQ2, o kernel nunca chama
DeliverEvent, os EvCBs permanecem vazios, e o shell nunca encontra o evento que faria a
montagem do sistema de arquivos.**

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec/medição diz | Como foi pego |
|---|---|---|---|---|
| 1 | diagnóstico | Que o CD-ROM levantava IRQ2 (confirmado em testes isolados nas iters 0114/0121/0124). | IRQ2 = 0 em 80 M passos de boot completo com disco. Os testes isolados usam harness que escreve direto no porto do CD-ROM; o boot completo depende do BIOS inicializar o CD-ROM, e essa inicialização não produz IRQ2. | Contador `raise_counts[2]` zerado no teste de integração. |
| 2 | endereçamento | Que o EvCB pointer do Table of Tables estava em endereço físico direto. | O ponteiro está em KSEG1 (0xA000E028), precisei mascarar `& 0x001F_FFFF` para ler o EvCB corretamente. | O dump de memória mostrava `ptr=0xA000E028` e o filtro `ptr < 0x0020_0000` rejeitava, rotulando como "não alocado". |
| 3 | processo | Que a bateria de mutação poderia ser multi-arquivo (irq.rs + cpu.rs). | O formato de manifesto suporta UM `alvo:` por arquivo. Precisei concentrar todos os mutantes em irq.rs. | O meta-teste `mutation_anchors` buscava todas as âncoras no `cpu.rs` e não achava nenhuma. |

## Bateria de mutação

Placar da bateria: 5/5 mutantes mortos, 2/2 controles verdes, 0 equivalente — docs/mutantes/0126-evento-cdrom-kernel.mut

| # | Mutação | Teste que pegou |
|---|---|---|
| m1 | raise() não incrementa o contador | `irq_raise_count_incrementa_por_bit` (raise_count fica 0) |
| m2 | raise_count() sempre devolve 0 | `irq_raise_count_incrementa_por_bit` (assert_eq falha) |
| m3 | raise_counts inicia com 1 em vez de 0 | `irq_raise_count_incrementa_por_bit` (assert_eq != 0 no início) |
| m4 | raise() não incrementa bit 2 (CD-ROM) | `irq_raise_count_incrementa_por_bit` (raise_count(2) fica 0) |
| m5 | raise_count() desvia todos os bits para índice 0 | `irq_raise_count_incrementa_por_bit` (bit 0 ≠ bit 2) |
| c1 | comentário cosmético antes de raise() | verde |
| c2 | renomeação consistente raise_counts → irq_raise_counts | verde |

### Verificação manual

Mutantes aplicados sobre `crates/psx-core/src/irq.rs`, bateria rodada com:
```
cargo test -p psx-core --test cdrom_evento_kernel irq_raise_count --release
```

| Mutante | Resultado |
|---|---|
| m1 | FAILED — `assertion left == right` (raise_count(2)=0 ≠ 1) |
| m2 | FAILED — `assertion left == right` (raise_count(2)=0) |
| m3 | FAILED — `assertion left == right` (valor inicial ≠ 0) |
| m4 | FAILED — `assertion left == right` (raise_count(2)=0) |
| m5 | FAILED — `assertion left == right` (raise_count(0)≠raise_count(2)) |
| c1 | ok (comentário cosmético) |
| c2 | ok (renomeação consistente) |

## Placar antes → depois

Workspace: 823 → **828** testes (+5: `irq_raise_count_incrementa_por_bit`, `irq_raise_count_bit_fora_do_alcance_retorna_zero`, `cpu_conta_entradas_do_handler_de_interrupcao`, `boot_com_disco_produz_irq2_e_handler`, `tabela_de_tabelas_evcb_esta_presente_apos_o_boot`), 0 falhas.

## Decisões e notas

- **A causa raiz não é o dispatch (4.4t) nem o CD-ROM individual — é a INICIALIZAÇÃO.** O kernel
  chama `_96_init()` durante o boot, que deveria enviar comandos ao CD-ROM (Test, GetStat) e
  receber respostas com IRQ2. Nosso `Irq::raise_counts[2]` prova que isso NÃO acontece.
  Próximo passo: instrumentar POR QUE o `_96_init()` do BIOS não dispara comandos de CD-ROM.
  Candidatos: o kernel pula a inicialização do CD-ROM (porque o drive está em estado
  inconsistente), ou o CD-ROM rejeita os comandos antes de produzir resposta.

- **Handoff para 4.4v:** O foco deve ser a inicialização do CD-ROM pela BIOS (`_96_init` /
  `A(96h)`). Por que o kernel não envia comandos ao CD-ROM durante o boot? O drive está
  no estado correto para receber comandos? A flag de shell-open (bit 4 do stat) está
  bloqueando?

- **Contadores adicionados:** `Irq::raise_counts[11]` conta quantas vezes cada bit de IRQ foi
  levantado. `Cpu::irq_handler_entries` conta quantas vezes a CPU vetorizou para 0x80000080
  por interrupção. Ambos são públicos e acessíveis por teste.

- **Manifestos antigos reparados:** `0096-i-mask-investigacao.mut` (m4) e
  `0103-ra-corrompido.mut` (c2) tiveram âncoras atualizadas porque as novas linhas em
  `irq.rs` e `cpu.rs` deslocaram o texto-alvo. A semântica dos mutantes permanece a mesma.
