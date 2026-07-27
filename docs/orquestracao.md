# Orquestração — como este projeto é conduzido

> Diário de decisões sobre o PROCESSO (não sobre hardware). Entradas datadas, append-only.
> É insumo direto do relatório final (`docs/relatorio.md`).

## Papéis

- **Trabalhador** — opencode + DeepSeek (`deepseek/deepseek-chat`), disparado por
  `scripts/oc-iter.ps1`. Escreve todo o código de emulação seguindo
  `.claude/skills/iterate/SKILL.md`. Custo por iteração ~duas ordens de grandeza menor que
  um modelo de fronteira (medido no gb-rs: US$ 0,11 vs US$ 5–8).
- **Orquestrador** — Claude (terminal interativo). Não escreve código de emulação. Faz:
  pré-voo do loop, monitoramento, **revisão adversarial de todo PR** (`docs/prompts/review.md`;
  modelo de vendor diferente do autor = revisão cruzada real), merge, métricas, fechamento de
  marco (checagem de premissa com o usuário + atualização do relatório).

## 2026-07-27 — bootstrap e o que herdamos do gb-rs

O projeto anterior (gb-rs, emulador de Game Boy, github.com/Deivison-Costaa/gb-rs) serviu de
experimento-piloto: 98 commits, 871 testes, US$ 145,46 em 55 execuções medidas. Este projeto
nasce corrigindo, por design, falhas que lá foram *medidas* (não opinadas):

1. **Protocolo com fonte única.** No gb-rs, CLAUDE.md (10 passos) e SKILL.md (12 passos)
   divergiam, e a bateria de mutação não estava em nenhum dos dois — só em prosa no STATUS.
   Um experimento controlado (Opus×Sonnet) mostrou o efeito: o agente que não inferia o rigor
   da prosa entregava bateria 0-de-2. Aqui o protocolo mora SÓ no SKILL; CLAUDE.md é índice.
2. **Contexto como recurso.** Medição do gb-rs: ~150k tokens/turno, sendo 41% arquivos de
   teste; cortar o STATUS não resolveu porque o custo é função do tamanho do repositório.
   Daí as regras R7/R8, o teto de 500 linhas por fonte, o mapa.md e os índices de seção nos
   docs de referência.
3. **Métricas sem memória humana.** O metricas.csv do gb-rs congelou duas vezes (26 PRs sem
   série numa delas) porque dependia de alguém rodar um script. Aqui o runner appenda a linha
   ao fim de cada execução e `metrics_freshness.rs` reprova lag > 1 iteração.
4. **Formato de commit blindado.** Lá o formato degradou na troca de agente (era convenção
   verbal); um commit ficou para sempre como `feat(cpu): CALL cc,u16 (    )` por aspas erradas.
   Aqui: template de PR + job `commit-lint` na CI.
5. **Checagem de premissa.** Um mal-entendido de enunciado (GBA vs DMG) sobreviveu 37
   iterações porque todos os controles olhavam para dentro do item. Aqui cada fechamento de
   marco relê o objetivo e pergunta ao usuário.
6. **Relatório incremental.** O marco de apresentação do gb-rs ficou 100% aberto até o fim
   ("o risco não é o emulador ficar incompleto — é ele ficar bom e o relatório não existir").
   Aqui fechar marco exige atualizar `docs/relatorio.md`.

Decisões de bootstrap: repo público `psx-rs`; **merge commit obrigatório** (squash/rebase
desabilitados) para preservar os commits test→feat→docs de cada PR — pedido explícito do
usuário após o histórico squashado do gb-rs; proteção de branch entra junto com a CI (exigir
check inexistente bloquearia os primeiros PRs). Iterações 0003–0005 executam os itens
0.5→0.4→0.3 nessa ordem (dependência: meta-testes exigem docs e CI existentes) — primeira
evidência de que iteração (cronológica) ≠ item (temático).

## 2026-07-27 — o smoke test pagou por si em uma hora

O 0008b falhou duas vezes antes de o modelo sequer ser chamado, e as duas falhas eram
exatamente da familia que o gb-rs documentou como "armadilhas do gh/aspas":

1. Shim npm: `Start-Process opencode` nao executa o .ps1 do npm; e o .cmd degrada aspas —
   um `--version` citado dentro do prompt virou flag do CLI, que imprimiu a versao e saiu
   com codigo 0. "ok" de exit code nao e "ok" de execucao (agora ha guarda: JSON < 1 KB =
   falha:sem-execucao).
2. Quoting do proprio orquestrador: `\"` (habito bash) nao escapa aspas em PowerShell; o
   argumento quebrou e fragmentos do prompt vazaram para outros parametros, corrompendo uma
   linha do metricas.csv (corrigida em 0008d, preservando a falha como dado).

Regra operacional fixada: prompts para oc-iter passam em string single-quoted do PowerShell
(escape de apostrofo = ''), e oc-iter chama o opencode.exe real, sem shims.

## 2026-07-27 — fechamento do M0

M0 completo no dia do bootstrap: 12 PRs mergeados, todos por merge commit com a trinca
test/feat/docs visivel na main, protecao de branch valendo para admin (push direto recusado,
GH006), CI verde em todos.

Numeros do pipeline ate aqui (docs/metricas.csv): 11 execucoes do orquestrador (custo em
assinatura, nao medido em dolar), 3 execucoes do trabalhador DeepSeek - 1 falha de infra
(falha:sem-execucao, 365 ms) e 2 iteracoes completas: 0008b US$ 0,0094 (49 steps, 3 min) e
0009 US$ 0,0145 (62 steps, 4 min). Ordem de grandeza confirmada: ~1 centavo por iteracao
nesta fase (gb-rs pagava US$ 5-8 no modelo de fronteira).

A revisao adversarial (papel novo, inexistente no gb-rs) achou defeito real em 2 de 2 PRs do
trabalhador: fmt fora de ordem derrubando a CI, regressao de comportamento sem-args, commit
sem escopo (pego pelo commit-lint - primeira captura real do guard), mensagem em ingles e
checkbox do ROADMAP nao marcado (virou regra explicita no SKILL). Nenhum achado de logica de
emulacao ainda - o codigo era trivial; o teste de fogo da revisao vem com delay slots (M1).

Correcao de rumo de processo: itens 0.3-0.5 executados fora de ordem por dependencia
(meta-testes exigem docs e CI); iteracao e cronologica, item e tematico.

## 2026-07-27 — decisao do usuario na checagem de premissa: revisao em lote

Premissa do projeto confirmada no fechamento do M0. Mudanca de cadencia decidida pelo
usuario: revisar cada PR do trabalhador custa caro no orquestrador (modelo de fronteira,
assinatura) enquanto o trabalhador custa ~US$ 0,01 - a economia inverteu o gargalo. Novo
regime a partir do M1:

- Trabalhador roda em lote: oc-loop -AutoMerge (merge com checks verdes, sem revisao previa).
- Guards sempre ativos por PR: CI (fmt/clippy/teste), meta-testes de processo, commit-lint,
  bateria de mutacao exigida pelo SKILL.
- Revisao adversarial do orquestrador passa a ser POR MARCO, em lote, sobre o diff acumulado;
  achados viram iteracoes de fix (sufixo letra). O campo "Revisao cruzada" dos docs de
  iteracao e preenchido nessa revisao de lote.
- Metricas: runner grava em logs/metrics-pending.csv (nao rastreado) e o worker incorpora no
  commit docs da iteracao seguinte - mantem arvore limpa entre iteracoes do loop e o lag <= 1
  do metrics_freshness.

Risco aceito e registrado: um erro de logica pode compor por algumas iteracoes ate a revisao
de marco; mitigacao real sao os testes de hardware (Amidog no 1.11) e a bateria de mutacao.

## 2026-07-27 — regra de tamanho estava invertida (iter 0015b)

O `file_size.rs`, escrito por mim no M0, reprovava **arquivo fonte** com mais de 500 linhas
e ignorava `tests/`. Ao ler o handoff da 0015 — que mandava fatiar o `cpu.rs` "porque o teto
manda" — o usuário corrigiu a premissa: a regra dele sempre foi a de **comentários** (≤5%,
reprova em 10%); o tamanho de arquivo fonte, "se fizer sentido, não tem problema", e o teto
"pros testes é uma boa, pros arquivos reais, não".

A correção não é cosmética, é o oposto do que estava valendo, e o motivo original ficava
melhor servido pela regra dele: no gb-rs quem comia contexto eram os testes (41% por turno),
não o `mcycle.rs`. O teto agora varre `crates/*/tests/`, e `src/` não tem teto — cortar
módulo por contagem de linha produz fronteira artificial, que é pior de ler que um arquivo
longo e coeso.

Efeito imediato: `cpu_branch_delay.rs` estava em 494 linhas, seis do limite. Virou
`cpu_jumps.rs` (113) e `cpu_branches.rs` (367), com os encoders de opcode extraídos para
`tests/support/asm.rs` — que é também o que a R8 pede, já que um item futuro sobre desvios
passa a pagar 113 ou 367 linhas de contexto, não 494.

Bateria de mutação do meta-teste: arquivo de teste plantado com 600 linhas → reprovado;
controles: fonte plantada com 900 linhas → verde (comportamento novo desejado), e
`comment_density.rs` intacto. 1/1 pego, 2/2 controles.

Registro honesto de onde o erro nasceu: o plano do M0 derivou "teto de 500 linhas por
arquivo fonte" do diagnóstico do gb-rs sem confirmar com o usuário, e a regra sobreviveu
15 iterações sem ser questionada porque nenhum arquivo tinha chegado perto do limite. Só
apareceu quando o `cpu.rs` chegou a 440 e o handoff começou a exigir refactor por causa
dela.
