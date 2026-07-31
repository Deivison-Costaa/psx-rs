<!-- Custo, tokens e duração NÃO entram aqui: são medidos por scripts/oc-iter.ps1 em
     docs/metricas.csv, casados por head_antes/head_depois. -->

# 0115 — sio-portas-16bits

- **Data:** 2026-07-30
- **Item do roadmap:** 4.4j
- **Objetivo:** fazer a resposta do controle chegar ao driver da BIOS, destravando o laço de
  `JOY_STAT.1` em `0x000045C4`.

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| psx-spx | § Peripheral I/O Ports | docs/reference/14-io-map.md |
| psx-spx | § SIO_CTRL | docs/reference/17-sio.md |
| psx-spx | § SIO_STAT | docs/reference/17-sio.md |
| psx-spx | § SIO_TX_DATA | docs/reference/17-sio.md |
| psx-spx | § SIO_MODE | docs/reference/17-sio.md |

O capítulo do SIO **não existia** em `docs/reference/`: a 0091 (item 6.1) o consultou online e
seguiu sem baixar. R1 manda baixar antes de implementar, então esta iteração começou por
acrescentar `serialinterfacessio.md` ao `scripts/fetch-reference-docs.ps1` (capítulo `17-sio.md`,
mesmo SHA pinado — nenhum outro arquivo mudou).

Duas linhas do mapa de I/O decidem o item: `1F80104Ah 2 JOY_CTRL` e `1F801044h 4 JOY_STAT`. São
registradores de 2 e 4 bytes, e o que o driver da BIOS usa é o **byte alto**: § SIO_CTRL põe
`DSR Interrupt Enable` no bit 12 e `SIO0 port select` no bit 13; § SIO_STAT põe
`Interrupt Request` no bit 9. Tudo isso mora em `1F80104Bh` e `1F801045h`.

## Como o item foi encontrado (medição antes de código)

Harness `padwait` (descartável): decodifica todo load/store cujo endereço cai em
`0x1F801040..0x1F80104F`, imprimindo passo, PC, **tamanho do acesso** (do opcode) e valor.
O tamanho era o dado que faltava. Trecho da corrida antes da correção:

```
  passo   89125458  pc=0x0000454C   sh 0x1F80104A  val=0x00001003
  passo   89125466  pc=0x00004584   sb 0x1F801040  val=0x00000001
  passo   89125598  pc=0x00004590  lhu 0x1F80104A  val=0x00000000
  passo   89125603  pc=0x000045A4   sh 0x1F80104A  val=0x00000010   <-- deveria ser 1013h
  passo   89125611  pc=0x000045C4  lhu 0x1F801044  ... (6,1 M vezes)
```

O driver escreve `JOY_CTRL = 1003h` (TXEN | DTR=/CS | DSR-IRQ-Enable) com **`sh`**, e nosso
`write16` decompõe a meia-palavra em duas escritas de byte passando o mesmo `phys` nas duas —
o parâmetro `offset` existia e era usado pelo scratchpad, pelos timers e pelo CD-ROM
(`phys - 0x1F80_1800 + offset`), mas o braço do SIO0 o ignorava. Resultado: `03h` em `104Ah`,
depois `10h` **de novo** em `104Ah`. O `JOY_CTRL` terminava `0010h`: `/CS` solto, TXEN desligado.

A partir daí a cadeia é mecânica: com `/CS` alto, `send_byte` não roda, o RX FIFO fica vazio,
`JOY_STAT.1` nunca sobe, e o laço de `0x000045C4` gira para sempre. O `10h` escrito no passo
89 125 603 é o próprio defeito se realimentando: o driver lê `JOY_CTRL` em `0x00004590` para
reescrevê-lo com o bit de ack, e a leitura de 16 bits tinha o mesmo furo — devolvia `0010h` em
vez de `1003h`, então o `OR` com `10h` deu `0010h` em vez de `1013h`.

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | endereçamento | O handoff da 0114 dizia "resposta do controle no SIO0" e o alvo era `sio.rs`: assumi que faltava modelar o pad ou o /ACK. | § Peripheral I/O Ports: `JOY_CTRL` tem 2 bytes e `JOY_STAT` tem 4. Quem quebra não é o dispositivo, é o decodificador de endereço do barramento. | Medição antes de código. O `padwait` mostrou o `sh 0x1F80104A val=0x1003` seguido de um `sh ... val=0x0010` — valor que a BIOS nunca escreveria. `sio.rs` não foi tocado nesta iteração. |
| 2 | endereçamento | `write16`/`read16` já sabiam quebrar a meia-palavra em bytes, então bastava o dispositivo estar certo. | Nada na spec; o contrato é do nosso próprio `region_write_byte(phys, kseg, offset, val)`. | O braço do SIO0 recebia `offset` e não o usava. Os outros quatro braços do mesmo `match` usam. Um parâmetro ignorado num braço de `match` não é aviso do compilador — é usado nos vizinhos. |
| 3 | flags | A suíte do SIO0 já tinha 9 testes (0091, item 6.1) e estava verde, então o caminho estava coberto. | — | `sio_digital_pad.rs` fala com `Sio::new()` **direto**, sem barramento: `sio.write_ctrl(0x0002)` nunca passa por `write16`. Um subsistema pode ter suíte verde e estar inalcançável pela CPU. Ver invariante 25. |

## Bateria de mutação

Placar da bateria: 6/6 mutantes mortos, 2/2 controles verdes, 0 equivalente — docs/mutantes/0115-sio-portas-16bits.mut

| # | Mutação | Teste que pegou |
|---|---|---|
| m1 | leitura de 16 bits do SIO0 ignora o `offset` (o defeito, lado da leitura) | `read16_de_joy_ctrl_traz_o_byte_alto_certo` |
| m2 | escrita de 16 bits do SIO0 ignora o `offset` (o defeito, lado da escrita) | `write16_em_joy_ctrl_entrega_o_byte_alto_em_0x1f80104b` |
| m3 | `write16` entrega o byte alto no offset 0 — mesma perda um nível acima | `write16_em_joy_mode_entrega_o_byte_alto` |
| m4 | `read16` compõe o byte alto lendo o offset 0 | `read16_de_joy_stat_traz_o_bit9_de_interrupcao_do_byte_alto` |
| m5 | byte alto do `JOY_CTRL` descartado — some o DSR-IRQ-Enable e o slot | `write16_em_joy_ctrl_entrega_o_byte_alto_em_0x1f80104b` |
| m6 | byte alto do `JOY_STAT` lê zero — some o bit 9 | `read16_de_joy_stat_traz_o_bit9_de_interrupcao_do_byte_alto` |
| c1 | soma do offset com operandos trocados (cosmético) | verde |
| c2 | porta calculada numa ligação antes da chamada (cosmético) | verde |

## Placar antes → depois

Workspace: 758 → **766** testes (8 novos em `sio_portas_16bits.rs`), 0 falhas.

Efeito medido no boot da BIOS real (SCPH1001), fora da suíte, com o `padwait`:

| | antes | depois |
|---|---|---|
| acessos aos portos do SIO0 em 120 M passos | 6 174 893 | 2 487 |
| `JOY_CTRL` depois do `sh 1003h` | `0010h` | `1003h` |
| ack em `0x000045A4` | `0010h` (solta o /CS) | `1013h` |
| PC depois do laço de `JOY_STAT` | preso em `0x000045C4` | segue para `0x000045D8` |
| polling dos dois slots (`1003h`/`3003h`) | não acontecia | uma vez por quadro |

Com o disco do Crash Bandicoot injetado (400 M passos), o TTY **passa** do critério de aceitação:
depois de `PS-X Control PAD Driver Ver 3.0` aparecem linhas novas, `GPU timeout:QUE=( 5, 5),...`,
e o PC sai da região do driver de pad e passa a circular pelo driver de GPU do kernel
(`0x800511DC`, `0x80051308`, `0x8005131C`) e por `0x00001C28`. Antes ficava em `0x000045D4` de
90 M até 380 M passos.

## Revisão cruzada (orquestrador)

Sem achados que barrem o merge. O que foi conferido no diff, e não só na suíte:

- **A correção é de duas linhas e não inventa comportamento.** `region_read_byte` e
  `region_write_byte` passaram a somar o `offset` que já recebiam, exatamente como os quatro
  braços vizinhos do mesmo `match`. Nenhuma linha de `sio.rs` mudou.
- **Efeito colateral verificado nos limites.** `read_byte`/`write_byte` do `Sio` casam endereço
  por igualdade e caem em `_ => 0` / `_ => {}` fora da faixa, então `phys + offset` estourando
  para `1F801041h` (byte alto de um `sh` no `JOY_TX_DATA`) ou `1F801050h` (SIO1) é descartado em
  vez de virar acesso a registrador errado. Antes do patch, o `sh` no `JOY_TX_DATA` enviava
  **dois** bytes; agora envia um, como manda § SIO_TX_DATA (bits 8-31 não usados).
- **Buraco conhecido, deixado em aberto de propósito (R4).** Escrita de 32 bits em
  `1F801044h..1F80104Fh` continua caindo no braço-sumidouro de `region_write32`
  (`0x1F80_1041..=0x1F80_105F => true`): um `sw` em `JOY_MODE`/`JOY_CTRL` é engolido em silêncio.
  Não é regressão (era assim antes) e a BIOS usa `sh` no caminho medido. Anotado como 10.48.
- **Preview do RX FIFO não modelado.** § SIO_RX_DATA diz que os bytes 1-3 de uma leitura de
  32 bits são preview das entradas 2-4 do FIFO; devolvemos zero. A própria spec diz que o
  registrador "should only be accessed as an 8-bit register". Fora de escopo.
- **Gates do projeto:** `purity`, `file_size`, `comment_density`, `roadmap_size`, `status_size`,
  `spec_citations`, `mutation_manifest` e `mutation_anchors` verdes.

## Decisões e notas

- **O handoff apontou o dispositivo errado, e isso é o dado.** A 0114 entregou "arquivo-alvo:
  `crates/psx-core/src/sio.rs`" com a evidência certa (o laço de `JOY_STAT.1`) e a conclusão
  errada (que faltava resposta do pad). O pad já respondia desde a 0091; o que faltava era o
  barramento entregar `/CS`. Medir o **tamanho** do acesso, não só o endereço, foi o que separou
  as duas hipóteses — e custou uma linha a mais no harness.
- **Por que a suíte verde não pegou.** Os 9 testes de `sio_digital_pad.rs` instanciam `Sio`
  diretamente. É a segunda vez que um subsistema correto fica inalcançável (a 0114 foi a
  primeira, com o `irq_pending()` sem chamador); virou a invariante 25.
- **Próximo degrau, já medido.** O kernel agora chega ao driver de GPU e imprime
  `GPU timeout:QUE=(n,n),CODE=(0,0,00FFFFFF)` em laço. É o item 4.4k.
