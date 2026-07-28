# Relatório final (rascunho incremental)

> Atualizado obrigatoriamente no fechamento de cada marco (regra do protocolo — lição do
> gb-rs, onde o relatório ficou para o fim e quase não existiu). Consolidação: item 11.1.

## 1. O que foi construído

Emulador de PlayStation 1 em Rust (psx-rs), três crates (core puro / CLI headless / app
desktop egui). Estado por marco: ver ROADMAP.md.

Em 28/07/2026, com o M1 fechado: o emulador carrega uma BIOS real, faz sideload de executáveis
PS-EXE, executa o R3000A completo (ALU, shifts, loads/stores com delay slot, branches, mult/div,
LWL/LWR/SWL/SWR, COP0 com mecanismo de exceção), tem scratchpad, cache isolation, hook de TTY
com `putchar`/`puts`/`printf`, e GPUSTAT com decodificação GP0/GP1. Roda suítes de teste de
hardware de terceiros e **produz veredito por teste**.

## 2. Como foi construído (o experimento)

Orquestrador (Claude) + trabalhador (opencode/DeepSeek); protocolo de fonte única com TDD e
bateria de mutação; meta-testes guardando o processo na CI. Detalhes e diário:
`docs/orquestracao.md`.

## 3. Métricas

Fonte: `docs/metricas.csv` (uma linha por execução, appendada pelo runner) e branch
`scoreboard-data`. Gráficos: item 11.2.

### 3.1 Custo e volume (fechamento do M1, 28/07/2026)

| | |
|---|---|
| Execuções do trabalhador | **59** |
| Custo total (trabalhador) | **US$ 1,87** |
| Tokens | 1,79 M entrada / 0,67 M saída |
| Iterações documentadas | 41 |
| Testes no workspace | 274 |
| PRs mergeados | 49 |

Custo por modelo:

| Modelo | Execuções | Custo | Média |
|---|---|---|---|
| `deepseek-v4-pro` | 24 | US$ 1,5607 | US$ 0,0650 |
| `deepseek-chat` | 12 | US$ 0,1779 | US$ 0,0148 |
| `deepseek-v4-flash` | 2 | US$ 0,0895 | US$ 0,0447 |
| `deepseek-reasoner` | 1 | US$ 0,0402 | US$ 0,0402 |

O `chat` é 4,4× mais barato por execução que o `v4-pro` e **nunca passou na revisão adversarial
num item de CPU** — as iterações 0009–0017 rodaram nele. Por execução aproveitada, o barato
custou infinito.

### 3.2 Retrabalho

| Resultado | Execuções |
|---|---|
| `ok` | 42 |
| `ok:correcao-pos-revisao` (todas as variantes) | 12 |
| `falha:sem-execucao` | 2 |
| `abortado` (handoff errado / troca de modelo) | 2 |
| `rejeitado:semantica` | 1 |

**Taxa de retrabalho: 20,3%** (12 rodadas de correção em 59 execuções). Nove itens exigiram
correção após revisão: 0018, 0020, 0021 (×2), 0025, 0027 (×2), 0029 (×2), 0031, 0035, 0036.

**30 das 41 iterações têm revisão cruzada preenchida** — a métrica que o gb-rs não tem, porque
lá a revisão adversarial nunca rodou.

### 3.3 Erros de primeira tentativa

**92 erros registrados** nos 41 docs de iteração. Por categoria:

| Categoria | N |
|---|---|
| API-Rust | 19 |
| endereçamento | 10 |
| processo | 7 |
| flags | 6 |
| *nenhum* (a suposição estava certa) | 6 |
| script / ambiente-host | 5 |
| hardware | 3 |
| bit-mapping | 2 |
| cobertura de teste | 4 |
| outros | 30 |

A concentração diz algo que não era óbvio no início: **o trabalhador erra mais em atrito com a
linguagem e em aritmética de endereço do que em entender o hardware.** A leitura de spec, quando
o handoff aponta arquivo e linha, raramente falha; o que falha é traduzi-la para Rust.

### 3.4 Placar de EXEs de teste

`scripts/scoreboard.ps1` sobre 51 arquivos em `tests/exes/` (ps1-tests build-158 + Amidog):

```
4 com veredito (1p/3f), 46 so com saida, 0 sem saida, 1 nao avaliados, de 51 arquivos
```

| Suíte | Status | Detalhe |
|---|---|---|
| `ps1-tests/cpu/cop` | pass | 2p/0f |
| `ps1-tests/cpu/code-in-io` | fail | 1p/2f |
| `ps1-tests/dma/otc-test` | fail | 3p/35f |
| `ps1-tests/gpu/gp0-e1` | fail | 5p/5f |

As 46 suítes em `tty` imprimem o banner da biblioteca do ps1-tests e param à espera de hardware
que ainda não existe (timers, vblank, DMA). O `1 nao avaliados` é um utilitário de host que veio
no zip e não é PS-EXE. Série histórica publicada pela CI na branch `scoreboard-data`.

## 4. Comparativo com o gb-rs

Baseline do projeto-piloto: US$ 145,46 / 55 execuções / 871 testes / DeepSeek US$ 0,11 por
iteração vs Claude US$ 5–8.

| | gb-rs | psx-rs (M1 fechado) |
|---|---|---|
| Execuções do trabalhador | 55 | 59 |
| Custo do trabalhador | US$ 145,46 | **US$ 1,87** |
| Testes | 871 | 274 (M1 de 11 marcos) |
| Revisão adversarial | nunca rodou | 30 de 41 iterações |
| Taxa de retrabalho medida | não medida | 20,3% |
| Erros de 1ª tentativa registrados | não registrados | 92 |

Com praticamente o mesmo número de execuções, o custo caiu **~78×**. A diferença não é o modelo
sozinho: no gb-rs o orquestrador escrevia o código; aqui ele revisa. O que o psx-rs gastou a
mais foi *disciplina de registro*, e é justamente o que produziu os números acima.

## 5. Padrões de falha medidos (o achado central até aqui)

Nenhuma das falhas caras deste projeto foi de emulação. As três que se repetiram:

1. **Afirmação sem execução.** Teste que retorna cedo por arquivo ausente e passa verde
   (0027, duas rodadas seguidas); bateria de mutação creditando mutantes a testes que não os
   pegavam (**três ocorrências**: 0027 M6 e C3, 0029 M8). Todas caíram do mesmo jeito: o
   revisor aplicar o mutante e rodar. A contramedida que entrou nos prompts — *"para cada
   afirmação, o comando que a prova, com a saída colada"* — produziu a primeira rodada sem
   nada a desfazer (1.11, terceira tentativa).
2. **Fechar um caminho de falha abre o vizinho.** A 0022 inventou mecanismo de hardware no
   texto do handoff; a regra criada na 0024 ("Spec tem que citar arquivo e seção") fez o texto
   ficar correto e a invenção migrou para as *armadilhas* (0025). Corrigir o nome de um arquivo
   de teste (0027) abriu espaço para errar o diretório dele na rodada seguinte.
3. **O placar mede o que é fácil, não o que importa.** Com o `printf` implementado, o critério
   "TTY não vazio" fez o placar saltar para 50/51 "passando" — sendo que os 50 só imprimiam o
   banner da biblioteca e paravam. E quando o veredito real entrou (1.13), a linha de resumo
   passou a *excluir* da manchete justamente as suítes que melhoraram: caiu de 50/51 para
   46/51 porque quatro suítes ganharam veredito. Nenhum teste pegaria isso — o CSV estava
   certo, a bateria estava certa; errado estava o que um humano lê.

Corolário para o método: **conferir o handoff antes de despachar virou o passo de maior
retorno.** Três defeitos distintos foram pegos assim, cada um valendo uma rodada inteira:
afirmação de hardware inventada (0032), handoff que não existia e era um ponteiro para outro
documento descrito como "revisado e aprovado" (0034), e esquema de dados que não representava
o caso real (0035).

## 6. Fechamentos de marco

| Marco | Data | Premissa reconferida | Observações |
|---|---|---|---|
| M0 | 2026-07-27 | sim — rumo confirmado; cadência alterada: revisão adversarial passa a ser em lote por marco (custo do orquestrador ≫ custo do trabalhador) | 12 PRs, 19 testes, pipeline validado: 2 iterações DeepSeek (US$ 0,0094 + US$ 0,0145), revisão adversarial achou 5 defeitos reais (fmt fora de ordem, regressão sem-args, commit sem escopo pego pelo commit-lint, commit em inglês, checkbox não marcado); 3 falhas de infra registradas como dado (shim npm ×2, quoting) |
| M1 | 2026-07-28 | sim — e a cadência voltou atrás: revisão **por PR**, não em lote (decisão do usuário em 27/07, após o lote parar em clippy e um handoff fundir 4 itens). Confirmada pelos números: 20,3% de retrabalho pego na revisão | 17 itens (1.1–1.14), 274 testes, 59 execuções, US$ 1,87. O emulador roda EXEs de hardware reais e produz veredito: `pass - testCop0Disabled` / `pass - testCop0Enabled`. Duas iterações consertaram o **ferramental** e não o emulador (0030: lançador quebrava com aspas no prompt e perdia a métrica da própria falha; commit-lint não revalidava título editado). Três achados viraram itens 10.3–10.5 em vez de virar nota perdida |
