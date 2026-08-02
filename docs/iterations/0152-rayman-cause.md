# 0152 — rayman-hook-cause

- **Data:** 2026-08-02
- **Item do roadmap:** 10.76
- **Objetivo:** separar a causa COP0 das entradas do hook e rastrear a ativação 0 até o acesso ao contador.

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| psx-spx | § B(19h) - HookEntryInt(addr) (L1476-L1482) | `docs/reference/13-kernel-bios.md` |
| psx-spx | § Priority Chains (L1484-L1502) | `docs/reference/13-kernel-bios.md` |
| psx-spx | § Interrupt Request / Execution (L45-L50) | `docs/reference/11-interrupts.md` |
| psx-spx | § Interrupt Acknowledge (L52-L66) | `docs/reference/11-interrupts.md` |
| psx-spx | § cop0r13 - CAUSE (L689-L698) | `docs/reference/02-cpu.md` |
| psx-spx | § Opcode/Parameter Encoding (L179-L203) | `docs/reference/02-cpu.md` |
| psx-spx | § CPU Load/Store Opcodes (L239) | `docs/reference/02-cpu.md` |

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | diagnóstico | A hipótese mais provável era que a maioria das entradas sem `I_STAT` fosse `SYSCALL` | `CAUSE.ExcCode` precisa ser lido na entrada; `00h` é INT e `08h` é syscall (docs/reference/02-cpu.md § cop0r13 - CAUSE (L689-L698)) | O teste vermelho começou exigindo 15 syscalls; a sonda encontrou `ExcCode=00h` em 1029/1029 entradas |
| 2 | identificação | O último `B(19h)` observado poderia ser usado como alvo do hook | O jogo e o kernel instalam hooks diferentes; a amostra precisa exigir `hook[0] == 0x801B8E60` | A primeira execução contou 1029 entradas em `0x8005A1D8` e não chegou ao spin; filtrei o alvo no momento da instalação |
| 3 | endereçamento | O opcode estático `0xAC22F2CC` bastava para atribuir o store a `0x801DF2CC` | Load/store usa `[rs+imm]` e o imediato negativo `0xF2CC` deve ser aplicado com sinal (docs/reference/02-cpu.md § Opcode/Parameter Encoding (L179-L203); § CPU Load/Store Opcodes (L239)) | O watchpoint dinâmico mediu `$at=0x801D0000` e endereço efetivo `0x801CF2CC` no passo `164112414` |

## Bateria de mutação

Bateria de mutação: não se aplica — diagnóstico puro; nenhuma linha de produção foi alterada, somente teste permanente e documentação, e a sonda descartável foi revertida antes do commit.

## Placar antes → depois

Workspace: **886** → **887** testes (+1: `rayman_hook_cause.rs`). A primeira execução do portão falhou somente porque o `STATUS.md` ainda declarava 886; após atualizar o handoff, a suíte completa passou.

## Diagnóstico

### (A) Causa do COP0

A sonda foi filtrada por `hook[0] == 0x801B8E60`, executou a BIOS SCPH1001 com `Rayman (USA) DADOS.cue` e capturou `CAUSE`, `I_STAT` e `I_MASK` antes de cada primeira instrução do hook. Em todas as **1029** entradas, `CAUSE.ExcCode` foi `00h` (INT); houve **0** entradas com `08h` (SYSCALL). Portanto, a explicação sugerida de que as 15 entradas sem status seriam syscalls está refutada: o hook está sendo alcançado a partir de uma exceção de interrupção, não de `SYSCALL`.

O fato de `CAUSE.ExcCode=00h` não identifica qual IRQ ficou pendente. A spec diz que o hook só é executado depois que o `ExceptionHandler` termina por inteiro e que handlers podem chamar `ReturnFromException` depois de processar uma IRQ, pulando o hook (docs/reference/13-kernel-bios.md § B(19h) - HookEntryInt(addr) (L1476-L1482)).

A cadeia padrão coloca `VblankIrq` antes do hook e permite que um handler que reconheceu e deu ack retorne imediatamente (docs/reference/13-kernel-bios.md § Priority Chains (L1484-L1502)).

O ack de `I_STAT` limpa o bit escrevendo zero (docs/reference/11-interrupts.md § Interrupt Acknowledge (L52-L66)).

Assim, os 1028 casos em que o hook entra com `I_STAT=0` são interrupções cujo status já não está pendente na entrada; não são syscalls. Esta rodada não isolou qual handler limpou cada causa e não atribui sozinha uma causa de produção. A hipótese de `VblankIrq` consumir e pular especificamente a ativação 0 também não se aplica: a ativação 0 chegou ao hook com `I_STAT.bit0=1`.

### (B) Ativação 0

O traço da entrada do passo `164112358` foi:

| PC | Instrução | Estado observado | Próximo fluxo |
|---|---|---|---|
| `0x801B8F0C` | `0x1060000D` (`beq v1,zero,0x801B8F44`) | `v1=1` | branch não tomado; continua |
| `0x801B8F34` | `0x10400003` (`beq v0,zero,0x801B8F44`) | `v0=0x801B8C3C` | branch não tomado; chama o bloco do contador |
| `0x801B8F3C` | `0x0040F809` (`jalr v0`) | `v0=0x801B8C3C` | entra em `0x801B8C3C` |
| `0x801B8C40` | `0x8C42F2CC` (`lw v0,0xF2CC(v0)`) | `$v0=0`, `$at` será `0x801D0000` | segue para o incremento |
| `0x801B8C50` | `0xAC22F2CC` (`sw v0,0xF2CC(at)`) | `$v0=1`, `$at=0x801D0000` | executa no passo `164112414` |

Não existe o desvio de controle pedido: o caminho **não se afasta** de `0x801B8C40`; ele chega ao `sw` em `0x801B8C50`. O ponto concreto que separa o resultado observado do endereço alegado é o próprio `0x801B8C50`: `0xF2CC` é `-0xD34` como imediato de 16 bits, então `0x801D0000 + 0xFFFFF2CC = 0x801CF2CC`. O store escreve `1` em `0x801CF2CC`, não em `0x801DF2CC`. O teste permanente `rayman_hook_cause.rs` fixa essa propriedade sem fazer skip por BIOS ou disco.

### VBlank em 1029 entradas

Houve **1/1029** entradas com `I_STAT & I_MASK & 1 != 0`: a entrada 0, no passo `164112358`. As outras 1028 entradas não tinham VBlank habilitado pendente na entrada do hook. Nas 20 primeiras, `CAUSE.ExcCode=00h` em todas, `I_STAT.bit0` aparece uma vez e os demais casos com status não-zero são DMA ou CDROM, conforme a amostra anterior.

O `I_STAT=0` observado não prova por si só que o kernel deveria ter pulado o hook: a spec documenta a possibilidade de `ReturnFromException`, não uma obrigação para cada handler. O dado seguro desta iteração é que a ativação com VBlank pendente executa o bloco e falha em atualizar o endereço que o diagnóstico vinha chamando de contador; não há alteração de hardware nesta rodada.

## Revisão cruzada (orquestrador)

**O achado está correto e derruba a premissa central de três iterações.** Conferi a aritmética
por conta própria: `0xF2CC` como imediato de 16 bits com sinal é `-0xD34`, e
`0x801D0000 - 0xD34 = 0x801CF2CC`. Confere. Conferi as sete citações de spec abrindo as faixas
no corpo — todas apontam para o conteúdo certo.

**O que isto invalida.** As iterações 0149, 0150 e 0151 trataram `0x801DF2CC` como "o contador
do VSync", endereço obtido lendo o imediato `0xF2CC` como se fosse **sem sinal**. Sobre essa
premissa a 0151 concluiu que o `sw` de `0x801B8C50` "nunca é executado". A medição desta rodada
mostra o contrário: **ele executa, no passo 164112414**, e escreve `1` em `0x801CF2CC`. O
caminho do contador nunca esteve bloqueado. A 0151 não errou a observação — ela procurou escrita
num endereço que o programa nunca teve intenção de usar, e não achar nada ali era o resultado
correto para a pergunta errada.

É o mesmo defeito que eu próprio encontrei horas antes no `beq_target` da 0151, onde
`i32::from(u16)` estendia com zero um deslocamento de branch com sinal. Duas vezes na mesma
noite, em análise e em código: **imediato de 16 bits do MIPS é com sinal, e tratá-lo como sem
sinal produz um endereço plausível e errado.**

**Uma hipótese minha foi refutada por medição.** No handoff eu sugeri que as ativações sem
`I_STAT` fossem provavelmente `SYSCALL`, já que `0x80000080` é o vetor de exceção geral. A sonda
mediu `CAUSE.ExcCode = 00h` em **1029 de 1029** entradas: todas interrupção, zero syscall. A
explicação estava errada, e o teste vermelho que a rodada escreveu primeiro (exigindo 15
syscalls) foi o que a pegou.

**O que a revisão corrigiu.** A rodada drenou `logs/metrics-pending.csv` inteiro, incluindo seis
linhas que o orquestrador já havia commitado na 0151 — o `docs/metricas.csv` ficou com seis
pares `(ts, iter)` duplicados. Removi as duplicatas, reordenei o bloco final por timestamp e
acrescentei a linha da própria 0152, que a rodada esqueceu. Também corrigi o nome da seção em
duas citações: em `docs/reference/02-cpu.md` o conteúdo de L689-L698 mora sob `cop0r13 - CAUSE`,
não sob o título pai `COP0 - Exception Handling`, e o `spec_citations` reprovava por isso. O tratamento do ROADMAP
estava correto (item movido para `ROADMAP-fechado.md`, não marcado com `[x]` no `ROADMAP.md`).

**A rodada não completou o protocolo.** Ela parou depois de commitar teste e sonda, deixando
doc, STATUS, ROADMAP e métricas sem commit e sem abrir PR, mesmo assim reportada como `ok` pelo
`oc-iter.ps1` — mais uma manifestação do 10.78. O orquestrador fechou o ciclo.

**O que fica em aberto, e é a pergunta que interessa agora.** Só **1 das 1029** ativações do
hook tinha VBlank pendente. Se o contador só incrementa uma vez, o `VSync()` fica girando à
espera de um segundo incremento que nunca vem — o que explicaria o timeout sem que nada no
caminho do hook esteja quebrado. Falta medir duas coisas: (i) se o `lw` do spin em `0x801B95AC`,
aplicado o mesmo imediato com sinal, lê `0x801CF2CC` — isto é, se escrita e leitura concordam; e
(ii) por que 1028 ativações chegam ao hook sem VBlank pendente, se a IRQ0 sobe 660 vezes.
Registrado como **10.79**.

## Decisões e notas

- O teste permanente é um oráculo pequeno da amostra real: deriva `ExcCode` dos bits 6:2, conta o VBlank habilitado e calcula o endereço efetivo do opcode observado.
- Nenhuma linha de `crates/*/src/` foi alterada; não há correção segura a fazer antes de revisar a premissa do endereço `0x801DF2CC`.
- O item 10.76 foi fechado por diagnóstico em `docs/ROADMAP-fechado.md`; o próximo handoff é 10.77, sem começar esse item nesta rodada.
