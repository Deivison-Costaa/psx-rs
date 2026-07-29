# 0055 — irq-controller

- **Data:** 2026-07-29
- **Item do roadmap:** 3.1
- **Objetivo:** implementar I_STAT/I_MASK no bus, conectar ao COP0 (CAUSE.IP bit10) e gerar excecao INT quando (I_STAT & I_MASK) != 0, SR.IEc=1 e SR.Im bit10=1.

## Revisão do PR anterior

Revisão do PR anterior (0054): sem achados.
- 1. Teste que não mede: 6 testes de propriedades + 5 mutantes todos mortos. OK.
- 2. Parâmetro não consumido → FIFO dessincronizado: sem comandos GP0 novos. OK.
- 3. Regra de borda trocada: sem rasterização. OK.
- 4. Campo de bit lido errado: sem manipulação de bits. OK.
- 5. Panic ou laço ilimitado: sem unwrap()/unsafe fora de teste. OK.
- 6. Citação de spec: confere-citacoes.ps1 verde. OK.
- 7. Escopo transbordado ou dívida não declarada: item 2.9 fechado, dívidas no doc. OK.

Nota: o manifesto de mutação 0054 foi criado em f7c1935 e deletado em a89328f — a deleção foi intencional porque o alvo era scripts PowerShell (rejeitados pelo meta-teste mutation_anchors.rs) e o doc da 0054 registra "Bateria de mutação: não se aplica".

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| psx-spx | I_STAT / I_MASK (L21-L22) | docs/reference/11-interrupts.md |
| psx-spx | Interrupt Request / Execution (L45) | docs/reference/11-interrupts.md |
| psx-spx | COP0 Interrupt Handling (L68) | docs/reference/11-interrupts.md |
| psx-spx | PSX specific COP0 Notes (L74) | docs/reference/11-interrupts.md |
| psx-spx | cop0r13 - CAUSE (L670) | docs/reference/02-cpu.md |
| psx-spx | cop0r12 - SR (L704) | docs/reference/02-cpu.md |
| psx-spx | Exception Vectors (L816) | docs/reference/02-cpu.md |
| psx-spx | Exception Priority (L832) | docs/reference/02-cpu.md |

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | endereçamento | Que o ROADMAP estava correto ao dizer "CAUSE.IP bit 2" | 11-interrupts.md L75: "PSX uses only cop0r13.bit10" — o campo IP ocupa bits 10-15 do CAUSE, e a PSX usa só o bit 10 | leitura da spec antes de codificar |
| 2 | API-Rust | Que o manifesto de mutação apontava irq.rs como alvo, bastando um `arquivo:` para desviar para cpu.rs | O campo `alvo:` do manifesto define o arquivo padrão; mutações em outros arquivos precisam de `arquivo:` explícito | mutantes.ps1 falhou: "@@DE não encontrada" — corrigi o `alvo:` para `crates/psx-core/src/cpu.rs` |
| 3 | API-Rust | Que o m3 podia ter @@PARA vazio (remover todo o bloco de atualização do CAUSE.IP) | O meta-teste mutation_anchors.rs rejeita @@PARA vazio (mutante inerte) | CI reprovou — troquei por substituição (`self.cop0[13] &= !(1 << 10)`) |
| 4 | API-Rust | Que o K1 podia renomear `sr` para `status_reg` (só trocar o nome da variável) | A renomeação quebrou o uso de `sr` abaixo, virando mutante em vez de controle | Erro de compilação no mutantes.ps1 — troquei por adição de comentário |

## Bateria de mutação

Bateria de mutação: 5/5 mutantes mortos, 2/2 controles verdes, 0 equivalente — docs/mutantes/0055-irq-controller.mut

Placar da bateria: 5/5 mutantes mortos, 2/2 controles verdes, 0 equivalente - ./docs/mutantes/0055-irq-controller.mut

| Mutante | Teste que o pegou |
|---|---|
| m1 (SR.IEc ignorado) | `interrupcao_nao_dispara_sem_sr_iec` |
| m2 (SR.Im bit10 ignorado) | `interrupcao_nao_dispara_sem_sr_im_bit10` |
| m3 (CAUSE.bit10 sempre zero) | `cause_bit10_reflete_irq_pendente` |
| m4 (ExcCode 01h em vez de 00h) | `interrupcao_dispara_excecao_int` |
| m5 (vetor 8000_0040h para INT) | `interrupcao_dispara_excecao_int` |

## Placar antes → depois

Workspace: **408** → **418** testes (408 existentes + 10 cpu_irq).

## Revisão cruzada (orquestrador)

<!-- Preenchido pelo Claude na revisão do PR -->

## Decisões e notas

1. **CAUSE.bit10 é dinâmico, não latch.** Atualizado no início de cada `step()` com base em `bus.irq().pending()`. A spec diz: "is NOT a latch, ie. it gets automatically cleared as soon as (I_STAT AND I_MASK)=zero" (11-interrupts.md L75-77).
2. **I_STAT acknowledge:** escrever 0 limpa o bit, escrever 1 não altera. A fórmula `self.stat &= val | !0x7FF` implementa isso para bits 0-10.
3. **I_MASK mascara bits 0-10** (0x7FF). Bits 11-15 são sempre zero, 16-31 são garbage.
4. **Vblank ainda não conectado.** O GPU tem `enter_vblank()`/`exit_vblank()` mas não chama `irq.raise(0)`. Isso será feito quando o scheduler de eventos estiver ativo (ROADMAP 3.4 — Timers).
5. **O ROADMAP 3.1 diz "CAUSE.IP bit 2"** — é um erro de digitação; o correto é bit 10 (a PSX usa só o bit 10 dos 6 bits IP disponíveis no COP0).
