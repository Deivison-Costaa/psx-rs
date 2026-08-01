<!-- Custo, tokens e duração NÃO entram aqui: são medidos por scripts/oc-iter.ps1 em
     docs/metricas.csv, casados por head_antes/head_depois. -->

# 0129 — deliverevent-diagnostico

- **Data:** 2026-07-31
- **Item do roadmap:** 4.4x
- **Objetivo:** Medir se `DeliverEvent(F0000003h, 20h)` ocorre e discriminar entre as duas
  hipóteses do handoff: (a) nosso CD-ROM não gera a INT2 do último comando; (b) a INT2 chega
  mas o handler da BIOS não invoca o DeliverEvent.

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| psx-spx | § B(07h) - DeliverEvent (L1642) | docs/reference/13-kernel-bios.md |
| psx-spx | § BIOS Event Summary (L1735) | docs/reference/13-kernel-bios.md |

## Instrumentação (só observação, zero mudança de comportamento)

1. `psx-cli`: resolve o endereço de `DeliverEvent` em runtime lendo a B-table via o stub em
   `0x000000B0` (lui+addiu do dispatcher, depois B-table[07h]) e loga `class($a0)`/`spec($a1)`
   a cada entrada. Resolvido: `B-table[07h] = 0x00001B44`.
2. `bus.rs`/`cdrom.rs`: cada IRQ2 física do CD-ROM logada com `intsts` e comando pendente.
3. Trace de `--trace-pcs` agora inclui `a1($5)` (teste da iteração).

## Medição

```
.\target\release\psx-cli.exe --bios bios/SCPH1001.BIN --disc "..\roms\extraido\Crash Bandicoot (USA).cue" --trace-pcs 0x00001EC8 --max-steps 200000000
```

Resultado (janela de 200 M steps; idêntico na janela de 92 M para tudo que cabe nela):

- `DeliverEvent(F0000003h, 20h)` **OCORRE 10 vezes** — steps 87 010 146, 87 484 677,
  87 938 621, 88 449 963, 88 880 658, 89 640 133 (6 antes do último TestEvent) e mais 4 na
  rajada final. Também: `F0000003h/200h` 76×, `F0000003h/10h` 2× (DMA finished,
  docs/reference/13-kernel-bios.md L1745).
- O loop de `TestEvent` do shell **PARA**: último poll no step 89 906 602 (total 7 627 polls,
  mesmo número da 0128) — nenhum poll nos 110 M steps seguintes.
- IRQs físicas do CD-ROM: 107 no total, TODAS entre cycles 214,6 M e 221,4 M (≈ steps
  89–92 M): 14× INT3 (primeira resposta), 7× INT2 (completo), **86× INT1 (dados de setor)**
  com `Pause` (0x09) pendente — o shell leu ~86 setores do disco.
- TTY cresce de 473 → 725 bytes após o step 92 M: `SetGraphDebug:level:1,type:0 reverse:0`
  e os "bad hankaku code" — o shell inicializou os gráficos e desenhou.
- Dos steps ~92 M até 200 M: **silêncio total do CD-ROM** (zero comandos novos) e só os pares
  periódicos VBlank (`F2000003h/2`, docs/reference/13-kernel-bios.md L1778) + default-IRQ
  (`F0000001h/1000h`, docs/reference/13-kernel-bios.md L1783) — 324 cada. O shell está em
  loop de VBlank.

## Veredito

**As DUAS hipóteses do handoff caem.** O pipeline inteiro funciona: nosso CD-ROM gera as
INTs, o handler da BIOS entrega `DeliverEvent(F0000003h, 20h)`, o shell sai do loop de
TestEvent, lê ~86 setores e desenha a tela. A premissa "spec=20h nunca é entregue" (herdada
da 0128) era inferência a partir de `v0=0` nos polls — não medição de DeliverEvent. A nova
fronteira do boot: depois de desenhar, o shell fica em VBlank sem pedir mais nada ao drive
até o step 200 M — descobrir o que ele exibe/espera (candidato: já é a tela de logo — comparar
VRAM com a referência do DuckStation).

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | nenhum (premissa herdada) | Que `DeliverEvent(F0000003h,20h)` nunca ocorria (afirmado no STATUS pela 0128) | docs/reference/13-kernel-bios.md § BIOS Event Summary (L1738): a BIOS abre spec=20h internamente e o entrega via handler | Instrumentação direta do endereço B-table[07h]: 10 ocorrências medidas |
| 2 | API-Rust | Que `--disc` aceitava o `.bin` | n/a — o CLI espera o `.cue` (parse_cue) | Primeiro lançamento morreu com "stream did not contain valid UTF-8" |

## Bateria de mutação

Placar da bateria: 5/5 mutantes mortos, 2/2 controles verdes, 0 equivalente — docs/mutantes/0129-deliverevent-diagnostico.mut

Bateria MANUAL (invariante 29) — alvo `crates/psx-cli/src/main.rs`, assassino
`cargo test -p psx-cli --test deliverevent_diagnostico` (aplicado → rodado → revertido, um a um):

| id | mutação | resultado |
|---|---|---|
| m1 | rótulo `a1($5)` → `x1($5)` | MORREU (0.06s) |
| m2 | valor de a1 vem de `regs[2]` | MORREU (0.06s) |
| m3 | valor de a0 vem de `regs[5]` | MORREU (0.06s) |
| m4 | `eprintln!` → `println!` (trace para stdout) | MORREU (0.06s) |
| m5 | valor de v0 vem de `regs[4]` | MORREU (0.06s) |
| c1 | comentário antes de `fn run` | SOBREVIVEU (esperado) |
| c2 | comentário na declaração de `deliver_event_pc` | SOBREVIVEU (esperado) |

O stub `trace_format_inclui_a1_na_saida` em `crates/psx-core/tests/deliverevent_diagnostico.rs`
existe só para o portão `bateria_nomes_de_teste_existem` (mesmo padrão da 0128).

## Placar antes → depois

838 → 840 testes no workspace (o teste da iteração no psx-cli + o stub do portão no psx-core;
contagem canônica do portão `placar_do_status_bate_com_a_contagem_de_testes`).

## Revisão cruzada (orquestrador)

<!-- Preenchido pelo Claude na revisão do PR: achados no formato de docs/prompts/review.md, ou "sem achados". -->

## Decisões e notas

- Janela de 200 M steps escolhida por causa da invariante 30 (medida negativa exige janela
  além do horizonte conhecido): a ausência de novos comandos de CD após ~92 M foi confirmada
  com 108 M steps de margem, não no limite da janela.
- Guardas executadas antes de medir: nenhum `opencode` vivo na árvore e
  `cargo clean -p psx-core --release` (corolário do rlib da invariante 30, pago na 0126) —
  o build recompilou `psx-core` e `psx-cli` do zero.
- A instrumentação de IRQ2 imprime de dentro do `psx-core` (bus/cdrom). É observação pura
  (eprintln + 2 getters), mas fica a nota: se a pureza R3 apertar, mover para callback.
- Invariante 32 criada: o pipeline de eventos do CD-ROM está ELIMINADO como gargalo do boot.
