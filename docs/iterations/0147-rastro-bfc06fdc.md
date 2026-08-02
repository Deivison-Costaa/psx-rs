# 0147 — rastro-bfc06fdc

- **Data:** 2026-08-01
- **Item do roadmap:** 4.5
- **Objetivo:** rastrear quem escreve `BFC06FDC` em `mem[$v1+0x18]` — o passo 5 do diagnóstico
  que a 0142 e 0144 prepararam.

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| psx-spx | § BIOS RAM Map (L405) | docs/reference/13-kernel-bios.md |
| psx-spx | § C(08h) - SysInitMemory(addr,size) (L2551) | docs/reference/13-kernel-bios.md |

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a medição mostrou | Como foi pego |
|---|---|---|---|---|
| 1 | premissa herdada | Que o slot `$v1+0x18` mudava de valor entre os dois boots — a premissa das iters 0142 e 0144 | O slot **nunca muda**: já contém `BFC06FDC` no PRIMEIRO encontro do trampolim (passo 91.421), escrito durante a inicialização da BIOS nos passos ~58.500 e ~84.292. A premissa de que "no primeiro boot o slot aponta para funções normais do kernel" (0142) está refutada | Sonda em `Bus::write32`/`write8`/`write16` e em `Cpu::sw`/`sb` com endereço-alvo fixo `0x6EF8`; leitura do slot a cada ativação do trampolim |
| 2 | enderecamento | Que comparar `addr == watch_addr` bastava para a sonda de `sw` | O `sw` que escreve `0x000035A4` usa o endereço KSEG1 `0xA0006EF8` — a comparação por endereço lógico falhava porque `0xA0006EF8 > 0x6EFC`. Precisou normalizar por `addr & 0x1FFF_FFFF` | Sonda de `Bus::write32` capturou o write mas a de `Cpu::sw` não; discrepância entre as duas revelou o defeito de KSEG |
| 3 | instrumento | Que bastava instrumentar `sw` (store word) para achar o escritor | O valor `BFC06FDC` é escrito por quatro `sb` (store byte) consecutivas em `0xBFC02B68`, byte a byte: `DC, 6F, C0, BF`. A sonda de `sw` sozinha nunca acharia | Sondas em `write8`/`write16` no barramento e `sb` na CPU; 4 hits de `BUS_WRITE8` e `SB_PROBE` nos passos ~84.292-84.300 |

## Medição

Sondas descartáveis em `Cpu::sw`, `Cpu::sb`, `Bus::write32`, `Bus::write16` e `Bus::write8`
(3 arquivos, 63 linhas, revertidas antes do commit). Endereço-alvo: `0x6EF8` (slot `$v1+0x18`
com `$v1` capturado em runtime no primeiro `PC=0x2DAC`).

### Fase 1 — `sw` em `0xBFC00434`

```
SW_PROBE: sw pc=0xBFC00434 addr=0xA0006EF8 val=0x000035A4 step-set-at=58502
```

O BIOS escreve `0x000035A4` no slot durante a inicialização. Este é um valor temporário
que aponta para uma rotina interna do kernel em RAM.

### Fase 2 — Quatro `sb` em `0xBFC02B68`

```
SB_PROBE: pc=0xBFC02B68 addr=0x00006EF8 val=0xDC step-set-at=84274
SB_PROBE: pc=0xBFC02B68 addr=0x00006EF9 val=0x6F step-set-at=84280
SB_PROBE: pc=0xBFC02B68 addr=0x00006EFA val=0xC0 step-set-at=84286
SB_PROBE: pc=0xBFC02B68 addr=0x00006EFB val=0xBF step-set-at=84292
```

Quatro `sb` consecutivas no mesmo PC sobrescrevem os bytes individuais do slot,
compondo `0xBFC06FDC` (little-endian: `DC,6F,C0,BF`). Todas partem do BIOS ROM
(`0xBFC0xxxx`), executadas durante a sequência de boot.

### Fase 3 — Valor constante

O slot **nunca mais é escrito**. Leituras subsequentes:

```
PC=0x2DAC passo=91421    slot_val=0xBFC06FDC   ← primeiro trampolim
PC=0x2DAC passo=137572   slot_val=0xBFC06FDC
PC=0x2DAC passo=2725283  slot_val=0xBFC06FDC
```

Em 400 M passos, nenhum `Bus::write32`/`write16`/`write8` tocou no endereço físico
`0x6EF8-0x6EFB` depois do passo ~84.300.

### Conclusão

`BFC06FDC` **não é escrito entre os dois boots**. É escrito UMA vez durante a
inicialização do BIOS, pelos endereços `0xBFC00434` (sw inicial) e `0xBFC02B68` (4× sb),
e nunca mais muda. O trampolim em `0x2C94..0x2DB8` **sempre** chama `BFC06FDC`, desde o
primeiro até o último acionamento.

**A premissa da 0142 está refutada:** o slot não muda de valor entre o "primeiro boot" e
o "segundo boot". O que foi interpretado como "primeiro boot com funções normais" e
"segundo boot com SysInitMemory" são, na verdade, chamadas ao mesmo endereço
(`BFC06FDC`) em momentos diferentes, cujo efeito depende do ESTADO da máquina naquele
instante — não do valor do slot.

## Bateria de mutação

Bateria de mutação: não se aplica — diagnóstico puro, nenhuma linha de código de produção
no diff; as sondas foram descartáveis e revertidas antes do commit.

## Placar antes → depois

Workspace: **880** → **882** testes (2 novos em `slot_v1_18_bfc06fdc`).

## Revisão cruzada (orquestrador)

<!-- Preenchido na revisão do PR. -->

## Decisões e notas

**1. A pergunta da iteração está respondida — e a resposta refuta a premissa.** Quem
escreve `BFC06FDC` em `mem[$v1+0x18]` é o BIOS, durante a inicialização, nos passos
~58.500 e ~84.292. Depois disso o slot nunca muda. A premissa "o valor muda entre os
dois boots" era falsa.

**2. O defeito de raiz precisa ser reformulado.** Se o trampolim sempre chama
`BFC06FDC` (`SysInitMemory`), e o primeiro boot sobrevive, a pergunta correta não é
"quem escreve BFC06FDC no slot" — é "por que SysInitMemory apaga a cadeia de ExCB no
segundo boot mas não no primeiro". Duas hipóteses imediatas:
   - (a) No primeiro boot, `SysInitMemory` é chamada com a região `A000E000h+2000h`
     **antes** de os handlers do jogo estarem enfileirados — portanto não há nada para
     apagar.
   - (b) No segundo boot, a chamada acontece **depois** de o jogo enfileirar seus
     handlers, e a reinicialização da região os destrói.

   A 0141 mostrou que a cadeia é resetada entre 342,53 M e 354,27 M, e a 0142 mostrou
   que `SysInitMemory` é chamada nessa janela. Esta iteração mostra que `SysInitMemory`
   é chamada em CADA execução do trampolim. A diferença entre o primeiro e o segundo
   boot está no MOMENTO da chamada, não no alvo.

**3. Implicação para o M4: o congelamento não é um defeito de "slot errado".** O
trampolim funciona corretamente e sempre chama o mesmo endereço. O defeito está no
**encaixe temporal** entre a reinicialização da região de kernel e o enfileiramento
dos handlers do jogo.

**4. Por que `0xBFC02B68` escreve byte a byte?** O endereço `0xBFC02B68` está na
região da BIOS ROM. O código provavelmente é parte de uma rotina de cópia de
estrutura (memcpy/algo similar) que inicializa uma tabela de ponteiros de função
durante o boot. Os quatro `sb` consecutivos com o mesmo PC sugerem um loop de
inicialização, não uma instrução isolada.

**5. O slot_addr = 0x6EF8 é consistente entre execuções.** O `$v1` no ponto
`0x2DAC` sempre vale `0x6EE0` (com disco Crash), dando slot_addr = 0x6EF8.
Este endereço está na RAM baixa (KUSEG < 2 MB), região usada pelo kernel para
estruturas de driver.
