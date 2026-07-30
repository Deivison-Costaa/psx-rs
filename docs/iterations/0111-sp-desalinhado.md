# 0111 — sp-desalinhado

- **Data:** 2026-07-30
- **Item do roadmap:** 4.4h
- **Objetivo:** achar e corrigir o segundo mecanismo que desalinha `$sp` (o crash do `$ra=4` no
  passo 85 544 264). **Resultado: era o load delay slot — a escrita da instrução seguinte tem de
  cancelar o load pendente para o mesmo registrador.** Um fix de 4 linhas destrava o boot inteiro:
  o logo completo aparece (losango com quatro pontas, "SONY" e "COMPUTER ENTERTAINMENT" em
  azul-escuro) e a BIOS liga 640×480 entrelaçado. **2.2d, 2.2e e 2.2f caíram junto** — eram
  fotografias de um boot que morria no meio.

## Revisão do PR anterior

PR #127 (iter 0110), do próprio orquestrador: quatro checks verdes, mergeado no início da rodada.

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| psx-spx | § Caution - Load Delay (L251) — o delay slot LÊ o valor velho; o texto não diz quem vence quando o delay slot ESCREVE o mesmo registrador | docs/reference/02-cpu.md |
| psx-spx | § Unaligned Load/Store (L320) — lwl/lwr encadeados no mesmo registrador funcionam sem nop, indício de que escrita posterior convive com load pendente | docs/reference/02-cpu.md |
| psx-spx | § GPU Status Register (L1001) — GPUSTAT.23: 0=Enabled, 1=Disabled (para o handoff do item novo 2.10) | docs/reference/03-gpu.md |

A regra em si ("a escrita da instrução no delay slot vence; o load é descartado") **não está na
spec local** — a prova é empírica: a BIOS SCPH1001 em `0x8004723C-40` faz `beq` (não tomado) com
`lw $ra,0x24($sp)` no delay slot, seguido de `jal`. No console real esse código funciona, logo o
link do `jal` sobrevive ao load pendente. No pipeline do R3000 os dois escrevem em WB e o `jal`,
emitido um ciclo depois, escreve por último.

## A cadeia da medição (pilha-sombra de jal/jr no harness, sem tocar o psx-core)

1. Watchpoint da 0108 reconfirmado: o slot `0x801FFDCC` fica intacto; o epílogo em `0x8003FA18`
   lê de outro lugar porque **o `$sp` da saída difere do da entrada**.
2. A função do epílogo (`0x8003F910`) foi entrada UMA vez (passo 19,2 M) e devolvida limpa; no
   passo 85 544 254 o corpo dela é **re-entrado por um retorno** com `$ra=0x8003F9E0` velho de
   66 M passos, guardado por outro quadro.
3. Pilha-sombra (jal/jalr empilha, jr $ra confere): primeiro evento anômalo do cacho é
   `jr@0x80040864` com **`$ra=0`** no passo 85 543 677; o `$sp` na saída da função `0x800404F0`
   está **0x48 abaixo** do da entrada (o slot certo continuava com `0x800404AC`).
4. Sensor de deriva de `$sp` no corpo da `0x800404F0`: **um único evento**, passo 20 116 580 — o
   callee chamado pelo `jal` em `0x80040524` devolve com `$sp` 0x28 mais baixo (`jr@0x8004EEB8`).
5. Trace instrução a instrução da janela (840 passos): em `0x8004723C`, `lw $ra,0x24($sp)` no
   delay slot de um `beq` não tomado carrega `$ra=0x8004052C` **com delay**; o `jal` seguinte
   escreve o link `0x80047248`… e o nosso load pendente aplicava **depois**, esmagando o link.
   O callee salva o `$ra` errado; o retorno pula a metade final de `0x80047210` — inclusive o
   `addiu $sp,+0x28` do epílogo. O `$sp` fica 0x28 baixo, dormente por 65 M passos, e estoura
   como `$ra=4` (o valor que estava no slot deslocado desde o passo 133 574).

## O fix

`cpu.rs`: `set_reg` registra o GPR escrito pela instrução corrente (`written_gpr`); o load
pendente só aplica se `written_gpr != Some(reg)`. Quatro linhas. O caminho de exceção não muda
(02-cpu.md § Caution - Load Delay (L251): IRQ entre load e instrução seguinte completa o load —
comportamento já existente).

## O que caiu junto (medido a 120 M e 300 M passos)

- Boot passa do passo 85 544 264 e **estabiliza no laço de espera de VSync** (`PC=0x80059DCC`).
- `GPUSTAT=0x144E220D`: **bit19=1 (480 linhas), bit22=1 (entrelaçado)** — o modo que a 0110
  declarou incognoscível só é programado DEPOIS do antigo ponto de morte.
- O despejo da VRAM mostra a tela da referência inteira: "SONY" azul-escuro, losango completo
  com "S" vazado, "™", "COMPUTER ENTERTAINMENT" azul-escuro. As CLUTs da linha 480 contêm a
  paleta real (0x7FFF etc.) — na cena de 480 linhas o losango termina em y≈368 e não as toca.
- Fundo termina em `B4B4B4` (fade completo, sem crash no meio) — a referência externa mostra
  branco; capturas de emuladores maduros mostram cinza-claro. Registrado como diferença aberta
  no item 2.2e fechado (nota), não como defeito conhecido.
- **Item novo 2.10:** `framebuffer_for_display()` lê GPUSTAT.23 invertido (03-gpu.md § GPU
  Status Register (L1001): 0=Enabled; gpu.rs:446 devolve None quando 0). O desktop mostraria
  "Display desligado" com o display ligado. Os testes d1/d2 da iteração 0053 codificam a
  polaridade errada e precisarão virar.

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que o comportamento real é | Como foi pego |
|---|---|---|---|---|
| 1 | ferramenta | Que comparar `$sp` no instante do `jr` com o `$sp` da chamada detectasse quadro não restaurado | O idioma MIPS restaura `$sp` no **delay slot do `jr`** — a pilha-sombra acusou centenas de falsos positivos | Todos os "divergentes" tinham delta igual ao tamanho do quadro; o corretor passou a somar o `addiu $sp` do delay slot |
| 2 | ferramenta | Que `cargo build` + exe presente = binário atual | O `vramshot.exe` era da iteração 0110 (fonte nem existia mais em `src/bin`; o exe velho sobreviveu aos builds) — quase medi o fix com o binário sem o fix, e os logs fantasma do GPU denunciaram | Os prints `GP1_08`/`E3` não existiam em lugar nenhum do fonte (`grep` vazio); `cargo clean --release` + rebuild fez o exe SUMIR, expondo que o fonte não estava lá |
| 3 | hardware | Que o R3000A completasse o load pendente por cima da escrita seguinte (era o comportamento implementado e havia teste afirmando isso como "assumido") | A escrita da instrução no delay slot vence; o load é descartado. A BIOS depende disso para `$ra` | O trace da janela de 840 passos mostrou o `jal` perdendo o link; o teste "comportamento_assumido" da iteração de load delay foi virado com a prova |
| 4 | processo | Que o diagnóstico da 0108 ("o prólogo nunca salvou `$ra` nesse slot") apontasse para a função do epílogo | O prólogo salvou, sim — noutra posição, porque quem desalinhou foi um retorno atravessado 65 M passos antes, em OUTRA função | A pilha-sombra achou o primeiro retorno anômalo longe do sintoma; o watchpoint sozinho olhava o lugar certo do jeito errado |

## Bateria de mutação

Placar da bateria: 5/5 mutantes mortos, 2/2 controles verdes, 0 equivalente — docs/mutantes/0111-sp-desalinhado.mut

| id | mutação | morto por |
|---|---|---|
| m1 | guarda invertida (`==`) | todos os 6 do arquivo (aplica só em conflito e descarta o resto) |
| m2 | guarda sempre verdadeira (defeito original) | escrita_alu, jal_no_delay_slot, delay_slot_le_valor_velho |
| m3 | `set_reg` nunca registra | os mesmos três |
| m4 | registra `idx^1` | os mesmos três |
| m5 | limpeza pré-execute vira `Some(31)` | lw_para_ra_sem_conflito (guarda criada para ele) |

## Placar antes → depois

Workspace: **735** → **741** testes (6 novos + 1 virado em `cpu_load_delay.rs`).

## Revisão cruzada (orquestrador)

Iteração inteira do orquestrador.

## Decisões e notas

1. **A regra veio do comportamento da BIOS, não de uma linha de spec** — está registrado na tabela
   de spec acima e na invariante 23. Se o Amidog `psxtest_cpu` entrar no scoreboard um dia, ele
   cobre esse caso e vira a fonte canônica.
2. **2.2d/e/f fechados por consequência**, com a ressalva do fundo (`B4B4B4` vs branco do render
   externo) anotada no ROADMAP. Nenhum código de GPU mudou nesta iteração.
3. **Harnesses de medição arquivados** em `psx-estado/instrumentacao/` (pilha-sombra `shadowstack.rs`,
   sensor de deriva `spdrift.rs`, trace `fulltrace.rs`, watchpoint `watchslot.rs`, desassemblador
   `crashdump.rs`, `crashtrace.rs`, `vramshot.rs` com CLUTs+fundo). Nenhum vive no repo.
4. **Manifesto 0100 reancorado de novo** — as linhas 2.2d/e/f do ROADMAP que ele usava de âncora
   mudaram ao fechar os itens; m2 agora ancora no item aberto 2.9.
5. O boot agora para no laço de VSync esperando o próximo passo do M4 (CD-ROM) — é o item 4.4.
