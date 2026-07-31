<!-- Custo, tokens e duração NÃO entram aqui: são medidos por scripts/oc-iter.ps1 em
     docs/metricas.csv, casados por head_antes/head_depois. -->

# 0116 — dma-dicr-irq3

- **Data:** 2026-07-30
- **Item do roadmap:** 4.4k
- **Objetivo:** entregar a interrupção de conclusão do DMA (IRQ3) ao `I_STAT`, modelando o `DICR`
  com flags de conclusão e flag mestre calculado.

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| psx-spx | § DICR - DMA Interrupt Register | docs/reference/04-dma.md |
| psx-spx | § DMA Channel Control | docs/reference/04-dma.md |
| psx-spx | § Interrupt Request / Execution | docs/reference/11-interrupts.md |

Três frases do § DICR definem o registrador inteiro. Os flags 24-30 *"are set ONLY if BOTH bit
(16+n) AND bit 23 are enabled (unlike interrupt flags in I_STAT, which are always set regardless
of whether the respective IRQ is masked)"*. O bit 31 *"is a simple readonly flag that is
recalculated on every write"*, por `IF b15=1 OR (b23=1 AND b(24-30)>0)` — e as máscaras por canal
**não** entram nessa conta. E: *"Upon 0-to-1 transition of Bit 31, the IRQ3 flag in I_STAT gets
set"*, que é a mesma regra de borda da invariante 24.

## Como o item foi encontrado (medição antes de código)

Harness `dmawait` (descartável), rodando BIOS + disco e registrando todo acesso a
`0x1F801080..0x1F8010FF` com o tamanho decodificado do opcode. O que ele mostrou, na janela em
que o kernel imprime `GPU timeout`:

| | valor medido |
|---|---|
| `I_MASK` | `0x0000000D` — bits 0 (VBLANK), 2 (CDROM) e **3 (DMA)** habilitados |
| escritas no `DICR` | **3** em 200 M passos, a última `0x88840000` (master b23 + máscara do canal 2 + ack do canal 3) |
| `I_STAT` ao fim | `0x00000000` |
| acessos ao `D2_CHCR` | 2462 — o DMA da GPU roda, em modo linked-list (`CHCR=0x01000401`) |

Ou seja: o kernel liga o IRQ3 no `I_MASK`, arma o `DICR` com master e máscara de canal, dispara
transferências que **completam** — e nunca recebe interrupção. `write_dicr` guardava o valor cru
(`self.dicr = val`) e não existia um só `raise(3)` no repositório. É a terceira ocorrência da
invariante 24 (4.4d/I_MASK, 4.4i/IRQ2 e agora esta).

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | flags | O `DICR` era um registrador comum: guardar o valor escrito bastava (era o que `write_dicr` fazia desde a 4.3a). | § DICR: os bits 24-30 são "write 1 to reset", o bit 31 é somente-leitura e **recalculado a cada escrita**, e os bits 7-14 não existem. Escrever o valor cru inventa um `b31` gravável e transforma o ack em set. | Teste escrito antes; o mutante m5 é exatamente a versão em que escrever 1 seta em vez de reconhecer. |
| 2 | flags | Com as máscaras por canal desligadas depois de uma conclusão, o `b31` cairia junto. | § DICR: *"the per-channel enable bits (b16-22) do not factor into the bit 31 calculation... Once a flag bit is set, it contributes to the master flag regardless of whether the channel enable is still on."* | Escrito como teste (`bit31_nao_olha_as_mascaras_por_canal`) a partir da leitura da spec, antes de implementar — foi o único ponto em que a spec contrariou a intuição sem custar retrabalho. |
| 3 | **diagnóstico** | Que fechar este buraco faria o `GPU timeout` parar. Escrevi isso no handoff da 0115 como "candidato medido, não confirmado" — e mesmo assim era a expectativa. | — | **A medição depois do patch refutou.** O IRQ3 passou a ser entregue e o handler do kernel roda (escritas no `DICR`: 3 → 508; acessos ao porto: 5 → 7063), mas o TTY continua repetindo `GPU timeout`. O defeito era real; não era a causa. Ver invariante 26. |

## Bateria de mutação

Placar da bateria: 7/7 mutantes mortos, 2/2 controles verdes, 0 equivalente — docs/mutantes/0116-dma-dicr-irq3.mut

| # | Mutação | Teste que pegou |
|---|---|---|
| m1 | conclusão de canal não marca flag nenhum (o defeito original) | `flag_do_canal_sobe_com_mascara_e_master_ligados` |
| m2 | flag ignora a máscara por canal (b16-22) | `flag_nao_sobe_sem_a_mascara_do_canal` |
| m3 | flag ignora o master enable (b23) | `flag_nao_sobe_sem_o_master_do_bit23` |
| m4 | bus error (b15) deixa de forçar o flag mestre | `bit15_de_bus_error_forca_o_bit31` |
| m5 | escrever 1 nos bits 24-30 seta em vez de reconhecer | `escrever_1_no_flag_limpa_o_flag_e_derruba_o_bit31` |
| m6 | IRQ3 por nível em vez de borda do bit 31 | `i_stat_bit3_e_de_borda_e_nao_volta_sozinho` |
| m7 | linha de IRQ3 nunca sobe | `borda_de_0_para_1_do_bit31_levanta_i_stat_bit3` |
| c1 | condição do mestre numa ligação antes do `if` (cosmético) | verde |
| c2 | máscara dos flags como deslocamento (cosmético) | verde |

## Placar antes → depois

Workspace: 766 → **775** testes (9 novos em `dma_dicr_irq3.rs`), 0 falhas.

Efeito medido no boot real (SCPH1001 + disco), fora da suíte, com o `dmawait`:

| | antes | depois |
|---|---|---|
| acessos ao porto `DICR` | 5 | 7063 |
| escritas no `DICR` | 3 | 508 |
| `DICR` ao fim da corrida | `0x88840000` (valor cru guardado) | `0x84840000` (b31 e flag do canal 2 calculados) |
| handler de DMA do kernel | nunca rodou | roda, e reconhece o flag a cada conclusão |

**Critério de aceitação do item: NÃO cumprido.** O 4.4k pedia que o TTY parasse de repetir
`GPU timeout`; ele continua. O que a iteração entrega é o `DICR` correto e o IRQ3 chegando ao
kernel — necessário, e comprovadamente não suficiente.

## Revisão cruzada (orquestrador)

Sem achados que barrem o merge, com uma ressalva de escopo declarada acima.

- **Máscara de escrita conferida bit a bit.** Graváveis: 0-6 (`0x7F`), 15 (`0x8000`), 16-22
  (`0x7F0000`), 23 (`0x800000`) — `0x00FF807F`. Os bits 7-14 e 31 não são graváveis; os 24-30
  entram só pela via do ack. A primeira versão da máscara saiu `0x00FF803F` (perdia o bit 6, o
  controle de slice do canal 6) e foi corrigida antes do commit.
- **Onde o serviço é chamado.** `service_dma_irq` roda depois de escrita em `0x1F801080..EC`
  (onde o `CHCR` dispara a transferência que completa) e depois de escrita no `DICR` (onde o ack
  derruba o `b31` e permite a próxima borda). Mesmo padrão de `service_sio_irq`/`service_cdrom_irq`;
  fora do `tick_timers` de propósito (seria varredura por tempo, não evento — R2).
- **A borda é a mesma regra da 0114.** `take_irq3_edge` memoriza o nível anterior; o ack do `DICR`
  recalcula o `b31` na hora, então a conclusão seguinte produz borda nova. É o teste
  `nova_conclusao_depois_do_ack_levanta_irq3_de_novo`.
- **Buraco conhecido, deixado aberto (R4).** O bit 15 (bus error) é gravável mas nada no emulador
  o **levanta**: transferir para fora da RAM hoje é silenciosamente ignorado (`if offset + 4 <=
  ram.len()`). O bit existe e é lido corretamente; quem o produz não. Anotado como 10.49.
- **Gates do projeto:** `purity`, `file_size`, `comment_density`, `roadmap_size`, `status_size`,
  `spec_citations`, `mutation_manifest`, `mutation_anchors` e `mutation_battery` verdes.

## Decisões e notas

- **Hipótese confirmada como defeito, refutada como causa.** O handoff da 0115 dizia
  "candidato medido, NAO confirmado: nao existe um so `raise(3)`". Estava certo sobre o defeito e
  errado sobre o efeito. Registrar isso importa mais do que o patch: o padrão da invariante 24
  ("`pub fn` sem chamador = subsistema desligado") acha buracos reais com facilidade, e por isso
  mesmo tenta se vender como explicação do sintoma que estava sendo investigado. Virou a
  invariante 26.
- **O que sobra medido para o próximo degrau.** Ao fim da corrida, `GPUSTAT = 0x184E260A`:
  bit 28 (pronto para bloco de DMA) = 1, bit 27 = 1, e **bit 26 (Ready to receive Cmd Word) = 0**.
  O `gpu.rs` abaixa o bit 26 enquanto um comando espera parâmetros e o levanta ao completar, o que
  faz do 26 preso em zero a hipótese barata: algum comando entrou pelo linked-list e ficou faminto
  por parâmetros que nunca vieram, e o driver da GPU espera esse bit antes de enviar. **Não
  confirmado** — é o item 4.4l, e a primeira coisa a fazer nele é medir qual comando ficou pendente,
  não corrigir o parser por intuição.
