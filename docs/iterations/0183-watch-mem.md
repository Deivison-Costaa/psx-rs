<!-- Custo, tokens e duracao ficam em docs/metricas.csv, medidos pelo runner. -->

# 0183 — watch-mem

- **Data:** 2026-08-03
- **Item do roadmap:** 0183.1.
- **Objetivo:** parar de responder "quem escreveu neste endereço?" com desmontagem manual.
- **Fonte:** orquestrador.

## Spec consultada

Nenhuma: ferramenta de diagnóstico, não hardware emulado.

## Por que

Na 0182 gastei horas desmontando RAM à mão para descobrir quem apagava o bit 0 do `I_STAT`.
Improvisei então uma comparação antes/depois de cada passo, e em **dois minutos** ela deu a
resposta que a leitura estática não tinha dado: `0x00004A1C` 530 vezes, `0x8005A298` 512, o
handler do jogo 3.

Essa improvisação virou `--watch-mem`. Ela **não é específica de jogo nenhum** — é a pergunta
"quem escreveu aqui, e de que instrução" aplicada a qualquer endereço, em qualquer título, e
serve igualmente para as suítes de hardware do ps1-tests.

```
psx-cli --bios bios/SCPH1001.BIN --disc <cue> --max-steps N --watch-mem 0x801CF5F4,0x1F801070
watch 801CF5F4: passo=697263154 pc=0x80132FA8 de=0x0001A900 para=0x0001AA00
```

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a medição diz | Como foi pego |
|---|---|---|---|---|
| 1 | teste | Que dois testes (uma linha existe; o último valor bate com o dump) cobrissem o instrumento. | Não é assunto de spec. | **Quatro dos seis mutantes sobreviveram.** Reportar sempre, nunca atualizar o valor lembrado, culpar o PC seguinte e nascer com baseline zero — todos passavam. Um instrumento de diagnóstico que mente é pior que não ter instrumento, porque a resposta dele fecha a pergunta. |

As quatro lacunas viraram quatro asserções que expressam a propriedade, não a forma:

- **Ruído:** menos de 1000 linhas em 3 M passos (reportar sempre daria uma por passo).
- **Falso positivo de baseline:** observar a ROM da BIOS (`0xBFC00000`), que nunca muda e não é
  zero, tem de dar **zero** linhas.
- **Culpa certa:** decodifico o opcode no PC reportado e exijo que seja um `store`. Culpar a
  instrução seguinte cai num `nop` ou coisa pior.
- **Endereço inválido** reprova em vez de observar em silêncio o endereço errado.

## Bateria de mutação

Placar da bateria: **6/6 mutantes mortos, 2/2 controles verdes, 0 equivalente.**

Rodada **à mão**: `scripts/mutantes.ps1` recusa alvo fora do `psx-core` (achado 10.58), e o
`.resultado` traz cabeçalho dizendo isso. O procedimento foi o mesmo do script — aplicar o par
(de,para), rodar o teste do manifesto, restaurar — e está reproduzível a partir do manifesto.

Primeira execução: 2/6. Depois de fechar as lacunas: 6/6.

## Placar antes → depois

Workspace: **1024 → 1029** testes.

| Pergunta | antes | depois |
|---|---|---|
| "quem escreveu neste endereço?" | desmontagem manual, horas | `--watch-mem`, um comando |

## Revisão cruzada (orquestrador)

Rodada de autoria do orquestrador. O que sustenta o instrumento não é a suíte verde da primeira
tentativa — que era falsa — e sim a bateria depois das lacunas fechadas.

## Decisões e notas

**O custo é uma leitura por endereço por passo.** É diagnóstico, não caminho quente: sem a flag
o laço não muda. Não vale otimizar antes de doer.

**Cuidado com sobreajuste ao Rayman.** O instrumento é genérico de propósito. O uso que motivou
ele foi o Rayman, mas a pergunta que responde vale para qualquer jogo e para as 21 suítes do
oráculo de hardware — e é justamente o tipo de ferramenta que evita transformar investigação de
um título em conhecimento que não se reaproveita.
