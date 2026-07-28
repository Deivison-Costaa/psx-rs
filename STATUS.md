# STATUS

> Memória do projeto entre iterações. O contexto do agente é descartado a cada iteração;
> este arquivo não. Teto de 16 KB imposto por `status_size.rs`. Curto e verdadeiro.

## Última iteração concluída

**0029** — Hook de printf A(3Fh) com expansão de % → Amidog imprimindo no TTY (ROADMAP 1.11b).
Suporte a `%%`, `%c`, `%s`, `%d`/`%i`, `%u`, `%x`/`%X` com argumentos de A1/A2/A3 e pilha em
`[SP+10h..]`. Teto de 1 MiB no laço principal de varredura da string de formato (correção H2
da revisão adversarial). Especificadores fora do escopo (`%o`, `%n`, etc.) emitidos como
literal. `%` no fim da string emitido antes do break. Bateria 8/8, 3/3. Scoreboard 50/51
produziram saída (rótulo `tty`/`sem-saida`, veredito real no 1.12). 241 testes no workspace.
Ver `docs/iterations/0029-cpu-printf-hook.md`.

## Próxima tarefa

**ROADMAP 1.12** — CI: job scoreboard ligado + primeiro placar real no histórico.

**Spec:** O item não tem spec de hardware. Os arquivos de referência são:

| Fonte | Seção | Arquivo local |
|---|---|---|
| ROADMAP | Item 1.12 (L32) | `ROADMAP.md` |
| Scoreboard | script completo (L1-96) | `scripts/scoreboard.ps1` |
| CI workflow | job check (L1-24) | `.github/workflows/ci.yml` |
| Fetch de EXEs | script de download (L1-67) | `scripts/fetch-test-exes.ps1` |
| BIOS | nota 1: local e hash | `STATUS.md` L52-53 |

**Arquivos-alvo:**
- `.github/workflows/ci.yml` — novo job `scoreboard` após `check`, com steps: checkout → rust → cache → fetch-test-exes → build psx-cli → roda `scripts/scoreboard.ps1` → commita `logs/scoreboard.csv` na branch `scoreboard-data`.
- `scripts/scoreboard.ps1` — já invoca `psx-cli --bios --exe` de verdade (resolvido na 0029); o job de CI só precisa chamá-lo.

**Armadilhas:**
1. **BIOS é gitignored** (`STATUS.md` L52-53). O scoreboard já emite `sem-bios` quando `bios/SCPH1001.BIN` não existe. O job de CI precisa ou baixar a BIOS de um secret, ou aceitar `sem-bios` como estado legítimo.
2. **EXEs são gitignored** — `scripts/fetch-test-exes.ps1` baixa do GitHub Releases. O job de CI precisa rodá-lo antes do scoreboard, e se falhar (sem token, rate-limit), o scoreboard roda vazio (0/0) sem quebrar o job.
3. **Branch `scoreboard-data` é órfã.** Commits nela são append-only (`logs/scoreboard.csv`), nunca force-push. O job precisa de `actions/checkout` com `fetch-depth: 0` ou checkout separado da branch de dados. Documente o mecanismo no doc da iteração.
4. **Timeout:** o Amidog `psxtest_cpu` roda ~0,3 s com step-limit de 50M. O timeout do job por EXE em `scripts/scoreboard.ps1` está em 120s — se um EXE travar, o job pode levar `N * 120s`. Considere timeout de job no workflow (`timeout-minutes`).
5. **Nem todo `.exe` em `tests/exes/` é um PS-EXE.** O zip do ps1-tests traz utilitários de host (`ps1-tests/tools/diffvram-windows-amd64.exe`); o glob `-Include *.exe` os pega, o `load_psexe` reprova no magic e o placar registra `fail-erro`, poluindo a série. Filtrar pelos 8 primeiros bytes (`PS-X EXE`, `16-cdrom-file-formats.md` L1163) em vez da extensão, ou dar status próprio a esse caso. Medido em 28/07: 51 arquivos varridos, 1 é binário de host.
6. **O status `tty`/`sem-saida` NÃO é veredito de teste.** O scoreboard hoje mede "produziu saída no TTY", não se o EXE passou ou falhou. O critério de veredito real (ler a saída de cada suite e extrair `pass`/`fail`) é trabalho do 1.12 — não "conserte" o placar mexendo no limiar de bytes do TTY.

**Testes de aceitação:**
- A1: `cargo test -p psx-cli --test cli_runner psxtest_cpu_sideload_executa_sem_panico` passa quando `psxtest_cpu.exe` existe (sideload + execução sem pânico, PC em KSEG0).
- A2: `scripts/scoreboard.ps1` roda sem erro e produz `logs/scoreboard.csv` com header + pelo menos 1 linha.
- A3: O job `scoreboard` no `ci.yml` está verde em PRs que não quebram o core (roda com `if: success()` após `check`).

## Repositório

- `main` protegida a partir da iter 0004; 1 PR por item; merge commit (nunca squash);
  commits test→feat→docs; título de PR validado pela CI.
- **Escopo de commit é UM único identificador `[a-z0-9-]`.** `feat(bus,cpu)` reprova no
  `commit-lint`; quando a mudança toca dois módulos, escolha o principal e cite o outro no
  resumo. Custou uma reescrita de 4 mensagens no PR #36 (ver `0022-scratchpad-isc.md`).
- Iterações são cronológicas e nem sempre na ordem dos itens (0003↔item 0.5, p.ex.);
  o vínculo real está no título do PR e no doc da iteração.

## Placar de testes

Workspace: **241** testes (10 meta-testes + 8 bus_bios + 2 bios_flag + 1 version + 12 bus_scheduler + 9 bus_scratchpad_isc + 8 cpu_fetch_decode + 26 cpu_alu + 14 cpu_shifts + 19 cpu_load_delay + 24 cpu_branches + 7 cpu_jumps + 20 cpu_mult_div + 27 cpu_unaligned_load_store + 10 cpu_cop0_regs + 14 cpu_exception_mechanism + 1 cpu_exception_estado_previo + 9 cpu_tty_hook + 11 cpu_printf_hook + 10 cli_runner).

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
