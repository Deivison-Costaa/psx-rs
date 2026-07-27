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
