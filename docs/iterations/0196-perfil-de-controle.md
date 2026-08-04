# 0196 — perfil-de-controle

- **Data:** 2026-08-03
- **Item do roadmap:** 9.4
- **Objetivo:** jogar de controle: gilrs no app desktop, um vocabulário puro de entrada no
  core, perfis remapeáveis e uma tela para mexer neles.

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| psx-spx | § Normal Mode - Command 42h "B" - Read Buttons (and analog inputs when enabled) (L1449) | docs/reference/10-controllers-memcards.md |

A ordem dos 16 bits já estava no `pad_script::BUTTONS` desde o runner headless; este item
reusa a mesma tabela em vez de escrever outra — duas listas de bits divergiriam.

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | API-Rust | Que o perfil sairia em TOML, como o resto da configuração | Enum com payload (`EixoNegativo(0)`) vira tabela aninhada em TOML; um arquivo que existe para ser editado à mão ficaria ilegível | Escrito antes de rodar, ao montar o `Vec<(Entrada, u8)>`. Virou um formato de linha `entrada = botao`, com comentário `#` — e três testes só do parser |
| 2 | nenhum | Que "perfil PlayStation" e "perfil Xbox" fossem mapeamentos diferentes | São o mesmo: no gilrs, `South`/`East` são posições físicas, e as duas famílias põem confirmar embaixo. A diferença real é com controle estilo Nintendo, em que A/B são trocados | Ao escrever a tabela de defaults. Os perfis viraram "Padrão" e "Faces trocadas", que é a distinção que existe de verdade |

## Bateria de mutação

Placar da bateria: 12/12 mutantes mortos, 2/2 controles verdes, 0 equivalente — docs/mutantes/0196-perfil-de-controle.mut

| Mutante | O que quebra | Quem pegou |
|---|---|---|
| m1 | polaridade da palavra do pad invertida | `um_botao_apertado_nao_aperta_os_outros` |
| m2 | religar soma ligação em vez de trocar | `religar_a_mesma_entrada_troca_o_botao_em_vez_de_somar` |
| m3 | `liga` devolve cópia sem a ligação nova | `ligar_devolve_perfil_novo_sem_mexer_no_antigo` |
| m4 | `desliga` apaga o perfil inteiro | `desligar_tira_so_a_entrada_pedida` |
| m5 | faces trocadas idêntico ao padrão | `perfil_de_faces_trocadas_difere_so_nas_duas_faces` |
| m6 | eixo vertical invertido | `direcional_analogico_cai_no_direcional_digital` |
| m7 | nome de eixo não volta | `nome_de_entrada_vai_e_volta_para_todas_as_variantes` |
| m8 | perfil gravado com o número do bit | `perfil_vira_texto_de_linha_e_volta_igual` |
| m9 | comentário não é cortado | `texto_com_lixo_e_comentario_nao_derruba_a_leitura` |
| m10 | nome da entrada lido com espaço | idem |
| m11 | palavra começa zerada | `nada_apertado_deixa_a_palavra_toda_em_um` |
| m12 | perfil padrão sem o analógico | `direcional_analogico_cai_no_direcional_digital` |

## Placar antes → depois

Workspace: 1175 → 1192 testes.

Não há medição contra hardware aqui: o teste de que o gilrs enxerga o controle certo é
plugar um. O que a bateria prova é que a camada pura entre "o gilrs disse South" e "o bit 14
do pad" está coberta.

## Revisão cruzada (orquestrador)

## Decisões e notas

- **`psx-core` não conhece gilrs.** `Entrada` é um vocabulário próprio (Sul, Leste, ombros,
  DpadCima, EixoNegativo(n)…) e o frontend traduz `gilrs::Button` para ele. R3 mantido, e a
  camada que decide o que cada botão faz fica testável sem controle plugado.
- **Remapear devolve perfil novo** (`liga`/`desliga` recebem `&self`). O perfil em edição
  não pode virar o perfil em uso antes de o usuário gravar.
- **Uma entrada física aciona um botão só**, então religar substitui. O contrário — dois
  botões no mesmo gatilho — não tem uso e esconde erro de configuração. O inverso é
  permitido: d-pad e analógico apontam ambos para `left`, de propósito.
- **Teclado e controle valem ao mesmo tempo**, por união das duas palavras. Quem larga o
  controle e pega o teclado não precisa trocar de modo.
- **Zona morta de 0,5 no analogico** (`crates/psx-desktop/src/gamepad.rs`). Controle usado
  solta valor de repouso longe de zero; sem zona morta o personagem anda sozinho. Valor alto
  de propósito: aqui o analógico só emula o d-pad digital, não há precisão a preservar.
- **Sem controle o app não muda de comportamento**: `Gilrs::new()` falhando só imprime um
  aviso e o teclado segue valendo.
- O perfil mora em `controles.txt`, ao lado da pasta de cartões. Tela: **F10**.
