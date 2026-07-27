# 0016 — MULT/MULTU/DIV/DIVU + HI/LO

- **Data:** 2026-07-27
- **Item do roadmap:** 1.6
- **Objetivo:** Implementar MULT, MULTU, DIV, DIVU, MFHI, MTHI, MFLO, MTLO com registradores hi/lo na CPU.

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| psx-spx | Multiply/divide (L329) | docs/reference/02-cpu.md |

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | API-Rust | `self.reg(rs) as i64` faz sign-extend corretamente | u32 → i64 zero-extende; precisa de `as i32 as i64` para sinalizar | `mult_negativo` falhou: hi=1999 em vez de 0xFFFF_FFFF |
| 2 | endereçamento | `encode_special(MFHI, 0, 0, 8)` escreve no registrador 8 | MFHI usa campo `rd` (bits 11..15), não `rs` (bits 21..25) | `mfhi_le_hi` e `mflo_le_lo` falharam — rd=0 (R0, ignorado) |
| 3 | nenhum | Expectativa de `mult_64bits_hi_lo` calculada errada | 0x1000_0001 * 0x0002_0000 = 0x2000_0002_0000, hi=0x2000, lo=0x20000 | Teste esperava hi=0x20000; corrigido |


## Bateria de mutação

Placar: **7/7 mutantes pegos, 2/2 controles verdes**.

| # | Mutação | Teste que pegou |
|---|---|---|
| 1 | MULT usa u64 (unsigned) em vez de i64 | `mult_negativo` |
| 2 | MULT: hi=0 sempre (ignora parte alta) | `mult_64bits_hi_lo`, `mult_negativo` |
| 3 | MULTU: hi=0 sempre | `multu_basico`, `multu_grande` |
| 4 | DIV sem tratar divisão por zero (panic) | `div_por_zero_rs_positivo`, `div_por_zero_rs_negativo` (panic) |
| 5 | DIV sem tratar overflow 0x80000000/-1 (panic) | `div_overflow_80000000_por_menos_1` (panic) |
| 6 | DIV: lo=0 para div por zero (sinal errado) | `div_por_zero_rs_positivo`, `div_por_zero_rs_negativo` |
| 7 | DIVU sem tratar divisão por zero (panic) | `divu_por_zero` (panic) |

Controles:
1. Renomear `a`/`b` para `x`/`y` em MULT — passou (18/18)
2. Reordenar MFHI/MTHI/MFLO/MTLO no match — passou (18/18)

## Placar antes → depois

129 → **149** testes no workspace (18 do item + 2 da revisão cruzada).

## Revisão cruzada (orquestrador)

**A tabela de erros de divisão está correta nas quatro linhas, e é o que a spec destaca
como contraintuitivo.** `02-cpu.md § Multiply/divide` diz explicitamente que o hardware
**não** gera exceção em divisão por zero nem no overflow, e tabela os valores: `divu x/0 →
Hi=Rs, Lo=FFFFFFFFh`; `div 0..+7FFFFFFFh/0 → Hi=Rs, Lo=-1`; `div -80000000h..-1/0 → Hi=Rs,
Lo=+1`; `div -80000000h/-1 → Hi=0, Lo=-80000000h`. As quatro estão implementadas e as
quatro têm teste. `MULT` sign-extende com `as i32 as i64` e `MULTU` zero-extende com `as
u64`, cada um com teste de parte alta. O handoff aponta para o 1.7, dentro do M1 — sem a
reincidência de escopo das iterações 0010 e 0015.

### Achado 1 — SEVERIDADE MÉDIA — CI vermelha por lint que o clippy local não conhece

O job `check` reprovou em `cpu.rs:299` com `clippy::manual_checked_ops` ("manual checked
division"): `divu` testava `rt_val == 0` e depois dividia. O trabalhador rodou `cargo
clippy -D warnings` local e viu verde; a remediação automática do `oc-loop` rodou
`clippy --fix` e não achou nada para corrigir — porque **o clippy local não tem esse
lint**. Stable local: 1.92.0 (2025-12-08). CI: `dtolnay/rust-toolchain@stable`, que instala
a última stable — o log da falha aponta para a documentação do clippy **1.97.0**. Cinco
versões de defasagem.

É a primeira falha de CI desta série que **não é do código do emulador e nem do processo**,
e sim do ambiente: o passo 7 do protocolo ("fmt + clippy + test") só significa alguma coisa
se o toolchain local for o mesmo da CI. Corrigido em duas frentes: `divu` reescrito com
`checked_div`/`checked_rem` (mesmo comportamento, o `_` do match cobre o divisor zero e
devolve os valores tabelados), e `rustup update stable` na máquina de desenvolvimento. O
STATUS agora manda sincronizar o toolchain antes do clippy.

Fica a decisão em aberto para uma iteração de infra: **pinar** o toolchain com
`rust-toolchain.toml` (local e CI na mesma versão fixa, bump deliberado) em vez de perseguir
`stable` dos dois lados. Pinar é mais determinístico e combina com a tese do projeto; o
custo é lembrar de subir a versão. Registrado em `docs/orquestracao.md`, não decidido aqui.

### Achado 2 — SEVERIDADE MÉDIA — buraco de cobertura: DIVU sem sinal não era testado

Os 18 testes entregues **não distinguem DIVU de DIV**. Prova: troquei o corpo de `divu`
por divisão com sinal (`(rs_val as i32) / (rt_val as i32)`) e a suíte inteira passou:
**18/18 verdes com o mutante vivo**. Os dois testes de DIVU usavam `rs=100`, valor em que
com e sem sinal dão o mesmo resultado.

A bateria de mutação da iteração tem 7 mutantes e nenhum ataca a *assinatura* do DIVU —
todos atacam divisão por zero, overflow ou a parte alta do produto. Vale a comparação com o
MULTU, que **tem** teste com bit alto (`multu_basico`, `rs=0x8000_0000`): o trabalhador
lembrou do sinal na multiplicação e esqueceu na divisão.

Fechado com dois testes (a implementação já estava certa — isto é cobertura, não correção):
`divu_com_dividendo_de_bit_alto_e_sem_sinal` (0xFFFFFFFF/2 = 0x7FFFFFFF, não 0) e
`divu_de_80000000_por_ffffffff_nao_e_o_caso_especial_do_div` (sem sinal o quociente é 0 e o
resto é 0x80000000; a linha `div -80000000h/-1` da tabela vale só para o DIV com sinal).
Com o mutante aplicado, o primeiro falha por valor e o segundo por pânico de overflow —
**1/1 pego, e os 18 originais continuam verdes** como controle.

### Achado 3 — o trabalhador escalou a dúvida de fatiamento em vez de fatiar sozinho

`cpu.rs` passou de 500 linhas (521) e o STATUS entregue trazia um comentário HTML pedindo
decisão do orquestrador, em vez de cortar o módulo por contagem. É exatamente o
comportamento que a regra corrigida na 0015b queria produzir — na 0015 o handoff mandava
"começar fatiando". Respondido: `cpu.rs` continua inteiro; o comentário saiu do STATUS
(que tem orçamento de 16 KB) e a resposta ficou aqui.

## Nota sobre stalls (dívida aceita)

A spec tabela a latência (`multu` 6/9/13 ciclos conforme `rs`, `div/divu` fixo em 36) e diz
que ler `hi`/`lo` com a operação em curso **trava a CPU**. Nada disso é observável enquanto
a CPU não cobrar ciclos do scheduler; a dívida está registrada no doc e não bloqueia o item.

## Decisões e notas

- Stalls (ciclos de MULT/DIV) registrados como dívida: serão observáveis quando o scheduler cobrar ciclos da CPU. A implementação atual é funcional mas não contabiliza latência.
