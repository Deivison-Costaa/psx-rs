<!-- Custo, tokens e duração NÃO entram aqui: são medidos por scripts/oc-iter.ps1 em
     docs/metricas.csv, casados por head_antes/head_depois. -->

# 0137 — diagnostico-streaming

- **Data:** 2026-08-01
- **Item do roadmap:** 4.5 (novo item-pai: 1º frame do jogo)
- **Objetivo:** nomear, por medição, o congelamento do Crash pós-boot (jogo roda, carrega
  954KB de WAD, não desenha nenhum frame).

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| psx-spx | edge-trigger L46-47; ordem de ack L57-66; CAUSE.b10 não-latch L74-81 | docs/reference/11-interrupts.md |
| psx-spx | modos/target/IRQ dos timers (arquivo inteiro, 101 linhas) | docs/reference/05-timers.md |

## Método (diagnóstico puro; sondas descartáveis nunca commitadas)

Cadeia de 9 medições: histograma de comandos de CD emitidos; contagem de INT1/INT2
entregues; dump de I_STAT/I_MASK e dos registradores do Timer 2 no estado travado; dump
da fila de streaming do jogo (0x8005C91C); disasm do scanner e do laço de espera a partir
da RAM em runtime (o EXE é realocado — o arquivo contém "PS-X" onde a RAM tem código);
sonda de $ra no printf da BIOS; sonda de chamadas C0 do kernel; sonda de acks do bit 6.
Painel multi-agente (3 análises independentes + juiz adversarial) escolheu a causa e
desenhou o experimento decisivo entre cada rodada de sondas.

## Hipóteses refutadas (todas por medição, nenhuma por opinião)

| # | Hipótese | Refutação |
|---|---|---|
| 1 | comandos de CD faltantes | o jogo só usa comandos que temos; os 2 que caem no braço genérico (GetStat, Demute) recebem a resposta certa por coincidência |
| 2 | IRQ preso re-entrando handler | I_STAT=0x0000 no estado travado |
| 3 | Timer 2 morto | mode=0x1E58/target=0x1000 decodificados; eventos F2000002 na cadência exata (~32k ciclos) do início ao fim |
| 4 | VBlank do kernel não entregue ao jogo | 3.602 DeliverEvent classe F2000003 até o fim do run |
| 5 | leitura de CD corrompendo/travando | 477 INT1 + 35 INT2 limpos; última requisição fecha com Pause+INT2; o jogo PARA de pedir por conta própria |
| 6 | GPU/DrawSync (bits 26/28/31) | o laço travado não toca MMIO de GPU/DMA; bit31 toggla no evento VBLANK |
| 7 | lhu 16-bit de I_STAT no sumidouro | caminhos 16-bit do IRQ existem (bus.rs); o próprio jogo lê I_MASK=0x4D por lhu |
| 8 | vetorização ignorando SR.IEc | cpu.rs:64 checa IEc+IM+IP corretamente |

## Mecanismo medido do congelamento

1. A fila de streaming do jogo (0x8005C91C) tem 38 páginas em estado 1 ("carregada");
   o scanner (0x800135A4+, disasm no dossiê) só consome estado 2; a promoção 1→2 nunca
   roda.
2. O `intr timeout(0040:004d)` sai de $ra=0x8003EAF8 — o WaitIntr do runtime do jogo,
   que faz poll CRU de `0x40 & I_STAT & I_MASK` (bit 6, TMR2) com orçamento de 0x800
   tentativas (disasm: slti com 0x801 em 0x8003EACC).
3. O jogo ENFILEIROU dois handlers próprios com prioridade 0 na cadeia de interrupção do
   kernel (SysEnqIntRP, elementos 0x80140004 e 0x80140014) — no hardware real eles
   reivindicam o TMR2 antes do handler do kernel — e DEPOIS OS REMOVEU (SysDeqIntRP de
   0x14 e de um 0x24 que nunca entrou no log): assinatura de init que falhou e foi
   desfeito (rollback).
4. Sem os handlers do jogo, o kernel acka o bit 6 sozinho (221.520 acks com o padrão
   `0xFFFFFFBF`) e o poll do jogo nunca vê o bit — congelamento permanente.

## Veredito do painel (juiz adversarial)

Causa mais provável do gatilho do rollback: laço de espera com orçamento fixo de
iterações perdendo a corrida porque o modelo de ciclos subcusta instruções fora da guarda
de loads — a MESMA classe do "VSync: timeout" da BIOS consertado na 0104; a guarda de
custo em cpu.rs:187 cobre só os opcodes 0x20-0x26 (LWC2/SWC2 pagam 1 ciclo; dívida 10.45
já registrada). A confirmar na próxima iteração ANTES de codificar.

## Próxima tarefa concreta (handoff)

Dump da estrutura dos elementos 0x80140004/0x14/0x24 (verifier/handler do jogo) + sonda
no chain walk do kernel (quem é chamado, o que o verifier lê, por que devolve "não é
meu") durante a janela do init do LIBSN. Se confirmar a corrida de timing: goldens de
custo por instrução no padrão da 0104 (valor citado de docs/reference/02-cpu.md, nunca
ajustado ao sintoma) + gate `intr timeout: 2→0` + implementação do trabalhador
(gpt-5.6-luna em effort max, já configurado).

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | extracao-spec | disasm estático do EXE serviria para os PCs amostrados | o Crash realoca código em runtime; o arquivo tem "PS-X" onde a RAM tem o laço | dump da RAM em runtime divergiu do arquivo |
| 2 | timing | primeira leitura do painel apontou VSync/IRQ0 como muro | o VSync timeout é único e não recorre; o congelamento real é o rollback do init + poll órfão do TMR2 | sondas de $ra e C0 |

## Bateria de mutação

Bateria de mutação: não se aplica — zero linhas de código de produção nesta iteração;
diagnóstico puro com sondas descartáveis revertidas antes do commit (precedentes: 0133 e
0135); nada mutável foi introduzido.

## Placar antes → depois

861 → 861 (sem mudança).

## Revisão cruzada (orquestrador)

O risco de diagnóstico é afirmar sem executar: cada linha do mecanismo acima tem a sonda
correspondente nos arquivos do scratchpad (cdcmd.txt, fila-dump.txt, code-dump.txt,
printf-ra.txt, c0-ack.txt), e as três sondas de código foram revertidas (git status limpo
antes deste commit). A sentença do painel está preservada no journal do workflow.

## Decisões e notas

- Item-pai NOVO e nomeado (4.5) em vez de sub-item reativo — regra do plano de saída.
- O custo por rodada de sonda (~5 min de boot 600M) dominou o dia; a dívida "harness de
  medição com janela de step para trace" fica anotada para o relatório.
