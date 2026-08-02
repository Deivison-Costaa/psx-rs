# 0146 — amostragem-sondagem

- **Data:** 2026-08-01
- **Item do roadmap:** 10.70
- **Objetivo:** o laço de `testevent_descritor.rs` varria a tabela EvCB inteira a cada passo da
  CPU. Sondar por amostragem, **sem** que a amostragem cegue o teste.

## Spec consultada

Nenhuma seção de hardware. Item de custo de suíte; a referência é a medição abaixo, feita com
sondas descartáveis sobre o próprio laço.

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a medição mostrou | Como foi pego |
|---|---|---|---|---|
| 1 | desenho do conserto | Que o certo era **guarda de janela** (`if (85_000_000..=92_000_000).contains(&step)`), copiando o idioma que o `evento_consumo_shell.rs:127` já usa, porque amostrar arriscaria pular a janela | A condicao "spec=20h e spec=8000h ambos prontos" **persiste**: observada estavel de 87,0 M a 87,9 M, isto e por >= 900 k passos. Com essa persistencia, amostragem simples da o mesmo ganho **e e mais robusta** — a guarda de janela cravaria 85 M..90 M como lei a partir de UMA execucao | Sonda descartavel que, em vez de sair ao achar, continuou ate 88 M imprimindo o estado a cada 100 k |
| 2 | alvo da bateria | Que os mutantes atacariam "o mapeamento de descritor de evento" em `crates/psx-core/src/` | `descritor_para_indice` e helper **local do arquivo de teste**, nao codigo de producao. O teste nao cobre funcao isolada: afirma que o kernel EMULADO monta a EvCB com spec=20h no indice 1 e spec=8000h no indice 4 | Busca por `fn descritor_para_indice` em `crates/psx-core/src/` nao retornou nada |
| 3 | formato do manifesto | Que declarar `teste:` no cabecalho e sobrescreve-lo em alguns registros funcionaria | `scripts/mutantes.ps1` tem **duas** ramificacoes `"teste"` no mesmo `switch`, e o `switch` do PowerShell executa TODAS as que casam: um `teste:` de registro sobrescreve tambem o do CABECALHO, para todos os registros seguintes. Placar saiu 2/5 com m3, m4 e m5 "sobrevivendo" por estarem sendo rodados contra o teste errado | O placar 2/5 nao batia com a analise; a linha de cabecalho impressa pelo script dizia `teste: custo_de_sondagem` quando o manifesto declarava `testevent_descritor`. Registrado como divida **10.71** |
| 4 | oraculo x ambiente | Que uma bateria verde na minha maquina estava provada. Os mutantes m3 e m4 originais so eram matados pelo teste caro, que precisa da BIOS e do disco | A CI reprovou com **3/5**: m3 e m4 sobreviveram em **0,3 s**, contra 11-12 s locais. O tempo denuncia — na CI o teste caro nao roda, faz `SKIP` por falta dos arquivos, que sao material com copyright e **nunca** vao estar la. Mutante que sobrevive por falta de oraculo mede o AMBIENTE, nao o codigo | CI do PR #163, job `mutantes`. Reproduzido localmente movendo `bios/` para fora do repositorio |

## Medição

### Onde o evento realmente acontece (sonda descartável, revertida)

```
primeiro EvCB visto no passo   86.987.801
laco SAIU no passo             86.988.128   (teto 90.000.000)
```

**A varredura cara rodava 87 M de vezes sem ter nada para achar.** O evento mora nos últimos 3 %
do laço; entre "primeiro EvCB aparece" e "os dois prontos" há 327 passos.

### Por quanto tempo a condição persiste

```
passo 87.000.000: idx20=Some(1) idx8000=Some(4)
passo 87.100.000: idx20=Some(1) idx8000=Some(4)
...
passo 87.900.000: idx20=Some(1) idx8000=Some(4)
```

Estável por **≥ 900 k passos**. É esse número que autoriza amostrar: com passo de 10 k, a margem
é de **90×**. E é ele, não intuição, que vira o teto de `custo_de_sondagem`.

### Custo

| estado | `testevent_descritor` |
|---|---|
| antes da 0145 (`opt-level = 0`, sondagem a cada passo) | 528 s |
| depois da 0145 (`opt-level = 1`, sondagem a cada passo) | 100 s |
| **depois da 0146 (amostragem a cada 10 k)** | **12,7 s** |

**41× no total** entre o estado inicial e o final.

E no portão inteiro (`cargo test --all`), somando as duas iterações:

| estado | suíte completa |
|---|---|
| antes da 0145 | 842 s |
| depois da 0145 (`opt-level = 1`) | 191 s |
| **depois da 0146 (amostragem)** | **121 s** |

**7,0× no portão.** O passo 7 do protocolo deixa de ser o gargalo da rodada: era mais longo que
a janela de travamento do trabalhador (`$TravamentoMin = 25 min`) em disco frio.

## Bateria de mutação

Placar da bateria: 5/5 mutantes mortos, 2/2 controles verdes, 0 equivalente —
`docs/mutantes/0146-amostragem-sondagem.mut`.  Resultado em
`docs/mutantes/0146-amostragem-sondagem.resultado`.  Alvo dentro de `crates/psx-core/`, entao
rodada pelo `scripts/mutantes.ps1` — a primeira desta sequencia que **nao** precisou de runner
manual.

m1 (passo 10 k → 2 M): morto por `sondagem_nao_pode_cegar_o_teste`.
m2 (passo 10 k → 1): morto por `sondagem_nao_roda_a_cada_passo`.
m3 (`deve_sondar` com deslocamento errado): morto por `deve_sondar_so_nos_multiplos_do_passo`.
m4 (`deve_sondar` sempre verdadeiro — nunca pula): morto pelo mesmo.
m5 (base do descritor desalinhada em um): morto por `descritor_decode_index_correto`.

**Verificada nos DOIS ambientes**, porque a primeira versao passava so em um:

| | com BIOS e disco | sem BIOS (como a CI) |
|---|---|---|
| versao original | 5/5 | **3/5** — m3 e m4 sobreviviam |
| versao final | 5/5 | **5/5** |

O conserto foi extrair a decisao de amostragem para `deve_sondar(passo)`, uma fn pura com teste
proprio. Antes, a unica coisa capaz de matar m3/m4 era o teste caro; agora existe um oraculo que
roda em qualquer ambiente. O teste caro continua sendo oraculo local — nao foi trocado, foi
**acompanhado**.

**O m4 e a confirmacao mais limpa da iteracao.** Ele faz `deve_sondar` devolver sempre `true`,
isto e, restaura exatamente a sondagem a cada passo. Rodando com BIOS, ele levou **106,1 s** —
o custo de antes da iteracao, reproduzido de proposito. Nao e argumento, e medicao.

## Placar antes → depois

Workspace: **876** → **880** testes (3 em `custo_de_sondagem`, 1 em `deve_sondar_so_nos_multiplos_do_passo`).

## Revisão cruzada (orquestrador)

Iteracao do orquestrador (`fonte=orquestrador`).

Ponto que merecia ceticismo: **o teto de 100 k em `custo_de_sondagem` e uma medicao de UMA
execucao promovida a lei?** Em parte sim — por isso o teto tem 9× de folga sobre os 900 k
observados, e o piso e frouxo (1 k). Se a persistencia mudar, o m3 continua sendo a rede: ele
falha se a varredura parar de enxergar, independentemente do valor da constante.

Segundo ponto: a mudanca e num arquivo de TESTE, o que normalmente seria "otimizar o proprio
medidor". O que impede isso de virar auto-engano e a bateria rodar o oraculo caro (m3, m4, m5)
com o codigo amostrado — se a amostragem tivesse cegado o teste, esses tres teriam sobrevivido.

## Decisões e notas

**1. Amostragem venceu a guarda de janela por medicao.** Ver erro 1. A guarda de janela daria o
mesmo ganho, mas embutiria "o evento acontece entre 85 M e 90 M" no codigo — uma constante que
so vale para esta BIOS, este disco e este modelo de ciclos.

**2. Bateria verde numa maquina nao e bateria verde.** O achado mais util da iteracao nao foi
sobre amostragem: foi que um mutante pode sobreviver por **ausencia de oraculo** em vez de por
robustez do codigo, e que isso se parece com sucesso quando se olha so o placar local. O sinal
que denunciou foi o TEMPO — 0,3 s onde o esperado eram 11 s. Vale como heuristica de revisao:
mutante que morre ou sobrevive rapido demais provavelmente nao exercitou o que se pensa.

**3. O que esta iteracao NAO faz.** Nao toca nos dois lacos de `cdrom_evento_kernel.rs` que
queimam 150 M passos sem saida antecipada (divida **10.68**), nem conserta o `switch` do
`mutantes.ps1` (divida **10.71**). R4.
