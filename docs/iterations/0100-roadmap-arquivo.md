# 0100 — roadmap-arquivo

- **Data:** 2026-07-30
- **Item do roadmap:** 10.39
- **Objetivo:** tirar do `ROADMAP.md` os marcos 100% fechados, movendo-os para
  `docs/ROADMAP-fechado.md`, e criar a regra que impede o inchaço de voltar.

## Revisão do PR anterior

Não há PR de trabalhador a revisar: as quatro rodadas anteriores não produziram PR revisável (duas
travadas no provedor, duas mortas na parede). Os PRs **#114 e #115**, abertos por uma sessão zumbi,
ficaram deliberadamente **sem merge** — medi o efeito do que eles propõem e o TTY do boot é byte a
byte idêntico ao da `main`, além de a correção de hardware não ter sido conferida contra a spec (R1).

## Spec consultada

Nenhuma seção de spec de hardware. O item é organização do próprio roadmap.

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que o comportamento real é | Como foi pego |
|---|---|---|---|---|
| 1 | teste | Que `roadmap.contains("docs/ROADMAP-fechado.md")` provasse que o ponteiro existe | O caminho aparece **duas** vezes: no cabeçalho e na linha do próprio item 10.39. Apagar o ponteiro deixa o teste verde pela outra ocorrência | Bateria em 4/5, o m3 sobreviveu. Corrigido afirmando a propriedade real: o ponteiro tem de estar no cabeçalho, antes do primeiro marco |
| 2 | processo | Que capar o comprimento das linhas fechadas resolvesse o teto | Medido: mesmo cortando toda linha `[x]` a 75 chars sobrariam só 430 bytes, e o M10 cresce a cada defeito achado. Band-aid, não conserto | Modelei a economia de quatro tetos diferentes antes de escolher; arquivar marco inteiro libera 2 638 bytes de uma vez |
| 3 | ferramenta | Que a bateria casaria a âncora num arquivo escrito por mim | `mutantes.ps1` monta a agulha com `"\n" + de + "\n"`: só casa arquivo com fim de linha **LF**. Minhas edições gravaram CRLF, contra o `eol=lf` do próprio `.gitattributes` | `edicao '@@DE' encontrada 0 vez(es)`, mensagem que não diz nada sobre fim de linha. Achei comparando o caractere seguinte à âncora (13 = CR) |

## Bateria de mutação

Placar da bateria: 5/5 mutantes mortos, 2/2 controles verdes, 0 equivalente — docs/mutantes/0100-roadmap-arquivo.mut

| Registro | Rótulo | Testes que pegaram, conforme o `.resultado` |
|---|---|---|
| m1 | M6 fica 100% fechado e continua na escada | `marco_totalmente_fechado_nao_fica_no_roadmap` |
| m2 | M2 fica 100% fechado e continua na escada | `marco_totalmente_fechado_nao_fica_no_roadmap` |
| m3 | ponteiro para o histórico some do cabeçalho | `roadmap_aponta_para_o_arquivo_de_fechados` |
| m4 | id do histórico reaparece na escada | `nenhum_item_aparece_nos_dois_arquivos` |
| m5 | segundo id repetido entre escada e histórico | `nenhum_item_aparece_nos_dois_arquivos` |
| c1 | rótulo de item aberto reescrito, id e estado intactos | sobreviveu |
| c2 | segunda linha do ponteiro reescrita, caminho mantido | sobreviveu |

As atribuições foram lidas do `.resultado` gerado pela máquina.

## Placar antes → depois

Workspace: **695** → **699** testes (+4: `roadmap_arquivo`).

O efeito medido é de espaço: `ROADMAP.md` de **9 824** para **7 430** bytes, contra o teto de
10 000 de `roadmap_size.rs`. A folga sai de **176 bytes** (dois itens novos derrubariam a CI) para
**2 570** — cerca de 25 itens. `docs/ROADMAP-fechado.md` fica com 2 898 bytes e não tem teto,
porque é histórico e ninguém o lê para decidir o que fazer a seguir.

## Revisão cruzada (orquestrador)

Iteração inteira do orquestrador, durante uma parada deliberada do loop pedida pelo usuário
("não tem como construir numa fundação ruim").

## Decisões e notas

1. **Arquivar marco inteiro, não capar linha.** O teto existe para manter a escada legível; o que
   o gasta é a quantidade de itens, e 37 deles (M0, M1, M3) já estavam fechados há muito. A regra
   nova é estrutural — marco que fecha sai — em vez de cosmética.
2. **O teste veio antes e mede a regra, não o estado de hoje.** `marco_totalmente_fechado_nao_fica_no_roadmap`
   falha de novo quando M2, M4 ou M6 fecharem, e a correção é mover, não editar o teste.
3. **`nenhum_item_aparece_nos_dois_arquivos` protege contra a duplicata, não contra a perda.**
   Está dito aqui porque a bateria não cobre remoção silenciosa de item do histórico: para isso
   seria preciso uma contagem esperada versionada, que passaria a exigir edição a cada item novo.
   Escolhi não pagar esse atrito; a perda ficaria visível no diff do PR.
4. **`mutantes.ps1` só casa âncora em arquivo LF.** Não consertei aqui (R4); normalizei o
   `ROADMAP.md` para LF, que é o que o `.gitattributes` já manda. Fica como item 10.40.
