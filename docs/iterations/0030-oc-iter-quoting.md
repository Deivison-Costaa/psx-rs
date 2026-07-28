<!-- Custo, tokens e duração NÃO entram aqui: são medidos por scripts/oc-iter.ps1 em
     docs/metricas.csv, casados por head_antes/head_depois. -->

# 0030 — oc-iter-quoting

- **Data:** 2026-07-28
- **Item do roadmap:** nenhum (iteração de processo, sem código de emulação)
- **Objetivo:** consertar dois defeitos do lançador `scripts/oc-iter.ps1` que uma rodada
  perdida expôs: o prompt sendo quebrado em vários argumentos, e a perda da linha de métrica
  justamente quando a rodada falha.

## Spec consultada

Nenhuma: não há hardware envolvido.

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que era verdade | Como foi pego |
|---|---|---|---|---|
| 1 | quoting | Que passar o prompt entre aspas em `Start-Process -ArgumentList` bastava, porque tinha funcionado em todas as rodadas anteriores | `-ArgumentList` **não reescapa**: as aspas viram delimitador literal na linha de comando, e a primeira aspa *dentro* do prompt fecha a região citada. O que vem depois é tokenizado — e token começando com `-` vira flag do CLI | A rodada das 11:52 de 28/07 saiu com o opencode imprimindo o **help** e um JSON de 0 byte. O culpado foi o `->` da frase "remover o teto -> teste trava" |
| 2 | erro-silencioso | Que a guarda `falha:sem-execucao` (JSON < 1 KB) cobrisse o caso de arquivo vazio | Arquivo de 0 byte faz `Get-Content -Raw` devolver `$null`, e `[regex]::Matches($null, ...)` **lança exceção** na linha 75 — antes do `Add-Content` que grava a métrica. A guarda rotulava a falha, mas a linha nunca era escrita | Mesma rodada: `ParentContainsErrorRecordException`, e `logs/metrics-pending.csv` sem nenhuma linha nova |

## O que foi medido

Com um script que só imprime os próprios argumentos, e o mesmo prompt que derrubou a rodada:

```
modo ANTIGO ("`"$prompt`""):    46 argumentos
                                arg[4]=remover  arg[5]=o  arg[6]=teto
modo NOVO   (aspas escapadas):   4 argumentos
                                arg[3]=<prompt inteiro>
```

O prompt tinha 14 aspas duplas (número par, o que mascara o problema em metade dos casos: o
texto entre pares de aspas volta a ficar "dentro"). O `->` caiu num trecho ímpar — fora — e
o yargs do opencode leu como flag desconhecida.

Por que só agora: os prompts anteriores também tinham aspas, mas nenhum token começando com
`-` caiu fora delas. Não é bug novo — é bug antigo que só agora encontrou o gatilho.

## Bateria de mutação

Não se aplica: mudança em `scripts/`, fora de `crates/`. A verificação equivalente é a
contagem de argumentos acima, com o antes e o depois medidos no mesmo prompt.

## Placar antes → depois

241 → 241 testes (inalterado).

## Revisão cruzada (orquestrador)

Iteração feita pelo próprio orquestrador: o defeito é da ferramenta de orquestração, não do
código de emulação. A rodada perdida foi despachada por mim, com um prompt escrito por mim.

## Decisões e notas

1. **A linha de métrica da rodada perdida foi escrita à mão** em `logs/metrics-pending.csv`
   (o caminho normal, incorporado ao `docs/metricas.csv` no PR da 0029), com
   `resultado=falha:sem-execucao` e custo/tokens/steps zerados — foi o que de fato aconteceu
   (o opencode saiu no parse de argumentos, sem consumir token nenhum). A duração ficou em `0`
   porque o script morreu antes de parar o cronômetro; o valor real seriam poucos segundos.
   Apagar a rodada da série seria mais cômodo e falsificaria o registro: a série tem que
   mostrar que **uma rodada em nove desta sessão morreu no lançamento**.
2. **Escapar as aspas em vez de trocar `Start-Process` por `ProcessStartInfo`.** O
   `ProcessStartInfo.ArgumentList` do .NET escapa sozinho e seria mais robusto, mas obriga a
   drenar stdout/stderr em thread separada — o opencode emite ~500 KB de JSON e encheria o
   buffer do pipe. A troca de uma linha resolve o defeito observado sem trocar o mecanismo de
   redirecionamento que já funciona.
3. **O que continua frágil:** uma barra invertida imediatamente antes de uma aspa no prompt
   ainda quebraria o escape. Não vi nenhuma nas nove rodadas até aqui; fica registrado em vez
   de tratado, para não gastar complexidade em caso hipotético.
