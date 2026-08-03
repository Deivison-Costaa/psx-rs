# 0187 — spu-vozes-adpcm

- **Data:** 2026-08-03
- **Item do roadmap:** 7.1 e 7.2
- **Objetivo:** sair do SPU que so guardava RAM para as 24 vozes tocando de verdade —
  ADPCM, contador de pitch com interpolacao gaussiana, envoltoria ADSR, sweep de volume
  e mixer estereo a 44,1 kHz.

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| psx-spx | § Sample Data (SPU-ADPCM) (L225) | docs/reference/08-spu.md |
| psx-spx | § Flag Bits (in 2nd byte of ADPCM Header) (L237) | docs/reference/08-spu.md |
| psx-spx | § Pitch Counter (L296) | docs/reference/08-spu.md |
| psx-spx | § 4-Point Gaussian Interpolation (L324) | docs/reference/08-spu.md |
| psx-spx | § SPU Volume and ADSR Generator (L438) | docs/reference/08-spu.md |
| psx-spx | § Envelope Operation depending on Shift/Step/Mode/Direction (L507) | docs/reference/08-spu.md |
| psx-spx | § SPU Voice Flags (L598) | docs/reference/08-spu.md |
| psx-spx | § SPU Control and Status Register (L658) | docs/reference/08-spu.md |
| psx-spx | § Pos/neg Tables (L978) | docs/reference/15-cdrom-format.md |

Os valores-ouro dos testes nao vieram do emulador: a tabela gaussiana foi extraida do
proprio `.md` por script e conferida pela propriedade que a spec enuncia (as quatro
entradas de um indice somam 7F7Fh..7F81h — medi 7F80h em i=0 e 7F7Fh em i=128), e as
sequencias de ADPCM e de envoltoria foram calculadas a mao a partir do pseudocodigo.

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | endereçamento | Que a tabela gaussiana estava dividida em quatro grupos de 128 e que `gauss[0FFh]` era o ultimo valor do primeiro grupo | Os rotulos `entry 000h..07Fh`, `080h..0FFh`, `100h..17Fh`, `180h..1FFh` sao de 128 entradas cada; `gauss[0FFh]` = 12C7h, nao 019Ch | A soma de controle dava 23798 em vez de ~32640. Com a indexacao certa deu 32640 (7F80h), dentro da faixa que a spec promete |
| 2 | flags | Que a transicao ataque → decay → sustain gasta um ciclo por fase | Com nivel de sustain 0Fh o alvo e 8000h, acima do teto de 7FFFh: o decay termina no instante em que comeca | O mixer entregava exatamente metade do esperado (14278 em vez de 28558) — um passo de decay indevido. Virou um laco de transicao em `tick_adsr` |
| 3 | saturação-gte | Que `AdsrStep = AdsrStep * AdsrLevel / 8000h` podia ser divisao truncada | A spec nao desambigua, mas com truncamento o passo vira 0 para nivel < 4096 e o release exponencial nunca chega a zero | Escrito como SAR 15 (piso) depois de ver que a alternativa trava a voz ligada para sempre. Registrado aqui porque e escolha, nao leitura |
| 4 | timing | Que a voz produz amostra audivel ja no primeiro ciclo depois do key on | O key on zera a envoltoria; a amostra do ciclo N usa o nivel ANTES do passo, entao o primeiro ciclo sai mudo por construcao | Dois testes falharam com `left: 0`. Ajustados para medir a partir do quarto ciclo, que e quando a janela gaussiana tambem esta cheia |
| 5 | nenhum | Que declarar `teste:` no cabecalho do manifesto bastaria | Achado 10.71: `mutantes.ps1` herda o ULTIMO `teste:` visto, entao os registros sem alvo proprio rodaram contra o arquivo errado | Primeira rodada deu 9/18 com m1..m9 "sobrevivendo". Com `teste:` em todos os 20 registros: 16/18 |

## Bateria de mutação

Placar da bateria: 18/18 mutantes mortos, 2/2 controles verdes, 0 equivalente — docs/mutantes/0187-spu-vozes-adpcm.mut

| Mutante | O que quebra | Quem pegou |
|---|---|---|
| m1 | coeficiente negativo do filtro sem sinal | `adpcm_filtro_2_soma_115_do_anterior_e_menos_52_do_penultimo` |
| m2 | predicao sem o arredondamento de +32 | idem |
| m3 | nibble 8h..Fh deixa de ser negativo | `adpcm_nibble_de_8_a_f_e_negativo` |
| m4 | amostra da a volta em vez de saturar | `adpcm_satura_o_resultado_em_16_bits_com_sinal` |
| m5 | shift do cabecalho ignorado | `adpcm_filtro_2_...` |
| m6 | repeat marcado pelo loop-end e nao pelo loop-start | `flag_loop_start_copia_o_endereco_corrente_para_o_repeat` |
| m7 | End+Mute e End+Repeat trocados | `loop_end_sem_repeat_forca_release_com_nivel_zero` |
| m8 | loop-end nao salta para o repeat | `key_on_limpa_o_bit_de_endx_da_voz` |
| m9 | key on nao zera a envoltoria | `key_on_copia_o_start_address_e_zera_a_envoltoria` |
| m10 | direcao e fase invertidas | `envoltoria_ataque_linear_shift_0_sobe_14336_por_ciclo` |
| m11 | queda exponencial vira linear | `envoltoria_decay_exponencial_cai_pela_metade` |
| m12 | taxa toda em um recebe o minimo de 1 | `envoltoria_com_taxa_toda_em_um_nunca_avanca` |
| m13 | ataque exponencial nao desacelera acima de 6000h | `envoltoria_ataque_exponencial_desacelera_acima_de_6000h` |
| m14 | pitch nao e limitado a 4000h | `pitch_acima_de_3fffh_e_limitado_a_4000h` |
| m15 | PMON sem o deslocamento de 8000h | `pmon_usa_a_amplitude_da_voz_anterior_como_fator_de_passo` |
| m16 | volume fixo sem o dobro do campo de 14 bits | `volume_fixo_e_o_dobro_do_campo_de_14_bits` |
| m17 | janela gaussiana com nova e mais velha trocadas | `gauss_da_entrada_zero_pesa_a_amostra_nova_com_menos_um` |
| m18 | mudo do SPUCNT ignorado | `spucnt_com_bit14_zerado_silencia_a_saida` |

**m6 e m12 sobreviveram na primeira rodada** e sao os dois achados de medicao da iteracao:

- m6: o teste do loop-start punha a flag no bloco de partida, e o `key_on` ja escreve
  `repeat = start`. Com a flag no SEGUNDO bloco (0202h) o teste passou a distinguir.
- m12: 64 ciclos nao separam "nunca anda" de "anda com incremento 1" — o contador so
  cruza 8000h no ciclo 32768. O teste agora roda 32769 ciclos.

## Placar antes → depois

Workspace: 239 → 269 testes (30 novos, 16 em `spu_vozes_adpcm` e 14 em `spu_adsr_mixer`).
`cargo clippy --workspace --all-targets -- -D warnings` limpo.

Custo em parede: 200 M passos da BIOS levaram 15,2 s com o SPU ligado contra 16,0 s sem
ele (mesma maquina, medicoes seguidas) — o evento de 768 ciclos e o laco de 24 vozes com
saida antecipada para voz desligada nao aparecem no ruido.

## Revisão cruzada (orquestrador)

## Decisões e notas

- **O SPU passou a andar pelo scheduler (R2)**, com `EventId(SPU_TICK)` a cada
  `CPU_CYCLES_PER_SAMPLE = 768` ciclos, e nao por poll do barramento.
- **A decodificacao do bloco de partida acontece no key on**, nao no primeiro ciclo:
  e o que faz o loop-start do bloco inicial valer antes de qualquer amostra sair.
- **O bit15 (SPU Enable) e o bit14 (Mute) so cortam a saida do mixer**; as vozes
  continuam avancando por baixo. E o comportamento que os testes fixam, e escolhi ele
  porque desligar o motor faria o ENDX parar de acender, que a spec nao autoriza.
- **`voice_out(n)` e o VxOUTX da spec** (pos-ADSR, pre-volume-de-canal): e o que
  alimenta o PMON da voz seguinte e o buffer de captura das vozes 1 e 3.
- O anel de saida tem teto de 8192 quadros: o runner headless nunca drena, e sem teto
  o `Vec` cresceria 44100 quadros por segundo emulado.
- Reverb, ruido por voz ligado ao NON e entrada de CD-DA/XA ficam para a 0188 (item 7.4);
  o gerador de ruido ja avanca por ciclo, mas nenhum teste ainda mede a sequencia.
