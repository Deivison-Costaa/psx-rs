<!-- Custo, tokens e duração NÃO entram aqui: são medidos por scripts/oc-iter.ps1 em
     docs/metricas.csv, casados por head_antes/head_depois. -->

# 0205 — timer-lhu-lbu-clear

- **Data:** 2026-08-06
- **Item do roadmap:** 10.52 (achado legado, iteração de origem 0118)
- **Objetivo:** um `lhu`/`lbu` no registrador MODE do timer tem que limpar os bits 11/12
  (target/FFFFh alcançado), igual um `lw` (`read32`) já faz — hoje só o acesso de 32 bits tem
  o efeito colateral de "clear on read".

## Spec consultada

Nenhuma nova — o comportamento de "clear on read" dos bits 11/12 já estava correto e testado
para `read32` (`Timers::read32`); este achado é sobre estender o MESMO efeito colateral já
implementado pros caminhos de acesso de 8 e 16 bits do barramento (`region_read_byte`/
`read16`), não sobre uma semântica de hardware nova.

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a medição mostrou | Como foi pego |
|---|---|---|---|---|
| 1 | escopo do fix | Que trocar `peek32` por `read32` dentro de `region_read_byte` bastaria pra corrigir tanto `lbu` quanto `lhu` | `read16` chama `region_read_byte` DUAS vezes (uma por byte) — se as duas chamadas passassem a usar `read32` (com efeito colateral), a segunda chamada veria os bits JÁ limpos pela primeira, corrompendo o segundo byte do halfword composto. Precisei de um braço próprio em `read16` pro intervalo de timers (mesmo padrão já usado pra SPU/GPU logo acima), chamando `Timers::read32` uma única vez e extraindo os dois bytes do mesmo instantâneo | Escrevi um terceiro teste especificamente pra essa armadilha (`read16_no_meio_da_transferencia_nao_corrompe_o_segundo_byte`) antes de implementar, comparando o valor composto exato — não só os bits 11/12 isolados |
| 2 | teste | Que o valor esperado do terceiro teste seria só `0x0008 \| 0x0800 = 0x0808` | `cargo test` mostrou `0x0C08`, não `0x0808` — `Timers::write32` seta o bit 10 (armado/pulso de IRQ) como parte da própria escrita em MODE (`timers.rs:92`), independente do meu fix. O erro era no valor esperado do teste, não na implementação | Reli `timers.rs::write32` antes de "corrigir" a implementação pra bater com o teste — a implementação estava certa, o teste que tinha o cálculo errado |

## Bateria de mutação

Placar da bateria: 4/4 mutantes mortos, 2/2 controles verdes, 1 equivalente —
`docs/mutantes/0205-timer-lhu-lbu-clear.mut`.

- m1 (`region_read_byte` volta a usar `peek32`): morto.
- m2 (braço de `read16` vira no-op): morto.
- m3 (braço de `read16` existe mas usa `peek32`): morto.
- m4 (`region_read_byte` não alinha o offset antes de chamar `read32`): morto.
- m5 (braço de `read16` não alinha o offset): **equivalente** — só muda o resultado num
  endereço (`T0_MODE+2` = 1F801106h) que a spec não documenta como registrador real do timer
  e que o achado 10.52 não trata; sem citação de spec pra fixar o que aconteceria lá, não
  inventei comportamento.
- c1 (mascara de shift em binário vs decimal): verde.
- c2 (ordem da soma no `byte_index`): verde.

## Placar antes → depois

Workspace: **1264** → **1267** testes (3 novos em `timers_lhu_lbu_clear.rs`).

## Revisão cruzada (orquestrador)

Sem achados — esta iteração foi conduzida pelo próprio orquestrador (exceção vigente em
`docs/orquestracao.md`; ver STATUS.md).

## Decisões e notas

**1. `docs/mutantes/0118-timer-portas-16bits.mut` arquivado.** Dois mutantes (m1/m2) ancoravam
o `peek32` que este fix removeu — arquivados com `arquivada:`, mesmo padrão já usado
repetidamente nesta sessão pra manifestos antigos que perderam a âncora por uma mudança
legítima e não relacionada ao achado original deles.

**2. Não toquei em `region_write_byte`.** O caminho de escrita byte-a-byte do timer já usa
`peek32` (sem efeito colateral) de propósito — uma escrita não deve disparar o "clear on
read", e trocar isso seria escopo novo sem base na spec nem no achado.

**3. Branch paralela.** Este item foi feito em paralelo com a 0205 do achado 10.49 (DICR bus
error), em worktrees/branches separadas — as duas vão divergir em STATUS/achados/ROADMAP-
fechado até reconciliar no merge, igual aconteceu com o lote anterior desta sessão.
