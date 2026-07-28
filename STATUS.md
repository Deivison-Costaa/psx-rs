# STATUS

> Memória do projeto entre iterações. O contexto do agente é descartado a cada iteração;
> este arquivo não. Teto de 16 KB imposto por `status_size.rs`. Curto e verdadeiro.

## Última iteração concluída

**0021** — Mecanismo de exceção (ROADMAP 1.8b): overflow em `ADD`/`ADDI` com `rt` intacto,
`syscall` (08h), `break` (09h) no vetor próprio `0x8000_0040`, AdEL/AdES com `BadVaddr`,
bits BD/BT. ExcCode escrito direto em `self.cop0[13]`, sem passar pela máscara da 1.8a.
**Duas rodadas de revisão adversarial**: a primeira achou três lacunas de comportamento
(CAUSE apagando os bits Sw; nenhum empilhamento do SR na entrada, com o `rfe` da 1.8a
desempilhando o nada; load pendente descartado pela exceção); a segunda achou dois mutantes
que escapavam da suíte inteira. 15 testes, 12/12 mutantes pegos, 3/3 controles verdes.
Ver `docs/iterations/0021-cpu-exception-mechanism.md`.

## Próxima tarefa

**ROADMAP 1.9** — Scratchpad (1KB), isolamento de cache (`SR.Isc`) e registradores de memory
control. Primeiro item do M1 que mexe no **bus**, não na CPU.

O handoff do trabalhador propunha um 1.8c (Reserved Instruction + Coprocessor Unusable) aqui.
Decisão do orquestrador: **vai para depois do 1.11**, com a prioridade escolhida pelo que o
Amidog `psxtest_cpu` reprovar. Motivo na armadilha 1 abaixo — o 1.9 conserta corrupção de RAM
em curso, e está no caminho crítico do 1.10 (ver o BIOS imprimir). RI/CpU não bloqueiam nada
e são exatamente o tipo de dívida que o scoreboard do 1.11 mede melhor do que eu estimo.
A derivação do 1.8c está preservada nas notas.

**Escopo:** decodificação de região no `Bus` para (a) Scratchpad 1KB em `1F800000h..1F8003FFh`,
(b) memory control em `1F801000h..1F801023h` e `1F801060h` (RAM_SIZE), (c) o BCC em
`FFFE0130h` (KSEG2), e (d) `SR.Isc` (bit 16) fazendo os stores da CPU não chegarem à memória.

**NÃO inclui:** i-cache de verdade (linhas, tags, fill) — o `Isc` aqui só engole store;
Bus Error para regiões não usadas; espelhos de RAM configuráveis por `RAM_SIZE`; write queue;
scratchpad executável. Cada um vira nota de dívida.

**Spec:** `docs/reference/01-memory-map.md` — `Memory Map` (L2), `KUSEG,KSEG0,KSEG1,KSEG2`
(L26), `Scratchpad` (L91), `Memory Mirrors` (L119), `Memory Exceptions` (L133).
`docs/reference/12-memory-control.md` — portas (L6–L36), `RAM_SIZE` (L152), `BCC` (L203).
`docs/reference/02-cpu.md` — `cop0r12 - SR` (L624), bit 16 `Isc`.

### Armadilhas nomeadas

1. **Isto não é "falta implementar", é corrupção ativa em curso.** `Bus::ram_offset` faz
   `phys & 0x1F_FFFF` para **todo** endereço que não seja BIOS. Consequência aritmética:
   `0x1F80_0000 & 0x1F_FFFF = 0x00_0000` — o scratchpad é hoje um **alias do endereço 0 da
   RAM**; `0x1F80_1060 & 0x1F_FFFF = 0x00_1060` — cada escrita do BIOS em memory control
   sobrescreve RAM real; `0xFFFE_0130 & 0x1F_FFFF = 0x1E_0130` — o BCC sobrescreve RAM em
   1.9 MB. As três faixas ficam dentro dos 64 KB de RAM reservados ao kernel ou em uso.
   Comece medindo isso com teste que falha, não implementando.
2. **A decodificação de região vem ANTES do fallback de RAM, e vale para os seis acessos.**
   Repare que `read32` já testa a faixa da BIOS mas `write32` **não** — escrever na BIOS hoje
   cai na RAM. Não conserte isso aqui (é dívida separada), mas não repita o padrão: a nova
   decodificação vale para `read8/16/32` e `write8/16/32`.
3. **KSEG2 não passa pela máscara de região, e está certo assim.** `to_physical` devolve `addr`
   intacto no braço `_`, que é o comportamento correto para `0xFFFE_0130`. O defeito está no
   `ram_offset`, que mascara depois. Não mexa em `to_physical` para "consertar" o KSEG2.
4. **O comentário do braço `0b010` em `to_physical` está errado.** Diz `KUSEG:
   0x0000_0000..0x1FFF_FFFF`, mas `addr >> 29 == 0b010` é `0x4000_0000..0x5FFF_FFFF`; o KUSEG
   baixo cai no braço `_` e funciona por coincidência (`addr & 0x1FFF_FFFF == addr` ali).
   Corrija o comentário em uma linha, respeitando R7.
5. **Scratchpad NÃO é espelhado em KSEG1.** A spec é explícita: "The Scratchpad is mirrored
   only in KUSEG and KSEG0, but not in KSEG1", e a tabela do memory map traz `--` na coluna
   KSEG1 da linha Scratchpad. `0xBF80_0000` **não** é scratchpad. O correto seria Bus Error
   (`Memory Exceptions`: "Bus Error -> Unused Memory Regions"), que não existe ainda — devolva
   0 / ignore a escrita e marque como **comportamento ASSUMIDO** com resolução no 1.11.
6. **O `Bus` não pode conhecer o COP0.** `Isc` mora no `SR`, que é da CPU. Decida onde a
   checagem entra (CPU antes de chamar o bus, ou flag passado ao bus) e **justifique no doc** —
   mas um `Bus` que precise de `&Cpu` reprova a revisão.

### Testes de aceitação OBRIGATÓRIOS

Literais derivados por duas rotas (regra da 0017e). Regra nova, da 1.8b: **todo item que
escreve em região/registrador persistente precisa de um caso com o vizinho sujo antes** — sem
isso um alias passa despercebido exatamente como passou na 1.8b.

**D1 — Scratchpad é memória própria, não alias da RAM.** `write32(0x0000_0000, 0xAAAA_AAAA)`,
depois `write32(0x1F80_0000, 0x5555_5555)`. Exigido: `read32(0x0000_0000) == 0xAAAA_AAAA` **e**
`read32(0x1F80_0000) == 0x5555_5555`. Rota 1: o mapa lista Scratchpad como região própria de
1K, separada da Main RAM. Rota 2: `0x1F80_0000 & 0x1F_FFFF = 0`, que prova o alias hoje. O
primeiro assert é o que importa — sem ele, um scratchpad ainda aliasado passa.

**D2 — Espelho em KSEG0 sim, em KSEG1 não.** `write32(0x1F80_0010, 0xC0DE_C0DE)`. Exigido:
`read32(0x9F80_0010) == 0xC0DE_C0DE` e `read32(0xBF80_0010) == 0` (ASSUMIDO; a mensagem do
assert diz que é suposição, no estilo do
`load_pendente_e_commitado_antes_da_excecao_comportamento_assumido`).

**D3 — Limite superior.** `write32(0x1F80_03FC, 0x1234_5678)` é o último word válido e tem de
ler de volta. 1KB = `0x400` bytes → último word alinhado em `0x400 - 4 = 0x3FC`; a spec dá a
faixa fechada `1F800000h..1F8003FFh`.

**D4 — `Isc=1` engole o store.** Sem isolamento, `sw` de `0xDEAD_BEEF` em `0x0000_0200`. Depois
`mtc0` com `SR = 0x0001_0000`, `sw` de `0x0000_0000` no **mesmo** endereço, `mtc0` com `SR = 0`,
e `lw`. Exigido: **`0xDEAD_BEEF`**. Rota 1: "When isolated, all load and store operations are
targetted to the cache instead of main memory". Rota 2: o kernel usa `Isc` com `FFFE0130h` para
invalidar a i-cache no boot; se esses stores fossem para a RAM, o BIOS escreveria lixo por cima
do próprio código antes de chegar ao shell.

**D5 — Memory control não corrompe a RAM.** `write32(0x1F80_1000, 0x1F00_0000)` e
`write32(0x1F80_1060, 0x0000_0B88)`. Exigido: `read32(0x0000_1000) == 0` e
`read32(0x0000_1060) == 0` — a RAM **não** foi tocada — e `read32(0x1F80_1060) == 0x0000_0B88`.
Os dois asserts sobre a RAM são o coração; sem eles, guardar o valor num campo novo passa mesmo
continuando a corromper.

**D6 — BCC em KSEG2.** `write32(0x001E_0130, 0x1111_1111)` como testemunha, depois
`write32(0xFFFE_0130, 0x0001_E988)`. Exigido: `read32(0xFFFE_0130) == 0x0001_E988` **e**
`read32(0x001E_0130) == 0x1111_1111`. Rota 1: memory map, "FFFE0130h (in KSEG2) 0.5K Internal
CPU control registers (Cache Control)". Rota 2 — derivação bit a bit da coluna "usually" da
tabela BCC: RAM(3), DS(7), IBLKSZ(8), IS1(11), RDPRI(13), NOPAD(14), BGNT(15), LDSCH(16) →
`0b1_1110_1001_1000_1000` = `0x1E988`, que é o valor que o BIOS de fato escreve. As duas rotas
concordam.

## Repositório

- `main` protegida a partir da iter 0004; 1 PR por item; merge commit (nunca squash);
  commits test→feat→docs; título de PR validado pela CI.
- Iterações são cronológicas e nem sempre na ordem dos itens (0003↔item 0.5, p.ex.);
  o vínculo real está no título do PR e no doc da iteração.

## Placar de testes

Workspace: **203** testes (10 meta-testes + 8 bus_bios + 2 bios_flag + 1 version + 12 bus_scheduler + 8 cpu_fetch_decode + 26 cpu_alu + 14 cpu_shifts + 19 cpu_load_delay + 24 cpu_branches + 7 cpu_jumps + 20 cpu_mult_div + 27 cpu_unaligned_load_store + 10 cpu_cop0_regs + 14 cpu_exception_mechanism + 1 cpu_exception_estado_previo).

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
