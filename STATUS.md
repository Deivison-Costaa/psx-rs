# STATUS

> Memória do projeto entre iterações. O contexto do agente é descartado a cada iteração;
> este arquivo não. Teto de 16 KB imposto por `status_size.rs`. Curto e verdadeiro.

## Última iteração concluída

**0027** — Sideload de PS-EXE no psx-cli + Amidog psxtest_cpu no scoreboard (ROADMAP 1.11).
Terceira rodada de correção após duas revisões adversariais. A4 agora executa de verdade
(dois `..` em `exe_dir()`), afirma PC em KSEG0 sem mentir sobre TTY (printf A(3Fh) pendente).
Scoreboard funcional (quoting de caminhos com espaço corrigido), `amidog/cpu` = `fail`.
ROADMAP 1.11b adicionado para printf A(3Fh). Bateria de mutação refeita do zero (7/7, 3/3).
230 testes no workspace. Ver `docs/iterations/0027-sideload-psexe.md`.

**0028** — Passo zero do 1.11b (sem código): medido, com instrumentação descartável, que o
`psxtest_cpu` chama `printf` A(3Fh) uma única vez (`"args: %d\n"`) e depois trava esperando
GPUSTAT.26, que é o item 2.1. Handoff do 1.11b reescrito com as linhas de spec citadas e com
o critério de sucesso corrigido. Ver `docs/iterations/0028-spec-printf.md`.

**0023** — Iteração de processo (sem código): incorporação das métricas pendentes da 0022 e
registro do erro de escopo múltiplo em commit reprovado pelo `commit-lint`.

**0024** — Iteração de processo (sem código): o handoff do 1.10 descrevia A0h/B0h como
códigos de `syscall`; a spec diz que são endereços de chamada (`jal 0xA0`) com o número da
função em R9. Corrigido contra `13-kernel-bios.md` antes de despachar o trabalhador. Ver
`docs/iterations/0024-handoff-1.10-corrigido.md`.

**0025** — Hook de TTY (A0h/B0h) (ROADMAP 1.10): hook no `Cpu::step` que detecta PC físico
`A0h`/`B0h`, lê R9 e emite putchar/puts para buffer no Bus (`tty_push`/`take_tty`). 7
testes (D1-D6 + D2b), 5/5 mutantes pegos, 2/2 controles verdes. O handoff corrigido pela
iter 0024 foi essencial: A0h/B0h como endereços de `jal`, não como códigos de syscall.
putchar grava byte cru (sem expansão TAB/LF) — decisão deliberada. Ver
`docs/iterations/0025-cpu-tty-hook.md`.

**0026** — Iteração de processo (sem código): baixado o capítulo `16-cdrom-file-formats.md`,
que traz o layout do header PS-EXE e faltava no repositório; handoff do 1.11 reescrito com os
offsets citados linha a linha. Ver `docs/iterations/0026-spec-formato-psexe.md`.

## Próxima tarefa

**ROADMAP 1.11b** — Hook de `printf` A(3Fh) com expansão de `%` → Amidog imprimindo no TTY.

O 1.11 fechou com o `psxtest_cpu` rodando e registrado como `fail` no placar: TTY de zero
bytes. A causa foi medida, não suposta — ver `docs/iterations/0028-spec-printf.md`.

**Spec:** `docs/reference/13-kernel-bios.md` **L2703-2740** — A(3Fh) Printf.

| Fato | Linha |
|---|---|
| `in: A0` = ponteiro para string terminada em 0; argumentos em `A1,A2,A3,[SP+10h..]` | L2705-2706 |
| Usa `putchar` internamente e expande os chars `09h` e `0Ah` | L2709-2710 |
| Códigos de escape (`c s i d D u U o O p x X n`) | L2712-2719 |
| Prefixos entre `%` e o código (`+`, espaço, `NNN`, `.NNN`, `*`, `-`, `#`, `0`, `L`, `h`, `l`) | L2721-2733 |
| Só octal/decimal/hex; **não** tem binário | L2739 |
| `puts` A(3Eh)/B(3Fh) é função DIFERENTE (não resolve `%`) | L2742-2746 |
| `putchar` A(3Ch)/B(3Dh) | L2776 |

**Arquivos-alvo:** `crates/psx-core/src/cpu.rs` (o `match (fn_idx, phys)` do hook do 1.10 —
hoje cobre `(0x3C,0xA0)|(0x3D,0xB0)` putchar e `(0x3E,0xA0)|(0x3F,0xB0)` puts; falta
`(0x3F,0xA0)` printf), `crates/psx-core/tests/cpu_printf_hook.rs` (arquivo novo).

### O que o Amidog realmente chama — medido em 28/07

Instrumentando 50M passos do `psxtest_cpu.exe`, ele chama `printf` **exatamente uma vez**:

```
printf #1 fmt="args: %d\n" a1=00000000 a2=00000000 a3=00000000
```

Ou seja: para o placar sair de `fail`, basta `%d` — mas implemente o conjunto abaixo, que é
uma feature coerente, não só o caso do teste.

**Escopo desta iteração (R4):** `%%`, `%c`, `%s`, `%d`/`%i`, `%u`, `%x`/`%X`, e os argumentos
vindos de A1/A2/A3 e depois da pilha em `[SP+10h..]`. **Fora do escopo:** `%o`, `%n`, `%p`,
larguras, `.precisão`, `*`, e os prefixos de sinal/padding — para um especificador não
suportado, emita a sequência literal (`%o` sai como `%o`) e **liste no doc da iteração o que
ficou de fora**. Não silencie: especificador engolido em silêncio vira bug invisível depois.

### Armadilhas

1. **`printf` é A(3Fh); `puts` é A(3Eh) — e B(3Fh) também é `puts`.** O `match` do hook casa
   `(fn_idx, phys)`: `(0x3F, 0xA0)` é printf, `(0x3F, 0xB0)` é puts. Errar isso foi o achado
   F1 da iteração 0025; o `match` atual já está certo, só falta o braço novo.
2. **Ler registrador com `reg_with_pending`, nunca `self.regs[n]` direto.** O hook roda antes
   do commit do load delay slot; ler cru dá o valor velho quando o argumento vem de um `lw`
   no delay slot do `jal` (achado F2 da 0025, já com teste).
3. **Argumento nº 4 em diante vem da pilha do chamador em `[SP+10h..]`** (L2706), não de um
   registrador. Se não for implementar, não finja que foi: hoje o Amidog só usa A1.
4. **A varredura da string tem que ter teto**, como o `puts` do 1.10 (1 MiB). Ponteiro
   inválido não pode virar laço infinito.
5. **`%d` é decimal com sinal de 32 bits; `%u` é sem sinal.** `-1` sai `-1` em `%d` e
   `4294967295` em `%u`. Teste os dois com o mesmo valor de entrada.
6. **Depois do printf, o Amidog trava esperando a GPU — e isso NÃO é bug desta iteração.**
   Medido: ele entra num laço em `80014DF0` (`lw r2,[1F801814h]` / `and r2,r2,0400_0000h` /
   `beq r2,r0,-4`), isto é, espera **GPUSTAT.26 "Ready to receive Cmd Word"**
   (`03-gpu.md` L1028; porta em L147). O bus devolve `0` para todo o range `1F801024h..1F801FFFh`
   (catch-all da iter 0022), então o bit nunca acende. Isso é o item **2.1** do ROADMAP.
   O sucesso desta iteração é o TTY conter `args: 0\n` — **não** o `psxtest_cpu` completar.

### Testes de aceitação

**A1 — `%d`.** `jal 0xA0` com R9=3Fh, R4 → `"n=%d\n"`, R5 = 42. `take_tty()` = `b"n=42\n"`.

**A2 — sinal.** Mesma string com R5 = `0xFFFF_FFFF`: `%d` dá `n=-1`, e com `"%u"` dá
`n=4294967295`.

**A3 — `%s` e `%c` e `%%`.** `"%s=%c 100%%\n"` com R5 apontando para `"ok"` e R6 = `b'X'` →
`b"ok=X 100%\n"`.

**A4 — hex.** `"%x %X\n"` com R5 = `0xDEAD_BEEF` → `b"deadbeef DEADBEEF\n"`.

**A5 — especificador fora do escopo sai literal.** `"%o\n"` → `b"%o\n"` (e a limitação
registrada no doc).

**A6 — o EXE real.** `cargo test -p psx-cli --test cli_runner -- --nocapture` e o
`psxtest_cpu` passa a imprimir `args: 0`. Ajuste o teste do 1.11
(`psxtest_cpu_sideload_executa_sem_panico`) para afirmar isso, e rode
`./scripts/scoreboard.ps1` — `amidog/cpu` tem que virar `pass`. **Cole a saída dos dois
comandos no doc da iteração**: nesta casa, afirmação sem comando que a prove não vale (foi o
que custou três rodadas no 1.11).


## Repositório

- `main` protegida a partir da iter 0004; 1 PR por item; merge commit (nunca squash);
  commits test→feat→docs; título de PR validado pela CI.
- **Escopo de commit é UM único identificador `[a-z0-9-]`.** `feat(bus,cpu)` reprova no
  `commit-lint`; quando a mudança toca dois módulos, escolha o principal e cite o outro no
  resumo. Custou uma reescrita de 4 mensagens no PR #36 (ver `0022-scratchpad-isc.md`).
- Iterações são cronológicas e nem sempre na ordem dos itens (0003↔item 0.5, p.ex.);
  o vínculo real está no título do PR e no doc da iteração.

## Placar de testes

Workspace: **230** testes (10 meta-testes + 8 bus_bios + 2 bios_flag + 1 version + 12 bus_scheduler + 9 bus_scratchpad_isc + 8 cpu_fetch_decode + 26 cpu_alu + 14 cpu_shifts + 19 cpu_load_delay + 24 cpu_branches + 7 cpu_jumps + 20 cpu_mult_div + 27 cpu_unaligned_load_store + 10 cpu_cop0_regs + 14 cpu_exception_mechanism + 1 cpu_exception_estado_previo + 9 cpu_tty_hook + 10 cli_runner).

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
