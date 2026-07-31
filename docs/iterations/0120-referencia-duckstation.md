<!-- Custo, tokens e duração NÃO entram aqui: são medidos por scripts/oc-iter.ps1 em
     docs/metricas.csv, casados por head_antes/head_depois. -->

# 0120 — referencia-duckstation

- **Data:** 2026-07-31
- **Item do roadmap:** 4.4o
- **Objetivo:** rodar a mesma BIOS com o mesmo disco num emulador de referência e diferenciar a
  sequência de comandos do CD-ROM contra a nossa. **Iteração de diagnóstico: sem código de
  produção.**

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| psx-spx | § First Response (INT3) (or INT5 if failed) | docs/reference/06-cdrom.md |
| psx-spx | § First Response | docs/reference/06-cdrom.md |
| psx-spx | § GetID | docs/reference/06-cdrom.md |

## O oráculo externo

DuckStation portátil (`psx-estado/.../duckstation/app`), configurado para a **mesma** BIOS
(`bios/SCPH1001.BIN`, via `SearchDirectory`), `PatchFastBoot = false` (boot completo, como o
nosso), região NTSC-U, `LogLevel = Debug` e `LogToFile = true`. Disco: o mesmo
`Crash Bandicoot (USA).cue`. Corrida de 45 s, log preservado em
`psx-estado/referencias/duckstation-cdrom-boot.txt`.

## A diferença, em uma linha

```
  REFERÊNCIA                        NOSSO
  Getstat    Stat=0x02              Getstat  (stat = 0x02)
  GetID                             — nada, para sempre —
  Getstat    Stat=0x02
  GetID
  Setloc     00:02:04
  SeekL      00:02:04
  Setmode    0x80
  ReadN      00:02:04
  DataSector 00:02:04 LBA=154
  ...
  (Executable path: 'SCUS_949.00'; System booted in 494.70ms)
```

Dois achados, e o primeiro elimina uma família inteira de suspeitos:

1. **O nosso stat byte está certo.** A referência responde `Stat=0x02` ao `Getstat` — idêntico ao
   nosso. Toda hipótese do tipo "o shell desiste porque o status diz outra coisa" morre aqui,
   incluindo a hipótese 3 da 0119 (bit de motor), que o experimento já tinha refutado.
2. **Falta exatamente um comando: `GetID`.** No hardware real o BIOS emite `GetID` imediatamente
   depois do `Getstat`, e a partir daí a cadeia inteira (`Setloc`/`SeekL`/`Setmode`/`ReadN`) roda e
   o executável `SCUS_949.00` é carregado. O nosso `cdrom.rs` **implementa** o `0x1A`; o BIOS é que
   nunca o pede.

## O defeito encontrado a caminho (não é a causa provada)

Medindo a janela do `Getstat` no nosso emulador, com watch das entradas em `0x80000080`:

```
  passo 87464254  W porta1 = 0x01     (comando Getstat)
  passo 87464256  entra no handler, I_STAT = 0x00000004   <-- IRQ2, DOIS passos depois
```

O INT3 é entregue **em zero ciclo**. A spec, § First Response: *"Nop (normal) 000c4e1h
0004a73h..003115bh"* — a primeira resposta leva em média **0xC4E1 = 50 401 ciclos**, no mínimo
0x4A73 = 19 059. Nós respondemos antes da instrução seguinte ao `sw` do comando.

Isso é um defeito real por três razões independentes: contraria a spec, viola o R2 (a resposta
devia ser um evento no `scheduler`, não um efeito colateral imediato da escrita no porto), e é
plausível como causa do sintoma — a interrupção chega no meio da escrituração do driver, que ainda
não terminou de armar o estado da operação. **Plausível não é provado** (invariante 26): quem
fecha isso é a iteração que implementar o atraso e medir se o `GetID` aparece.

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a medição diz | Como foi pego |
|---|---|---|---|---|
| 1 | diagnóstico | Que a divergência apareceria como uma resposta diferente do drive (status, licença, região). | A resposta é byte a byte igual à da referência. A divergência é de **iniciativa**: o BIOS real pede mais, o nosso para de pedir. | Primeira linha do log da referência: `Getstat Stat=0x02`, idêntico ao nosso. |
| 2 | timing | Que responder um comando na hora fosse conservador e inofensivo — "no pior caso, rápido demais". | § First Response: a primeira resposta demora dezenas de milhares de ciclos. Entregar em zero ciclo faz a IRQ pré-emptar o próprio código que acabou de escrever o comando. | Só apareceu porque instrumentei a entrada no handler junto com os portos: `87464254` escreve, `87464256` já está no handler. |

## Bateria de mutação

Bateria de mutação: não se aplica — esta iteração não altera código de produção. Ela entrega a
sequência de comandos da referência (preservada em `psx-estado/referencias/`), a confirmação de que
o nosso stat byte é idêntico ao real, a identificação do comando que falta (`GetID`) e o defeito de
timing da primeira resposta, com os números da spec.

**Duas iterações de diagnóstico seguidas (0119 e 0120).** É o custo de ter entrado num sintoma sem
oráculo; a 4.4p já é item de código, com alvo e números definidos.

## Placar antes → depois

Workspace: **790** testes, 0 falhas (inalterado — nenhuma linha de produção mudou).

## Revisão cruzada (orquestrador)

- **A referência é comparável de propósito.** Mesma BIOS, mesmo disco, `PatchFastBoot = false`.
  Um boot rápido teria pulado justamente o trecho em disputa.
- **O log foi preservado fora do repositório**, em `psx-estado/referencias/`, como todo artefato de
  medição. Nada de imagem de disco ou binário no repo.
- **Árvore limpa**, `crates/psx-core/src/bin` removido, diff só de documentação.
- **Gates:** `roadmap_size`, `status_size`, `status_handoff`, `spec_citations` e `mutation_battery`
  verdes.

## Decisões e notas

- **O oráculo externo pagou na primeira corrida.** A invariante 27 nasceu na 0119 depois de quatro
  refutações por instrumentação; aqui, 45 s de emulador de referência deram o que três harnesses não
  deram. Fica o registro de que o custo de montar o oráculo (configurar log, casar BIOS e disco) é
  menor do que o de mais uma rodada de instrumentação cega.
- **Próximo degrau, e agora é código.** Item 4.4p: entregar a primeira resposta do CD-ROM pelo
  `scheduler`, com o atraso da spec (média 0xC4E1 ciclos), em vez de dentro da escrita no porto. O
  critério de aceitação é o sintoma, não o relógio: **o `GetID` tem de aparecer depois do
  `Getstat`**. Se não aparecer, o atraso continua sendo a correção certa pela spec, mas a causa é
  outra — e aí o próximo passo é diferenciar o TTY do kernel contra a referência.
