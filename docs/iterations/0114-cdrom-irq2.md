<!-- Custo, tokens e duração NÃO entram aqui: são medidos por scripts/oc-iter.ps1 em
     docs/metricas.csv, casados por head_antes/head_depois. -->

# 0114 — cdrom-irq2

- **Data:** 2026-07-30
- **Item do roadmap:** 4.4i
- **Objetivo:** entregar a interrupção do drive de CD-ROM ao `I_STAT` (IRQ2), por borda, para o
  boot da BIOS sair do logo e seguir a inicialização.

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| psx-spx | § `0x1f801803` (read, banks 0 and 2): `HINTMSK` | docs/reference/06-cdrom.md |
| psx-spx | § `0x1f801803` (write, bank 1): `HCLRCTL` | docs/reference/06-cdrom.md |
| psx-spx | § `0x1f801803` (read, banks 1 and 3): `HINTSTS` | docs/reference/06-cdrom.md |
| psx-spx | § Interrupt Request / Execution | docs/reference/11-interrupts.md |
| psx-spx | § Interrupt Acknowledge | docs/reference/11-interrupts.md |

Duas frases decidiram a implementação inteira. Do § HINTMSK: *"The CD-ROM drive fires an
interrupt whenever (HINTMSK & HINTSTS) is non-zero"* — a condição é exatamente o
`irq_pending()` que já existia no módulo e que ninguém lia. Do § Interrupt Request / Execution:
*"The interrupt request bits in I_STAT are edge-triggered, ie. they get set ONLY if the
corresponding interrupt source changes from 'false to true'"* — é o que separa a implementação
certa da errada, e o que a bateria de mutação mede em m2.

## Como o item foi encontrado (medição antes de código)

O handoff da 0113 listava três candidatos para o que a BIOS espera depois do logo, em ordem:
CD-ROM, timer, campo par/ímpar do GPUSTAT.31. A medição decidiu entre eles antes de qualquer
linha de implementação, com o harness `shellwait` (histograma de PC + decodificação de todo
load/store cujo endereço cai em `0x1F801800..3`):

- **Os dois laços quentes não eram o bloqueio.** O maior (`0x80059DA4`) espera o contador de
  frames em `0x80079D9C`, escrito 429 vezes pelo callback de VSync em `0x8005A5E0` — ou seja, a
  base de tempo funciona. O segundo (`0x80059D54`) espera GPUSTAT.31 mudar; o bit alterna no
  `VBLANK_EXIT`. Ambos são laços com orçamento de 0x8000 giros e saída por timeout.
- **O CD-ROM parava seco.** Foram exatamente 10 acessos aos portos em 300 M passos, o último no
  passo 86 927 369: `HINTMSK=1Fh`, depois `Test(19h)` com parâmetro `20h`. Depois disso, nada.
  E `cdrom.irq_pending()` ficava `true` para sempre: a fonte estava pedindo interrupção e não
  havia um só `raise(2)` no repositório (`grep` por `raise(` achava só os bits 0, 7 e os timers).
- **Confirmação por experimento, revertido em seguida.** Um `irq.raise(2)` provisório em
  `tick_timers` levou o boot de 10 para 43 acessos ao CD e fez o TTY imprimir
  `System Controller ROM Version 97/01/10 c2` (a resposta do próprio `Test(20h)`) e
  `PS-X Control PAD Driver Ver 3.0`. Hipótese confirmada; o patch foi jogado fora e a
  implementação de verdade começou pelo teste.

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | flags | Bastava `raise(2)` enquanto `(HINTMSK & HINTSTS)` fosse não-zero — foi o que o experimento de confirmação fez. | § Interrupt Request / Execution: os bits do `I_STAT` são de **borda**, setados só na transição false→true da fonte. Por nível, o kernel que segue a ordem de ack da spec (primeiro `I_STAT`, depois o porto) tomaria um IRQ novo no mesmo instante. | Escrito como teste antes de implementar (`i_stat_bit2_e_de_borda_nao_volta_sozinho_sem_hclrctl`); o mutante m2 é justamente a versão por nível e morre nesse teste. |
| 2 | flags | Com a borda modelada, a segunda resposta de um comando (Init 0Ah → INT3 → INT2) apareceria sozinha. | § HCLRCTL: depois do ack, *"the result FIFO is drained and if there's been a pending command, then that command gets send to the controller"* — o ack **baixa** a linha, e só então a segunda resposta a levanta de novo. | Vermelho real: 6 dos 7 testes passaram na primeira implementação e `segunda_resposta_do_init_tambem_levanta_irq2` falhou. Como `write8` limpava o `HINTSTS` e entregava a segunda resposta dentro da mesma chamada, a borda ficava invisível na fronteira. Corrigido baixando a linha no próprio ack. |
| 3 | flags | O ack do `HCLRCTL` sempre derruba a linha. | § HCLRCTL: `INTSTS` é um valor de 3 bits, e um ack parcial (escrever `01h` sobre INT3) vira INT2 — a fonte continua não-zero. | **Mutante sobrevivente.** A primeira bateria deu 5/6: m6 (`irq_line.set(false)` em vez de recalcular) sobreviveu porque nenhum teste fazia ack parcial. Teste `ack_parcial_do_hclrctl_deixa_a_fonte_alta_e_nao_gera_borda_nova` acrescentado, bateria refeita: 6/6. |

## Bateria de mutação

Placar da bateria: 6/6 mutantes mortos, 2/2 controles verdes, 0 equivalente — docs/mutantes/0114-cdrom-irq2.mut

| # | Mutação | Teste que pegou |
|---|---|---|
| m1 | `take_irq2_edge` sempre `false` (o defeito 4.4i original) | `hintmsk_e_hintsts_nao_zero_levanta_i_stat_bit2` |
| m2 | IRQ por nível em vez de borda | `i_stat_bit2_e_de_borda_nao_volta_sozinho_sem_hclrctl` |
| m3 | escrita no `HINTMSK` não habilita nada | `hintmsk_e_hintsts_nao_zero_levanta_i_stat_bit2` |
| m4 | ack do `HCLRCTL` não baixa a linha | `segunda_resposta_do_init_tambem_levanta_irq2` |
| m5 | linha memorizada presa em `true` | `nova_borda_depois_do_ack_do_hclrctl_levanta_de_novo` |
| m6 | ack do `HCLRCTL` baixa a linha sem olhar os bits restantes | `ack_parcial_do_hclrctl_deixa_a_fonte_alta_e_nao_gera_borda_nova` |
| c1 | nível calculado em duas etapas (cosmético) | verde |
| c2 | máscara de 3 bits em binário (cosmético) | verde |

## Placar antes → depois

Workspace: 750 → **758** testes (8 novos em `cdrom_irq2.rs`), 0 falhas.

Efeito medido no boot da BIOS real (SCPH1001), fora da suíte, com o harness `shellwait`:

| | antes | depois |
|---|---|---|
| acessos aos portos do CD-ROM | 10 | 43 |
| último acesso ao CD | passo 86 927 369 | passo 87 402 755 |
| TTY final | `ResetCallback: _96_remove ..` | `PS-X Control PAD Driver Ver 3.0` |
| onde para | laço de VSync em `0x80059DCC` | `0x000045C4`, esperando `JOY_STAT` |

## Revisão cruzada (orquestrador)

<!-- Preenchido pelo Claude na revisão do PR: achados no formato de docs/prompts/review.md, ou "sem achados". -->

## Decisões e notas

- **Onde o serviço é chamado.** Copiei o padrão que já existia para o SIO (`service_sio_irq`,
  chamado depois de cada escrita nos portos): `service_cdrom_irq` roda depois de toda escrita em
  `0x1F801800..3`, nos dois caminhos (`region_write32` e `region_write_byte` — `write16` passa
  por este último). `HINTSTS` só muda em `send_command` e `deliver_second`, e ambos são
  alcançados por escrita em porto, então não existe borda fora desses pontos. Não foi para o
  `tick_timers` de propósito: ali seria uma varredura por tempo, não um evento (R2).
- **`irq_pending()` já existia e já estava certo.** O bug não era de cálculo, era de fiação: o
  módulo sabia dizer que queria interromper e ninguém perguntava. Vale como padrão de busca —
  um `pub fn` sem chamador em `psx-core` é candidato a subsistema desligado.
- **O que a medição derrubou.** Os dois outros candidatos do handoff da 0113 (timer e
  GPUSTAT.31) estão refutados como causa do boot travado, mas a medição deixou um dado a
  registrar: os dois laços de espera da BIOS têm orçamento de 0x8000 giros e o nosso frame gasta
  ~230 k passos, então eles frequentemente saem por **timeout** em vez de por sucesso. É folga
  de tempo, não bloqueio; anotado como candidato 10.47.
- **Próximo degrau, já medido.** O boot agora para em `0x000045C4`, um laço
  `lhu $t4,4($s1) / andi 2 / beq` com `$s1 = 0x1F801040`: espera `JOY_STAT.1` (RX FIFO não
  vazia), a resposta do controle. É o item 4.4j.
