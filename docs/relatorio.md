# Relatório final (rascunho incremental)

> Atualizado obrigatoriamente no fechamento de cada marco (regra do protocolo — lição do
> gb-rs, onde o relatório ficou para o fim e quase não existiu). Consolidação: item 11.1.

## 1. O que foi construído

Emulador de PlayStation 1 em Rust (psx-rs), três crates (core puro / CLI headless / app
desktop egui). Estado por marco: ver ROADMAP.md.

## 2. Como foi construído (o experimento)

Orquestrador (Claude) + trabalhador (opencode/DeepSeek); protocolo de fonte única com TDD e
bateria de mutação; meta-testes guardando o processo na CI. Detalhes e diário:
`docs/orquestracao.md`.

## 3. Métricas

Fonte: `docs/metricas.csv` (uma linha por execução, appendada pelo runner) e branch
`scoreboard-data`. Gráficos: item 11.2.

- Custo/tokens/duração por iteração e por modelo: (a consolidar)
- Erros de primeira tentativa por categoria: (a consolidar dos docs de iteração)
- Aprovações de EXEs de teste por commit: (a consolidar do scoreboard)

## 4. Comparativo com o gb-rs

Baseline do projeto-piloto: US$ 145,46 / 55 execuções / 871 testes / DeepSeek US$ 0,11 por
iteração vs Claude US$ 5–8. Comparar: custo total, custo por item entregue, taxa de retrabalho,
% de PRs com achado na revisão cruzada (métrica nova — no gb-rs a revisão nunca rodou).

## 5. Fechamentos de marco

| Marco | Data | Premissa reconferida | Observações |
|---|---|---|---|
| M0 | 2026-07-27 | sim — rumo confirmado; cadência alterada: revisão adversarial passa a ser em lote por marco (custo do orquestrador ≫ custo do trabalhador) | 12 PRs, 19 testes, pipeline validado: 2 iterações DeepSeek (US$ 0,0094 + US$ 0,0145), revisão adversarial achou 5 defeitos reais (fmt fora de ordem, regressão sem-args, commit sem escopo pego pelo commit-lint, commit em inglês, checkbox não marcado); 3 falhas de infra registradas como dado (shim npm ×2, quoting) |
