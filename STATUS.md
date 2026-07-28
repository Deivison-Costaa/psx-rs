# STATUS

> Memória do projeto entre iterações. O contexto do agente é descartado a cada iteração;
> este arquivo não. Teto de 16 KB imposto por `status_size.rs`. Curto e verdadeiro.

## Última iteração concluída

**0016** — MULT/MULTU/DIV/DIVU + HI/LO (ROADMAP 1.6): MULT (SPECIAL 0x18), MULTU (0x19),
DIV (0x1A), DIVU (0x1B), MFHI (0x10), MTHI (0x11), MFLO (0x12), MTLO (0x13) + campos
`hi`/`lo` na struct `Cpu`. 20 testes em `cpu_mult_div.rs`.
Bateria de mutação: 7/7 pegos, 2/2 controles verdes.
Erro de primeira tentativa: (1) `u32 as i64` zero-extende em vez de sign-extender para
MULT (corrigido com `as i32 as i64`); (2) testes de MFHI/MFLO usavam `rs` em vez de `rd`
no encode; (3) expectativa de `mult_64bits_hi_lo` calculada errada.
A revisão adversarial achou a CI vermelha por lint que o clippy local (desatualizado) não
conhece e um buraco de cobertura — DIVU implementado com sinal passava nos 18 testes; +2
testes. Ver `docs/iterations/0016-cpu-mult-div.md`.

**0017 — REPROVADA na revisão adversarial, PR #27 fechada sem merge** (LWL/LWR/SWL/SWR,
ROADMAP 1.7). Primeira iteração rejeitada do projeto. Os 20 testes passavam porque
codificavam o mesmo modelo errado da implementação. Custo registrado no CSV como
`rejeitado:semantica`. Diagnóstico completo e o que a segunda tentativa tem de fazer
diferente: `docs/iterations/0017-cpu-unaligned-load-store.md`.

**0017d — trabalhador migra para `deepseek/deepseek-v4-pro`.** Da 0009 à 0017 o trabalhador
rodou em `deepseek-chat` (geração anterior) só porque era o default do `oc-iter.ps1`, nunca
por decisão. A primeira tentativa da 0018 (1.7 de novo) foi **abortada aos 18min36 no meio do
passo 5** para trocar de modelo; branch preservada como `abandonada/0018-lwl-chat-v3`, linha
`abortado:troca-de-modelo` nas métricas. Eixo de comparação agora é v4-pro × v4-flash.
Ver `docs/iterations/0017d-modelo-v4-pro.md`.

## Próxima tarefa

**ROADMAP 1.7 (SEGUNDA TENTATIVA)** — LWL/LWR/SWL/SWR. A PR #27 foi **reprovada na revisão
adversarial**: implementação e testes compartilhavam o mesmo modelo errado, então os 20
testes passavam e a bateria de mutação não pegava nada. Leia
`docs/iterations/0017-cpu-unaligned-load-store.md` **antes de começar** — ele descreve os
dois defeitos. Comece do zero a partir da main; não recupere a branch antiga.

Spec: `docs/reference/02-cpu.md`, seções **Unaligned Load/Store** e **Unaligned Load/Store
(Details)** (índice: L235, L257). Leia as duas.

Armadilha 1 — **`[N*4+0]` na tabela da spec é ENDEREÇO DE BYTE, não a parte alta do valor
da palavra.** Em little-endian o byte em `N*4+0` é o byte **menos** significativo da palavra
lida em `N*4`: depois de `write32(0x1000, 0xAABBCCDD)`, o byte em `[0x1000]` vale `0xDD`.
Então `LWL` com endereço `0x1000` ("transfer upper 8bit of Rt from [N*4+0]") põe **`0xDD`**
em `rt[31:24]` — quem responder `0xAA` reproduziu o erro da PR #27. Derive as quatro
posições de cada opcode dessa regra; o merge precisa **deslocar** a via de byte, não só
mascarar. O que não é transferido fica intacto (nem zero- nem sign-extend), no registrador
e na memória.

Armadilha 2 — **LWL e LWR têm de enxergar um ao outro sem delay.** A spec mostra o idioma
logo acima da tabela (`lwl r2,$0003(t0)` seguido de `lwr r2,$0000(t0)`, "no delay required
between these ... although both access r2"). Hoje o merge lê `self.reg(rt)`, que ainda é o
valor antigo quando o load anterior está no delay slot — o resultado do `lwl` some. O merge
tem de usar o valor do load pendente quando ele for para o mesmo registrador.

**Teste de aceitação obrigatório** (o item não fecha sem ele): `[0..3] = DD CC BB AA`,
`[4..7] = 44 33 22 11`, `t0 = 1`; `lwl r2,3(t0)` seguido de `lwr r2,0(t0)` e um `nop` tem de
deixar `r2 = 0x44DDCCBB` — a palavra desalinhada formada pelos bytes `[1][2][3][4]`. Esse é
o idioma da spec e o motivo de os quatro opcodes existirem.

Teste: `crates/psx-core/tests/cpu_unaligned_load_store.rs`. **Use** `tests/support/asm.rs`
(`bus_with_bios_empty`, `encode_i_type`, `nop`) em vez de recriar helpers, e nomeie os
testes em português, como em `cpu_mult_div.rs`.

`cpu_mult_div.rs` tem 283 linhas — dentro do teto de 500. `cpu.rs` passou de 500 linhas e
**continua inteiro**: o orquestrador respondeu a dúvida levantada na 0016 — o teto vale só
para teste, e fatiar por contagem seria pior que um arquivo coeso. O corte virá quando a
coesão pedir (candidato natural: COP0/exceções em módulo próprio, no 1.8).

**Toolchain é pinado** em `rust-toolchain.toml` (1.97.1) desde a 0017c: o rustup resolve a
versão sozinho, local e CI rodam o mesmo compilador, e o clippy que você vê é o que a CI vê.
Não rode `rustup update` esperando efeito aqui — subir de versão é iteração própria.

## Repositório

- `main` protegida a partir da iter 0004; 1 PR por item; merge commit (nunca squash);
  commits test→feat→docs; título de PR validado pela CI.
- Iterações são cronológicas e nem sempre na ordem dos itens (0003↔item 0.5, p.ex.);
  o vínculo real está no título do PR e no doc da iteração.

## Placar de testes

Workspace: **151** testes (10 meta-testes + 8 bus_bios + 2 bios_flag + 1 version + 12 bus_scheduler + 8 cpu_fetch_decode + 26 cpu_alu + 14 cpu_shifts + 19 cpu_load_delay + 24 cpu_branches + 7 cpu_jumps + 20 cpu_mult_div).

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
2. **Dívida de overflow trap (fecha no item 1.8).** `ADDI` está implementado sem trap,
   idêntico a `ADDIU` — a spec (02-cpu.md, `arithmetic instructions`) manda excetuar e
   deixar `rt` intacto no overflow. `ADD` (secondary 0x20) e `SUB` (0x22) nem existem no
   match: dão `unimplemented!`. Quando o 1.8 trouxer o mecanismo de exceção, os três
   entram juntos. Autorizado no handoff da 0012, mas é dívida, não comportamento correto.
3. **Load delay × escrita no mesmo registrador: comportamento ASSUMIDO, não verificado
   (resolve no item 1.11).** Quando a instrução do delay slot escreve o registrador
   destino do load (`lw r10,..` seguido de `ori r10,..`), a nossa implementação faz o
   **load vencer**. A spec local não decide: `02-cpu.md § Caution - Load Delay` só diz que
   o registrador "não é atualizado até o próximo opcode ter completado", o que fala de
   leitura, não de precedência de escrita. Não mudamos sem evidência (R1). O teste
   `load_delay_vs_escrita_no_mesmo_registrador_comportamento_assumido` fixa o que fazemos
   hoje e nomeia a dúvida, para que uma futura mudança seja deliberada. Ponto de
   resolução: Amidog `psxtest_cpu` no item 1.11 — se ele reprovar, inverter a ordem em
   `Cpu::step` (commitar o load antes de executar, escrevendo num banco de saída).
4. **BcondZ com `rt` fora da tabela: comportamento ASSUMIDO (resolve no item 1.11).** O
   opcode 01h só tem `rt`=00h/01h/10h/11h tabelados em `02-cpu.md § Opcode/Parameter
   Encoding`; a spec local não diz o que `rt`=02h..0Fh/12h..1Fh fazem. Assumimos **no-op
   silencioso** (nem desvia nem linka). O teste
   `bcondz_rt_fora_da_tabela_comportamento_assumido` fixa isso e diz na asserção que é
   suposição. Se o Amidog `psxtest_cpu` reprovar, o critério a testar primeiro é o de
   hardware conhecido: bit16 sozinho decide BLTZ/BGEZ e o link ocorre quando os bits
   20..17 valem 1000b — o que faria `rt`=02h agir como BLTZ.
5. **Dívida do bit BD / delay slot para o item 1.8.** `Cpu` sinaliza desvio pendente com
   `branch_target: Option<u32>`, consumido em `step` ANTES de executar a instrução. Isso
   basta para o desvio, mas apaga a informação "a instrução atual está num delay slot",
   que o 1.8 precisa para setar `CAUSE.BD` e apontar `EPC` para o branch (e não para o
   delay slot) — a própria spec cita o caso em `§ JALR cautions`. Quem fizer o 1.8 tem de
   guardar esse flag junto com o endereço do branch, não deduzi-lo depois.
