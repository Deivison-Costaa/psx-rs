# 0145 — perfil-de-build

- **Data:** 2026-08-01
- **Item do roadmap:** 10.69
- **Objetivo:** o `Cargo.toml` da raiz nao declarava nenhum perfil, entao toda a suite rodava em
  `opt-level = 0`. Declarar perfis otimizados **sem** comprar a velocidade com as checagens de
  execucao.

## Spec consultada

Nenhuma secao de hardware. Item de infraestrutura de build; a referencia e a documentacao do
Cargo sobre perfis (`opt-level`, `debug-assertions` e `overflow-checks` como chaves
independentes, e `test` herdando de `dev`).

Bateria de mutação: aplicada (ver abaixo).

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a medição mostrou | Como foi pego |
|---|---|---|---|---|
| 1 | perfis do Cargo | Que `[profile.test]` sozinho podia deixar a lib `psx-core` sem otimizar, porque teste de integracao linka a lib como dependencia e dependencia compila sob `dev` — e que por isso a escolha do perfil era decisiva | Os tres arranjos custam o mesmo: `test` sozinho 77 s, `dev` sozinho 76 s, os dois 76 s, contra 528 s de baseline. A escolha e indiferente. Duas razoes se somam: `test` **herda** de `dev`, e ~95 % do trabalho quente esta no proprio crate de teste (a varredura de EvCB), nao na lib | Medicao com `CARGO_PROFILE_TEST_OPT_LEVEL` / `CARGO_PROFILE_DEV_OPT_LEVEL`, que testam a hipotese sem tocar no repositorio |

O erro nao mudou a decisao — declarar os dois perfis continua sendo o certo, por ser explicito —
mas mudou a **justificativa**: nao e "senao a lib fica lenta", e "para que a intencao esteja
escrita e seja mutavel".

## Medição

### Qual perfil carrega o ganho (teste `testevent_descritor`)

| arranjo | execução |
|---|---|
| baseline, tudo em `opt-level = 0` | **528,1 s** |
| `CARGO_PROFILE_TEST_OPT_LEVEL=1` | 77 s |
| `CARGO_PROFILE_DEV_OPT_LEVEL=1` | 76 s |
| ambos | 76 s |

### Qual nível otimiza melhor (medido antes, na máquina livre)

| opt-level | execução | binário |
|---|---|---|
| 0 | 528,1 s | — |
| **1** | **70,3 s** | 8069 KB |
| 2 | 94,9 s | 8116 KB |
| 3 | 94,0 s | 8124 KB |
| s | 98,6 s | 7911 KB |

`1` ganha nos dois eixos: e o mais rapido a executar **e** o mais barato a compilar entre os
otimizados. A hipotese intuitiva de que binario menor seria mais rapido (cache de instrucoes)
esta **refutada**: `s` produz o menor binario e e o mais lento dos otimizados.

### Suíte completa

`cargo test --all`: **842 s** antes → **246 s** na primeira passada (que inclui recompilar tudo
com o perfil novo) e **191 s** em regime, com o build quente. Ganho de **4,4x** no portão.

## Bateria de mutação

Placar da bateria: 5/5 mutantes mortos, 2/2 controles verdes, 0 equivalente —
`docs/mutantes/0145-perfil-de-build.mut`.  Resultado em
`docs/mutantes/0145-perfil-de-build.resultado`.  Alvo e o `Cargo.toml` da raiz, fora de
`crates/psx-core/`, entao `mutantes.ps1` o pula (`mutantes.ps1:366`, invariante 29) e a bateria
foi aplicada por runner manual.

m1 (`[profile.dev] opt-level` 1 → 0): morto por `perfil_dev_otimiza`.
m2 (`[profile.test] opt-level` 1 → 0): morto por `perfil_test_otimiza`.
m3 (dev `debug-assertions` → false): morto por `otimizacao_preserva_debug_assertions`.
m4 (dev `overflow-checks` → false): morto por `otimizacao_preserva_overflow_checks`.
m5 (test `debug-assertions` → false): morto por `otimizacao_preserva_debug_assertions`.
c1 (`codegen-units` no dev) e c2 (`incremental` no test): sobreviveram, como esperado.

Os ancoras sao multilinha e incluem o cabecalho da secao de proposito: `opt-level = 1` aparece
duas vezes no arquivo, e o casamento e por linha inteira com exigencia de ocorrencia unica.

## Placar antes → depois

Workspace: **872** → **876** testes (4 novos em `perfil_de_build`).

## Revisão cruzada (orquestrador)

Iteracao do orquestrador (`fonte=orquestrador` na `metricas.csv`), como as 0134–0137, 0139 e
0143: mexe na infraestrutura que hospeda o trabalhador, nao em emulacao.

Ponto que merecia ceticismo e foi verificado: **o teste afirma `opt-level != 0`, nao
`opt-level == 1`.** Prender o valor exato transformaria uma medicao de hoje em lei; o invariante
que importa e "o portao nao roda sem otimizacao". A escolha do `1` esta registrada na tabela de
medicao acima, que e onde uma decisao de ajuste deve morar.

## Decisões e notas

**1. `debug-assertions` e `overflow-checks` explicitos nao sao decoracao.** Sao chaves separadas
de `opt-level`: o padrao de `dev`/`test` ja e `true`, e mexer so em `opt-level` os preservaria de
qualquer forma. Declara-los serve para que a bateria possa mata-los — m3, m4 e m5 existem
exatamente para provar que a suite percebe se alguem comprar velocidade desligando checagem.

**2. Por que o ganho e tao grande.** Nao e otimizacao de compilador sobre codigo de emulacao: e
que ~95 % do custo esta em sondagem escrita no proprio crate de teste (item 10.70, proxima
iteracao). `opt-level = 1` acelera essa sondagem; a 0146 remove a maior parte dela.

**3. O que esta iteracao NAO faz.** Nao mexe em `[profile.release]` nem em nenhum laco de teste.
R4.
