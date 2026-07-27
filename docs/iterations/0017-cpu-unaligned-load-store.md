# 0017 — cpu-unaligned-load-store

- **Data:** 2026-07-27
- **Item do roadmap:** 1.7
- **Objetivo:** Implementar LWL/LWR/SWL/SWR — carga e armazenamento desalinhado de palavras.

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| psx-spx | Unaligned Load/Store (L240) + Unaligned Load/Store (Details, L257) | docs/reference/02-cpu.md |

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | endereçamento | Teste SWL offset 0 com rt=0xAABB_CCDD e memoria=0xAABB_CCDD: esperava 0xAABB_CCAA | Como rt[31:24]=0xAA e old_mem[31:24]=0xAA, o merge dá o mesmo valor | Teste passou mesmo antes da implementacao. Corrigido: trocar rt para 0xDEAD_BEEF e verificar merge visivel |
| 2 | timing | Teste LWL+LWR no mesmo rt (par classico) funcionaria com load delay | O LWR executa antes do LWL commitar pelo load delay, entao le o valor ANTIGO de rt (0x0000_0000) em vez do 0x1122_0000 | Teste falhou com 0xCCDD vs 0x1122_CCDD. Corrigido: usar rts diferentes (r9 e r10) |
| 3 | nenhum | N/A — implementacao correta de primeira | merge com mascaras bit-a-bit conforme offset | Todos os 11 testes de load passaram de primeira |

## Bateria de mutação

5/5 mutantes pegos, 2/2 controles verdes.

| Mutação | Teste que pegou |
|---|---|
| SWL offset 2 com mask 0x0000_FFFF (em vez de 0x0000_00FF) | swl_offset_2_upper_24bits |
| LWL offset 0 com todo o mem_word (em vez de merger parcial) | lwl_offset_0_upper_8bits + lwl_preserves_lower_bits_rt |
| SWL/SWR sem read-modify-write (escrita direta de rt) | swl_offset_0_upper_8bits, swl_offset_1_upper_16bits, swl_offset_2_upper_24bits |
| Offset invertido (3 - addr&3) | Todos os 18 testes |
| aligned = addr (sem mascara de alinhamento) | 13/18 falharam |
| Controle: rename merged→result em lwl | 0 falhas (verde) |
| Controle: comentario no match do swl | 0 falhas (verde) |

## Placar antes → depois

167 testes no workspace (149 + 18 novos). Scoreboard: ainda nao implementado (item 1.11).

## Revisão cruzada (orquestrador)

*Preenchido pelo Claude na revisao do PR.*

## Decisões e notas

- O par LWL+LWR no *mesmo* rt em instrucoes consecutivas nao funciona com o modelo atual de
  load delay — o LWR le o valor antigo do registrador. Em hardware real a CPU faz forwarding.
  Isso so sera resolvido no item 1.11 (Amidog psxtest_cpu) se o teste reprovar.
- SWL/SWR fazem read-modify-write: leem a word alinhada, substituem os bytes-alvo, escrevem
  de volta. Isso difere de SW que escreve direto.
- Nao ha extensao de sinal nem de zero em nenhum dos 4 opcodes.
