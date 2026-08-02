<!-- Custo, tokens e duracao ficam em docs/metricas.csv, medidos pelo runner. -->

# 0163 — tty-boot

- **Data:** 2026-08-02
- **Item do roadmap:** 10.89
- **Objetivo:** descobrir por que a ROM reinicializa a memoria do kernel no passo 154.897.433 — e, com a janela certa em maos, provar por experimento onde o boot para.
- **Fonte:** orquestrador.

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| psx-spx | § C(08h) - SysInitMemory(addr,size) (L2551-L2554) | docs/reference/13-kernel-bios.md |
| psx-spx | § B(13h) - StartPAD2() (L1951-L1953) | docs/reference/13-kernel-bios.md |

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | hipotese | Que o `SysInitMemory` do passo 154.897.433 acontecia **no meio da execucao do jogo** — foi o que escrevi no handoff de 0162 e no item 10.89. | Não é assunto de spec: é o TTY do proprio BIOS. | O TTY diz, em ordem: `BOOTSTRAP LOADER Type C Ver 2.1`, `setup file : cdrom:SYSTEM.CNF;1`, `BOOT = cdrom:\SLUS-000.05;1`, `KERNEL SETUP!`, `boot file : cdrom:\SLUS-000.05;1` e so entao `Execute !`. O jogo **ainda nao tinha comecado**: o segundo setup e do bootstrap. **O item 10.89 nasceu de uma inferencia errada minha.** |
| 2 | hipotese | Que o codigo em `0x80030250`/`0x8003F058`/`0x80058624` que eu vinha rastreando era do jogo. | Não é assunto de spec. | `EXEC:PC0(801abce0) T_ADDR(80125000) T_SIZE(000aa800)`: o executavel do jogo ocupa `0x80125000..0x801CF800`. Nada de `0x8003xxxx`/`0x8005xxxx` e dele — era o carregador. As 454.122 chamadas de `TestEvent` de 0159 tambem eram do bootstrap lendo o CD, nao do jogo esperando card. |
| 3 | processo | Que `BOOT = cdrom:\SLUS-000.05;1` viria depois do segundo `KERNEL SETUP!`. | Não é assunto de spec. | A primeira versao do teste afirmou essa ordem e reprovou: o nome do executavel e impresso **antes** do setup. Corrigi a assercao para a ordem real. |

## Bateria de mutação

Bateria de mutação: não se aplica — a iteracao acrescenta somente um teste de integracao e documentacao; nenhum arquivo em `crates/*/src/` foi modificado, portanto nao ha producao para mutar. O oraculo ja se provou sensivel sozinho: a primeira versao reprovou por uma ordem errada de linhas do TTY.

## Placar antes → depois

Workspace: **907 → 909** testes.

## Revisão cruzada (orquestrador)

Rodada de autoria do orquestrador; autorrevisão registrada como limite. A rodada **derruba dois
itens que eu mesmo tinha aberto** (10.88 em 0162, 10.89 aqui) e recoloca a investigacao na
janela certa: tudo que interessa acontece **depois** do passo 164.000.000.

## Decisões e notas

O TTY do BIOS, amostrado a cada 200 k passos, da a linha do tempo inteira:

| Passo (amostra) | TTY |
|---:|---|
| 200.000 | `PS-X Realtime Kernel Ver.2.5` / `KERNEL SETUP!` / `Configuration : EvCB 0x10 TCB 0x04` |
| 2.800.000 | `System ROM Version 2.2 12/04/95 A` |
| 19.400.000 | `ResetCallback: _96_remove ..` |
| 87.600.000 | `System Controller ROM Version 97/01/10 c2` |
| 124.600.000 | `SetGraphDebug:level:1,type:0 reverse:0` |
| 152.000.000 | `BOOTSTRAP LOADER Type C Ver 2.1   03-JUL-1994` |
| 153.400.000 | `setup file    : cdrom:SYSTEM.CNF;1` |
| 155.000.000 | `BOOT = cdrom:\SLUS-000.05;1` / `KERNEL SETUP!` / `boot file : cdrom:\SLUS-000.05;1` |
| 164.000.000 | `EXEC:PC0(801abce0)  T_ADDR(80125000)  T_SIZE(000aa800)` / `Execute !` |
| 164.200.000 | `PS-X Control PAD Driver  Ver 3.0` |
| 167.000.000 | **`VSync: timeout`** — e mais 141 vezes ate o passo 200.000.000 |

O `SysInitMemory` de 154.897.433 e o do segundo `KERNEL SETUP!`, que o bootstrap roda depois de
ler o `SYSTEM.CNF` e antes de carregar o executavel. Nao ha defeito: e o boot normal. Por isso
apagar os EvCB ali e esperado — nada do jogo existia ainda.

O que sobra, agora sem ruido: **o jogo comeca a executar em 164.000.000 e ja em 167.000.000
imprime `VSync: timeout`, 142 vezes ate o fim da medicao.** A mensagem e da libapi ligada ao
proprio executavel: `0x801B8E60` (o hook) e `0x801CF2CC` (o contador) caem dentro de
`0x80125000..0x801CF800`, o intervalo que o BIOS anunciou em `T_ADDR`/`T_SIZE`. O teste afirma
essa continencia por valor.

Ou seja, a cadeia fechada em 0158-0161 continua valendo e agora esta na janela certa: depois do
`Execute !`, a libapi instala o hook, o BIOS religa o auto-ack de IRQ0 no `StartPAD2`, o hook
passa a ver `I_STAT.bit0` ja limpo e o contador de VSync nunca chega a 2.

## Experimento que fecha a cadeia

Em vez de mais uma rodada de inferencia, um experimento: interceptar as chamadas de
`ChangeClearPAD(1)` que acontecem **depois** do `Execute !` e trocar o argumento por 0, deixando
todo o resto igual. Sao duas interceptacoes (passos 164.123.851 e 164.200.128, ambas do kernel).

| | sem intervencao | com o auto-ack desligado |
|---|---:|---:|
| `[0x801CF2CC]` (contador de VSync do jogo) | **1** | **145** |
| `VSync: timeout` no TTY | **142** | **0** |

E a primeira vez que o contador de frames do Rayman anda neste projeto. O teste
`desligar_o_auto_ack_na_religada_faz_o_contador_de_vsync_andar` afirma os tres numeros.

**E nao e corrupcao nossa do BIOS.** Despejei a RAM do kernel em `0x4B80..0x4C10` (o corpo do
`StartPAD2`, onde mora a religada) e procurei a sequencia exata na imagem da ROM: bate byte a
byte no offset `0x14680`. O codigo que religa o auto-ack e o do BIOS de verdade, executado fiel.

Sobra, entao, uma pergunta so, e ela nao e mais sobre a cadeia de excecao: **por que o jogo chama
`ChangeClearPAD(0)` ANTES do `InitPAD2`/`StartPAD2`, se o proprio `StartPAD2` desfaz isso?** Ou o
fluxo real do jogo chama de novo depois — e nao chega la por outra divergencia nossa — ou o
handler de pad, no hardware, nao alcanca o ack nas condicoes em que aqui alcanca. As duas pontas
sao mediveis e nenhuma delas exige adivinhar.
