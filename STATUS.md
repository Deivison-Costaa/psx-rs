# STATUS

> Memória do projeto entre iterações. O contexto do agente é descartado a cada iteração;
> este arquivo não. Teto de 16 KB imposto por `status_size.rs`. Curto e verdadeiro.

## Última iteração concluída

**0022** — Cache isolation + scratchpad + memory control (ROADMAP 1.9): decodificação
de região no Bus para Scratchpad 1KB (`1F800000h..1F8003FFh`), memory control stubs
(`1F801000h..1F801023h` + `1F801060h`), BCC (`FFFE0130h`) e `SR.Isc` (bit 16) suprimindo
stores no CPU. 6 testes, 5/5 mutantes pegos, 1/1 controle verde. Armadilha conhecida:
D3 precisou de testemunha RAM dupla porque o alias do range errado passava o readback
simples. Ver `docs/iterations/0022-scratchpad-isc.md`.

**0023** — Iteração de processo (sem código): incorporação das métricas pendentes da 0022 e
registro do erro de escopo múltiplo em commit reprovado pelo `commit-lint`.

**0024** — Iteração de processo (sem código): o handoff do 1.10 descrevia A0h/B0h como
códigos de `syscall`; a spec diz que são endereços de chamada (`jal 0xA0`) com o número da
função em R9. Corrigido contra `13-kernel-bios.md` antes de despachar o trabalhador. Ver
`docs/iterations/0024-handoff-1.10-corrigido.md`.

## Próxima tarefa

**ROADMAP 1.10** — Hook de TTY (A0h/B0h) → BIOS imprimindo no console.

Primeiro item que conecta CPU + Bus + saída visível.

> **O handoff anterior deste item estava errado e foi corrigido na iter 0024 contra a spec.**
> A0h/B0h **não** são códigos de `syscall`: são **endereços de chamada**. O código chama
> `jal 0x000000A0` (ou `0xB0`, `0xC0`) com o **número da função em R9 (`$t1`)**, argumentos em
> R4-R7 e retorno em R2. `syscall` é outra tabela (SYS-Functions, número em R4). Não invente:
> leia `13-kernel-bios.md` L496 antes de codificar.

Funções de TTY deste item, **exatamente como a spec as numera**:

| Função | Chamada | R9 | Argumento |
|---|---|---|---|
| `putchar(char)` | `jal 0xA0` | `3Ch` | R4 = caractere (byte baixo) |
| `putchar(char)` | `jal 0xB0` | `3Dh` | R4 = caractere (byte baixo) |
| `puts(src)` | `jal 0xA0` | `3Eh` | R4 = ponteiro para string terminada em `00h` |
| `puts(src)` | `jal 0xB0` | `3Fh` | R4 = ponteiro para string terminada em `00h` |

**Escopo:** um hook no `Cpu::step` que dispara quando o PC **físico** (`pc & 0x1FFF_FFFF`)
vale `A0h` ou `B0h`; se R9 casar com uma das quatro entradas acima, o byte (ou a string lida
byte a byte via `bus.read8`) vai para um buffer de saída no `Bus`
(`tty_push` / `take_tty`), e a execução **segue normalmente** — o hook só observa, não desvia
o PC nem sintetiza um retorno.

**NÃO inclui:** as demais funções A/B/C; `getchar`/`gets` (entrada); expansão de TAB/LF do
`putchar` real (ver armadilha 3); impressão no psx-cli (só o buffer no core; o CLI drena
depois, no item do runner).

**Spec:** `docs/reference/13-kernel-bios.md` — L496 (`A-Functions`, convenção de chamada),
L481 (`Parameters, Registers, Stack`), L2776 (`putchar`), L2742 (`puts`).

**Arquivos-alvo:** `crates/psx-core/src/cpu.rs` (hook no `step`), `crates/psx-core/src/bus.rs`
(buffer + `tty_push`/`take_tty`), teste novo `crates/psx-core/tests/cpu_tty_hook.rs`.

### Armadilhas

1. **Não precisa de BIOS nem de mini-handler.** A armadilha registrada antes (vetor
   `0x8000_0040`, handler em RAM) pressupunha o mecanismo errado de `syscall`. Com o hook em
   `jal 0xA0`, o teste monta `jal` + delay slot em RAM, ajusta R9/R4 e passa a executar até o
   PC chegar em `A0h`. Nenhuma BIOS envolvida.
2. **Espelhos de segmento.** O PC pode chegar como `0x000000A0`, `0x800000A0` ou `0xA00000A0`.
   Compare o endereço **físico**, nunca o virtual.
3. **`puts(0)` imprime `<NULL>`.** A spec é explícita (L2746-2749): se R4 aponta para um `00h`
   nada é impresso, mas se R4 é `00000000h` a saída é os seis caracteres `<NULL>`, sem CR/LF.
4. **String sem terminador trava o emulador.** `puts` deve ter um teto de bytes lidos
   (proponha 1 MiB) e parar, em vez de varrer a memória para sempre.
5. **O `putchar` real expande TAB e LF** (L2778-2780: `09h` → espaços até o próximo múltiplo
   de 8; `0Ah` → `0Dh,0Ah`). Como o hook **observa** em vez de substituir a função, este item
   grava o byte **cru**. Decisão deliberada, não esquecimento: registre como nota ASSUMIDA no
   doc da iteração, com ponto de resolução = comparar com a saída de uma BIOS real.

### Testes de aceitação

**D1 — putchar por A0h.** RAM com `jal 0xA0` + delay slot, R9=`0x3C`, R4=`'X'`. Executa até o
PC valer `0xA0`. Exigido: `take_tty()` devolve `b"X"`.

**D2 — putchar por B0h usa outro número.** Mesmo cenário com `jal 0xB0` e R9=`0x3D`. Exigido:
`b"X"`. E com `jal 0xB0` + R9=`0x3C` (número de A0h na tabela errada): **nada** é emitido.

**D3 — puts lê até o `00h`.** R9=`0x3E`, R4 aponta para `"oi\0"` em RAM. Exigido: `b"oi"`, sem
o terminador.

**D4 — `puts(0)` emite `<NULL>`.** R9=`0x3E`, R4=`0`. Exigido: `b"<NULL>"`.

**D5 — Número desconhecido é ignorado.** R9=`0xFF` em `jal 0xA0`. Exigido: buffer vazio, sem
pânico, e a execução continua (o hook não altera PC nem registradores).

**D6 — Espelho KSEG0.** Salto para `0x800000A0` com R9=`0x3C`. Use `jalr` (o alvo vem de um
registrador): `jal` monta o alvo com os 4 bits altos do PC e, a partir de KUSEG, não alcança
`0x8...`. Exigido: mesmo resultado de D1.

## Repositório

- `main` protegida a partir da iter 0004; 1 PR por item; merge commit (nunca squash);
  commits test→feat→docs; título de PR validado pela CI.
- **Escopo de commit é UM único identificador `[a-z0-9-]`.** `feat(bus,cpu)` reprova no
  `commit-lint`; quando a mudança toca dois módulos, escolha o principal e cite o outro no
  resumo. Custou uma reescrita de 4 mensagens no PR #36 (ver `0022-scratchpad-isc.md`).
- Iterações são cronológicas e nem sempre na ordem dos itens (0003↔item 0.5, p.ex.);
  o vínculo real está no título do PR e no doc da iteração.

## Placar de testes

Workspace: **209** testes (10 meta-testes + 8 bus_bios + 2 bios_flag + 1 version + 12 bus_scheduler + 6 bus_scratchpad_isc + 8 cpu_fetch_decode + 26 cpu_alu + 14 cpu_shifts + 19 cpu_load_delay + 24 cpu_branches + 7 cpu_jumps + 20 cpu_mult_div + 27 cpu_unaligned_load_store + 10 cpu_cop0_regs + 14 cpu_exception_mechanism + 1 cpu_exception_estado_previo).

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
   `docs/reference/02-cpu.md`, seção cop0r16-r31 - Garbage (L805).
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
