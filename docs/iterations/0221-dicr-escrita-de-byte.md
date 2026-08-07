<!-- Custo, tokens e duração NÃO entram aqui: são medidos por scripts/oc-iter.ps1 em
     docs/metricas.csv, casados por head_antes/head_depois. -->

# 0221 — dicr-escrita-de-byte

- **Data:** 2026-08-07
- **Item do roadmap:** 0214.4
- **Objetivo:** o banco de DMA passa a decodificar os bytes 1-3 de DPCR/DICR, que o
  driver de streaming STR da Sony usa para ligar e desligar a interrupção de fim de DMA.

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| psx-spx | § 1F8010F4h - DICR - DMA Interrupt Register (R/W) | docs/reference/04-dma.md |
| psx-spx | § Caution - 8/16-bit writes to certain IO registers | docs/reference/02-cpu.md |

`04-dma.md` L144-159 define o registrador: bits 16-22 são a máscara por canal, o bit 23 é o
master enable, os bits 24-30 são flags de conclusão reconhecidos por escrita de 1 e o bit 31
é só leitura. Os bits 16-23 caem inteiros no **byte 2** (1F8010F6h) e os flags no **byte 3**.

## O defeito, medido

Tomb Raider I trava porque o decodificador de FMV é chamado com um único setor em RAM em vez
do quadro completo de 9. A desmontagem do overlay carregado no passo 212.215.000 mostra a
cadeia real:

- `StGetNext` (0x800700B8) devolve o início do quadro quando o descritor em
  `[0x801DF7E8]` está no estado 2; `StFreeRing` (0x8006FFBC → 0x80070064) libera
  `numChunks` células e adianta o cursor de leitura em `numChunks`. Ou seja: **uma chamada de
  `StGetNext` equivale a um quadro inteiro**, não a um setor.
- Quem marca o estado 2 é o callback de fim de DMA3 (0x8006FE44), despachado pelo
  tratador de IRQ de DMA em 0x8005FF84.
- O jogo **só habilita a interrupção de fim de DMA3 no setor que carrega o último chunk do
  quadro**. O parâmetro chega em `sp+20` de 0x80070B40 e vale 1 apenas no ramo
  `chunkNo == numChunks-1` (0x80070970 → 0x800709EC). O helper aplica isso em 0x80070BC8-0x80070C2C:
  `lbu a0,2(v1)` / `or`-`and` / `sb a0,2(v1)`, com `v1 = 1F8010F4h` — **read-modify-write de
  um byte em 1F8010F6h**.

No nosso barramento os dois lados desse RMW estavam quebrados:

- `region_read_byte` casava só os endereços-base (`1F801080h..10ECh | 10F0h | 10F4h`), então
  `lbu` em 1F8010F6h caía no ramo genérico e devolvia **0 fixo** (medido: `a0=0x00000000`
  em todas as passagens por 0x80070C2C).
- A escrita de byte no mesmo endereço caía no ramo `1F801061h..1FFFh => true` e era
  **descartada em silêncio**.

Resultado: a máscara do canal 3 nunca era desligada, o IRQ de DMA3 subia a cada setor, o
descritor virava READY depois do chunk 0 e o decodificador saía correndo pela RAM — foi assim
que ele zerou o vetor de exceção em 0x80000080.

Sequência medida do índice de início de quadro (`[0x801DF7E4]`), antes → depois:

```
antes:  0,1,2,3,4,5,...  (um por setor, 147k passos de intervalo)
depois: 0,9,0x12,0x1B,0(wrap),8,0x11,0x1A,...  (nove por quadro)
```

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | endereçamento | a condição de "quadro pronto" seria um contador de chunks no jogo | 04-dma.md L152-159: os bits 16-22 é que decidem se a conclusão levanta o flag | desmontar 0x80070B40 e ver o `sb` em 1F8010F6h |
| 2 | endereçamento | o intervalo do banco de DMA em `region_read_byte` cobria os 4 bytes de cada registrador | o registrador ocupa 1F8010F4h..10F7h | `--trace-pcs 0x80070C2C` mostrou `a0=0` em todas as passagens |

## Bateria de mutação

Placar da bateria: 5/5 mutantes mortos, 2/2 controles verdes, 0 equivalente — docs/mutantes/0221-dicr-escrita-de-byte.mut

| Registro | Mutação | Teste que pegou |
|---|---|---|
| m1 | byte 3 vira gravação direta em vez de ack | `sb_no_quarto_byte_do_dicr_nao_acende_flag_que_estava_apagado` |
| m2 | byte 2 escreve sem preservar o resto do registrador | `sb_no_terceiro_byte_do_dicr_liga_a_mascara_do_canal` |
| m3 | máscara gravável esquece os bits 16-23 | `sb_no_terceiro_byte_do_dicr_liga_a_mascara_do_canal` |
| m4 | byte de ack tratado como byte comum | `sb_no_quarto_byte_do_dicr_reconhece_o_flag_de_conclusao` |
| m5 | deslocamento do byte conta em 4 bits | `lbu_no_terceiro_byte_do_dicr_devolve_mascara_e_master` |

O manifesto 0204 (m4) foi reancorado no novo intervalo e rerrodado: 6/6 mortos, 2/2 controles.

## Placar antes → depois

Workspace: 1385 → 1393 testes, todos verdes. `clippy --all-targets --workspace -D warnings` limpo.

Medição a 600M passos, linha de base = a27ece7 (merge do PR #234):

| Jogo | Base: PCs distintos nos últimos 10% | Depois | VRAM base | VRAM depois |
|---|---|---|---|---|
| Tomb Raider | 60 (topo em 0x80000080, o vetor de exceção) | 43 (0x80059EC8/0x800700D4, laço de streaming) | zera a partir do dump 8 | muda até o dump 24 |
| Tomb Raider II | 35 | 41 | — | — |
| Tomb Raider III | 60 (0x80001B58, RAM baixa) | 45 (0x00078F7C, código do jogo) | zera a partir do dump 12 | logo da Eidos legível |
| Silent Hill | 25 | 44 | congela em 4484 no dump 20 | logo da Sony e título legíveis |
| Final Fantasy IX | 11 | 11 | — | — |

Regressão a 400M nos 10 que funcionavam: 8 idênticos ao byte. Tekken 3 e Resident Evil 2
mudaram **para frente** — Tekken 3 sai da tela de título parada e entra no FMV de atração
(VRAM de 7052/7052/7065 estático para 10301/10318/10289 variando); RE2 sai de 4505 congelado
para 4720/5981 variando.

## Revisão cruzada (orquestrador)

<!-- Preenchido pelo Claude na revisão do PR: achados no formato de docs/prompts/review.md, ou "sem achados". -->

## Decisões e notas

- **Não mexi na regra de 32 bits do endereço-base.** `write8_gpr_completo` continua tratando
  `sb`/`sh` em 1F801080h..10ECh, 10F0h e 10F4h como escrita de palavra com o GPR inteiro
  (02-cpu.md L309-313). A mudança vale só para os bytes 1-3, onde a regra antiga não era
  "GPR inteiro" e sim "descarta". Aplicar "GPR inteiro" também em 10F6h **quebraria** o jogo:
  o valor armazenado é 88h/80h e, como palavra, apagaria o master enable do bit 23. Fica
  registrada a tensão: a spec não diz o que acontece num `sb` em offset diferente de zero de
  um registrador de 32 bits, e o software embarcado é a única evidência disponível aqui.
- **Falta em aberto:** o FMV agora roda de ponta a ponta mas sai ruidoso nos quatro jogos.
  O logo da Eidos (TR3) e o título do Silent Hill são legíveis sobre um fundo granulado. É um
  defeito separado, provavelmente no MDEC ou no decodificador BS, e não estava visível antes
  porque nenhum FMV chegava a decodificar.
- Final Fantasy IX não mudou em nada: continua com 11 PCs distintos em 0x800A9A6C-0x800A9A78.
  Trava por outro motivo.
