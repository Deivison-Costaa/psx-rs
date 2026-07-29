# STATUS

> Memória do projeto entre iterações. O contexto do agente é descartado a cada iteração;
> este arquivo não. Teto de 16 KB imposto por `status_size.rs`. Curto e verdadeiro.

## Última iteração concluída

**0072** — Correção de registro e revisão do PR #85 (ROADMAP 10.24): o job `scoreboard` da CI
sai verde medindo zero, porque sem BIOS o script rotula as 51 suítes como `sem-bios` e encerra 0.
Das 1982 linhas publicadas na branch `scoreboard-data`, 1981 são `sem-bios`. Corrige a nota 2 da
iteração 0068 e preenche a revisão cruzada da 0071.

Antes dela, **0071** — DPCR como gate de habilitação no DMA (ROADMAP 10.19). Medido no hardware:
`ps1-tests/dma/otc-test` foi de `6p/34f` para `7p/30f`, e `testOtcStandardWithMasterDisabled`
passou. Primeira correção do projeto guiada pela suíte de hardware.

## Próxima tarefa

**ROADMAP 10.20 — DMA — o que o OTC grava em cada endereço.**
`ps1-tests/dma/otc-test` ainda reprova 30 subtestes e todos são o mesmo defeito. Para um buffer
de 4 palavras, `testOtcStandard:28-31` espera `buf[0]=0xFFFFFF`, `buf[1]=&buf[0]`,
`buf[2]=&buf[1]`, `buf[3]=&buf[2]`: terminador no índice mais BAIXO, cada palavra apontando para
a de baixo. Nós gravamos o espelho — terminador em MADR, no índice mais alto, e ponteiros para
cima.
**O sentido de varredura já está certo e a spec confirma:** em `docs/reference/04-dma.md`, seção
"1F801088h+N\*10h - D#\_CHCR - DMA Channel Control (Channel 0..6) (R/W)" (L84), as restrições do
DMA6 dizem que o bit 1 do D6_CHCR é sempre 1, com `increment=-4` (L117-119). O OTC desce, e
`try_execute_otc` já desce. O defeito é o VALOR gravado em cada endereço e a ponta onde vai o
terminador: descendo a partir de MADR, cada palavra deve receber o endereço que será visitado a
seguir (o de baixo), e o terminador cai na última palavra escrita, que é a mais baixa. A seção
"DMA Register Summary" (L27) chama o canal 6 de "reverse clear OT" (L35).
**Armadilha:** `crates/psx-core/tests/dma_otc.rs:79-81` afirma hoje o espelho, com as mensagens
"ultimo slot = end marker" e "slot N-1 aponta para slot N". Esse teste é o que certificou o
defeito, e TEM de mudar junto — o teste próprio não é o árbitro, a spec e o `otc-test` são.
Confirme na mesma passagem a segunda diferença: gravamos o ponteiro dobrado em 21 bits
(`& 0x1F_FFFC`), e os valores esperados pelo `otc-test` são de 24 bits.
Arquivos-alvo: `crates/psx-core/src/dma.rs`, `crates/psx-core/tests/dma_otc.rs`.

**Depois desta, em ordem de evidência de hardware:** 10.21 (bit 15 do GPUSTAT sem o gate de
GP1(09h), 3 subtestes), 10.22 (mask bit, 2 subtestes), e o 4.3b abaixo, que não tem suíte de
hardware medindo.

**ROADMAP 4.3b — CDROM — Acoplar DiscLayout + dados do .bin.**
Substituir o buffer stub (`data_buffer` preenchido com `(i+1) & 0xFF`) por dados reais do arquivo .bin, usando o `DiscLayout` (item 4.2b). ReadN/ReadS devem ler setores do BIN a partir da posição definida por Setloc. Armadilha: o `Cdrom` hoje não tem referência ao `DiscLayout` nem ao buffer `.bin`; `Bus` precisa injetá-los ou o `Cdrom` precisa guardar uma referência.
Spec, em `docs/reference/06-cdrom.md`: seção "ReadN/ReadS" (L924).
Sequência de entrega do setor, na seção "CDROM Incoming Data / Buffer Overrun Timings" (L928) do mesmo arquivo: "Copy Data to Main RAM" (L940).
Arquivos-alvo: `crates/psx-core/src/cdrom.rs` (injetar DiscLayout + buffer BIN, ler setor real no deliver_second), `crates/psx-core/src/bus.rs` (passar dados do BIN para o Cdrom).

## Repositório

- `main` protegida a partir da iter 0004; 1 PR por item; merge commit (nunca squash);
  commits test→feat→docs; título de PR validado pela CI.
- **Escopo de commit é UM único identificador `[a-z0-9-]`.** `feat(bus,cpu)` reprova no
  `commit-lint`; quando a mudança toca dois módulos, escolha o principal e cite o outro no
  resumo. Custou uma reescrita de 4 mensagens no PR #36 (ver `0022-scratchpad-isc.md`).
- Iterações são cronológicas e nem sempre na ordem dos itens (0003↔item 0.5, p.ex.);
  o vínculo real está no título do PR e no doc da iteração.

## Placar de testes

Workspace: **556** testes (10 meta-testes + 8 bus_bios + 2 bios_flag + 1 version + 12 bus_scheduler + 9 bus_scratchpad_isc + 8 cpu_fetch_decode + 26 cpu_alu + 14 cpu_shifts + 19 cpu_load_delay + 24 cpu_branches + 7 cpu_jumps + 20 cpu_mult_div + 29 cpu_unaligned_load_store + 10 cpu_cop0_regs + 14 cpu_exception_mechanism + 1 cpu_exception_estado_previo + 9 cpu_tty_hook + 11 cpu_printf_hook + 11 cpu_opcode_reservado + 11 cpu_irq + 13 dma_otc + 14 dma_gpu + 12 cdrom_dma + 7 dma_dpcr_gate + 13 timers + 11 timers_sync + 9 timers_dotclock_hblank + 14 timers_irq + 16 gpu_status_gp0_gp1 + 9 ci_scoreboard + 9 cli_runner + 21 gpu_vram_transfers + 20 gpu_triangulos_flat_gouraud + 20 gpu_linhas_retangulos + 4 gpu_linhas_retangulos_continuacao + 6 gpu_textura_15bpp + 6 gpu_texturas_4bpp_8bpp + 5 gpu_texture_window + 6 gpu_semi_transparencia + 7 gpu_dithering + 8 gpu_mask_bit + 7 gpu_display_regs + 9 gpu_timing_vblank + 6 gpu_framebuffer + 3 gpu_desktop_egui + 6 gpu_scoreboard + 13 cdrom_regs + 11 cdrom_seek_pause + 11 cdrom_bin_cue + 10 cdrom_read + 1 spec_citations + 2 mutation_manifest + 2 mutation_anchors + 5 mutation_battery + 1 mutation_reconciliation).

## Bloqueios

(nenhum)

## Invariantes

1. **Imediato de endereçamento é SINALIZADO.** Todo load/store (`lw/sw/lb/lh/...`) e todo
   `addi/addiu/slti/sltiu` sign-extendem o campo de 16 bits: `(instr & 0xFFFF) as u16 as
   i16 as u32`. Só a família lógica (`andi/ori/xori`) zero-extende. Violado na iter 0011
   (SW), pego na revisão adversarial; qualquer item novo que leia um imediato reconfere
   esta linha antes de escolher a extensão.

## Notas

1. BIOS local: `bios/SCPH1001.BIN` (MD5 924E392ED05558FFDB115408C263DCCF), gitignored,
   validada na iter 0009 (item 0.9). Nunca commitar.
2. **Load delay × escrita no mesmo registrador: comportamento ASSUMIDO, não verificado
   (resolve no item 1.11).** Quando a instrução do delay slot escreve o registrador
   destino do load (`lw r10,..` seguido de `ori r10,..`), a nossa implementação faz o
   **load vencer**. A spec local não decide: `02-cpu.md § Caution - Load Delay` só diz que
   o registrador "não é atualizado até o próximo opcode ter completado", o que fala de
   leitura, não de precedência de escrita. Não mudamos sem evidência (R1). O teste
   `load_delay_vs_escrita_no_mesmo_registrador_comportamento_assumido` fixa o que fazemos
   hoje e nomeia a dúvida, para que uma futura mudança seja deliberada. Ponto de
   resolução: Amidog `psxtest_cpu` no item 1.11 — se ele reprovar, inverter a ordem em
   `Cpu::step` (commitar o load antes de executar, escrevendo num banco de saída).
3. **BcondZ com `rt` fora da tabela: comportamento ASSUMIDO (resolve no item 1.11).** O
   opcode 01h só tem `rt`=00h/01h/10h/11h tabelados em `02-cpu.md § Opcode/Parameter
   Encoding`; a spec local não diz o que `rt`=02h..0Fh/12h..1Fh fazem. Assumimos **no-op
   silencioso** (nem desvia nem linka). O teste
   `bcondz_rt_fora_da_tabela_comportamento_assumido` fixa isso e diz na asserção que é
   suposição. Se o Amidog `psxtest_cpu` reprovar, o critério a testar primeiro é o de
   hardware conhecido: bit16 sozinho decide BLTZ/BGEZ e o link ocorre quando os bits
   20..17 valem 1000b — o que faria `rt`=02h agir como BLTZ.
4. **EPC e BadVaddr são graváveis via MTC0 — comportamento ASSUMIDO (resolve no item
   1.11).** A spec marca ambos como (R), mas o comportamento sob escrita não está
   documentado localmente. Implementados como R/W na 1.8a. Os testes
   `epc_gravavel_comportamento_assumido` e `badvaddr_gravavel_comportamento_assumido`
   fixam o comportamento atual. Se o Amidog `psxtest_cpu` reprovar, adicionar `if reg ==
   8 || reg == 14 { return; }` em `cop0_write`.
5. **TAR (cop0r6) é R/W — comportamento ASSUMIDO (resolve no item 1.11).** Mesmo
   critério de EPC/BadVaddr: spec marca (R), implementado como R/W sem evidência
   contrária.
6. **Registradores N/A do COP0 (r0-r2, r4, r10, r32-r63) não disparam exceção — dívida
   do 1.8c, a agendar depois do 1.11.** Leitura retorna 0, escrita é ignorada. O comportamento correto é Reserved
   Instruction Exception (excode=0Ah).
7. **Acesso ao COP0 em User mode com COP0 disabled — dívida do 1.8c, a agendar depois do 1.11.** Acessar qualquer
   registrador do COP0 que não seja garbage (r16-r31), ou executar RFE, em User mode com
   COP0 disabled (SR.bit1=1 e SR.bit28=0) gera Coprocessor Unusable Exception (excode=0Bh).
   Os registradores garbage r16-r31 podem ser acessados nesse estado sem exceção. Fonte:
   `docs/reference/02-cpu.md`, seção cop0r16-r31 - Garbage (L885).
8. **E1 — Entrada de exceção preserva bits Sw (8-9) e IP (10-15) do CAUSE.** A escrita
   do ExcCode agora usa máscara: `self.cop0[13] = (self.cop0[13] & !0xC000_007C) | cause`,
   gravando apenas BD (bit31), BT (bit30) e ExcCode (bits 2-6). O erro original
   (`self.cop0[13] = cause`) zerava os bits Sw, contradizendo a spec que diz "clear them
   before returning from the exception handler" — instrução de software que só faz sentido
   se o hardware não limpar sozinho. Corrigido na revisão adversarial do PR #35.
9. **E2 — Empilhamento de SR na entrada da exceção: comportamento ASSUMIDO (resolve no
   item 1.11).** O inverso exato do RFE: bits 0-1 (IEc/KUc) → bits 2-3 (IEp/KUp), bits 2-3
   (IEp/KUp) → bits 4-5 (IEo/KUo), bits 0-1 zerados. A spec local NÃO documenta o push,
   apenas o RFE que desempilha. Sem o push, um handler que execute RFE restaura lixo nos
   bits IEc/KUc. Ponto de resolução: Amidog `psxtest_cpu`. Testes:
   `sr_e_empilhado_na_entrada_da_excecao` e `sr_push_seguido_de_rfe_restaura_os_bits_0_3`.
10. **E3 — Load delay commitado antes da exceção: comportamento ASSUMIDO (resolve no
   item 1.11).** O acesso à memória do `lw` já ocorreu quando a exceção da instrução
   seguinte é reconhecida; o valor pendente é commitado antes do desvio para o handler.
   A spec local não tem evidência sobre este caso (R1). Escolha (a) entre duas opções
   igualmente plausíveis. Teste:
   `load_pendente_e_commitado_antes_da_excecao_comportamento_assumido`.

11. **A spec se contradiz sobre o EPC do `syscall` — ASSUMIDO (resolve no item 1.11).**
   `cop0r14 - EPC` diz que o registrador guarda "the address at which an exception occured",
   o que dá `EPC = endereço do próprio syscall`; a seção `exception opcodes` descreve o
   handler examinando `[epc-4]` para ler o opcode que causou a exceção, o que só fecha se
   `EPC` apontasse para a instrução **seguinte**. Implementamos a primeira leitura, que é a
   do registrador em si e a que os testes B2/B4 fixam. As duas leituras se reconciliam se
   `[epc-4]` for descuido de redação sobre o caso BD (onde `EPC` de fato aponta 4 bytes
   antes). Não mudamos sem evidência (R1). Ponto de resolução: Amidog `psxtest_cpu`.
12. **`file_size.rs`: `cpu_exception_mechanism.rs` está em 487 linhas de 500.** O próximo
    teste de exceção vai para `cpu_exception_estado_previo.rs`, criado na segunda rodada de
    revisão do PR #35, ou para um arquivo novo com nome próprio. Não corte casos existentes.
13. **Linhas INCLUEM a coordenada inferior-direita (`docs/reference/03-gpu.md` L361-362); polígonos EXCLUEM (`docs/reference/03-gpu.md` L323).**
    As duas funções de rasterização têm regras de borda OPOSTAS. `render_single_line` usa
    Bresenham com break em `x==x1 && y==y1` (inclusivo); `render_triangle` usa
    `xr.min(area_x2 + 1)` (exclusivo). Não reusa uma na outra.
