# 0197 — config-toml

- **Data:** 2026-08-03
- **Item do roadmap:** 9.5
- **Objetivo:** tirar BIOS, pastas, vídeo e áudio da linha de comando e pôr num `psx-rs.toml`
  editável pela tela de ajustes.

## Spec consultada

Nenhuma. Configuração de aplicativo não é hardware do PS1.

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | nenhum | Que valor fora de faixa devia ser recusado na leitura | Recusar faz o app não abrir por causa de um número que o usuário digitou errado num arquivo de texto | Escrito antes de rodar. Ficaram as duas coisas separadas: `valida()` diz o que está errado (aparece na tela), `ajustada()` grampeia para o app funcionar. Os testes exigem que `valida` rode ANTES do grampeamento — o mutante m7 mata quem inverte isso |
| 2 | nenhum | Que pasta vazia no arquivo era inofensiva | `""` como pasta de cartões faria o `.mcd` ser gravado na raiz do processo, fora de qualquer pasta | Teste `pasta_vazia_cai_no_padrao_em_vez_de_virar_raiz`; mutantes m4 e m5 (o m5 cobre pasta só de espaços) |

## Bateria de mutação

Placar da bateria: 12/12 mutantes mortos, 2/2 controles verdes, 0 equivalente — docs/mutantes/0197-config-toml.mut

| Mutante | O que quebra | Quem pegou |
|---|---|---|
| m1 | escala grampeada só por cima | `escala_fora_da_faixa_e_grampeada_nas_pontas` |
| m2 | volume não grampeado | `volume_e_slot_fora_da_faixa_sao_grampeados` |
| m3 | slot não grampeado | idem |
| m4 | pasta vazia vira caminho vazio | `pasta_vazia_cai_no_padrao_em_vez_de_virar_raiz` |
| m5 | pasta só de espaços é aceita | idem |
| m6 | BIOS vazia não é reclamada | `valida_aponta_a_bios_faltando` |
| m7 | validação usa a faixa já grampeada | `valida_reclama_de_faixa_antes_de_grampear` |
| m8 | volume fora de faixa só acima de 255 | idem |
| m9 | áudio desligado não zera o ganho | `audio_desligado_zera_o_ganho_seja_qual_for_o_volume` |
| m10 | ganho dividido por 255 | `volume_vira_ganho_linear_entre_zero_e_um` |
| m11 | escala padrão vira 3 | `padrao_e_utilizavel_sem_editar_nada` |
| m12 | áudio nasce desligado | idem |

## Placar antes → depois

Workspace: 1192 → 1208 testes (12 no `psx-core`, 4 no `psx-desktop`).

## Revisão cruzada (orquestrador)

## Decisões e notas

- **O `toml` ficou no `psx-desktop`, não no `psx-core`.** O que tem decisão — padrões,
  faixas, validação, ganho — é puro e mora no core, com bateria de mutação. Serializar já é
  problema resolvido pelo crate. A consequência é que a prova de que o arquivo é TOML de
  verdade tem de morar no frontend: são os quatro `#[cfg(test)]` em `ajustes.rs`
  (ida-e-volta, chave faltando, arquivo ausente, arquivo quebrado). **Esses quatro não
  entram na bateria** — `scripts/mutantes.ps1` só roda `-p psx-core` (10.33/10.58).
- **Arquivo ausente não é erro**; arquivo ilegível é. Sem `psx-rs.toml` o app abre no
  padrão e grava na primeira vez que o usuário clicar em Gravar. Com um TOML quebrado ele
  também abre, mas diz o que houve — sobrescrever calado apagaria a configuração de alguém.
- **`#[serde(default)]` no `Config` inteiro**: TOML escrito à mão com três chaves funciona,
  e acrescentar campo nesta struct não invalida os arquivos que já existem.
- **A linha de comando virou só dois argumentos:** `--config <arquivo>` e `--bios <arquivo>`
  (esta última também aceita posicional, como antes). Pasta de jogos, de cartões e de saves
  saíram das flags — mudar de pasta é ajuste, não invocação.
- Escala e filtro valem no quadro seguinte; BIOS, pastas e slot inicial só no próximo jogo,
  e a tela avisa isso.
