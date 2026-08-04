# 0193 — biblioteca-identidade

- **Data:** 2026-08-03
- **Item do roadmap:** 9.1
- **Objetivo:** dizer quem é um disco sem bootá-lo — serial, região e rótulo — e transformar
  isso na tela de biblioteca que faltava para o app desktop carregar um jogo.

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| psx-spx | § System Area (prior to Volume Descriptors) (L1123) | docs/reference/15-cdrom-format.md |
| psx-spx | § Primary Volume Descriptor (sector 16 on PSX disks) (L1163) | docs/reference/15-cdrom-format.md |
| psx-spx | § Format of a Directory Record (L1262) | docs/reference/15-cdrom-format.md |

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | endereçamento | Que dava para andar no diretório somando `33 + LEN_FI + padding` | LEN_DR já inclui o System Use, que em disco CD-XA (todo PSX) tem 14 bytes; somar 33+LEN_FI cai no meio do registro seguinte | Escrito antes de rodar, ao ler § Format of a Directory Record. Virou o teste `diretorio_avanca_por_len_dr_e_nao_por_33_mais_o_nome` e o mutante m4 |
| 2 | endereçamento | Que o registro de tamanho zero encerra o diretório inteiro | Encerra o **setor**: o diretório continua no setor seguinte, e o resto de cada setor é preenchido com zeros | Pego pelo par de testes `registro_de_tamanho_zero_encerra_o_diretorio` (um setor) e `diretorio_maior_que_um_setor_e_percorrido_inteiro` (dois). A função pura para no zero; quem itera setor a setor é o `identifica` |
| 3 | nenhum | Que o rótulo do volume serviria de título | O Rayman tem Volume Identifier vazio: o rótulo saiu `?` na medição contra o disco real | Medido com `--disc-info`. O título passou a ser o nome do arquivo, e o rótulo virou detalhe |

## Bateria de mutação

Placar da bateria: 12/12 mutantes mortos, 2/2 controles verdes, 0 equivalente — docs/mutantes/0193-biblioteca-identidade.mut

| Mutante | O que quebra | Quem pegou |
|---|---|---|
| m1 | licença lida do início do setor, não do offset 20h | `licenca_de_cada_regiao_sai_do_setor_4` |
| m2 | PVD aceito sem a assinatura CD001 | `pvd_sem_assinatura_cd001_e_recusado` |
| m3 | LBA e tamanho da raiz trocados | `pvd_entrega_raiz_e_rotulo` |
| m4 | diretório avança 33+LEN_FI | `diretorio_avanca_por_len_dr_e_nao_por_33_mais_o_nome` |
| m5 | registro zero não encerra o setor | `registro_de_tamanho_zero_encerra_o_diretorio` |
| m6 | setor bruto sempre lido como Mode1 | `setor_bruto_de_2352_tem_os_dados_no_offset_certo_por_modo` |
| m7 | serial mantém o sublinhado | `boot_vira_serial_normalizado` |
| m8 | serial mantém o ponto do 8.3 | idem |
| m9 | primeira linha com `=` vira BOOT | idem |
| m10 | contagem de setores truncada | `identifica_o_disco_inteiro_a_partir_dos_setores` |
| m11 | busca para no primeiro setor da raiz | `diretorio_maior_que_um_setor_e_percorrido_inteiro` |
| m12 | PVD procurado no setor 17 | `identifica_o_disco_inteiro_a_partir_dos_setores` |

## Placar antes → depois

Workspace: 1137 → 1152 testes.

Medição contra disco real, por `psx-cli --disc-info` (lê 4 setores por seek, não a imagem
inteira):

| Disco | serial | região | rótulo | trilhas |
|---|---|---|---|---|
| Crash Bandicoot (USA) | SCUS-94900 | NTSC-U | SCUS-94900 | 1 |
| Rayman (USA) | SLUS-00005 | NTSC-U | *(vazio)* | 51 |
| Rayman (USA) DADOS | SLUS-00005 | NTSC-U | *(vazio)* | 1 |

Os dois seriais batem com o catálogo real. O `.cue` multi-trilha e o só-de-dados do Rayman
dão a mesma identidade, que é o esperado: a identidade mora na trilha 1.

## Revisão cruzada (orquestrador)

## Decisões e notas

- **`app/` dentro do `psx-core`.** A lógica pura de frontend (biblioteca, e nos itens
  seguintes config, mapeamento e sessão) mora em `crates/psx-core/src/app/`, não numa crate
  nova. Motivo medido: `scripts/mutantes.ps1` só roda `cargo test -p psx-core`
  (achados 10.33/10.58), então código testável fora do `psx-core` **não teria bateria de
  mutação** — que é o portão de qualidade do projeto. R3 continua valendo: `app/` não faz
  I/O e não trouxe dependência nenhuma. Quem lê arquivo é o frontend.
- **A varredura não lê a imagem inteira.** `disco::identifica` abre o `.bin` e faz seek para
  os 4 setores que o ISO 9660 pede. Ler 700 MB por jogo para descobrir o serial tornaria a
  biblioteca inutilizável com meia dúzia de discos.
- **Título é o nome do arquivo.** Não há título de jogo no disco — só o Volume Identifier,
  que no Rayman está vazio. Inventar um banco de dados de títulos seria escopo novo; o
  serial e a região, que saem do disco, aparecem como detalhe.
- **Disco ilegível continua na lista**, com identidade vazia. Esconder o jogo faria o
  usuário procurar defeito no emulador onde há um `.cue` quebrado.
- `psx-cli --disc-info <cue>` expõe a mesma rotina sem janela — foi como esta iteração se
  mediu, e serve de diagnóstico quando um disco não aparece direito na lista.
