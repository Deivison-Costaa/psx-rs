<!-- Custo, tokens e duração NÃO entram aqui: são medidos por scripts/oc-iter.ps1 em
     docs/metricas.csv, casados por head_antes/head_depois. -->

# 0207 — cdrom-getloc

- **Data:** 2026-08-06
- **Item do roadmap:** 10.103 (GetlocL/GetlocP stub)
- **Objetivo:** GetlocL (10h) e GetlocP (11h) parem de cair no braço genérico de `send_command`
  e devolvam a resposta real da spec.

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| psx-spx | § GetlocL - Command 10h (L1052-1071) | docs/reference/06-cdrom.md |
| psx-spx | § GetlocP - Command 11h (L1073-1088) | docs/reference/06-cdrom.md |
| psx-spx | § Command Table 08h..1fh (L566-567) | docs/reference/06-cdrom.md |

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | timing | Que dava para medir "GetlocL falha durante o seek" escrevendo o comando e avançando o relógio até a 2a resposta do SeekL vencer. | A spec não trata escalonamento de eventos do emulador. | A 2a resposta só é agendada depois do ACK (10.53), com atraso menor que o 1o response de qualquer comando novo (`0x4A00` contra `0xC4E1`) — não dá para "pegar" o `seeking=true` por tick simples, a 2a resposta do SeekL sempre vence antes. Descartei a checagem de seek explícita: cobri as duas condições que dava para isolar sem essa corrida (motor parado, playing) mais a condição medida em hardware real (setor nunca lido), registrada abaixo em "decisões e notas". |
| 2 | mutação | Que testar `getlocl_erro_80h_com_motor_parado` e `..._durante_play` sem antes ler um setor já isolava cada checagem. | Não é assunto de spec. | `scripts/mutantes.ps1 -Iter 0207`: m1 (remove checagem de motor) e m2 (remove checagem de play) sobreviveram — nos dois testes, `has_last_sector` também estava falso (nenhum ReadN foi emitido), então a checagem "nenhum setor lido ainda" mascarava a mutação. Corrigido lendo um setor primeiro nos dois testes, isolando cada condição. |
| 3 | duplicação | Que copiar o cálculo de offset do setor (`checked_sub(150)` + `* 2352`) direto de `read_sector_from_disc` para a nova `read_sector_header` não teria custo. | Não é assunto de spec. | `mutation_anchors` reprovou: as mesmas duas linhas passaram a existir 2x no arquivo, e os manifestos antigos `0132-pregap-150.mut`/`0136-motor-respostas.mut` que as ancoravam pararam de bater (achavam 2 ocorrências em vez de 1). Extraí `sector_bin_offset` compartilhada; as duas baterias antigas foram reexecutadas (5/5+2/2 e 6/6+2/2) e continuam batendo. |

## Bateria de mutação

Placar da bateria: 7/7 mutantes mortos, 2/2 controles verdes, 0 equivalente — docs/mutantes/0207-cdrom-getloc.mut

m1/m2/m3 removem cada termo do OR de erro do GetlocL (motor, play, setor-nunca-lido) — mortos
por `getlocl_erro_80h_com_motor_parado_apos_leitura`, `..._durante_play` e
`..._antes_de_qualquer_leitura` respectivamente. m4 troca o INT3 de sucesso por INT2 — morto
por `getlocl_devolve_...`. m5 faz o ReadN nunca marcar `has_last_sector` — morto pelo mesmo
teste (viraria erro 80h em vez dos 8 bytes esperados). m6 remove a checagem de motor do
GetlocP — morto por `getlocp_erro_80h_com_motor_parado`. m7 zera a posição relativa do
GetlocP — morto por `getlocp_devolve_trilha_index_posicao_relativa_e_absoluta`.

Reexecutadas por âncora envelhecida (a refatoração de `sector_bin_offset` moveu as linhas que
elas ancoram): `0132-pregap-150` (5/5 mutantes, 2/2 controles) e `0136-motor-respostas` (6/6,
2/2) — ambas continuam batendo, sem regressão.

## Placar antes → depois

Workspace: **1264 → 1271** testes (7 novos em `cdrom_getloc.rs`).

## Revisão cruzada (orquestrador)

Rodada de autoria do orquestrador; a revisão adversarial é do mesmo agente que escreveu — fica
registrado em vez de escondido, como em 0159/0206.

O que dá para afirmar com medição independente do julgamento de quem escreveu: os 7 testes
falhavam antes da implementação (braço genérico devolvia só o stat byte) e passam depois; os
mutantes m1/m2 só morreram depois de eu perceber, medindo a bateria e não assumindo, que os
testes originais não isolavam a condição pretendida.

**Limitação conhecida, não coberta por teste:** § GetlocL (L1066-1071) de docs/reference/06-cdrom.md
também lista "falha durante o seek" como condição independente de "nenhum setor lido ainda". A
implementação atual não tem uma checagem de `seeking` dedicada — na prática, um GetlocL logo
após um SeekL (antes de qualquer ReadN) já cai na condição "nenhum setor lido ainda" e erra
pelo motivo certo por coincidência; mas um GetlocL emitido durante um SeekL que ocorre *depois*
de outro setor já ter sido lido (`has_last_sector=true` de uma leitura anterior, motor ligado,
não tocando o play) devolveria os dados do setor antigo em vez de INT5, divergindo da spec. Não
consegui isolar essa condição num teste sem depender de uma corrida entre dois agendamentos do
scheduler (2a resposta do SeekL sempre vence a 1a resposta de qualquer comando novo emitido
depois do ACK — ver erro de primeira tentativa #1) — fica para quem tiver uma abordagem melhor
de testar timing concorrente do scheduler sem depender da corrida.

## Decisões e notas

**`GetlocL` falha antes de qualquer leitura, medido em hardware real, não na spec.** § GetlocL
(L1062-1071) de docs/reference/06-cdrom.md só documenta falha durante play e durante seek. Mas
`docs/iterations/0175-cdrom-oraculo.md` mediu no hardware real do oráculo: `GetlocL failed,
IRQ=5` logo no início da suíte `cdrom/getloc`, antes de qualquer ReadN ter completado, mesmo com
o motor já girando (`GetStat -> 0x02`). Tratei isso como uma terceira condição de erro
(`has_last_sector`), citando a medição de hardware da 0175 como base — não é uma leitura livre
da spec, é o gabarito real preenchendo uma lacuna que a spec deixa aberta.

`GetlocP` não tem essa terceira condição: a spec descreve que ele lê Subchannel Q continuamente
(inclusive durante seek), não um buffer de setor — só falha com o motor parado.

`GetlocP` usa `read_pos_mm/ss/ff` como posição "atual" do drive, e `trilha_em`/`subtrai_msf` (já
usados pelo relato de posição do Play, item 4.4ad/0136) para resolver trilha/index e a posição
relativa. Isso herda uma imprecisão pré-existente e não nova desta iteração: `read_pos_*` só é
atualizado por Play/ReadN/ReadS, não por SeekL sozinho (`0x15` não grava `read_pos_*` em
nenhuma fase) — um GetlocP logo após um SeekL sem ReadN prévio reportaria uma posição
desatualizada. Não abri achado novo para isso porque é o mesmo comportamento (ausência de
atualização de `read_pos_*` pelo SeekL) que já afeta outros consumidores dessas variáveis;
registrar aqui para quem for medir isso depois.
