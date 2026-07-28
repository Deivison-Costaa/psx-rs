# 0018 — LWL/LWR/SWL/SWR (segunda tentativa)

- **Data:** 2026-07-27
- **Item do roadmap:** 1.7
- **Objetivo:** Implementar LWL/LWR/SWL/SWR com vias de byte corretas (deslocamento, não máscara) e merge via `load_delay` para o idioma do par LWL+LWR.

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| psx-spx | Unaligned Load/Store + Unaligned Load/Store (Details) | docs/reference/02-cpu.md L240, L257 |

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | endereçamento | Que `[N*4+0]` na tabela da spec é a parte alta da palavra (big-endian mental). | `[N*4+0]` é endereço de byte; em LE, é o byte menos significativo da palavra. LWL no endereço alinhado põe o byte baixo da memória em `rt[31:24]`. | Teste de aceitação `lwl_offset_0_carrega_8bits_superiores` com golden value derivado da spec (0xDD→rt[31:24]=0xDDFFFFFF), não da implementação. *(Corrigido em 2026-07-27: o doc original grafava `lwl_k0_transfere_byte_alto_no_msb` — nome que nunca existiu no arquivo — e o golden value `0xDDFFFFFFFF` com um F extra, erro de digitação.)* |
| 2 | delay-slot | Que LWL e LWR seguem o load delay normal (valor pendente não disponível). | LWL e LWR enxergam um ao outro sem delay — a spec documenta o idioma "no delay required between these (although both access r2)". | Teste `lwl_seguido_de_lwr_no_mesmo_registrador_sem_nop_entre_eles`: rt começa em 0, sem o merge via `load_delay` o LWR usa rt=0 e a contribuição do LWL some. *(Corrigido em 2026-07-27: o doc original grafava `lwl_lwr_enxergam_um_ao_outro_sem_delay` — nome que nunca existiu no arquivo.)* |

A PR #27 (primeira tentativa, rejeitada) sofria dos dois defeitos. Os testes passavam porque codificavam o mesmo modelo errado da implementação. Nesta segunda tentativa, os golden values vêm da tabela derivada no handoff do STATUS, ancorada em bytes literais.

## Bateria de mutação

Placar: **7/7 mutantes pegos, 1/1 controles verdes.**

| # | Tipo | Mutação | Teste que pegou |
|---|---|---|---|
| 1 | erro | LWL k=0: `(word & 0xFF) << 24` → `word >> 24` (byte errado) | `lwl_offset_0_carrega_8bits_superiores` |
| 2 | erro | LWR k=1: `word >> 8` → `word >> 9` (shift errado) | `lwr_offset_1_carrega_24bits_inferiores` |
| 3 | erro | SWL k=1: `val >> 16` → `val >> 8` (byte errado de rt) | `swl_offset_1_armazena_16bits_superiores` |
| 4 | erro | SWR k=1: `val << 8` → `val << 16` (shift errado) | `swr_offset_1_armazena_24bits_inferiores` |
| 5 | erro | `reg_with_pending` ignora load_delay (sempre retorna `self.regs`) | `lwl_seguido_de_lwr_no_mesmo_registrador_sem_nop_entre_eles` |
| 6 | erro | LWL k=1: máscara `0x0000_FFFF` → `0xFFFF_0000` (máscara invertida) | `lwl_offset_1_carrega_16bits_superiores` |
| 7 | erro | SWR k=2: `val << 16` → `val << 24` (shift errado) | `swr_offset_2_armazena_16bits_inferiores` |
| C1 | controle | Renomear `aligned` → `base_addr` em LWL | Nenhum (verde) |

*(Corrigido em 2026-07-27: os nomes originais da tabela — `lwl_k0_transfere_byte_alto_no_msb`, `lwr_k1_transfere_tres_bytes_baixos`, `swl_k1_escreve_dois_bytes_altos`, `swr_k1_escreve_tres_bytes_baixos`, `lwl_lwr_enxergam_um_ao_outro_sem_delay`, `lwl_k1_transfere_dois_bytes_altos`, `swr_k2_escreve_dois_bytes_baixos` — não existiam no arquivo de teste. A bateria foi re-executada com mutações reais no fonte e os nomes acima são os que de fato pegaram cada mutante.)*

## Placar antes → depois

Workspace: **178** testes (151 anteriores + 27 de unaligned load/store). Meta-testes: 10.

## Revisão cruzada (orquestrador)

<!-- Preenchido pelo Claude na revisão do PR -->

## Decisões e notas

1. **`reg_with_pending`**: método novo em `Cpu` que consulta `load_delay` antes de ler o registrador. LWL e LWR usam esse método para ler `rt` durante o merge, permitindo que LWR enxergue o valor pendente do LWL no mesmo registrador. SWL e SWR usam `reg()` normal (stores não têm delay slot).
2. **Força-alinhamento**: Os quatro opcodes forçam alinhamento do endereço (`addr & !3`), como documentado na spec. O offset (`addr & 3`) seleciona quantos bytes transferir.
3. **Read-modify-write nos stores**: SWL e SWR leem a palavra alinhada, mascaram os bytes a escrever e fazem `write32` da palavra modificada. Bytes não afetados permanecem intactos.
4. **25 testes cobriam:** 4 níveis de offset × 4 opcodes (16), par LWL+LWR (1), delay entre LWL/LWR sem nop (1), registradores diferentes (1), preservação de vizinhos (2: `lwl_mantem_bits_nao_transferidos` e `lwr_mantem_bits_nao_transferidos`), imediato negativo (2: LWL e LWR), endereço forçado (2: SWL e SWR). **O round-trip SWL+SWR (1) citado na versão original deste item NÃO existia** — o teste de aceitação obrigatório do handoff não foi escrito, e a soma batia com 25 apenas porque "preservação de vizinhos" foi contado como 1 quando na verdade eram 2 testes. *(Corrigido em 2026-07-27: adicionados `round_trip_swl_swr_seguido_de_lwl_lwr` e `lwl_enxerga_load_delay_de_lw_no_mesmo_registrador`, elevando o total para 27.)*
