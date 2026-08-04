# 0195 — cartao-por-serial

- **Data:** 2026-08-03
- **Item do roadmap:** 9.3
- **Objetivo:** um memory card por jogo, escolhido sozinho pelo serial do disco, e uma tela
  que mostra o que há dentro dele.

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| psx-spx | § Directory Frames (Block 0, Frame 1..15) (L2684) | docs/reference/10-controllers-memcards.md |
| psx-spx | § Title Frame (Block 1..15, Frame 0) (in first block of file only) (L2750) | docs/reference/10-controllers-memcards.md |
| psx-spx | § Shift-JIS Character Set (16bit) (used in Title Frames) (L2794) | docs/reference/10-controllers-memcards.md |
| psx-spx | § Filename Notes (L2706) | docs/reference/10-controllers-memcards.md |

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | endereçamento | Que cada frame de diretório com estado "em uso" é um arquivo | 51h é o primeiro bloco; 52h e 53h são continuações do MESMO arquivo. Listar as três dá arquivo fantasma para todo save de 2+ blocos | Escrito antes de rodar, ao ler § Directory Frames. Virou `bloco_de_continuacao_nao_vira_arquivo_proprio` e o mutante m1 |
| 2 | nenhum | Que o Title Frame é sempre Shift-JIS de 16 bits | "the BIOS memory card manager does also accept 8bit characters 20h..7Fh" — os dois formatos aparecem em cartão real | Lido na mesma seção. O decodificador aceita os dois; mutante m9 prova que o caminho de 8 bits é testado |
| 3 | nenhum | Que os títulos de teste ("RAYMAN", "CRASH BANDICOOT") cobriam o Shift-JIS | **Mutante m7 sobreviveu**: nenhum título de teste tinha dígito, então deslocar a faixa `82h,4Fh..58h` não quebrava nada | Bateria. Título virou "RAYMAN 2 fase" — espaço, dígito, maiúscula e minúscula numa string só |
| 4 | nenhum | Que testar com uma imagem de 1000 bytes provava a checagem de tamanho | **Mutante m10 sobreviveu**: 1000 bytes são pequenos demais para o diretório caber, então a lista sai vazia com ou sem a checagem | Bateria. O teste passou a truncar um cartão VÁLIDO no meio: o diretório continua lá, e sem a checagem os arquivos seriam listados apontando para blocos que não existem |

## Bateria de mutação

Placar da bateria: 12/12 mutantes mortos, 2/2 controles verdes, 0 equivalente — docs/mutantes/0195-cartao-por-serial.mut

| Mutante | O que quebra | Quem pegou |
|---|---|---|
| m1 | bloco 53h vira arquivo próprio | `bloco_de_continuacao_nao_vira_arquivo_proprio` |
| m2 | arquivo apagado (A1h) volta à lista | `lista_so_os_arquivos_vivos_do_diretorio` |
| m3 | diretório começa no frame 2 | idem |
| m4 | nome lido do offset 0 | idem |
| m5 | blocos divididos por frame | `tamanho_em_blocos_sai_do_filesize` |
| m6 | Title Frame sem assinatura SC | `bloco_sem_assinatura_sc_fica_sem_titulo` |
| m7 | dígito Shift-JIS deslocado | `titulo_em_shift_jis_de_16_bits_vira_texto` (depois do conserto) |
| m8 | maiúscula lida como minúscula | idem |
| m9 | título ASCII de 8 bits ignorado | `titulo_em_ascii_de_8_bits_tambem_e_aceito` |
| m10 | cartão de tamanho errado é lido | `cartao_de_tamanho_errado_nao_lista_nada` (depois do conserto) |
| m11 | serial vai cru para o nome do arquivo | `serial_com_caminho_nao_escapa_da_pasta` |
| m12 | serial só de separadores não cai no padrão | `serial_vazio_cai_num_cartao_padrao` |

## Placar antes → depois

Workspace: 1163 → 1175 testes.

## Revisão cruzada (orquestrador)

## Decisões e notas

- **O serial é entrada externa.** Ele vem de dentro de um `.cue` que o usuário baixou;
  concatenar direto num caminho de arquivo é travessia de diretório. `nome_do_cartao`
  deixa passar só `[A-Za-z0-9-_]`, e serial vazio (ou só de separadores) cai em
  `sem-serial.mcd`. Testado nas duas convenções de barra.
- **Um cartão por jogo, em `cartoes/<serial>.mcd`.** O cartão de PS1 tem 15 blocos; um
  cartão único compartilhado enche e obriga o usuário a apagar save alheio para continuar.
  A pasta é criada na primeira gravação, e só quando o jogo grava de verdade.
- **A flag posicional de memory card saiu do app desktop.** Antes era
  `psx-desktop <BIOS> [cartao.mcd]`; agora é `--cartoes <pasta>` (padrão `cartoes/`), porque
  o arquivo em si deixou de ser escolha do usuário. O `psx-cli` mantém `--memcard <arquivo>`:
  no runner headless o cartão é instrumento de medição, não conveniência.
- **A tela de saves (F9) só lê.** Apagar arquivo de cartão mexe no `.mcd` que o jogo pode
  estar usando naquele instante; ficaria para um item próprio, com confirmação.
