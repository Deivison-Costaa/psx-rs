# STATUS

> Memória do projeto entre iterações. O contexto do agente é descartado a cada iteração;
> este arquivo não. Teto de 16 KB imposto por `status_size.rs`. Curto e verdadeiro.

## Última iteração concluída

**0033** — Opcode não implementado gera exceção RI/CpU em vez de panic (ROADMAP 1.14). 258 testes (11 novos + 3 renomeados). Bateria 7/7, 2/2. Nenhum `unimplemented!`/`panic!`/`unwrap()`/`expect()` no source.
Ver `docs/iterations/0033-cpu-opcode-reservado.md`.

## Próxima tarefa
**ROADMAP 2.1** — GPUSTAT + decodificação GP0/GP1. Primeiro item do M2.

O handoff anterior deste item (escrito na 0031) tinha quatro afirmações erradas, corrigidas e
registradas em `docs/iterations/0032-handoff-2-1.md`. Este aqui é o handoff de verdade, com as
linhas conferidas uma a uma com `grep -n` em 28/07.

**Spec:** `docs/reference/03-gpu.md` — linhas **absolutas do arquivo**:

| Fato | Linha |
|---|---|
| Portas: `1F801810h`-Write = GP0, `1F801814h`-Write = GP1, `1F801810h`-Read = GPUREAD, `1F801814h`-Read = GPUSTAT | L144-147 |
| Tabela de bits do GPUSTAT, bit a bit | L1002-1032 |
| **GP1(00h) Reset GPU — "Accordingly, GPUSTAT becomes 14802000h"** | L747-763 |
| GP1(01h) Reset Command Buffer | L767 |
| GP1(02h) Acknowledge IRQ1 (GPUSTAT.24) | L773 |
| GP1(03h) Display Enable (GPUSTAT.23) | L779 |
| GP1(04h) DMA Direction (GPUSTAT.29-30, e define o *significado* do bit 25) | L789 |
| GP1(08h) Display mode — mapa bit a bit para GPUSTAT.16-22 e .14 | L885-893 |
| GP0(E1h) Draw Mode / Texpage (GPUSTAT.0-10 e .15) | L492 |
| GP0(E6h) Mask Bit Setting (GPUSTAT.11-12) | L578 |
| GP0(00h) NOP e seus mirrors | L721, L734 |

**Arquivos-alvo:** `crates/psx-core/src/gpu.rs` (hoje tem 1 byte — módulo vazio),
`crates/psx-core/src/bus.rs` (mapear `1F801810h..1F801817h`),
`crates/psx-core/tests/gpu_status_gp0_gp1.rs` (arquivo novo).

**Escopo (R4):** GPUSTAT como espelho do estado que GP1(03h/04h/08h) e GP0(E1h/E6h) escrevem,
mais os bits de "pronto" fixos. **Fora do escopo:** rasterização, VRAM, transferências,
temporização, IRQ de verdade. Comando de renderização pode ser aceito e descartado — mas veja
a armadilha 5.

### Armadilhas

1. **Não monte o valor de reset bit a bit: a spec dá o número pronto.** Depois de GP1(00h),
   `GPUSTAT = 14802000h` (L763). Use-o como golden value do teste. Decompondo, são os bits
   28 (ready DMA), 26 (ready cmd), 23 (display disabled) e 13 (interlace field) — se a sua
   montagem der outro número, é a montagem que está errada, não a spec.
2. **Não existe campo de "versão da GPU" no GPUSTAT.** O handoff antigo afirmava que os bits
   19-20 eram revisão do hardware: são **Vertical Resolution** e **Video Mode** (L1021-1022).
   E o bit 28 é **Ready to receive DMA Block**, não odd/even — odd/even é o bit **31** (L1033).
3. **Bit 25 não é um bit de estado: o significado dele muda conforme GP1(04h)** (L1027-1031).
   Com DMA direction 0 é sempre zero; 1 = FIFO não cheio; 2 = igual ao bit 28; 3 = igual ao 27.
   Implementar como espelho condicional, não como flag guardada.
4. **O bus tem um catch-all** que devolve `0` para `1F801024h..1F801FFFh` desde a iteração 0022,
   e há testes que dependem dele (`bus_scratchpad_isc`). Ao abrir a janela da GPU, não quebre o
   catch-all nem os testes existentes — rode `cargo test --all` e compare o total antes/depois.
5. **GP0 tem comandos multi-palavra.** Se você aceitar comandos de renderização e descartá-los,
   precisa consumir o número certo de palavras de parâmetro, senão a próxima palavra de dados
   vira "comando" e o estado do GPUSTAT começa a mudar sozinho. Se não for implementar a
   contagem, restrinja-se a GP0(00h/01h/E1h..E6h) e **escreva no doc quais comandos ficaram de
   fora** — engolir comando em silêncio é o defeito que a 0029 registrou com `%o`.
6. **Escrita em `1F801814h` é GP1; leitura do mesmo endereço é GPUSTAT.** São coisas
   diferentes no mesmo endereço (L144-147). O mesmo vale para `1F801810h`: escrita é GP0,
   leitura é GPUREAD (não implementado — devolver `0` é aceitável, mas registre como stub).
7. **Não invente critério de pass/fail no scoreboard.** Isso é o item 1.13, que depende deste.

### Testes de aceitação

**A1 — valor de reset.** Após GP1(00h) via `sw` em `1F801814h`, um `lw` em `1F801814h`
devolve exatamente `0x1480_2000` (golden value, L763).

**A2 — GP1(08h) mapeia para GPUSTAT.** Escrever display mode com bits 0-7 variados e conferir
GPUSTAT.16-22 e .14 conforme a tabela de L885-893. Pelo menos dois valores distintos.

**A3 — GP0(E1h) e GP0(E6h).** Draw mode escreve GPUSTAT.0-10 e .15 (L492); mask bit escreve
GPUSTAT.11-12 (L578).

**A4 — GP1(03h) alterna o bit 23** e GP1(04h) escreve os bits 29-30, com o bit 25 seguindo a
regra da armadilha 3 nos quatro modos.

**A5 — o catch-all sobreviveu.** `cargo test --all` verde, e o total sobe apenas pelos testes
novos: 258 + N. Se algum teste antigo mudou de resultado, o item quebrou algo.

**A6 — o EXE real, SEM andaime.** Este é o teste que fecha o item:
`psx-cli --bios bios/SCPH1001.BIN --exe tests/exes/ps1-tests/cpu/cop/cop.exe` tem que imprimir

```
pass - testCop0Disabled
pass - testCop0Enabled
```

**sem** o stub temporário de GPUSTAT no `bus.rs` — hoje isso só sai com o andaime aplicado à
mão (medido na revisão da 0033). Depois deste item, sai porque a GPU existe. **Cole a saída no
doc.** E rode `./scripts/scoreboard.ps1`, colando a distribuição de status: a expectativa é
que várias suítes saiam do banner e produzam saída de verdade — meça, não estime.

## Repositório

- `main` protegida a partir da iter 0004; 1 PR por item; merge commit (nunca squash);
  commits test→feat→docs; título de PR validado pela CI.
- **Escopo de commit é UM único identificador `[a-z0-9-]`.** `feat(bus,cpu)` reprova no
  `commit-lint`; quando a mudança toca dois módulos, escolha o principal e cite o outro no
  resumo. Custou uma reescrita de 4 mensagens no PR #36 (ver `0022-scratchpad-isc.md`).
- Iterações são cronológicas e nem sempre na ordem dos itens (0003↔item 0.5, p.ex.);
  o vínculo real está no título do PR e no doc da iteração.

## Placar de testes

Workspace: **258** testes (10 meta-testes + 8 bus_bios + 2 bios_flag + 1 version + 12 bus_scheduler + 9 bus_scratchpad_isc + 8 cpu_fetch_decode + 26 cpu_alu + 14 cpu_shifts + 19 cpu_load_delay + 24 cpu_branches + 7 cpu_jumps + 20 cpu_mult_div + 27 cpu_unaligned_load_store + 10 cpu_cop0_regs + 14 cpu_exception_mechanism + 1 cpu_exception_estado_previo + 9 cpu_tty_hook + 11 cpu_printf_hook + 11 cpu_opcode_reservado + 6 ci_scoreboard + 9 cli_runner).

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
