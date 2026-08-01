<!-- Custo, tokens e duração NÃO entram aqui: são medidos por scripts/oc-iter.ps1 em
     docs/metricas.csv, casados por head_antes/head_depois. -->

# 0136 — motor-respostas

- **Data:** 2026-08-01
- **Item do roadmap:** 4.4ad
- **Objetivo:** fechar o motor de respostas do CD-ROM com gate de IRQ, timing por comando e
  leitura sequencial de setores.

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| psx-spx | § Responses (L1969-1982) | docs/reference/06-cdrom.md |
| psx-spx | § First Response (INT3) (L1984-2000) | docs/reference/06-cdrom.md |
| psx-spx | § Second Responses (L2002-2026) | docs/reference/06-cdrom.md |
| psx-spx | § Second Response (L2066-2076) | docs/reference/06-cdrom.md |
| psx-spx | § ReadN/ReadS (L924-926) | docs/reference/06-cdrom.md |

## Mecanismo

O estado do CD-ROM usa as duas flags documentadas para INT2 e INT1; INT3 é entregue pela
execução do comando. O gate impede a execução com IRQ pendente. Quando o comando é aceito
sem IRQ pendente, o bus cancela o evento de setor que ainda estava armado e agenda a primeira
resposta; isso escolhe uma ordem determinística permitida pelo mainloop sem transformar as
respostas em uma fila genérica.

Pause e Stop usam timings distintos para motor/leitura/parado, e apenas os dez comandos da
tabela da spec armam segunda resposta. ReadN e ReadS avançam a posição MSF após cada INT1 e
rearmam a leitura. Setmode (0Eh) responde INT3 e armazena o modo, mas o buffer segue fixo em
800h — o tamanho 924h pelo bit5 é dívida aceita (a revisão pegou a alegação invertida aqui:
`mode` não era lido em lugar nenhum).

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | scheduler | cancelar o evento de setor somente quando a primeira resposta fosse entregue bastaria | o mainloop não tem prioridade fixa entre setor e comando (`docs/reference/06-cdrom.md` L1947-1952); neste caminho o comando aceito precisa cancelar o setor já armado antes do prazo da resposta | `pause_durante_leitura_leva_o_tempo_de_5_setores` falhou com INT1 antes do INT3; correção moveu o sinal de cancelamento para o latch |
| 2 | timing | uma contagem global de comandos poderia distinguir a primeira execução de Pause | a tabela separa Pause em leitura, Pause já pausado e Stop por estado do drive (`docs/reference/06-cdrom.md` L2066-2076) | os goldens de Pause em idle e durante ReadN expuseram o atraso artificial; constantes passaram a depender do estado |
| 3 | processo | todos os manifestos antigos continuariam com âncoras válidas após a reestruturação do CD-ROM | âncoras envelhecidas devem ser atualizadas ou arquivadas, nunca ignoradas | `mutation_anchors` encontrou cinco manifestos históricos; foram arquivados com o placar preservado |

## Bateria de mutação

Placar da bateria: **6/6 mutantes mortos, 2/2 controles verdes, 0 equivalente** —
`docs/mutantes/0136-motor-respostas.mut`.

Mutantes M1/M2 cobrem o cancelamento de setor armado, M3/M6 cobrem timing e tipo da segunda
resposta do Pause, M4 cobre avanço de MSF e M5 cobre o pregap ao indexar o BIN. Os seis
falharam em `cdrom_motor`; os dois controles permaneceram verdes.

## Placar antes → depois

**850 → 857** testes no workspace. Verificação final: `cargo fmt --all -- --check`,
`cargo clippy --all-targets -- -D warnings` e `cargo test --all` verdes.

## Revisão cruzada (orquestrador)

Um achado: o doc alegava "escolhem o tamanho de buffer pelo Setmode", mas `mode` não era
lido em lugar nenhum e o buffer é fixo em 800h — alegação corrigida acima, 924h registrado
como dívida (padrão de erro de narrativa, mesmo das 0125-0127). A escolha de ordem
determinística (cancelar o setor armado no aceite do comando, em vez de reter a entrega)
foi revalidada contra 06-cdrom.md L1947-1949 e L1997-1999: ambas as ordens são permitidas;
a estrita fica nos goldens. Mutantes M1/M2 reaplicados na re-execução da bateria pelo
orquestrador antes do merge.

## Decisões e notas

- O item fecha a dívida 10.53: comando com INT pendente fica retido até o acknowledge.
- A ausência de fila real de IRQs e o limite de uma INT1 não entregue permanecem modelados
  por flags, conforme `docs/cdrom-comandos.md`.
- Física real de seek, áudio, overrun de oito slots, VideoCD, Unlock e GetQ continuam fora
  do escopo aceito do motor.
- O próximo muro medido não é GTE: `docs/spikes/sideload-crash.md` registra `VSync: timeout`
  seguido de permanência no vetor de exceção. O próximo diagnóstico deve tratar VSync/IRQ0 do
  jogo antes de abrir trabalho adicional de GTE.
