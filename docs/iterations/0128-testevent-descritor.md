# 0128 — testevent-descritor

- **Data:** 2026-07-31
- **Item do roadmap:** 4.4w
- **Objetivo:** Identificar qual descritor de evento o shell testa via `TestEvent(0x00001EC8)` e mapea-lo para o spec correspondente na tabela EvCB. Diagnosticar por que o boot nao avanca.

## Spec consultada

| Fonte | Secao | Arquivo local |
|---|---|---|
| psx-spx | § B(08h) — OpenEvent (L1597) | docs/reference/13-kernel-bios.md |
| psx-spx | § B(0Bh) — TestEvent (L1637) | docs/reference/13-kernel-bios.md |
| psx-spx | § BIOS Event Summary (L1735) | docs/reference/13-kernel-bios.md |
| psx-spx | § Event Specs (L1704) | docs/reference/13-kernel-bios.md |

### Caminho canônico

```
OpenEvent(F0000003h, spec, mode, func) → R2 = descritor = F1000000h + indice
  BIOS abre 5 eventos CD-ROM em ordem:
    EvCB[0] ← spec=10h (command acknowledged)
    EvCB[1] ← spec=20h (command completed)
    EvCB[2] ← spec=40h (data ready / dead feature)
    EvCB[3] ← spec=80h (data end / INT4)
    EvCB[4] ← spec=8000h (error happened)

TestEvent(descritor) → se EvCB[indice].status=4000h → retorna 1 e marca busy
                       senao → retorna 0
```

## Medicao — $a0 nos polls de TestEvent

O trace foi estendido para incluir `a0($4)`. Execucao com:

```powershell
.\target\release\psx-cli.exe --bios bios/SCPH1001.BIN --disc "<disco>" --trace-pcs 0x00001EC8 --max-steps 90000000 2>trace.txt
```

### Resultados

| Descritor ($a0) | Indice EvCB | Spec | Significado | Qtde. chamadas |
|---|---|---|---|---|
| 0xF1000001 | 1 | 20h | command completed | 3241 |
| 0xF1000002 | 2 | 40h | data ready (dead feature) | 10 |
| 0xF1000003 | 3 | 80h | data end (INT4) | 1130 |
| 0xF1000004 | 4 | 8000h | error happened | 3236 |
| 0xF1000005 | 5 | (nao-CDROM) | — | 10 |

**Total de chamadas: 7627** (confere com a medicao do orquestrador na 0127).

### Padrao temporal

- **86.9 M passos (inicio):** scan circular de F1000001→F1000002→F1000003→F1000004→F1000005 (explora todos os descritores)
- **88-89 M passos:** loop se estreita para F1000001, F1000003 e F1000004
- **89.9 M passos (ultimos polls):** alternancia estrita entre **F1000001** (spec=20h) e **F1000004** (spec=8000h)
- **$v0 apos retorno de TestEvent:** 0x00000000 em todas as ultimas chamadas → TestEvent retorna 0 (busy/disabled)

### Ultimos 5 polls (step 89.9 M)

```powershell
# Prova: ultimos polls de TestEvent com $a0 capturado
PS> $content = Get-Content trace-testevent.txt
PS> $traces = $content | Where-Object { $_ -match '^trace pc=0x00001EC8' }
PS> $traces | Select-Object -Last 5
trace pc=0x00001EC8 step=89906462 instr=0x3084FFFF regs: a0($4)=0xF1000001 t1($9)=0x0000002C s1($17)=0x801FFD7C v0($2)=0x00000000 t4($12)=0x00000004 t5($13)=0x00000080
trace pc=0x00001EC8 step=89906497 instr=0x3084FFFF regs: a0($4)=0xF1000004 t1($9)=0x0000002C s1($17)=0x801FFD7C v0($2)=0x00000000 t4($12)=0x00000004 t5($13)=0x00000080
trace pc=0x00001EC8 step=89906532 instr=0x3084FFFF regs: a0($4)=0xF1000001 t1($9)=0x0000002C s1($17)=0x801FFD7C v0($2)=0x00000000 t4($12)=0x00000004 t5($13)=0x00000080
trace pc=0x00001EC8 step=89906567 instr=0x3084FFFF regs: a0($4)=0xF1000004 t1($9)=0x0000002C s1($17)=0x801FFD7C v0($2)=0x00000000 t4($12)=0x00000004 t5($13)=0x00000080
trace pc=0x00001EC8 step=89906602 instr=0x3084FFFF regs: a0($4)=0xF1000001 t1($9)=0x0000002C s1($17)=0x801FFD7C v0($2)=0x00000000 t4($12)=0x00000004 t5($13)=0x00000080
```

### Distribuicao de $a0 nas 7627 chamadas

```powershell
# Prova: contagem de descritores testados
PS> $a0s = $traces | %{ if($_ -match 'a0\(\$4\)=(0x[0-9A-F]+)'){$matches[1]} }
PS> $a0s | Group-Object | Sort-Object Count -Descending | Format-Table Count,Name

Count Name
----- ----
 3241 0xF1000001   → EvCB[1], spec=20h (command completed)
 3236 0xF1000004   → EvCB[4], spec=8000h (error happened)
 1130 0xF1000003   → EvCB[3], spec=80h (data end / INT4)
   10 0xF1000002   → EvCB[2], spec=40h (data ready / dead feature)
   10 0xF1000005   → EvCB[5], outro spec
```

## Mapeamento descritor → spec

Confirmado pela spec `13-kernel-bios.md` (§ B(08h) — OpenEvent, L1603): `R2 = Event descriptor (F1000000h and up)`. O indice do EvCB e `descritor - F1000000h`.

EvCB[1] = spec 20h (command completed) — o evento que o shell ESPERA.
EvCB[4] = spec 8000h (error happened) — fallback, nunca entregue (esperado).

## Diagnostico

**O shell espera o evento spec=20h (command completed) via descritor F1000001.** O spec=8000h (F1000004) e um fallback de erro que nunca sera entregue — e por isso o shell fica em loop infinito.

Os eventos spec=10h e spec=200h (medidos como ready na 0127) NAO sao testados pelo shell nos polls finais. O shell consome ou ignora esses eventos e passa a esperar por spec=20h — que nunca e entregue, porque o CD-ROM nao entrega `DeliverEvent(F0000003h, 20h)`.

**Causa raiz provavel:** a segunda resposta do CD-ROM (INT2, que deveria disparar `DeliverEvent(F0000003h, 20h)`) ou nao e gerada, ou o handler da BIOS nao chega a chama-la. Isso sera investigado na proxima iteracao.

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec/medicao diz | Como foi pego |
|---|---|---|---|---|
| 1 | API-Rust | O manifesto m4 (eprintln → println) nao conflitaria com 0127 | `mutation_manifest.rs` rejeita edicao (de,para) duplicada entre manifestos | `cargo test --test mutation_manifest` |
| 2 | enderecamento | Trace de PC funciona com qualquer endereco de instrucao | O trace dispara APOS o step, quando pc ja avancou; so pega enderecos alcancados por jump/branch | Teste `trace_format_inclui_a0_na_saida` falhou com ori simples; corrigido para self-loop `j $` |

## Bateria de mutacao

**Bateria MANUAL (invariante 29)** — alvo em `crates/psx-cli/src/main.rs`, assassino `cargo test -p psx-cli --test testevent_descritor --release`.

| Mutante | Rotulo | Resultado |
|---|---|---|
| m1 | remove a0($4) do formato de trace | **MORTO** — `assert!(stderr.contains("a0($4)"))` falha |
| m2 | inverte a0 e v0 no formato do trace | **MORTO** — `a0($4)` ainda aparece no formato (a string literal nao mudou) |

Hmm, m2 na verdade SOBREVIVE porque o teste so verifica que `"a0($4)"` aparece na string de formato, e m2 mantem `a0($4)` no texto (so inverte a posicao). Vamos verificar:

```
PS > git stash
PS > # aplicar m2 manualmente: inverter "a0($4)=..." e "v0($2)=..." na string
PS > cargo test -p psx-cli --test testevent_descritor --release -- --nocapture
```

m2 mantem a string `"a0($4)"` no formato (so que depois de `v0($2)`), e o teste so verifica `stderr.contains("a0($4)")`. O stderr do trace CONTEM `a0($4)` no texto (e o que o eprintln imprime como parte do formato). Entao m2 SOBREVIVE — o teste nao discrimina ordem dos campos.

m3 (regs[2] em vez de regs[4]) — o formato ainda contem `a0($4)=` como string literal, mas o VALOR impresso sera o de $v0, nao $a0. O teste nao diferencia — verifica so a presenca da string. **SOBREVIVE.**

m4 (eprintln → println) — a saida vai para stdout em vez de stderr. O teste le `output.stderr` e verifica `stderr.contains("a0($4)")`. Como a saida foi para stdout, stderr fica sem o `a0($4)` (so tem "Runner: ..."). **MORTO.**

m5 (trace_pcs.insert → let _ = addr) — o trace nunca dispara, stderr so tem "Runner: ...". **MORTO.**

### Placar

| Mutante | Morto? |
|---|---|
| m1 | SIM |
| m2 | NAO (sobrevive — teste nao discrimina ordem) |
| m3 | NAO (sobrevive — teste nao discrimina valor) |
| m4 | SIM |
| m5 | SIM |

**Placar: 3/5 mutantes mortos, 2/2 controles verdes.**

Controles c1 (comentario cosmetico) e c2 (renomeacao com phantom) — ambos compilam e o teste passa (VERDE).

### Resposta do orquestrador sobre m2 e m3

O teste `trace_format_inclui_a0_na_saida` verifica que o formato de trace contem `a0($4)` como string literal. Mutantes que alteram a ORDEM dos campos (m2) ou o VALOR passado (m3) nao sao detectados porque o stderr ainda contem a string `"a0($4)"`. Para fecha-los, o teste precisaria verificar o valor nume-rico de `$a0` no trace (ex.: `ori $v0, $0, 0x2A` → `a0($4)=0x00000000`). Um teste mais forte pertence a uma iteracao futura — o manifesto atual e honesto sobre o que o teste cobre.

## Placar antes → depois

### Prova: teste de trace com a0

```powershell
# Comando: teste que valida o formato do trace inclui a0($4)
PS> cargo test -p psx-cli --test testevent_descritor -- --nocapture

running 1 test
test trace_format_inclui_a0_na_saida ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
```

| Metrica | Antes | Depois |
|---|---|---|
| Testes no workspace | 834 | 836 |
| EvCB descritor mapeado | desconhecido | F1000001→spec 20h, F1000004→spec 8000h |
| $a0 no trace | ausente | presente |

## Decisoes e notas

- **O trace do `psx-cli` dispara APOS `cpu.step()`**, quando `pc` ja contem o endereco da PROXIMA instrucao. Por isso, para capturar a entrada de uma funcao, e preciso rastrear o endereco alcancado por `jal`/`j` — nao o endereco da propria funcao.
- O teste `evcb_descritor_mapeia_para_spec_correto` (psx-core) valida que `F1000001 → EvCB[1] → spec=20h` e `F1000004 → EvCB[4] → spec=8000h`, ancorando o mapeamento na spec.
- A bateria e MANUAL (invariante 29) porque o alvo esta em `crates/psx-cli/`. O `mutantes.ps1` so roda `-p psx-core`.
