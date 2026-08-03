<!-- Custo, tokens e duração NÃO entram aqui: são medidos por scripts/oc-iter.ps1 em
     docs/metricas.csv, casados por head_antes/head_depois. -->

# 0174 — mdec-spu-oraculo

- **Data:** 2026-08-03
- **Item do roadmap:** 8.1 (fechado), 8.2 parcial (mono), 10.102, 10.103 (novos)
- **Objetivo:** lote C do oráculo de TTY — fechar `mdec/4bit`, `mdec/8bit` e
  `spu/memory-transfer`, e diagnosticar `mdec/step-by-step-log` sem tentar fechá-la.

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| psx-spx | § MDEC Status Register (L80), Control/Reset (L101), DMA (L114-124) | docs/reference/09-mdec.md |
| psx-spx | § MDEC(1) Decode Macroblock(s) (L129-139), MDEC(2) Set Quant Table(s) (L141-149), MDEC(3) Set Scale Table (L151-158) | docs/reference/09-mdec.md |
| psx-spx | § decode_monochrome_macroblock (L182-185), rl_decode_block (L187-206) | docs/reference/09-mdec.md |
| psx-spx | § real_idct_core (L241-267), y_to_mono (L287-296) | docs/reference/09-mdec.md |
| psx-spx | § Monochrome Macroblocks (L395-408), EOB (L458-466) | docs/reference/09-mdec.md |
| psx-spx | § D#_BCR (L61-76): formatos de SyncMode0 vs SyncMode1 | docs/reference/04-dma.md |
| psx-spx | § Commonly used DMA Control Register values (L184-192) | docs/reference/04-dma.md |
| psx-spx | § SPU Control Register SPUCNT (L659-676), SPU Status Register SPUSTAT (L678-697) | docs/reference/08-spu.md |
| psx-spx | § Sound RAM Data Transfer Address/Fifo/Control (L702-739), SPU RAM Manual Write (L741-753), SPU RAM DMA-Write/-Read (L755-773) | docs/reference/08-spu.md |

**Omissão registrada (R1):** a spec não detalha a ordem dos nibbles no empacotamento de
4 bits (§ Monochrome Macroblocks, L395-408 diz só "an 8x8 bitmap"). Usei o gabarito de
hardware real como oráculo: comparando `mdec/8bit/psx.log` (1 byte/pixel) com
`mdec/4bit/psx.log` (mesmo bloco), o nibble de 4 bits é `y_to_mono(pixel) >> 4`, pixel
par no nibble baixo e ímpar no alto — os dois primeiros bytes do dump batem exatamente
com essa regra (`f0` = `0x0 | (0xf<<4)` a partir de `00,ff`).

**Omissão admitida pela própria spec:** `real_idct_core` (L262-264) diz textualmente
"the hardware appears to be working roughly like that, still the results aren't
perfect" — implementei a fórmula exata do texto (nenhuma tentativa de reverse-engineer
o arredondamento real de outro emulador), e por isso `mdec/8bit` não bate byte a byte
mesmo com a decodificação correta (ver Placar abaixo).

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | flags | Bastava passar `self.output_signed` para a função que decide o XOR de `y_to_mono`. | § y_to_mono (L293, `docs/reference/09-mdec.md`): "if unsigned then Y=Y xor 80h" — o parâmetro é o inverso de `output_signed`. | `mdec_decode_8bit_bloco_heart_bate_com_algoritmo_da_spec` falhou com todo pixel em `0x80`/`0x7f` (o valor clampado antes do XOR, sem XOR nenhum). |
| 2 | API-Rust | Compor um registrador de 16 bits do SPU a partir de dois `write8` (caminho genérico `write16` do barramento) era seguro para qualquer porta. | § Sound RAM Data Transfer Fifo (L712-716, `docs/reference/08-spu.md`): a fifo é escrita-apenas; nada garante que ler de volta reflita o que acabou de ser escrito. | `spu_escrita_manual_fica_visivel_na_ram_via_dma_read` leu `"H",0,0,"e","l",0,0,"l"...` — cada `write16` parcial empurrava uma entrada nova na fifo em vez de uma só. |
| 3 | endereçamento | `D#_BCR` sempre tem o formato BS\*BA (blocksize\*blockcount). | § D#_BCR (L61-69, `docs/reference/04-dma.md`): SyncMode=0 usa um único campo BC; só SyncMode=1 usa BS\*BA. | `testDMAWriteToRamSyncMode0`/`testDMAReadToRamSyncMode0` do `spu/memory-transfer.exe` rodaram os 800M passos sem produzir mais nenhuma linha de TTY — a leitura teria escrito ~1 MB a partir de um buffer de 32 bytes na pilha do jogo. |
| 4 | timing | Uma DMA1 que pede mais palavras do que o MDEC decodificou deveria seguir a spec ao pé da letra e tratar BA=0 como `10000h` blocos. | § Data-Out Request (L108-112, `docs/reference/09-mdec.md`): "it gets cleared after reading the first some words of that block" — a DMA real se pauta pela disponibilidade de dado, não por um contador cego. | Medido, não por teste vermelho: `mdec/4bit`/`mdec/8bit` do ps1-tests pedem BS=0x20/BA=0 (`8`ou`16` palavras reais / `32` do bloco), e um motor sem custo por ciclo executaria a wraparound da spec (`0x200000` palavras) instantaneamente, sobrescrevendo a RAM inteira do processo em uma única chamada de `write32`. |

## Bateria de mutação

Placar da bateria: 6/6 mutantes mortos, 2/2 controles verdes, 0 equivalente —
`docs/mutantes/0174-mdec-spu-oraculo.mut` / `.resultado`.

| Mutante | O que muda | Teste assassino |
|---|---|---|
| m1 | remove a inversão de `output_signed` antes de `y_to_mono` (erro #1 acima) | `mdec_macroblock_decode` |
| m2 | troca a ordem dos nibbles no empacotamento de 4 bits | `mdec_macroblock_decode` |
| m3 | máscara do XOR unsigned vira `0x7F` em vez de `0x80` | `mdec_macroblock_decode` |
| m4 | `rl_decode_block` para de pular a própria entrada ao avançar `k` | `mdec_macroblock_decode` |
| m5 | `total_words` trata SyncMode0 como SyncMode1 (erro #3 acima) | `spu_memory_transfer` |
| m6 | `write_ram_halfword` grava os bytes do SPU em ordem trocada | `spu_memory_transfer` |
| c1 (controle) | `Current Block` do status do MDEC muda de 4 para 5 — nenhum teste lê esse campo | `mdec_registers_dma` |
| c2 (controle) | `current_address` inicial do SPU muda de 0 para 4 — sempre sobrescrito antes de usar | `spu_memory_transfer` |

## Placar antes → depois

Workspace: 953 → 966 testes (+13: 6 em `mdec_registers_dma`, 2 em `mdec_macroblock_decode`,
5 em `spu_memory_transfer`).

K/M do oráculo de TTY (K linhas divergentes de M; `diff` alinhado na âncora, por hunk —
mesmo método do handoff do lote), `--max-steps 800000000`:

| Suíte | Antes | Depois | Nota |
|---|---|---|---|
| `mdec/4bit` | 11/19 | **9/19** | `readDecodedDma` e o hexdump ainda divergem (ver Decisões); 2 das 8 linhas de hexdump agora batem byte a byte. |
| `mdec/8bit` | 11/19 | 11/19 | Bytes decodificados mais próximos do gabarito (ex.: `00 ff ff 00 ff ff 06 00` vs. `...04 00`), mas nenhuma das 8 linhas de 8 bytes bate inteira — `diff` por linha não pontua isso. |
| `spu/memory-transfer` | 9/11 | **7/11** | `testDMAWriteToRamSyncMode0`/`testDMAReadToRamSyncMode0` agora aparecem e passam (o gabarito é de um build anterior a essas duas). Resta `testDMAWriteTiming`/`testDMAReadTiming` — item 10.103. |
| `mdec/step-by-step-log` | 1662/1665 (contagem do handoff) | 1524/1665 (algoritmo de `scripts/lib/tty-veredito.ps1`, ver nota) | Não fechada, por instrução do lote. 1ª divergência real: item 10.102. |

`mdec/step-by-step-log`: os métodos de contagem não são diretamente comparáveis (o
handoff não registrou como os `1662/1665` foram medidos). Recontei com o algoritmo
canônico do projeto (`scripts/lib/tty-veredito.ps1`, replicado em Python só para medir,
sem tocar hardware): `python3 <script> real.txt psx.log` → `difere 1524/1665`.

## Revisão cruzada (orquestrador)

<!-- Preenchido pelo Claude na revisão do PR: achados no formato de docs/prompts/review.md, ou "sem achados". -->

## Decisões e notas

- **R4 dobrado nesta rodada, por decisão do usuário**: o lote fecha MDEC 8.1 completo,
  a metade mono do 8.2, e a infraestrutura de RAM/DMA do SPU (itens 7.1-7.4 ainda não
  tocam vozes/ADPCM) numa única iteração — o motivo é o custo de espera de suíte e CI,
  não o tamanho do código.
- **`mdec_readDecodedDma` em `mdec/4bit`/`mdec/8bit` é irrecuperável nesta rodada**: o
  `psx.log` bundlado no release `build-158` (pinado por `scripts/fetch-test-exes.ps1`) foi
  gerado por um `.exe` de ANTES do commit `40d2e3d` do próprio `ps1-tests`
  ("mdec/frame: added missing dma tests", 2021-02-06), que mudou `BS` de `0x08` para
  `0x20` em `mdec_readDecodedDma`. Com `BS=0x08` (gabarito), `bytes/4 / BS` dá um
  bloco exato (`8/8=1`); com `BS=0x20` (nosso `.exe` real, `build-158`), a mesma conta
  dá `8/32=0` — um `BA=0` que a spec manda tratar como `10000h` blocos
  (`docs/reference/04-dma.md` L76). Isso não é um defeito nosso: é o próprio binário do
  `ps1-tests` pedindo mais dado do que existe. O texto do `blockSize` impresso
  (`0x20` vs. o `0x8` do gabarito) e o endereço do buffer (dependente de pilha) NUNCA
  vão bater, com qualquer implementação correta de MDEC/DMA de nossa parte.
- **Decisão de segurança em `try_execute_dma1`**: em vez de seguir a wraparound
  `BA=0 → 10000h` ao pé da letra (o que, num motor sem custo por ciclo, escreveria
  `0x20 * 0x10000` = 2097152 palavras = 8 MB num processo com 2 MB de RAM, numa
  única chamada síncrona — sobrescrevendo o próprio código/pilha em execução), o
  canal transfere no máximo `mdec.output_len()` palavras e só marca `completed`
  quando esse total bate com o pedido. Isso é uma aproximação da spec (L108-112: a
  requisição de dado é pautada por disponibilidade, não por contagem cega) dentro das
  limitações do nosso motor de DMA atual (execução atômica, sem custo por ciclo) — não
  é uma hipótese de hardware, é uma escolha de engenharia registrada aqui.
- **`mdec/step-by-step-log` não foi fechada, por instrução do lote.** A 1ª divergência
  real (linha 9, após `mdec_reset...ok`) é `mdec_quantTable(addr=0x8001364c,...)` contra
  o gabarito `addr=0x80013ba4` — mesma família do problema acima (endereço/layout de
  build diferente do `.exe` que gerou o `psx.log`), não uma cadeia de decodificação
  errada. A partir da 1ª linha de `MDEC_STATUS`, o teste usa `colorDepth=3` (15bpp,
  caminho `yuv_to_rgb`) que não foi implementado nesta rodada (R5: sem suite do lote
  exercitando cor) — registrado como ROADMAP 10.102.
- **`testDMAWriteTiming`/`testDMAReadTiming` de `spu/memory-transfer` não fecham nesta
  rodada.** Exigem que a DMA leve mais de uma iteração do laço de polling do jogo para
  completar (`transferFinishedImmediately == false`) — nosso motor de DMA (como o de
  GPU/CDROM/OTC já existente) executa a transferência inteira de forma síncrona dentro
  da própria escrita do `CHCR`, sem custo por ciclo. Corrigir isso exigiria um DMA
  dirigido pelo `scheduler` (R2), o que é maior do que uma correção pontual — registrado
  como ROADMAP 10.103.
- **Cor (15/24bpp, `yuv_to_rgb`) do MDEC(1) fica para uma próxima iteração** (ROADMAP
  8.2 permanece aberto para essa metade): nenhuma suite deste lote exercita esse
  caminho, e R5 proíbe implementar sem teste vermelho primeiro.
