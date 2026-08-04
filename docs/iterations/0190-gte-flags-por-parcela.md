# 0190 — gte-flags-por-parcela

- **Data:** 2026-08-03
- **Item do roadmap:** 5.5 e 5.6
- **Objetivo:** fechar o GTE contra hardware: flags de overflow por parcela, bug do far
  color do MVMVA, GPF/GPL (que nao existiam) e um placar automatico contra o log de fuzz
  do ps1-tests.

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| psx-spx | § COP2 0180001h - 15 Cycles - RTPS - Perspective Transformation (single) (L481) | docs/reference/07-gte.md |
| psx-spx | § COP2 158002Dh - 5 Cycles - AVSZ3 - Average of three Z values (for Triangles) (L523) | docs/reference/07-gte.md |
| psx-spx | § COP2 0400012h - 8 Cycles - MVMVA(sf,mx,v,cv,lm) (L550) | docs/reference/07-gte.md |
| psx-spx | § COP2 190003Dh - 5 Cycles - GPF(sf,lm) - General purpose Interpolation (L641) | docs/reference/07-gte.md |
| psx-spx | § Details on "MAC+(FC-MAC)\*IR0" (L657) | docs/reference/07-gte.md |

## Oráculo

`tests/exes/ps1-tests/gte-fuzz/gte_valid_0xc0ffee_50.log` (gitignored) traz 22 comandos x
50 casos de hardware real, cada um com os 64 registradores de entrada e os 64 de saida.
O teste `gte_fuzz_hardware.rs` reproduz caso a caso e compara **todos** os registradores,
inclusive o FLAG. Sem o arquivo o teste se ignora sozinho.

**Placar: 889/1100 (80,8%) → 1100/1100 (100%).**

| Comando | Antes | Depois |
|---|---|---|
| RTPS | 45/50 | 50/50 |
| RTPT | 34/50 | 50/50 |
| MVMVA | 28/50 | 50/50 |
| AVSZ3 | 15/50 | 50/50 |
| AVSZ4 | 17/50 | 50/50 |
| GPF | 0/50 | 50/50 |
| GPL | 0/50 | 50/50 |
| outros 15 | 50/50 | 50/50 |

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | saturação-gte | Que os SZ do AVSZ podiam ser lidos como palavra de 32 bits | SZn sao unsigned de 16 bits; o fuzzer escreve lixo nos 16 bits de cima e o produto sai errado | AVSZ3 em 15/50 e AVSZ4 em 17/50, divergindo so no MAC0. Com a mascara: 50/50 nos dois |
| 2 | saturação-gte | Que o SZ3 do RTPS sai do MAC3 (32 bits) | § RTPS (L481) de docs/reference/07-gte.md: SZ3 = MAC3 SAR ((1-sf)\*12), mas sobre o acumulador de 44 bits — com sf=0 o truncamento em 32 bits inverte o sinal e o SZ3 satura no lado errado | Um caso esperava SZ3=FFFFh e media 0, arrastando divisao por zero, overflow de MAC0 e saturacao de SY2 |
| 3 | flags | Que o RTPT calcula o cue de profundidade nos tres vertices | O MAC0/IR0 de DQA/DQB sai so no ultimo; as flags dos vertices intermediarios nao existem no hardware | Um unico bit de diferenca (FLAG.12) em 3 dos 50 casos de RTPT |
| 4 | flags | Que o bug do far color do MVMVA era so "descartar as duas primeiras parcelas" | A conta descartada AINDA levanta flag de overflow do MAC e de saturacao de IR — esta ultima sempre como se lm=0 — e a saturacao do total tambem entra | Tres modelos diferentes medidos contra o log: so-descartado deu 45/50, so-total deu 48/50, os dois juntos deram 50/50 |
| 5 | endereçamento | Que o vetor de translacao do MVMVA vinha da matriz escolhida | § MVMVA (L550) de docs/reference/07-gte.md: `Tx` e escolhido por `cv`, `Mx` por `mx`. Com mx=2 e cv=0 o codigo usava far color no lugar de TR | Achado ao reescrever o laco; o log confirmou |
| 7 | nenhum | Que apontar a bateria de mutacao para o teste do placar bastava | O log e gitignored: na CI o teste se ignora sozinho e TODO mutante sobrevive. A bateria deu 12/12 aqui e 2/12 la | O job `mutantes` do PR reprovou. 21 casos do log foram embutidos em `gte_fuzz_embutido.rs` (5 deles escolhidos por medicao, um para cada mutante que a amostra inicial nao separava) |
| 6 | nenhum | Que o log de fuzz cobria todo o comportamento | Nenhum dos 50 casos de MVMVA tem cv=2 com lm=1 e trecho descartado negativo — a regra "como se lm=0" fica invisivel | Mutante m5 sobreviveu. Teste dedicado, montado a partir da spec, separa as duas leituras |

## Bateria de mutação

Placar da bateria: 12/12 mutantes mortos, 2/2 controles verdes, 0 equivalente — docs/mutantes/0190-gte-flags-por-parcela.mut

| Mutante | O que quebra | Quem pegou |
|---|---|---|
| m1 | overflow conferido so no total | `placar_do_fuzz_de_hardware_do_gte` |
| m2 | acumulador sem truncar em 44 bits | idem |
| m3 | translacao escolhida pela matriz | idem |
| m4 | far color entra no resultado | idem |
| m5 | flag do trecho descartado usa o lm | `far_color_do_mvmva_satura_o_trecho_descartado_como_se_lm_fosse_zero` |
| m6 | SZ3 do MAC3 truncado | `placar_do_fuzz_de_hardware_do_gte` |
| m7 | cue de profundidade nos tres vertices | idem |
| m8 | SZ somados em 32 bits | idem |
| m9 | AVSZ3 sem flag de MAC0 | idem |
| m10 | GPL sem deslocar a base | idem |
| m11 | GPF parte do MAC anterior | idem |
| m12 | interpolacao usa IR1 em vez de IR0 | idem |

A bateria roda contra `gte_fuzz_embutido.rs` — 21 casos do log copiados para dentro do
teste — porque o log inteiro e gitignored e nao existe na CI. O placar completo
(`gte_fuzz_hardware.rs`, 1100 casos) continua sendo a medicao de verdade, e se ignora
sozinho quando o arquivo falta. Unica excecao: m5 e creditado ao teste dedicado do far
color, que mora no arquivo do placar.

A bateria 0088 foi reexecutada (7/7, 2/2) depois de renovar duas ancoras que
envelheceram com a mascara dos SZ.

## Placar antes → depois

Workspace: 310 → 315 testes. O placar do GTE contra hardware saiu de 80,8% para 100%.

## Revisão cruzada (orquestrador)

## Decisões e notas

- **O item 5.6 esta cumprido pelo `gte-fuzz` do ps1-tests, nao pelo `psxtest_gte` do
  Amidog.** O fuzz da placar por registrador e roda em 0,4 s no `cargo nextest`; o Amidog
  exige boot completo, menu e leitura de VRAM, e mede menos por rodada. O EXE do Amidog
  segue em `tests/exes/amidog/gte/` para quem quiser a suite interativa.
- **A conta do MAC agora e uma so funcao (`acumula_mac`)**: parcela somada, flag conferida,
  acumulador truncado em 44 bits com sinal. RTPS, MVMVA e os comandos de cor passam por ela.
- **GPF e GPL nao existiam** — caiam no `_ => {}` do despacho e nao faziam nada. Nenhum
  teste do projeto os cobria antes deste.
- Cinco testes de Rayman com passo absoluto NAO se moveram nesta iteracao: o GTE mudou
  flags, nao contagem de ciclos.
