# 0198 — sessao-recentes

- **Data:** 2026-08-03
- **Item do roadmap:** 9.6
- **Objetivo:** fechar o M9 com o que faltava de conveniência: fast-forward, lista de
  recentes e contador de tempo de jogo por jogo.

## Spec consultada

Nenhuma. Nada disto é hardware do PS1.

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | nenhum | Que o tempo de jogo sairia do relógio de parede | Com fast-forward de 8× o relógio de parede mede o tempo do *emulador*, não o do jogador: uma hora de jogo apareceria como sete minutos | Escrito antes de rodar, ao ligar o multiplicador. O contador virou quadros ÷ 60 — segundos **emulados** —, então acelerar não distorce a estatística |
| 2 | nenhum | Que multiplicador 0 seria inofensivo (ninguém escolhe zero) | `passos_por_quadro(base, 0)` dá zero passo: o jogo congela e nada na tela explica por quê | Teste `multiplicador_zero_nao_congela_o_emulador` e mutante m6. O `.max(1)` é barato; o suporte a "por que meu jogo travou" não é |

## Bateria de mutação

Placar da bateria: 13/13 mutantes mortos, 2/2 controles verdes, 0 equivalente — docs/mutantes/0198-sessao-recentes.mut

| Mutante | O que quebra | Quem pegou |
|---|---|---|
| m1 | jogo registrado não entra na lista | `o_ultimo_jogado_fica_no_topo` |
| m2 | tempo substitui em vez de somar | `tempo_de_jogo_acumula_entre_sessoes` |
| m3 | entrada antiga vira duplicata | `rejogar_sobe_para_o_topo_sem_duplicar` |
| m4 | lista cresce sem limite | `lista_para_de_crescer_no_limite_e_perde_o_mais_antigo` |
| m5 | jogo sem serial entra na lista | `jogo_sem_serial_nao_entra_na_lista` |
| m6 | multiplicador zero congela | `multiplicador_zero_nao_congela_o_emulador` |
| m7 | velocidade avança duas casas | `velocidade_cicla_e_volta_para_um` |
| m8 | velocidade desconhecida vira 8× | `velocidade_desconhecida_volta_para_um` |
| m9 | minuto exato sai em segundos | `tempo_e_escrito_na_unidade_que_faz_sentido` |
| m10 | hora exata sai em minutos | idem |
| m11 | minutos da hora sem o resto | idem |
| m12 | serial desconhecido pega tempo alheio | `lista_vazia_nao_conhece_ninguem` |
| m13 | lista de velocidades pula o 4× | `velocidades_disponiveis_sao_potencias_de_dois` |

## Placar antes → depois

Workspace: 1208 → 1223 testes.

## Revisão cruzada (orquestrador)

## Decisões e notas

- **Os recentes moram no `psx-rs.toml`**, como `[[recentes]]`, e não num arquivo próprio.
  Três arquivos de estado do app (config, controles, recentes) seria um a mais do que o
  necessário: o campo é um `Vec` de escalares, que é exatamente o que TOML faz bem.
  O perfil de controle continua separado porque `Entrada` tem variante com payload
  (ver 0196).
- **A lista tem teto de 10 e derruba o mais antigo.** Sem teto, o arquivo cresce para
  sempre e a tela vira rolagem infinita.
- **Registrar devolve lista nova.** A gravação em disco só acontece se a lista mudou de
  verdade — sair do menu sem jogar não reescreve o arquivo.
- **Tempo em segundos emulados** (quadros ÷ 60), não relógio de parede. Ver erro 1.
- **F12 cicla 1× → 2× → 4× → 8× → 1×.** Alternar em vez de segurar: o app é de janela
  única, e tecla presa some quando a janela perde o foco. A velocidade atual aparece na
  barra de status; em 1× não aparece nada, para não poluir.
- **O áudio não acompanha o fast-forward**: em 8× o SPU produz oito vezes mais quadros do
  que a placa consome e o anel descarta o excesso (é o que o `Ring` já fazia por projeto).
  Fica áspero, de propósito — corrigir isso exige reamostragem dependente de velocidade, o
  que é item próprio, não efeito colateral deste.
