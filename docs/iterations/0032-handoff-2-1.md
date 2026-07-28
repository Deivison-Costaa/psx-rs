<!-- Custo, tokens e duração NÃO entram aqui: são medidos por scripts/oc-iter.ps1 em
     docs/metricas.csv, casados por head_antes/head_depois. -->

# 0032 — handoff-2-1

- **Data:** 2026-07-28
- **Item do roadmap:** nenhum diretamente (passo zero do 2.1; cria o item 1.14)
- **Objetivo:** conferir o handoff do 2.1 antes de despachá-lo, medir o que de fato acontece
  quando o GPUSTAT devolve o valor de reset da spec, e reordenar a escada com o que a medição
  mostrou.

## Spec consultada

| Fonte | Seção | Arquivo local (linha absoluta) |
|---|---|---|
| psx-spx | Portas `1F801810h`/`1F801814h`, leitura e escrita | `03-gpu.md` L144-147 |
| psx-spx | Tabela de bits do GPUSTAT | `03-gpu.md` L1002-1032 |
| psx-spx | GP1(00h) Reset GPU — **"GPUSTAT becomes 14802000h"** | `03-gpu.md` L747-763 |
| psx-spx | Opcode reservado → RI `excode=0Ah` | `02-cpu.md` L230, L874, L878 |
| psx-spx | LWC0/SWC0 → Coprocessor Unusable `excode=0Bh` | `02-cpu.md` L883-884 |

## Erros de primeira tentativa

Os erros abaixo são do **handoff do 2.1** escrito na iteração 0031, pegos nesta conferência
antes do despacho. É a quinta vez que citação não conferida entra num handoff (0022, 0024,
0026, 0029/G5, e esta).

| # | Categoria | O que o handoff afirmava | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | hardware | "GPUSTAT.19-20 são bits de versão = 2 (GPU revision 2), valor de reset documentado na spec" | **Não existe campo de versão no GPUSTAT.** Bit 19 = Vertical Resolution, bit 20 = Video Mode (`03-gpu.md` L1021-1022) | Abri a tabela de bits antes de despachar |
| 2 | hardware | "GPUSTAT.28 = 1 (odd/even field, valor de reset)" | Bit 28 = **Ready to receive DMA Block** (L1030). Odd/even é o bit **31** (L1033) | Mesma leitura |
| 3 | citação | "GPUSTAT (L1-180)" e "GP0/GP1 commands (L181-400)" | A seção do GPUSTAT começa em L1002; os comandos GP1 em L747+. As faixas foram inventadas | `grep -n` nos títulos das seções |
| 4 | número | Teste de aceitação A3: "o placar continua 50/50" | O placar é 50/**51** | Rodei o scoreboard |

O padrão que se repete: o handoff acerta a estrutura (arquivos-alvo, escopo, armadilhas
plausíveis) e erra o **conteúdo verificável**. Estrutura se copia do handoff anterior;
conteúdo exige abrir o arquivo.

## O que foi medido

Apliquei um stub temporário no `bus.rs` (**não commitado**) devolvendo o valor de reset que a
spec dá pronto:

```rust
0x1F80_1814 => Some(0x1480_2000),   // GPUSTAT após GP1(00h), 03-gpu.md L763
```

e rodei o `psx-cli` contra as suítes. Antes, todo EXE do ps1-tests imprimia ~150 bytes (o
banner `ResetGraph:`) e travava. Depois:

| EXE | TTY antes | TTY depois |
|---|---|---|
| `ps1-tests/cpu/access-time` | ~150 | 2 345 |
| `ps1-tests/cpu/io-access-bitwidth` | ~150 | 3 582 |
| `ps1-tests/gte/chopping` | ~150 | 13 430 |
| `ps1-tests/timers/timers` | ~150 | 384 |
| `ps1-tests/cpu/code-in-io` | ~150 | 1 135 336 |
| `ps1-tests/cpu/cop` | ~150 | **panic** |
| `amidog/cpu/psxtest_cpu` | 8 | 8 |

Três conclusões, todas com consequência:

1. **O bit 26 do GPUSTAT é mesmo a tranca.** Um `Some(0x1480_2000)` de quatro palavras faz as
   suítes saírem do laço e executarem de verdade. O 2.1 vale o que o handoff diz que vale.
2. **`cop.exe` derruba o emulador** assim que avança:
   ```
   thread 'main' panicked at crates\psx-core\src\cpu.rs:231:18:
   not implemented: opcode primary=38 nao implementado
   ```
   Primary `38h` é SWC0 — coprocessor store, que por spec (`02-cpu.md` L883-884) deve levantar
   **Coprocessor Unusable (`0Bh`)**, não Reserved Instruction (`0Ah`).
3. **O Amidog não muda** (8 bytes, `args: 0`): ele trava em outro ponto, não no GPUSTAT. Fica
   como pergunta aberta para quando o 2.1 estiver pronto — não assumir que o 2.1 o destrava.

## Bateria de mutação

Não se aplica: sem mudança em `crates/` (o stub do `bus.rs` foi revertido; `git status` limpo
antes do commit).

## Placar antes → depois

247 → 247 testes (inalterado).

## Revisão cruzada (orquestrador)

Iteração do próprio orquestrador. O erro nº 4 da tabela é meu tanto quanto do trabalhador: o
"50/50" veio de um handoff que eu revisei na iteração 0031 sem conferir o número.

## Decisões e notas

1. **Item 1.14 criado e colocado ANTES do 2.1.** Enquanto o `unimplemented!()` estiver no
   decodificador, cada suíte que alcança um opcode novo mata o processo em vez de virar uma
   linha de placar — e o 2.1 existe justamente para fazer as suítes avançarem. Fazer o 2.1
   primeiro seria abrir a comporta com o cano furado.
2. **O handoff do 1.14 exige o stub de GPUSTAT no teste de aceitação A4**, com aviso explícito
   de não commitar. Sem ele, o `cop.exe` nem chega ao opcode e o teste não prova nada. É o
   primeiro handoff do projeto que manda usar um andaime temporário; o risco de ele vazar para
   um commit está registrado aqui de propósito.
3. **O que NÃO foi decidido:** por que o Amidog `psxtest_cpu` para com 8 bytes mesmo com o
   GPUSTAT pronto. Medi que não muda, não medi a causa. Fica aberto em vez de virar palpite no
   handoff — foi exatamente esse tipo de palpite que gerou as iterações 0024 e 0026.
