# 0188 — spu-reverb-cdda

- **Data:** 2026-08-03
- **Item do roadmap:** 7.4
- **Objetivo:** fechar o SPU com o que faltava do mixer — reverb completo, ruido por voz
  (NON) e a entrada de audio do CD-ROM (CD-DA e XA-ADPCM) chegando ao mixer.

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| psx-spx | § SPU Noise Generator (L633) | docs/reference/08-spu.md |
| psx-spx | § SPU Reverb Registers (L877) | docs/reference/08-spu.md |
| psx-spx | § Reverb Formula (L947) | docs/reference/08-spu.md |
| psx-spx | § Reverb Disable (L991) | docs/reference/08-spu.md |
| psx-spx | § 1F801D98h - Voice 0..23 Reverb mode aka Echo On (EON) (L924) | docs/reference/08-spu.md |
| psx-spx | § decode\_28\_nibbles (L963) | docs/reference/15-cdrom-format.md |
| psx-spx | § Pos/neg Tables (L978) | docs/reference/15-cdrom-format.md |
| psx-spx | § 25-point Zigzag Interpolation (L991) | docs/reference/15-cdrom-format.md |

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | timing | Que o passo do ruido (bits 9-8 do SPUCNT) faz o LFSR andar mais devagar | O passo e SUBTRAIDO do temporizador: passo maior faz o LFSR andar MAIS RAPIDO, e quem decide a cadencia e a razao entre a recarga (20000h SHR shift) e o passo | O teste esperava "anda a cada dois ciclos" com shift 0Eh e passo 7, e mediu "anda todo ciclo". Refeito com shift 0Dh, comparando passo 4 contra passo 7 |
| 2 | endereçamento | Que preencher o primeiro grupo de 128 bytes de um setor XA bastava para o teste de historico | O setor tem 18 grupos de 4 blocos; os blocos vazios com filtro 0 zeram o preditor, entao old/older terminavam em zero | `assert_ne!(estado, default)` falhou com os dois lados zerados. O gerador de setor passou a preencher os 18 grupos e os 4 blocos |
| 3 | endereçamento | Que a area de reverb podia ser testada so com deslocamentos pequenos | Os enderecos sao relativos ao buffer corrente e enrolam em mBASE..7FFFEh; sem cruzar o teto o teste nao ve a diferenca entre enrolar e mascarar | Mutante m2 sobreviveu. Teste novo usa area de 80h bytes e um deslocamento de 100h, que da a volta duas vezes |

## Bateria de mutação

Placar da bateria: 17/17 mutantes mortos, 2/2 controles verdes, 0 equivalente — docs/mutantes/0188-spu-reverb-cdda.mut

| Mutante | O que quebra | Quem pegou |
|---|---|---|
| m1 | produto de volume dividido por 4000h | `apf2_com_volume_zero_devolve_o_que_esta_no_buffer_vezes_vlout` |
| m2 | endereco sem enrolar dentro da area | `endereco_de_reverb_enrola_dentro_da_area_e_nao_no_topo_da_ram` |
| m3 | reflexao le [mLSAME] em vez de [mLSAME-2] | `reflexao_soma_de_volta_a_amostra_de_duas_meias_palavras_antes` |
| m4 | vLIN ignorado | `reflexao_do_mesmo_lado_grava_lin_vezes_viir_no_buffer` |
| m5 | APF soma o deslocamento em vez de subtrair | `apf2_com_volume_zero_...` |
| m6 | bit7 deixa de cortar a escrita | `bit7_do_spucnt_corta_a_escrita_no_buffer_mas_nao_a_leitura` |
| m7 | endereco corrente sem o piso de mBASE | `endereco_corrente_anda_de_duas_em_duas_e_da_a_volta_no_topo` |
| m8 | reverb a 44,1 kHz | `reverb_roda_a_metade_da_taxa_do_mixer` |
| m9 | paridade do ruido sem o xor 1 | `ruido_avanca_como_lfsr_de_paridade` |
| m10 | passo do ruido fixo em 4 | `passo_do_ruido_vem_dos_bits_9_8_do_spucnt` |
| m11 | EON ignorado (toda voz vai ao reverb) | `eon_manda_a_saida_da_voz_para_o_reverb` |
| m12 | CD entra no mixer sem o bit0 | `volume_de_cd_entra_no_mixer_so_com_o_bit0_do_spucnt` |
| m13 | shift do XA sem o "12 menos" | `xa_desloca_o_nibble_por_12_menos_o_campo_de_shift` |
| m14 | nibble alto lido do lugar do baixo | `xa_nibble_alto_usa_o_proprio_par_de_shift_e_filtro` |
| m15 | CD-DA com canais trocados | `setor_cdda_vira_588_quadros_estereo_de_16_bits` |
| m16 | reamostragem invertida | `reamostragem_de_37800_para_44100_estica_sete_por_seis` |
| m17 | taxa de XA fixa em 37800 | `subcabecalho_diz_taxa_estereo_e_se_o_setor_e_de_audio` |

m2, m3 e m8 sobreviveram na primeira rodada (14/17) e cada um virou um teste novo —
todos os tres eram lacunas de configuracao do teste, nao de implementacao.

## Placar antes → depois

Workspace: 269 → 300 testes. `cargo clippy --workspace --all-targets -- -D warnings` limpo.

## Revisão cruzada (orquestrador)

## Decisões e notas

- A unidade de reverb roda **a cada dois ciclos de 44,1 kHz**, na forma "esquerda e
  direita juntas a 22050 Hz" que a spec escreve, e nao alternando os canais por ciclo.
  A spec diz que o hardware alterna; o resultado audivel e o mesmo e a forma escrita e
  a testavel.
- **`SPUCNT.bit7` corta so a escrita.** Com ele zerado o reverb continua lendo, que e o
  que § Reverb Disable (L991) de docs/reference/08-spu.md exige — e o que permite ao jogo
  zerar o buffer sem que o hardware o reencha.
- **A reamostragem de XA e vizinho mais proximo**, nao a interpolacao de 25 pontos que o
  hardware usa: a taxa fica certa (37800 e 18900 sobem para 44100) e o filtro nao. Fica
  registrado como achado 0188.1.
- A fila de audio do CD tem teto de 4 setores; se o jogo le mais rapido do que o SPU
  consome, o excedente e descartado em vez de crescer sem limite.
- O `Cdrom` passou a guardar `XaState` entre setores: § Old/Older Values (L1119) de
  docs/reference/15-cdrom-format.md diz que old/older atravessam a fronteira do setor.
