<!-- Custo, tokens e duracao ficam em docs/metricas.csv, medidos pelo runner. -->

# 0175 — cdrom-oraculo

- **Data:** 2026-08-03
- **Item do roadmap:** 10.103 (o que ficou aberto) e 10.108 (arreio sem disco).
- **Objetivo:** lote D do oráculo de TTY — `cdrom/disc-swap`, `cdrom/timing`, `cdrom/getloc`.
- **Fonte:** trabalhador (lote D, orquestrado em paralelo com A/B/C/E); **correção de rumo na
  revisão cruzada do orquestrador**.

**R4 dobrado a pedido do usuário.** A regra diz uma micro-funcionalidade por iteração; aqui o
lote inteiro fecha numa rodada porque o custo não é o código, é a espera de suíte e CI.

**Esta rodada terminou sem mudança de produção.** O defeito que ela julgou ter achado não
existia; o que existe é outra coisa, medida na revisão e registrada abaixo. Fica no acervo
inteira, com o caminho errado visível, porque é para isso que o acervo serve.

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| psx-spx | § Setloc - Command 02h,amm,ass,asect (L787-798) | docs/reference/06-cdrom.md |
| psx-spx | § Error Codes (L1020-1022) | docs/reference/06-cdrom.md |
| psx-spx | § GetlocL - Command 10h (L1052-1064) | docs/reference/06-cdrom.md |
| psx-spx | § GetlocP - Command 11h (L1073-1084) | docs/reference/06-cdrom.md |

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | hardware | Que Setloc **não** exigisse disco: § Setloc (L787-798) de docs/reference/06-cdrom.md só valida BCD e não menciona disco, então a checagem herdada da 0063 seria suposição sem base. | § Error Codes (L1020) do mesmo arquivo lista o comando **02h em primeiro lugar** entre os que devolvem erro 80h "when the disk is missing". A seção do Setloc é omissa; a de códigos de erro não é. | Revisão cruzada do orquestrador. Argumento de silêncio numa seção não vence texto explícito noutra — e R1 diz exatamente isso: ler a spec antes, e a spec inteira que trata do assunto. |
| 2 | medição | Que a suíte travar em `CdlSetloc` provasse defeito nosso. | Não é assunto de spec. | O gabarito de hardware mostra `GetStat -> 0x02` (motor girando) e `GetlocP succeeded - track 01 index 00`: as suítes de CD-ROM do ps1-tests foram gravadas **com disco na bandeja**. Nosso arreio as roda com `--exe` e sem `--disc`. A suíte travava porque falta mídia, não porque o Setloc esteja errado. |

## O que a medição mostrou de verdade

Montando um disco (`--exe` + `--disc`) e rodando `cdrom/getloc`, aparecem três divergências
reais que a rodada sem mídia escondia:

| Nossa saída | Gabarito de hardware |
|---|---|
| `GetStat -> 0x00` | `GetStat -> 0x02` — não ligamos o bit de motor girando |
| `GetlocL succeeded - [00:00:00] mode 0` | `GetlocL failed, IRQ = 5, status = 0x02` — deve falhar antes de qualquer leitura |
| `SetLoc failed, irq=5, status=0x01` | seek prossegue — falha mesmo **com** disco montado |

Isto é: remover a checagem de disco do Setloc não tocaria na causa. Com disco, o Setloc falha
assim mesmo, e por outro motivo ainda não isolado. Os três viram o item **10.108**, junto com o
arreio: enquanto o oráculo rodar as suítes de CD-ROM sem `--disc`, as contagens delas medem a
ausência de mídia, não a nossa fidelidade.

**10.53/10.55/10.56 lidos e descartados como causa.** O handoff pedia checar se algum destes
itens já abertos explicava `cdrom/timing`. Não explicam; continuam abertos e sem relação com o
que foi medido aqui.

## Bateria de mutação

Bateria de mutação: não se aplica — a rodada terminou sem mudança em `crates/*/src/`, e o
manifesto que existia media a correção que a revisão desfez.

## Placar antes → depois

Workspace: **953 → 953** testes. Nenhuma suíte do lote mudou de contagem:
`cdrom/disc-swap` 7/11, `cdrom/timing` 17/19, `cdrom/getloc` 41/45.

Nota de contagem: o trabalhador recontou `cdrom/getloc` com script ad-hoc e chegou a 40/44 em vez
dos 41/45 do CSV. A discrepância de 1 linha não foi isolada; fica registrada em vez de forçada a
bater.

## Revisão cruzada (orquestrador)

**Achado, severidade alta — regressão contra a spec.** O commit `ef281a0` removia o
INT5(stat,80h) do Setloc sem disco. § Error Codes (L1020) de docs/reference/06-cdrom.md lista
`02h` entre os comandos que devolvem 80h com disco ausente. Revertido em `1ce4dab`, junto com o
teste reescrito e o manifesto de mutação construído em cima dele. O teste
`setloc_sem_disco_retorna_int3_mas_stat_com_bit0` volta às asserções originais da 0063.

Vale registrar o que a rodada acertou: a leitura de `disc-swap` como bloqueio de infraestrutura,
o diagnóstico de que `GetlocL`/`GetlocP` caem no braço genérico de `send_command`, a recusa a
forçar a contagem a bater, e a nota sobre symlink e `.gitignore`. O erro foi de método num ponto
só — parar de ler a spec cedo demais.

## Decisões e notas

- **`disc-swap` não é "sem TTY nenhum"**, mas trava numa interação física (abrir/fechar a
  bandeja) que o hardware real espera de um operador. Sem jeito de scriptar isso no `psx-cli`,
  é bloqueio de infraestrutura, não defeito de hardware (10.103).
- **`GetlocL`/`GetlocP` são stub reais**: `send_command` cai no braço `_` genérico, que só
  devolve `stat_byte()`. Implementar exige TOC real e rastreamento de subchannel Q durante seek.
- **Ambiente:** os symlinks `bios` e `tests/exes` do setup do worktree não casam com o
  `.gitignore` (padrão com barra final não casa symlink-para-diretório), o que derrubava a
  checagem de árvore suja do `scripts/mutantes.ps1`. Contornado via `.git/info/exclude`, que não
  é versionado.
- Concorrência: uma suíte por vez, sem bateria nem `cargo test` em paralelo com o oráculo.
