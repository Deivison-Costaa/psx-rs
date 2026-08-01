<!-- Custo, tokens e duração NÃO entram aqui: são medidos por scripts/oc-iter.ps1 em
     docs/metricas.csv, casados por head_antes/head_depois. -->

# 0135 — tabela-cdrom

- **Data:** 2026-08-01
- **Item do roadmap:** 4.4ad (passo 1 de 3 — diagnóstico puro, zero código de produção)
- **Objetivo:** Ler a spec do CD-ROM INTEIRA (suspensão pontual e por escrito da R8,
  autorizada no plano de saída de 01/08) e produzir `docs/cdrom-comandos.md` — o design
  doc do motor de respostas, com citação de linha real para cada fato.

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| psx-spx | ARQUIVO INTEIRO (2182 linhas, 4 faixas + consolidação) | docs/reference/06-cdrom.md |
| psx-spx | ARQUIVO INTEIRO (edge-trigger L46-47; ordem de ack L57-66; CAUSE.b10 não-latch L74-81) | docs/reference/11-interrupts.md |

## Método

Extração delegada a 5 agentes (4 faixas de linhas + consolidador), com regra de ouro
"linha citada = linha real conferida com grep, nunca o índice do topo". Verificação do
orquestrador por amostragem, todas em 06-cdrom.md: L581-591, L925-926, L1073-1086,
L1968-2000, L2004-2014 e L2064-2076 de 06-cdrom.md — todas batem ("delegar a busca,
nunca a verificação").

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | extracao-spec | o consolidador reportou "conflito: opcode 0x11 é GetlocP E também 'não usado, retorna INT5'" | o `11h` da tabela geral (06-cdrom.md L581-591) é BYTE DE ERRO do payload INT5 dos opcodes não usados; GetlocP é o comando 11h (06-cdrom.md L1073) | verificação por amostragem do orquestrador; falso conflito removido do doc |

## Bateria de mutação

Bateria de mutação: não se aplica — zero linhas de código de produção nesta iteração;
diagnóstico puro que só produz documentação (docs/cdrom-comandos.md e o registro do
spike); nada mutável foi introduzido (precedente: 0133).

## Placar antes → depois

850 → 850 (sem mudança).

## Revisão cruzada (orquestrador)

A revisão É o passo 2 do método (amostragem de citações + eliminação do falso conflito) e
o preenchimento da seção "Decisões de escopo do motor" no próprio doc: dentro (modelo de
2 flags fiel ao hardware, gate de comando com INT pendente, 2ª resposta só nos 10 comandos
documentados, tabela de timing por comando, leitura sequencial com avanço de seek), fora
(áudio XA/CDDA/Play, física de seek, overrun além de 3 slots, VideoCD/Unlock/subchannel,
glitch de HINTSTS de consoles antigos) — cada exclusão vira dívida aceita no ROADMAP.

## Decisões e notas

- **Achado que muda o desenho do motor:** o hardware NÃO tem fila — tem 2 flags (INT2 e
  INT1 pendentes; INT3 é imediato, sem flag) e no máximo 1 INT1 não entregue
  (06-cdrom.md L1969-1982). O motor implementa o modelo de flags, não uma fila genérica.
- **Spike de sideload registrado em `docs/spikes/sideload-crash.md`** (fora do fluxo
  iterate, andaime não commitado): injeção pós-kernel do `SCUS_949.00` FUNCIONA — o jogo
  roda `CD_init`, driver de pad e `ResetGraph`, e trava em `VSync: timeout` + 100% dos
  PCs no vetor de exceção. **GTE ainda não é o muro; o muro seguinte é VSync/IRQ do
  jogo** — item-pai nomeado a abrir depois do motor. VRAM zerada (nada desenhado).
- A regra de ack em duas etapas (I_STAT primeiro, porta depois — 11-interrupts.md L57-66)
  e o CAUSE.b10 não-latch (L74-81) explicam mecanicamente a tempestade em 0x80000080 do
  spike: um IRQ que nunca é reconhecido re-entra no handler para sempre.
- Próximo passo (B2): goldens do orquestrador em `crates/psx-core/tests/cdrom_motor.rs`
  citando `docs/cdrom-comandos.md`; depois (B3) despacho do worker com R4 suspensa por
  escrito para implementar o motor até os goldens ficarem verdes.
