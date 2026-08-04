# 0189 — audio-cpal-anel

- **Data:** 2026-08-03
- **Item do roadmap:** 7.3
- **Objetivo:** levar os quadros que o SPU produz ate a placa de som: anel puro no
  `psx-core` e stream `cpal` no app desktop.

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| psx-spx | § Mono/Stereo Audio Output (L173) | docs/reference/08-spu.md |
| psx-spx | § Maximum Sound Frequency (L312) | docs/reference/08-spu.md |

Item de frontend: a spec entra so para fixar a taxa (44,1 kHz) e o formato estereo.

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | nenhum | Que bastava empurrar os quadros do SPU para o `cpal` | O dispositivo padrao do Linux costuma abrir a 48 kHz; sem conversao o audio toca 8,8% rapido e o anel esvazia | Escrito antes de rodar, ao ler `config.sample_rate()`. Virou um acumulador de fase no `Ring`, testado com 441 quadros -> 480 |
| 2 | nenhum | Que o teste podia avancar o barramento por `CPU_CYCLES_PER_SAMPLE` | O teste passa a andar junto com a constante que deveria medir | Mutante m8 dobrou o periodo para 1536 e sobreviveu. O teste agora usa o literal 768 e afirma a constante separadamente |

## Bateria de mutação

Placar da bateria: 8/8 mutantes mortos, 2/2 controles verdes, 0 equivalente — docs/mutantes/0189-audio-cpal-anel.mut

| Mutante | O que quebra | Quem pegou |
|---|---|---|
| m1 | anel cheio descarta o novo, nao o antigo | `anel_cheio_descarta_o_quadro_mais_antigo` |
| m2 | escala 32767 em vez de 32768 | `escala_usa_32768_para_o_minimo_bater_em_menos_um` |
| m3 | underrun deixa lixo no buffer | `falta_de_quadros_vira_silencio_e_conta_como_underrun` |
| m4 | canais trocados | `anel_entrega_os_quadros_na_ordem_em_que_entraram` |
| m5 | conversao de taxa invertida | `anel_converte_a_taxa_do_spu_para_a_taxa_do_dispositivo` |
| m6 | teto em amostras, nao em quadros | `anel_cheio_descarta_o_quadro_mais_antigo` |
| m7 | drenar copia em vez de esvaziar | `barramento_produz_um_quadro_a_cada_768_ciclos` |
| m8 | periodo do evento do SPU dobrado | idem |

## Placar antes → depois

Workspace: 300 → 310 testes.

Cinco testes do Rayman tiveram passo absoluto re-fixado em +6.372 (o mesmo deslocamento
uniforme em todos): com o SPU vivo o SPUSTAT passou a espelhar o SPUCNT e as esperas do
kernel terminam em vez de girar. E o achado 10.115 pela terceira vez. O
`rayman_evcb_descritores` deixou de fixar passo: agora dispara no primeiro instante em
que os dois descritores estao habilitados, e afirma a JANELA em vez do passo.

## Revisão cruzada (orquestrador)

## Decisões e notas

- **O anel mora no `psx-core` (puro) e o `cpal` no `psx-desktop`.** R3 continua valendo:
  a allowlist de `purity.rs` nao mudou. Alem disso, `mutantes.ps1` so roda `-p psx-core`
  (achado 10.33/10.58), entao codigo testavel fora dele nao teria bateria.
- **A conversao de taxa e por acumulador de fase**, vizinho mais proximo: para cada
  quadro de 44,1 kHz soma-se `output_hz` e emite-se enquanto passar de 44100. Com
  `output_hz == 44100` degenera em 1:1 exato, sem duplicar nem perder quadro.
- **Sem placa de som o emulador nao cai:** `AudioOut::new` registra o erro, deixa o anel
  vivo e o app segue com video. A tela mostra "Audio desligado".
- Dispositivo mono (1 canal) recebe a media dos dois canais; mais de 2 canais recebem a
  mesma media em todos — nada de silencio nos canais extras.
