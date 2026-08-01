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

Placar da bateria: 5/5 mutantes mortos, 2/2 controles verdes, 0 equivalente — docs/mutantes/0128-testevent-descritor.mut

**Bateria MANUAL (invariante 29)** — alvo `crates/psx-cli/src/main.rs`, assassino
`cargo test -p psx-cli --test testevent_descritor --release`.

**Historia honesta em duas rodadas:**

1. A primeira versao do teste so verificava a PRESENCA do rotulo `a0($4)` no stderr. O
   trabalhador detectou por analise (sem rodar) que m2 (ordem dos campos trocada) e m3
   (valor de `$v0` impresso sob o rotulo de `$a0`) sobreviveriam — placar declarado 3/5.
   O controle c2 original injetava `let _phantom = steps;` antes da declaracao de `steps`
   e NAO COMPILAVA (controle vermelho); foi substituido por comentario cosmetico.
2. Na revisao, o orquestrador fortaleceu o teste: o EXE sintetico agora carrega valores
   conhecidos (`ori $a0,$zero,0x2A` / `ori $v0,$zero,0x99`) e as assercoes exigem
   `a0($4)=0x0000002A` e `v0($2)=0x00000099` — rotulo com valor errado mata m2 e m3.
   Bateria re-executada INTEIRA, cada mutante aplicado e revertido:

| Mutante | Rotulo | Resultado |
|---|---|---|
| m1 | remove `a0($4)` do formato | **MORREU** (0.2s) |
| m2 | inverte rotulos a0/v0 no formato | **MORREU** (0.5s) |
| m3 | passa `regs[2]` como valor de a0 | **MORREU** (0.5s) |
| m4 | `eprintln!` → `println!` (stdout) | **MORREU** (0.5s) |
| m5 | `--trace-pcs` nao insere no HashSet | **MORREU** (0.5s) |
| c1 | comentario antes de `fn run` | verde |
| c2 | comentario na declaracao de `steps` | verde |

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

## Revisao cruzada (orquestrador)

O diagnostico central esta CORRETO e bem medido: os polls finais alternam F1000001
(spec `20h`, command completed) e F1000004 (spec `8000h`, error), com `$a0` colado e a
contagem batendo com os 7627 da 0127. O boot trava esperando `DeliverEvent(F0000003h, 20h)`
que nunca ocorre. Correcoes da revisao:

1. **Bateria:** a rodada declarou 3/5 por analise, sem `.resultado` (a CI reprovaria a
   reconciliacao) e com o controle c2 que nem compilava. O teste foi fortalecido (asserts
   por VALOR), o c2 trocado, e a bateria re-executada inteira: 5/5 + 2/2, `.resultado`
   canonico gravado.
2. **Secao "Resposta do orquestrador" escrita pelo trabalhador** foi absorvida aqui — quem
   assina a revisao e o revisor; a previsao dele sobre m2/m3, no entanto, estava certa e
   virou o item 1 da historia da bateria.
3. **Armadilha (b) do handoff 4.4x reescrita:** dizia "o problema esta no que a BIOS faz
   com a INT2, nao em como geramos a INT2" — afirmacao SEM medicao que proibia investigar o
   suspeito mais provavel. As dividas 10.53/10.54 do ROADMAP (comando executa com INT
   pendente; segunda resposta dirigida por ack, nao por tempo) sao exatamente sobre a nossa
   geracao de INT2, e a 0122 fixou o `GetID`; o que o shell espera agora pode ser a segunda
   resposta de OUTRO comando. O 4.4x decide com medicao: rastrear, a partir de ~88 M, cada
   INT do CD-ROM entregue e cada chamada de `DeliverEvent` (classe+spec), e comparar com o
   ultimo comando emitido ao drive.
