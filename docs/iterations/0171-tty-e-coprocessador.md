<!-- Custo, tokens e duracao ficam em docs/metricas.csv, medidos pelo runner. -->

# 0171 — tty-e-coprocessador

- **Data:** 2026-08-03
- **Item do roadmap:** 10.97, 10.98 e 10.99 (três numa rodada)
- **Objetivo:** tirar os dois artefatos que escondiam divergência real, e corrigir a primeira
  família que eles revelaram.
- **Fonte:** orquestrador.

**R4 dobrado a pedido do usuário.** A regra diz uma micro-funcionalidade por iteração; o usuário
pediu explicitamente os três itens juntos ("não tem pra que dividir tanto"), porque o custo de
uma rodada não é o código, é a espera de suíte e CI. Registro aqui para o histórico não mentir:
esta rodada tem três mudanças de produção e uma de CI.

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| psx-spx | § cop0r12 - SR (L744-747) | docs/reference/02-cpu.md |
| psx-spx | § cop0r16-r31 - Garbage (L892) | docs/reference/02-cpu.md |
| psx-spx | § mov [mem],cop0reg / mov cop0reg,[mem] - coprocessor cop0 load/store (L881) | docs/reference/02-cpu.md |

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a medição diz | Como foi pego |
|---|---|---|---|---|
| 1 | medição | Que os números de TTY que reportei a noite toda fossem reais. | Não é assunto de spec. | Ao remover a duplicação, `VSync: timeout` no Rayman caiu de 142 para **71** — metade exata. Todo número de TTY citado nas iterações 0163-0170 estava inflado em 2x. O teste `rayman_tty_boot` fixava `> 100`, ou seja, fixava o artefato. |
| 2 | spec | Que bastasse remover o prefixo `% ` do gabarito para alinhar. | Não é assunto de spec. | Só `cpu/cop` tem o prefixo (19 de 19 linhas); as outras 20 suítes não têm nenhuma. Remover por padrão seria palpite sobre formato. A regra virou: remover só quando **todas** as linhas não-vazias o têm. |
| 3 | hardware | Que a prosa do psx-spx bastasse para o Coprocessor Unusable: § mov [mem],cop0reg (L881) diz que load/store de cop0 "causes a Coprocessor Unusable Exception", sem qualificar. | O gabarito de hardware `cpu/cop/psx.log` do ps1-tests diz `pass - testSwc0Enabled`, isto é, **não lança** com CU0 ligado. | Conflito real entre prosa e hardware. Resolvido pela regra que satisfaz os dois: a isenção de modo kernel vale para `copN`, não para `lwcN`/`swcN`, e o bit CU manda em ambos. |
| 4 | teste | Que os testes de GTE já ligassem CU2. | § cop0r12 - SR (L746) de `docs/reference/02-cpu.md`: bit 30 é `CU2 COP2 Enable (GTE in PSX)`. | 28 testes em quatro arquivos `gte_*.rs` quebraram assim que passamos a checar. Eles nunca ligaram CU2 porque nós nunca checávamos — código real liga antes de usar o GTE. |

## As três mudanças

**10.97 — TTY duplicado.** O hook de alto nível (`printf` em `A0h/3Fh`, `puts` em `A0h/3Eh` e
`B0h/3Fh`) existia para o ambiente sem kernel, onde os vetores de syscall eram `jr $ra` e ninguém
mais emitia. Depois da 0170 o kernel é real: o hook escreve a string e a rotina da BIOS escreve de
novo, byte a byte, pelo `putchar` — que também interceptamos. Agora o hook de alto nível só emite
quando o vetor está stubado; o `putchar` emite sempre, porque é a saída do dispositivo.

**10.98 — alinhamento do oráculo.** Nosso TTY passou a trazer o banner de boot da BIOS, que o
gabarito não tem. `Get-TtyVeredito` agora alinha na primeira linha do gabarito que aparece na
nossa saída, conta as linhas de gabarito puladas até a âncora como divergência, e devolve
`sem-alinhamento` quando não há âncora nenhuma — em vez de inventar um K/M.

**10.99 — Coprocessor Unusable.** A regra do R3000A é o bit CU do SR, não a existência do
coprocessador: `cop1`/`cop3` com CU ligado **não** lançam (são no-op), e `cop2`/`swc2` com CU2
desligado **lançam**. Nosso despacho fazia o oposto nos dois casos.

## Bateria de mutação

Placar da bateria: **7/7 mutantes mortos, 2/2 controles verdes, 0 equivalente.**

Na primeira execução m5 e m7 sobreviveram — ambos apagam a distinção entre `copN` e
`lwcN`/`swcN` para a unidade 0, e o meu arquivo de teste só cobria `swc0` com CU0 **ligado**.
O assassino existia noutro arquivo (`cpu_opcode_reservado`), o que não vale: a bateria mede a
cobertura do teste da rodada. Fechei a lacuna com
`load_e_store_de_cop0_lancam_sem_cu0_mesmo_em_modo_kernel` e os dois morreram.

Os registros declaram `teste:` individualmente porque a rodada tem dois assassinos
(`cpu_tty_sem_duplicar` e `cpu_coprocessador_usavel`). Isso contorna o item **10.71** — a
ramificação `teste` de registro vaza para o cabeçalho no `switch` do PowerShell — de forma
inofensiva: se todos os registros declaram o seu, não sobra registro para herdar o vazamento.

## Placar antes → depois

Workspace: **939 → 952** testes.

**Oráculo de TTY (`K/M` = K linhas divergentes de M).** Antes da 0171 o arreio não alinhava e
contava o preâmbulo de boot como divergência, então toda suíte marcava `M/M` — 21 de 21 sem
nenhuma linha em comum. Depois:

| Suíte | ee5fc83 | c987251 (10.97+10.98) | c7c0851 (10.99) |
|---|---|---|---|
| gpu/gp0-e1 | 12/12 | **0/12 idêntico** | **0/12 idêntico** |
| gpu/mask-bit | 7/7 | **0/7 idêntico** | **0/7 idêntico** |
| cpu/cop | 19/19 | 7/19 | **1/19** |
| cdrom/disc-swap | 11/11 | 7/11 | 7/11 |
| cpu/code-in-io | 10/10 | 7/10 | 7/10 |
| mdec/4bit, mdec/8bit | 19/19 | 11/19 | 11/19 |
| dma/chain-looping | 11/11 | 9/11 | 9/11 |
| spu/memory-transfer | 11/11 | 9/11 | 9/11 |

Total: **0 → 2 idênticas**, e as 21 melhoraram. As duas idênticas são a primeira paridade
byte a byte com hardware real medida no projeto.

A única divergência que sobrou em `cpu/cop` é `testCop0InvalidOpcode`: o hardware **não** lança
com um `cop0cmd` inválido que não seja TLBxx, e nós lançamos `0Ah`. A spec local só fala dos
quatro TLB — § cop0cmd=01h,02h,06h,08h - TLBR,TLBWI,TLBWR,TLBP (L877-878) de docs/reference/02-cpu.md
— e é omissa sobre o resto da faixa. Fica registrado no ROADMAP; não
cabia nesta rodada porque exige descobrir qual encoding o teste usa.

## Revisão cruzada (orquestrador)

Rodada de autoria do orquestrador; autorrevisão registrada como limite. O que sustenta o
resultado é externo: dois gabaritos de hardware real passaram a bater byte a byte, e nenhuma das
21 suítes piorou.

## Decisões e notas

O `ci_workflow.rs` foi alterado **de propósito**. Ele exigia a string literal `cargo test --all`;
como `cargo test --all --doc` contém essa substring, trocar a suíte para o nextest teria deixado o
guarda verde sozinho. Agora ele exige `cargo nextest run --all-targets` e `cargo test --all --doc`
explicitamente. Motivo da troca: o `cargo test` roda os 139 binários em série, e a medição de
02/08 mostrou 189 s de CI concentrados em dez diagnósticos que emulam dezenas de milhões de
passos.
