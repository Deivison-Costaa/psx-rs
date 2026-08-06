<!-- Custo, tokens e duração NÃO entram aqui: são medidos por scripts/oc-iter.ps1 em
     docs/metricas.csv, casados por head_antes/head_depois. -->

# 0201 — silent-hill-hang

- **Data:** 2026-08-05
- **Item do roadmap:** 10.53 (achado legado, aberto desde a 0121)
- **Objetivo:** o usuário pediu pra jogar Silent Hill (USA); o jogo trava depois da tela de
  abertura. Investigar a causa e consertar o que der pra fechar nesta iteração.

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| psx-spx | § First Response (INT3) (or INT5 if failed) (L1984) | docs/reference/06-cdrom.md |

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | diagnóstico | Que o conserto do achado 10.53 (comando executa com INT pendente) resolveria o travamento do Silent Hill, porque o padrão de sintoma (CD-ROM para, só sobra vblank) batia com a descrição do achado | O repro de 400M passos do Silent Hill deu **byte-a-byte idêntico** antes e depois do conserto (mesmo `sh_err.log`, mesmas 4021 linhas, mesmos `total_cycles`) — o caminho que o conserto muda nunca é exercitado por esta partida específica do jogo. O achado 10.53 era real e valia a pena fechar, mas não é a causa do travamento do Silent Hill | Comparação byte-a-byte do `sh_err.log` gerado antes e depois do fix, com o mesmo comando de repro |
| 2 | modelo mental | Que `int2_pending`/`int1_pending` significam "há um INT2/INT1 sentado em `intsts` sem ack" | Essas flags caem para `false` no ack do INT3 (antes mesmo do INT2 ser *entregue* em `intsts`) — elas marcam "ainda devo uma segunda resposta", não "intsts tem INT sem ack". Um comando latchado depois desse ack mas antes do ack do INT2 real passava direto, porque `deliver_first()` só olhava essas flags, não `intsts` bruto | Escrevi um teste (`comando_novo_durante_int_pendente_e_entregue_apos_o_ack`) assumindo que `int2_pending` continuava `true` até o ack do INT2; ele falhou na PRIMEIRA asserção (`pre`), não na que eu esperava — obrigou a reler `send_command(0x0A)` e o handler de ack linha a linha |
| 3 | mutação | Que `if blocked { return true; }` (em vez de `false`) seria morto pelo teste novo, já que inverte o contrato do booleano | Sobreviveu: o efeito colateral observável (`scheduler.cancel(EventId(CDROM_SECOND))` no chamador) só diverge quando existe uma 2ª resposta JÁ AGENDADA e não disparada no exato instante de um `deliver_first()` bloqueado — e por construção do código (agendamento só acontece dentro do próprio handler de ack, que zera `intsts`), essa janela nunca coexiste com `intsts != 0` nos fluxos que os testes do repo exercitam. Troquei o mutante por um que ataca a própria condição (`pending_cmd.get().is_none()`), que o teste mata de verdade | `mutantes.ps1 -Iter 0201`: m3 sobreviveu em 1,1 s |

## Bateria de mutação

Placar da bateria: 5/5 mutantes mortos, 2/2 controles verdes, 0 equivalente —
`docs/mutantes/0201-silent-hill-hang.mut`.

- m1 (volta à checagem antiga `int2_pending||int1_pending`): morto — `ainda_bloqueado` passa a
  ver INT3 (3) em vez do INT2 retido (2).
- m2 (`!=` → `==`, invertido): morto — o próprio `init_ate_int3` já não entrega INT3 do Init.
- m3 (condição vira `pending_cmd.get().is_none()`): morto — `ainda_bloqueado` vê o GetStat
  executar na hora (3) em vez de ficar retido (2).
- m4 (`blocked = false`, nunca bloqueia): morto — mesma asserção de m3.
- m5 (`blocked = true`, sempre bloqueia): morto — o próprio Init nunca entrega INT3.
- c1 (renomear a variável local `blocked`): verde.
- c2 (trocar a ordem de `param_buf`/`param_count`, campos independentes): verde.

## Placar antes → depois

Workspace: **1242** → **1243** testes (1 novo em `cdrom_fila_int.rs`).

## Revisão cruzada (orquestrador)

Sem achados — esta iteração foi conduzida pelo próprio orquestrador (exceção vigente em
`docs/orquestracao.md`; ver STATUS.md).

## Decisões e notas

**1. Achado 10.53 fechado, mas não pela razão que motivou a iteração.** O usuário pediu pra
investigar por que Silent Hill trava depois da tela de abertura. A investigação achou e
consertou um defeito real e antigo (`deliver_first()` bloqueava por `int2_pending`/
`int1_pending` em vez de `intsts` bruto, violando `06-cdrom.md` L1984-1989), mas o repro do
Silent Hill deu **idêntico** antes e depois — o travamento tem outra causa, ainda não
identificada. Registrado como achado novo 0201.1 em `docs/achados.md`.

**2. Defeito de segundo nível achado pelo próprio conserto.** Corrigir `deliver_first()`
expôs que o *fixture* `setloc()` de seis arquivos de teste de CD-ROM (`cdrom_dma`,
`cdrom_pregap`, `cdrom_read`, `cdrom_seek_pause`, `cdrom_setor_mode2`, `dma_dpcr_gate`) nunca
ackava o INT3 do Setloc antes do próximo comando — o bug antigo mascarava isso porque Setloc
não seta `int1_pending`/`int2_pending`. `cdrom_motor.rs` já fazia o ack certo; os outros
arquivos foram alinhados ao mesmo padrão.

**3. O travamento do Silent Hill segue aberto.** Sintoma medido: o jogo desenha a tela de
abertura (VRAM muda entre os dumps 1-3 de `--dump-vram-every`), depois **para completamente**
por 150M+ passos — só sobra um par de eventos de vblank se repetindo
(`DeliverEvent class(a0)=0xF2000003/0xF0000001`). A última atividade de CD-ROM antes do
silêncio é um `CDROM_SECOND_IRQ2 intsts=0x02` (INT2) que nunca é seguido de outro evento de
CD-ROM — não dá pra saber, sem mais instrumentação, se o INT2 nunca chega a ser processado
pelo driver do jogo ou se o driver processa e entra num laço de espera por outra coisa (setor
específico, campo do `stat`, etc). Repro: `./target/release/psx-cli.exe --bios
bios/SCPH1001.BIN --disc "../roms/extraido/Silent Hill (USA).cue" --max-steps 400000000 --pad
--dump-vram-every 50000000 sh`.
