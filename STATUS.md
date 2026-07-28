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

**ROADMAP 1.11** — Sideload de PS-EXE no psx-cli + Amidog psxtest_cpu no scoreboard.

**Spec — layout do header (baixado na iter 0026):**
`docs/reference/16-cdrom-file-formats.md` **L1162-1184**. Header de 800h bytes, código/dados
logo depois. Campos, **exatamente como a spec os lista** (não renomeie para t_addr/b_addr —
esses nomes vêm de outras ferramentas, não do psx-spx):

| Offset | Campo |
|---|---|
| `000h-007h` | ID ASCII `PS-X EXE` |
| `010h` | Initial PC (tipicamente `80010000h`) |
| `014h` | Initial GP/R28 (tipicamente 0) |
| `018h` | **Destination Address in RAM** — para onde o corpo é carregado |
| `01Ch` | Filesize, múltiplo de 800h, **sem** contar o header |
| `020h`/`024h` | Data section addr/size (tipicamente 0) |
| `028h`/`02Ch` | BSS addr/size (`0` = nenhuma) |
| `030h`/`034h` | SP/R29 e FP/R30: base (tipicamente `801FFFF0h`, `0` = não mexer) e offset somado à base |
| `038h-04Bh` | Reservado para A(43h); a BIOS guarda RA,SP,R30,R28,R16 do chamador aqui |
| `800h...` | Código/dados, carregados no endereço de `018h` |

**Spec — funções da BIOS:** `docs/reference/13-kernel-bios.md` — A(41h) LoadTest (L1041),
A(43h) Exec (L1054-1063), A(51h) LoadExec (L1065-1095), Executable Memory Allocation
(L1150-1157). O 1.11 não lê setores de CD-ROM nem implementa LoadExec: é o sideload mínimo
para rodar um EXE de arquivo local.

**Arquivos-alvo:** `crates/psx-cli/src/main.rs` (args: `psx-cli --bios <BIOS> --exe <PS-EXE>`),
`crates/psx-core/src/cpu.rs` (estado do step após halt), `crates/psx-cli/tests/cli_runner.rs`
(teste de integração novo).

### Armadilhas

1. **O header de PS-EXE tem 800h bytes, mas os campos relevantes estão em [10h..4Bh].**
   O headerbuf que Exec recebe tem só 3Ch bytes. O offset no arquivo .psexe é diferente
   do offset no headerbuf — não confundir.
2. **Os endereços do header são VIRTUAIS (KSEG0, `8001xxxx`), não físicos.** O destino em
   `018h` é tipicamente `80010000h`; escrever esse valor direto como índice de RAM estoura.
   Passe pela tradução do Bus. (A versão anterior desta armadilha dizia que o PC inicial era
   KSEG1 `0xBFC0_xxxx` — errado, e removido na revisão do PR #39: `BFC00000h` é o reset
   entrypoint da BIOS ROM, `14-io-map.md` L275. A spec agora local diz `80010000h`,
   `16-cdrom-file-formats.md` L1166.)
2b. **Zerofill do BSS é word-a-word: endereço e tamanho são múltiplos de 4** (L1195). E
   `02Ch` = 0 significa *nenhuma* BSS — não zere nada nesse caso.
3. **BSS zerofill: b_addr pode ser > t_addr+t_size.** Zerar b_size bytes a partir de b_addr
   depois de carregar o código.
4. **Registradores iniciais:** PC (`010h`), GP/R28 (`014h`), SP/R29 e FP/R30 = base
   (`030h`) + offset (`034h`). **Base `0` significa "não mexa no SP/FP"** (L1188), não
   "SP=0". A spec ainda diz que o executável recebe R4 e R5 como parâmetros, "usually R4=1 e
   R5=0" (L1200-1202) — adote esses valores e registre como escolha. Demais registradores = 0
   (a spec não especifica; é o estado limpo que faz sentido no sideload).
5. **Critério de parada do runner — NÃO VERIFICADO.** A proposta é parar quando o PC repete o
   mesmo endereço (`JMP $`), mas nada na spec local diz que o Amidog termina assim; é palpite
   de quem escreveu este handoff. Trate como hipótese: implemente um teto de passos como
   parada primária (sempre correta) e, se for adotar a detecção de auto-loop, confirme com a
   saída real do `psxtest_cpu` e registre no doc da iteração.
6. **`scripts/fetch-test-exes.ps1` precisa ser rodado antes** para baixar `tests/exes/amidog/cpu/psxtest_cpu.psexe`.
   Verifique pré-existência no teste e pule com `#[ignore]` se faltar.

### Testes de aceitação

**A1 — Sideload de EXE mínimo.** Um .psexe sintético (montado no teste via bytes): magic "PS-X EXE",
PC=0x8000_0000, GP=0, SP base/offset em RAM, t_addr=0x8000_0000, t_size=4, código = `JMP $`.
Runner executa e detecta halt em ≤1000 steps.

**A2 — print "OK" via TTY.** EXE sintético com `jal 0xA0` + R9=0x3C + R4='O' + R4='K'`, seguido
de `JMP $`. `take_tty()` devolve `b"OK"`.

**A3 — Zero-fill do BSS.** EXE com b_size=8, b_addr logo após o código. Verifica que os 8 bytes
de RAM em b_addr são zero após o load.

**A4 — `psxtest_cpu` executa e o scoreboard registra o resultado.** `scripts/scoreboard.ps1`
roda com o runner funcional; o EXE do Amidog não precisa passar todos os testes (vai falhar
nos assumidos), mas o placar deve listar o EXE com status `pass` ou `fail` (não `sem-runner`).

**A5 — `--bios` ausente ou BIOS inválida → erro claro.** `psx-cli --exe foo.psexe` sem `--bios`
imprime mensagem de erro e sai com código ≠ 0.

## Repositório

- `main` protegida a partir da iter 0004; 1 PR por item; merge commit (nunca squash);
  commits test→feat→docs; título de PR validado pela CI.
- **Escopo de commit é UM único identificador `[a-z0-9-]`.** `feat(bus,cpu)` reprova no
  `commit-lint`; quando a mudança toca dois módulos, escolha o principal e cite o outro no
  resumo. Custou uma reescrita de 4 mensagens no PR #36 (ver `0022-scratchpad-isc.md`).
- Iterações são cronológicas e nem sempre na ordem dos itens (0003↔item 0.5, p.ex.);
  o vínculo real está no título do PR e no doc da iteração.

## Placar de testes

Workspace: **221** testes (10 meta-testes + 8 bus_bios + 2 bios_flag + 1 version + 12 bus_scheduler + 9 bus_scratchpad_isc + 8 cpu_fetch_decode + 26 cpu_alu + 14 cpu_shifts + 19 cpu_load_delay + 24 cpu_branches + 7 cpu_jumps + 20 cpu_mult_div + 27 cpu_unaligned_load_store + 10 cpu_cop0_regs + 14 cpu_exception_mechanism + 1 cpu_exception_estado_previo + 9 cpu_tty_hook).

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
