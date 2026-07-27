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
| M0 | | | |
