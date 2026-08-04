# 0191 — memory-card

- **Data:** 2026-08-03
- **Item do roadmap:** 6.3
- **Objetivo:** memory card de 128 KiB no endereco 81h do SIO0, com os comandos R/W/S da
  spec e imagem `.mcd` crua carregada e regravada pelo frontend.

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| psx-spx | § Reading Data from Memory Card (L2565) | docs/reference/10-controllers-memcards.md |
| psx-spx | § Writing Data to Memory Card (L2589) | docs/reference/10-controllers-memcards.md |
| psx-spx | § Get Memory Card ID Command (L2605) | docs/reference/10-controllers-memcards.md |
| psx-spx | § FLAG Byte (L2632) | docs/reference/10-controllers-memcards.md |
| psx-spx | § Data Size (L2667) | docs/reference/10-controllers-memcards.md |

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | endereçamento | Que devolver "sem /ACK" no setor invalido bastava para parar a transferencia | O mestre continua batendo o clock; sem marcar a maquina como abortada o passo seguinte indexa `data[]` fora do fim | Panico `range end index 131200 out of range` no teste do setor 400h. A maquina passou a entrar em `Modo::Abortado` |
| 2 | nenhum | Que a resposta ao terceiro byte separava pad de cartao no teste de roteamento | Os dois devolvem 5Ah ali (ID2 do cartao e ID do pad digital). Quem separa e a resposta ao byte de COMANDO: 41h contra o FLAG | O teste passou a olhar o segundo byte, nao o terceiro |
| 3 | flags | Que "escreveu igual ao buffer" servia para decidir 47h contra 4Eh | Reescrever zeros sobre zeros com checksum errado daria 47h por coincidencia | Trocado por um campo de resultado escrito no momento da gravacao |

## Bateria de mutação

Placar da bateria: 12/12 mutantes mortos, 2/2 controles verdes, 0 equivalente — docs/mutantes/0191-memory-card.mut

| Mutante | O que quebra | Quem pegou |
|---|---|---|
| m1 | FLAG inicial zerado | `cartao_vazio_tem_1024_quadros_de_128_bytes` |
| m2 | checksum sem os bytes de endereco | `leitura_segue_a_sequencia_de_bytes_da_spec` |
| m3 | MSB e LSB trocados | idem |
| m4 | setor fora de 0..3FFh aceito | `setor_acima_de_3ffh_devolve_ffffh_e_aborta` |
| m5 | escrita aceita qualquer checksum | `escrita_com_checksum_errado_devolve_4eh_e_nao_grava` |
| m6 | bit3 do FLAG cai na leitura | `bit3_do_flag_cai_na_escrita_e_nao_na_leitura` |
| m7 | ultimo byte ainda pede /ACK | `ack_cai_no_ultimo_byte_da_leitura` |
| m8 | comando invalido segue respondendo | `comando_invalido_aborta_logo_depois_do_byte_de_comando` |
| m9 | imagem de tamanho errado aceita | `imagem_crua_de_128_kib_e_aceita_e_devolvida_igual` |
| m10 | escrita nao marca a imagem como suja | `escrita_grava_o_setor_e_devolve_47h` |
| m11 | Get ID com 04h e 80h trocados | `get_id_devolve_a_sequencia_fixa_da_spec` |
| m12 | endereco 81h nao vai ao cartao | `sio_encaminha_o_endereco_81h_para_o_cartao_e_o_01h_para_o_pad` |

Quatro baterias antigas de SIO (0091, 0092, 0159, 0186) tiveram ancoras renovadas porque
o `send_byte` mudou de forma, e foram reexecutadas: 6/6, 5/5, 6/6 e 7/7, todos os
controles verdes.

## Placar antes → depois

Workspace: 313 → 325 testes.

## Revisão cruzada (orquestrador)

## Decisões e notas

- **A imagem e crua de 131072 bytes**, sem cabecalho — § Raw Memory Card Images (L2824) de docs/reference/10-controllers-memcards.md. Ler o arquivo e regrava-lo e
  do frontend: o `psx-core` continua puro (R3). O CLI ganhou `--memcard <arquivo.mcd>` e o
  app desktop aceita o caminho como segundo argumento.
- **A imagem so e regravada quando o jogo escreve**, pelo `take_dirty()`; um jogo que so
  le nunca toca o arquivo.
- **Cartao Sony, nao generico:** setor invalido devolve FFFFh no endereco confirmado e
  aborta sem mandar dados. A spec descreve os dois comportamentos; o generico mascara com
  3FFh e entrega o setor errado, que e pior para depurar.
- **O byte de fim da escrita e decidido na gravacao**, nao comparando o quadro com o
  buffer depois: 47h so quando o checksum bateu e os bytes foram para a imagem.
