<!-- Custo, tokens e duração NÃO entram aqui: são medidos por scripts/oc-iter.ps1 em
     docs/metricas.csv, casados por head_antes/head_depois. -->

# 0172 — cpu-oraculo

- **Data:** 2026-08-03
- **Item do roadmap:** 10.100, 10.3 (quatro suítes numa rodada)
- **Objetivo:** fechar o lote A (CPU/kernel) do oráculo de TTY: `cpu/cop`, `cpu/access-time`,
  `cpu/code-in-io`, `cpu/io-access-bitwidth`.
- **Fonte:** orquestrador (lote `logs/orquestrador/lote-A.txt`).

**R4 dobrado a pedido do usuário.** A regra diz uma micro-funcionalidade por iteração; aqui o
lote inteiro fecha numa rodada porque o custo não é o código, é a espera de suíte e CI — decisão
do usuário, registrada aqui para o histórico não mentir: esta rodada tem quatro suítes e cinco
correções de produção.

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| psx-spx | § cop0cmd=01h,02h,06h,08h - TLBR,TLBWI,TLBWR,TLBP (L876) | docs/reference/02-cpu.md |
| psx-spx | § Caution - 8/16-bit writes to certain IO registers (L309) | docs/reference/02-cpu.md |
| psx-spx | § Scratchpad (L114, L137-140) | docs/reference/01-memory-map.md |
| psx-spx | § Memory Exceptions (L156-160) | docs/reference/01-memory-map.md |
| psx-spx | § Exception Priority (L832) | docs/reference/02-cpu.md |
| psx-spx | § Interrupt Control (L76) / § MDEC Registers (L130) | docs/reference/14-io-map.md |

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec/medição diz | Como foi pego |
|---|---|---|---|---|
| 1 | hardware | Que "Bus Error → Unused Memory Regions" (L160) cobria o bloco de 4K de I/O Ports inteiro (1F801000h-1F801FFFh), então flaguei tudo ali como bus error de fetch. | O gabarito `cpu/code-in-io/psx.log`: `testCodeInInterrupts` e `testCodeInMDEC` lançam, mas `testCodeInDMA0`/`DMAControl`/`SPU` NÃO. A spec local é omissa sobre qual sub-bloco. | Rodei o binário real após a correção "ampla": os três testes que deviam passar (`wasExceptionThrown()==false`) passaram a falhar porque eu lançava onde não devia. Instrumentei `eprintln!` temporário no fetch para achar os 5 endereços exatos por instrumentação, não por leitura de seção única — a lição do lote D (Setloc/§ Error Codes) se aplica igual aqui: uma seção silenciosa não autoriza generalizar. |
| 2 | medição | Que, corrigido o bus error, `cpu/code-in-io` completaria e daria um K/M limpo. | SPU (0x1F801C00) não tem estado nenhum (M7 vazio): `region_read32` devolve zero fixo e a escrita é descartada. O teste escreve um "retorno" ali antes de saltar; nós ignoramos a escrita, então o fetch acha zero (NOP) e a CPU anda para sempre por SPU→EXPANSION2, sem terminar em 800 M passos. | `--max-steps` maior não ajudou (30 M e 800 M dão o mesmo resultado parcial). Não é regressão minha — é uma lacuna pré-existente que meu fetch bus error tornou alcançável pela primeira vez. Registrado como item novo (10.108) em vez de forçar um K/M inventado. |
| 3 | mutação | Que inserir o novo bloco de bus error entre o check de alinhamento e o fetch não afetaria manifestos antigos. | `mutation_anchors.rs`: dois registros do manifesto **0164** (`m4`, `c2`) ancoravam um bloco de 4 linhas que atravessava exatamente o ponto onde inseri código novo. | Suíte completa reprovou após o fix. Encurtei as âncoras (mesmo alvo semântico, sem a linha `let instr = ...` que deixou de ser vizinha) e rodei a bateria 0164 de novo — 6/6 mortos, 3/3 controles, sem regressão. |
| 4 | duplicação | Que copiar a lógica de `region_read32` para uma função nova (`dma_register_value`) não teria custo. | O meta-teste de âncoras do **meu próprio** manifesto 0172 reprovou: duas âncoras bateram 2-3 vezes porque o texto duplicado existia em dois lugares. | Refatorei `region_read32` para chamar `dma_register_value` em vez de duplicar — âncoras ficaram únicas e o código ficou menor. |

## Bateria de mutação

Placar da bateria: **8/8 mutantes mortos, 2/2 controles verdes, 0 equivalente** —
`docs/mutantes/0172-cpu-oraculo.mut`. Três testes assassinos (`cpu_cop0_invalid_opcode`,
`cpu_io_write_width_echo`, `cpu_fetch_bus_error`, mais `cpu_cop0_regs` para o m3) — todo
registro declara `teste:` (contorna o item 10.71). Verde na primeira execução.

## Placar antes → depois

Workspace: **953 → 972** testes (+2 do merge de `main`/0177; +17 novos: `cpu_cop0_invalid_opcode`
×3, `cpu_io_write_width_echo` ×8, `cpu_fetch_bus_error` ×6).

**Suítes do lote A (`K/M` = K linhas divergentes de M):**

| Suíte | antes (0172) | depois (0172) | o que fechou |
|---|---|---|---|
| `cpu/cop` | 1/19 | **0/19** | `testCop0InvalidOpcode`: só RFE (10h) e TLBxx (01h/02h/06h/08h) decidem algo; resto do `cop0cmd` é no-op, não 0Ah. |
| `cpu/io-access-bitwidth` | 31/67 | **25/67** | `DMA0_ADDR` e `DMAC_CTRL` (DPCR) fecham nas 3 tabelas (8/16/32 bit): `sb`/`sh` num registrador de DMA colocam os 32 bits do rt no barramento, mascarados pela lógica que já existia. |
| `cpu/code-in-io` | 7/10 (iter 0171) | 4 primeiras linhas corretas (RAM/Scratchpad/MDEC/Interrupts); resto **não mensurável** — trava (achado 10.108) | Bus error 06h em scratchpad + I_STAT/I_MASK + MDEC, os três comprovados por instrumentação. |
| `cpu/access-time` | 18/22 | 18/22 (adiado) | É o próprio ROADMAP 10.1 (timing model completo); o teste do ps1-tests diz "no assertions - please manually compare". Não cabia nesta rodada. |

`io-access-bitwidth` não fechou totalmente: restam I_MASK (ecoa os 32 bits crus, sem a máscara
de 11 bits que `Irq::write_mask` aplica — exigiria redesenhar o registro para guardar o valor
bruto além do funcional), SIO/JOY (registros de 16 bits com mascaramento próprio, fora do lote),
timers (bits "open bus" no read de 32 bits) e `Dma::write_dicr` (deixa passar o bit 6: grava
`0x340078` em vez de `0x340038` ao escrever `0x12345678` — bug em `dma.rs`, fora do lote A).

## Revisão cruzada (orquestrador)

Pendente — PR aberto para revisão adversarial do orquestrador.

## Decisões e notas

- **`fetch_causa_bus_error`/`write8_gpr_completo`/`write16_gpr_completo` são métodos novos em
  `Bus`**, não mudanças de assinatura de `write8`/`write16`/`read32` — 23 arquivos de teste de
  OUTROS lotes chamam essas funções diretamente; mudar a assinatura quebraria trabalho
  concorrente. O custo foi maior (métodos extras) pela segurança de zero blast radius.
- Não toquei em `dma.rs`, `irq.rs`, `sio.rs`, `gpu.rs`, `mdec.rs`, `spu.rs`, `timers.rs` —
  todos pertencem a outros lotes rodando em paralelo. Os achados que exigiriam tocá-los (DICR,
  I_MASK, SPU) ficam registrados, não corrigidos.
- Reparei o manifesto **0164** (âncoras) e re-rodei a bateria dele — ambos os commits estão
  nesta branch porque a quebra foi causada pela minha própria mudança nesta rodada.
