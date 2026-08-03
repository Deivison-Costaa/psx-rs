<!-- Custo, tokens e duracao ficam em docs/metricas.csv, medidos pelo runner. -->

# 0175 — cdrom-oraculo

- **Data:** 2026-08-03
- **Item do roadmap:** nenhum item prévio cobria o defeito achado (Setloc); 10.103 registrado
  para o que ficou aberto (GetlocL/GetlocP e disc-swap).
- **Objetivo:** lote D do oráculo de TTY — `cdrom/disc-swap`, `cdrom/timing`, `cdrom/getloc`.
- **Fonte:** trabalhador (lote D, orquestrado em paralelo com A/B/C/E).

**R4 dobrado a pedido do usuário.** A regra diz uma micro-funcionalidade por iteração; aqui o
lote inteiro fecha numa rodada porque o custo não é o código, é a espera de suíte e CI.

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| psx-spx | § Setloc - Command 02h,amm,ass,asect (L787-798) | docs/reference/06-cdrom.md |
| psx-spx | § First Response (INT3) (or INT5 if failed) (L1984-2000) | docs/reference/06-cdrom.md |
| psx-spx | § GetlocL - Command 10h (L1052-1064) | docs/reference/06-cdrom.md |
| psx-spx | § GetlocP - Command 11h (L1073-1084) | docs/reference/06-cdrom.md |

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | hardware (herdado da 0063) | Que Setloc exigisse disco inserido, como SeekL/ReadN — decisão #3 da 0063 ("comandos que exigem disco... Setloc"), nunca confirmada contra o texto da própria seção. | § Setloc (L787-798) só valida BCD; não menciona disco. A checagem pertence ao SeekL/ReadN, que fazem o seek/leitura de fato. | `cdrom/timing` (ps1-tests real, sem disco) trava em "Unexpected INT5!" assim que a bateria de 100 execuções chega em `CdlSetloc` — hardware real não trava aí. |

## As mudanças

**Defeito achado e corrigido: Setloc não exige disco.** `send_command(0x02)` retornava
INT5(stat,80h) sem disco inserido, além da validação de BCD já existente. A seção da spec não
prevê essa checagem — Setloc "só guarda o alvo do seek, sem ainda iniciar o seek". Removida a
checagem; Setloc agora só falha por BCD inválido (INT5,10h), igual à spec.

**10.53/10.55/10.56 lidos e descartados como causa.** O handoff pedia checar se algum destes
itens já abertos explicava `cdrom/timing`. Não explicam: o defeito real (Setloc/disco) é
diferente dos três, que continuam abertos e sem relação direta com a suíte medida aqui.

## Bateria de mutação

Placar da bateria: **5/5 mutantes mortos, 2/2 controles verdes, 0 equivalente** —
`docs/mutantes/0175-cdrom-oraculo.mut`.

| Mutante | Teste que o pegou |
|---|---|
| m1 (reintroduz a checagem de disco removida) | `setloc_sem_disco_retorna_int3_mas_stat_com_bit0` |
| m2 (para de validar o setor/ff do BCD) | `setloc_rejeita_setor_bcd_invalido` |
| m3 (troca a ordem dos bytes de erro) | `setloc_rejeita_segundo_bcd_invalido` |
| m4 (Setloc válido dispara INT5) | `setloc_com_bcd_valido_retorna_int3_e_armazena_posicao` |
| m5 (troca ss/ff ao guardar o alvo) | `setor_mode2_form1_le_os_dados_a_partir_de_0x18` (cdrom_setor_mode2) |

Registros declaram `teste:` individualmente (m5 mata em `cdrom_setor_mode2`, os demais em
`cdrom_seek_pause`) — contorna o item 10.71.

O nome do teste que expõe o defeito (`setloc_sem_disco_retorna_int3_mas_stat_com_bit0`) foi
preservado da iteração 0063: renomeá-lo quebraria `mutation_battery::bateria_nomes_de_teste_existem`,
que confere nomes creditados em `.resultado` antigos (mesmo arquivados) contra as fns reais. Só o
corpo (as asserções) mudou.

## Placar antes → depois

Workspace: **953 → 953** testes (nenhum teste novo — o único adicionado corrige um já existente
no lugar, ver acima).

**Oráculo de TTY, lote D (`K/M` = K linhas divergentes de M, alinhado na 1ª linha do gabarito
presente na nossa saída).** Medido com `psx-cli --bios bios/SCPH1001.BIN --exe <exe>
--max-steps 800000000`, sem disco montado (nenhuma imagem disponível para estas três suítes):

| Suíte | antes | depois | nota |
|---|---|---|---|
| `cdrom/disc-swap` | 7/11 | 7/11 (inalterado) | precisa de abertura/fechamento **físico** da bandeja; sem mecanismo de script no `psx-cli` para simular isso, não há como avançar sem inventar comportamento (10.103). |
| `cdrom/timing` | 17/19 | 17/19 (mesmo número, causa mudou) | antes travava ANTES de emitir `CdlSetloc`/`CdlSetmode` (bug nosso). Depois do fix, as duas linhas são alcançadas — ainda divergem numericamente (nosso modelo de timing de ACK é determinístico; o hardware real tem jitter de mainloop, item 10.1) — e a suíte avança até a seção "single-speed timing", onde um `ReadN` sem disco real produz o mesmo "Unexpected INT5!" que antes, agora por falta de mídia, não por defeito. |
| `cdrom/getloc` | 40/44 (medido; a tabela do lote dizia 41/45 — não bati a mesma contagem, ver nota) | 40/44 (inalterado) | antes: `Setloc` falhava (bug). Depois: `Setloc` passa e o próximo comando (`SeekL`) falha corretamente por falta de disco — mesmo sintoma superficial (`* Seek/SetLoc failed, irq=5, status=0x01`), causa diferente. `GetlocL`/`GetlocP` continuam stub (10.103) e a suíte pressupõe TOC de um disco de 74 min que não temos. |

Nota de contagem: recontei `cdrom/getloc` linha a linha (script de alinhamento ad-hoc, mesmo
método usado para `disc-swap`/`timing`, que bateram exatamente com a tabela do lote) e cheguei em
M=44 (45 linhas não-vazias do gabarito, contando `cdrom/header-valid-bit` como âncora e `Test
passed` como última linha, menos a âncora que não conta como divergência) em vez de 45. Não
insisti em achar a diferença de 1 — registro a discrepância em vez de forçar a bater.

## Revisão cruzada (orquestrador)

<!-- Preenchido pelo Claude na revisão do PR: achados no formato de docs/prompts/review.md, ou "sem achados". -->

## Decisões e notas

- **`disc-swap` não é "sem TTY nenhum"** (o critério literal do handoff), mas trava numa
  interação física (abrir/fechar a bandeja) que o hardware real espera de um operador humano.
  Sem um jeito de scriptar isso no `psx-cli`, tratar como bloqueado por falta de infraestrutura,
  não como defeito de hardware — registrado em 10.103 em vez de forçar um palpite de
  comportamento.
- **`getloc`/`GetlocL`/`GetlocP` são stub reais** (`send_command` cai no braço `_` genérico, que
  só devolve `stat_byte()`). Implementar direito exige TOC real (a suíte espera um disco de 74
  minutos, com múltiplas trilhas/índices) e rastreamento de posição de subchannel Q durante seek —
  maior que uma correção pontual e sem imagem de disco local para servir de oráculo. Registrado em
  10.103 em vez de forçar.
- **Ambiente de trabalho:** os symlinks `bios` e `tests/exes` (mandados pelo setup do worktree)
  não são reconhecidos pelo `.gitignore` porque um padrão com barra final (`/bios/`) não casa
  symlink-para-diretório em Git — só diretório de verdade. Isso derrubava a checagem de árvore
  suja do `scripts/mutantes.ps1`. Corrigido localmente via `.git/info/exclude` (não versionado,
  não afeta outros clones/worktrees), sem tocar no `.gitignore` do repositório.
- Concorrência: rodada medida uma suíte por vez (`cargo build -j4`, sem `mutantes.ps1` nem
  `cargo test` em paralelo com o oráculo de outro lote), conforme o handoff.
