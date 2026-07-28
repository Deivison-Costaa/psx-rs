# STATUS

> Memória do projeto entre iterações. O contexto do agente é descartado a cada iteração;
> este arquivo não. Teto de 16 KB imposto por `status_size.rs`. Curto e verdadeiro.

## Última iteração concluída

**0031** — Rodada de correção (I1-I4): `permissions: contents: write` no job, pré-filtro por extensão + magic bytes (51 linhas/varredura, sem readmes), publish só na main, graceful exit sem EXEs (ROADMAP 1.12). Scoreboard 50/51. Bateria 10/10, 2/2. 247 testes (6 ci_scoreboard).
Ver `docs/iterations/0031-ci-scoreboard-job.md`.

## Próxima tarefa
**ROADMAP 1.14** — Opcode não implementado gera exceção (RI 0Ah / CpU 0Bh) em vez de panic.

Item novo, criado a partir de uma medição (`docs/iterations/0032-handoff-2-1.md`): com o
GPUSTAT devolvendo o valor de reset da spec, as suítes do ps1-tests destravam e passam a
executar de verdade — e a primeira delas a avançar (`ps1-tests/cpu/cop/cop.exe`) **derruba o
emulador**:

```
thread 'main' panicked at crates\psx-core\src\cpu.rs:231:18:
not implemented: opcode primary=38 nao implementado
```

Enquanto o `unimplemented!()` estiver no decodificador, cada suíte que alcança um opcode novo
mata o processo em vez de virar uma linha de placar. Por isso este item vem **antes** do 2.1.

**Spec:** `docs/reference/02-cpu.md` (linhas absolutas do arquivo, conferidas com `grep -n`):

| Fato | Linha |
|---|---|
| Opcodes reservados → Reserved Instruction Exception, `excode=0Ah` | L230 |
| Ler cop0r0..r2/r4/r10/r32..r63 → RI `0Ah` | L874 |
| TLBR/TLBWI/TLBWR/TLBP → RI `0Ah` | L878 |
| **`mov [mem],cop0reg` / `mov cop0reg,[mem]` (LWC0/SWC0) → Coprocessor Unusable, `excode=0Bh`, NÃO 0Ah** | L883-884 |

O opcode que o `cop.exe` alcançou é o primary `38h` = SWC0, ou seja, o caso de `0Bh` — não o
de `0Ah`. Os dois códigos existem e não são intercambiáveis.

**Arquivos-alvo:** `crates/psx-core/src/cpu.rs` (o `_ => unimplemented!(...)` do `execute`, e o
mesmo padrão em `special`/`cop0_op` se houver), `crates/psx-core/tests/cpu_opcode_reservado.rs`
(arquivo novo).

### Armadilhas

1. **O mecanismo de exceção já existe** desde o item 1.8b (`raise_exception`, `pending_exception`,
   bit BD, EPC). Este item **não** reimplementa nada disso: só troca o `panic` por uma chamada
   ao mecanismo existente com o excode certo. Leia `cpu.rs` antes de escrever qualquer coisa.
2. **`0Ah` e `0Bh` não são o mesmo caso.** Coprocessor loads/stores (primary `30h..33h` e
   `38h..3Bh`) são `0Bh` (L883-884). Opcode simplesmente inexistente é `0Ah` (L230). Um teste
   para cada.
3. **Não confundir com o COP2/GTE.** `12h` (COP2) e `3Ah` (SWC2) pertencem ao GTE, que é item
   do M3 — hoje também caem no `unimplemented!`. Se você fizer todos virarem exceção, o GTE
   vai passar a levantar `0Bh` silenciosamente em vez de estourar; **registre isso no doc**,
   porque é uma mudança de comportamento que o M3 vai precisar desfazer.
4. **Não existe `unwrap`/`expect`/`panic!`/`unimplemented!` em produção (R6).** Se sobrar
   algum caminho de panic no decodificador depois deste item, ele é bug do item.
5. **O contador de ciclos e o delay slot continuam valendo.** Exceção levantada em delay slot
   já tem tratamento (bit BD) desde a 1.8b — o teste tem que cobrir opcode reservado dentro de
   delay slot, senão o item não fecha o que a 1.8b abriu.

### Testes de aceitação

**A1 — RI (`0Ah`).** Opcode primary inexistente (ex.: `3Fh`) executado: `CAUSE.excode == 0Ah`,
`EPC` aponta para a instrução, PC vai para o vetor de exceção.

**A2 — CpU (`0Bh`).** `SWC0` (primary `38h`): `CAUSE.excode == 0Bh`. Mesmo teste para `LWC0`
(primary `30h`).

**A3 — em delay slot.** Opcode reservado no delay slot de um `jal`: `CAUSE.BD == 1` e `EPC`
aponta para o **branch**, não para o delay slot (regra da 1.8b).

**A4 — o `cop.exe` deixa de derrubar o processo.** Com o stub temporário de GPUSTAT descrito
no doc da 0032 (`0x1F80_1814 => Some(0x1480_2000)` no `bus.rs`, **não commitar**), rodar
`psx-cli --bios bios/SCPH1001.BIN --exe tests/exes/ps1-tests/cpu/cop/cop.exe` e verificar que
o processo termina normalmente em vez de entrar em pânico. **Cole a saída no doc.** Sem o stub,
o `cop.exe` nem chega ao opcode — então A4 sem o stub não prova nada.

**A5 — nenhum panic sobrando.** `grep -rn "unimplemented!\|panic!\|unwrap()\|expect(" crates/psx-core/src/`
não devolve nada fora de teste. Cole a saída (vazia) no doc.

## Repositório

- `main` protegida a partir da iter 0004; 1 PR por item; merge commit (nunca squash);
  commits test→feat→docs; título de PR validado pela CI.
- **Escopo de commit é UM único identificador `[a-z0-9-]`.** `feat(bus,cpu)` reprova no
  `commit-lint`; quando a mudança toca dois módulos, escolha o principal e cite o outro no
  resumo. Custou uma reescrita de 4 mensagens no PR #36 (ver `0022-scratchpad-isc.md`).
- Iterações são cronológicas e nem sempre na ordem dos itens (0003↔item 0.5, p.ex.);
  o vínculo real está no título do PR e no doc da iteração.

## Placar de testes

Workspace: **247** testes (10 meta-testes + 8 bus_bios + 2 bios_flag + 1 version + 12 bus_scheduler + 9 bus_scratchpad_isc + 8 cpu_fetch_decode + 26 cpu_alu + 14 cpu_shifts + 19 cpu_load_delay + 24 cpu_branches + 7 cpu_jumps + 20 cpu_mult_div + 27 cpu_unaligned_load_store + 10 cpu_cop0_regs + 14 cpu_exception_mechanism + 1 cpu_exception_estado_previo + 9 cpu_tty_hook + 11 cpu_printf_hook + 6 ci_scoreboard + 9 cli_runner).

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
