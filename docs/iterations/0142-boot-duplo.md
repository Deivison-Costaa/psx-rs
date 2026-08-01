# 0142 — boot-duplo

- **Data:** 2026-08-01
- **Item do roadmap:** 4.5
- **Objetivo:** achar **o que** reseta a cadeia de exceções, mecanismo que a 0141 mediu mas não
  nomeou.

## Revisão do PR anterior

PR #158 (iter 0141). Revisão cruzada não aconteceu por falha de ferramental (revisor isolado em
`/tmp` sem acesso de leitura ao repo — dívida 10.65); em lugar dela, auto-ataque por medição, que
a conclusão sobreviveu.

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| psx-spx | § C(08h) - SysInitMemory(addr,size) (L2551) | docs/reference/13-kernel-bios.md |
| psx-spx | § C(07h) - InstallExceptionHandlers()  ;destroys/uses k0/k1 (L2546) | docs/reference/13-kernel-bios.md |

Trecho autoritativo (§ SysInitMemory):

> Initializes the address (A000E000h) and size (2000h) of the allocate-able Kernel Memory region,
> and, seems to deallocate any memory handles which may have been allocated via B(00h).

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a medição mostrou | Como foi pego |
|---|---|---|---|---|
| 1 | premissa | Que "segunda execução do boot" implicasse **reset de CPU**, e que a sonda no vetor de reset fosse encontrá-lo repetido | `reset_vector` dispara **uma única vez, no ciclo 0**. Não há reset nenhum: é código do BIOS sendo re-executado a partir da RAM/ROM com a CPU no mesmo estado | Sonda em `phys == 0x1FC00000` contando disparos |

## Medição

Sonda descartável nos marcos do caminho de (re)inicialização, carimbados com `bus.total_cycles()`.
Revertida antes do commit.

```
BOOT reset_vector          ciclo=0                          ← UMA vez, so
BOOT C0(07)                ciclo=318605        ra=BFC06864
BOOT C0(08)                ciclo=394124        ra=BFC06914   ← init normal
BOOT exe_entry_80030000    ciclo=14058474      ra=BFC0702C   ← boot do jogo
BOOT C0(08)                ciclo=354241830     ra=BFC06F4C   ← ****
BOOT A0(44)                ciclo=364458316     ra=800431E8
BOOT exe_entry_80030000    ciclo=394001939     ra=8002FF90
```

### A cadeia causal, fechada ponta a ponta

1. A 0141 mediu que o handler do jogo (`0x80140004`, prio 0) **sobrevive** às próprias chamadas de
   `SysDeqIntRP`, e que a cadeia é resetada entre os ciclos 342,53 M e 354,27 M.
2. **No ciclo 354.241.830 — dentro dessa janela — código do BIOS em `BFC06F4C` chama
   `C(08h) SysInitMemory`.**
3. A spec diz que `SysInitMemory` reinicializa a região de memória do kernel em **`A000E000h`**,
   tamanho `2000h`.
4. A 0141 mediu `[0x100] = A000E004`: **o array de ExCB vive dentro dessa região**. Reinicializá-la
   apaga as quatro cabeças de cadeia — e com elas o handler do jogo.
5. O kernel reenfileira em seguida apenas os handlers dele (chamadas 11-12 da 0141), e o `WaitIntr`
   do jogo passa a esperar para sempre um bit 6 que o kernel acka sozinho — o congelamento que a
   0137 descreveu.

**Não há reset de CPU.** O `reset_vector` dispara uma vez só, no ciclo 0. O que existe é
re-execução de código de boot do BIOS com a máquina em pleno funcionamento.

## Bateria de mutação

Bateria de mutação: não se aplica — diagnóstico puro, nenhuma linha de código de produção no diff; a sonda foi descartável e revertida antes do commit.

## Placar antes → depois

Workspace: **870** → **870** testes. Nenhum código de produção mudou, por desenho.

## Revisão cruzada (orquestrador)

<!-- Preenchido na revisão do PR. -->

## Decisões e notas

**1. O que ficou provado e o que não.** Está provado *o quê* apaga a cadeia (`SysInitMemory`
chamado de `BFC06F4C` no ciclo 354,24 M) e *por quê* isso apaga (a região reinicializada contém o
array de ExCB, pela spec e pela medição de `[0x100]`). **Não** está provado *por que* o BIOS
re-executa esse caminho aos 354 M. Essa é a pergunta da próxima iteração, e é onde mora o defeito.

**2. Duas leituras possíveis, e como distingui-las.**
   - **(a) Espúria (nossa):** algo no nosso emulador desvia o PC para `BFC069xx`/`BFC06Fxx` —
     exceção mal vetorizada, salto com alvo errado, ou retorno para `$ra` corrompido. Distinguir:
     rastrear o PC nos ~2000 passos antes do ciclo 354.241.830 e ver **de onde** se entra no BIOS.
   - **(b) Legítima (o jogo pede):** o Crash chama de propósito uma rotina do BIOS que reinicializa
     memória, e o defeito é o kernel não repor os handlers — ou o jogo esperar que reponha.
     Distinguir: se a entrada vier de código do jogo (`0x800xxxxx`) com `$ra` coerente, é (b).

   `ra=BFC06F4C` diz apenas que a **chamada a C0(08)** partiu do BIOS; não diz quem entrou no BIOS.

**3. Por que ainda não há conserto.** Três iterações seguidas (0137, 0141, 0142) foram diagnóstico
puro, e isso é deliberado: a 0137 nomeou um mecanismo errado ("rollback do init"), a 0141 o refutou,
e esta nomeou o mecanismo certo. Implementar sobre a hipótese da 0137 teria produzido goldens de
custo de ciclo para um sintoma cuja causa é outra. A dívida 10.45 (`cpu.rs:187` subcusta LWC2/SWC2)
**continua aberta e continua legítima** — só não é a explicação deste congelamento.

**4. Passo 3 já medido nesta mesma iteração: a leitura (a) está DESCARTADA.** Sonda de transição
RAM→BIOS na janela do `SysInitMemory` (ciclos 354.241.000–354.242.400) achou **seis** transições, e
a decisiva é:

```
ciclo=354241243  de=00002DB8 → 1FC06FDC  ra=00002DBC  epc=00001ED8  cause=00000000
ciclo=354241659  de=00002DB8 → 1FC06FDC  ra=00002DBC  epc=00001ED8  cause=00000000
```

`ra = 0x2DBC` é exatamente `0x2DB8 + 4`, `cause = 0`, `epc` inalterado. É um **`jal` normal**:
código do kernel em RAM (`0x2DB8`) chama deliberadamente a rotina do BIOS em `BFC06FDC`, que leva ao
`SysInitMemory` de `BFC06F4C` 171 ciclos depois. Não há exceção mal vetorizada, salto com alvo
errado nem `$ra` corrompido — as três formas da leitura (a).

Uma verificação de controle na mesma sonda, com a janela larga (354,10 M–354,26 M): 1405 transições
RAM→BIOS, **todas** de `0x1F0C` para três alvos alternados em `BFC07E8C/EAC/ED8`. É o trampolim de
chamada do kernel funcionando normalmente — serve de linha de base e mostra que a sonda não estava
inventando transições.

**Portanto vale a leitura (b): a reinicialização é pedida, não acidental.** O que falta agora é
identificar **que função do kernel mora em `0x2DB8`** e **quem a chama** — se o próprio jogo, via
alguma A/B/C-function, ou se é caminho interno do kernel disparado por outra coisa. Note que
`A(9Ch) SetConf`, o candidato óbvio (a spec diz que ele "deallocates all old ExCBs"), foi sondado e
**não** dispara nesta execução.
