<!-- Custo, tokens e duracao ficam em docs/metricas.csv, medidos pelo runner. -->

# 0164 — fetch-desalinhado

- **Data:** 2026-08-02
- **Item do roadmap:** 10.91
- **Objetivo:** buscar opcode em endereco desalinhado tem de levantar AdEL, nao executar o alvo.
- **Fonte:** orquestrador.

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| psx-spx | § jumps and branches (L477) | docs/reference/02-cpu.md |
| psx-spx | § Exception Codes (L693-L694) | docs/reference/02-cpu.md |
| psx-spx | § BadVaddr (L813) | docs/reference/02-cpu.md |

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | processo | Que a suite de hardware do Amidog nao rodava neste projeto — o item 10.5 do ROADMAP diz que ela "para apos `args: 0`". | Não é assunto de spec. | Bastou `scripts/fetch-test-exes.ps1` e rodar o EXE pelo `psx-cli`: a suite roda inteira e imprime `Result: 00000909` com 4.918 linhas de erro. O item 10.5 estava velho e ninguem tinha reconferido. |
| 2 | processo | Que o `scoreboard.ps1` daria o veredito da suite. | Não é assunto de spec. | Ele conta **bytes** de TTY e joga o texto fora: reporta `0 com veredito, 50 so com saida`. O veredito estava no texto o tempo todo (itens 10.23/10.24). |
| 3 | endereçamento | Que um salto para endereco desalinhado seria pego pelo `read32` do barramento. | § jumps and branches (L477) de `docs/reference/02-cpu.md`: *"jr/jalr can be used to jump to an unaligned address, in which case an address error (AdEL) exception will be raised on the next instruction fetch"*. | O `step()` nao conferia alinhamento nenhum no fetch. O que o Amidog via era o efeito colateral: o alvo mascarado era executado e, quando calhava de ser um store, saia `AdES (05h)` onde o hardware da `AdEL (04h)`. |
| 4 | API-Rust | Que dava para reusar o caminho de excecao existente so setando `pending_exception`. | Não é assunto de spec. | Esse caminho e processado **depois** de `execute`, e um fetch que falha nao pode executar nada. Extrai o corpo para `enter_exception(...)` e chamei dos dois lugares. |
| 5 | API-Rust | Que a extracao seria invisivel para o resto do projeto. | Não é assunto de spec. | `mutation_anchors` reprovou: a ancora `K1` do manifesto **0055** (`let sr = self.cop0[12];`) passou a casar duas vezes no arquivo. Desambiguei com uma linha de contexto e reexecutei a bateria de 0055 (5/5, 2/2). |

## Bateria de mutação

Placar da bateria: 6/6 mutantes mortos, 3/3 controles verdes, 0 equivalente — docs/mutantes/0164-fetch-desalinhado.mut

Nenhum morreu de erro de compilacao (`grep -cE '^error\[E'` no log da 0), 16 `panicked` de assercao,
e o oraculo nao depende de BIOS nem de disco. `m2` troca `AdEL` por `AdES` — e exatamente o
defeito que o Amidog reportava, e o teste o mata. Reexecutada por ancora envelhecida: `0055`
(5/5, 2/2).

## Placar antes → depois

Workspace: **909 → 915** testes.

**E, pela primeira vez, um placar de hardware:** `psxtest_cpu.exe` do Amidog sai de
`Result: 00000909` para `Result: 00000109`, e as 18 linhas de `jr`/`jalr exception error`
desaparecem (4.918 → 4.900 linhas de erro).

## Revisão cruzada (orquestrador)

Rodada de autoria do orquestrador; autorrevisão registrada como limite. Diferente das rodadas
anteriores, aqui existe **oraculo externo**: a correcao e confirmada por um teste de hardware de
terceiros, nao pelo meu julgamento. Os seis testes novos falharam antes do commit de producao e
passam depois; a bateria mata o mutante que reintroduz o codigo de excecao errado.

## Decisões e notas

§ Exception Codes (L693) de `docs/reference/02-cpu.md` da `04h AdEL — Address error, Data load or
Instruction fetch` e `05h AdES — Address error, Data store`. Nosso `step()` lia o opcode sem
conferir alinhamento, entao um `jr` para `0x2001` nao vetorizava: o `read32` mascarava o endereco
e a CPU executava a palavra vizinha. O `AdES (05h)` que o Amidog via nao era o erro em si — era o
que sobrava depois, quando a palavra executada por acidente era um store.

A correcao confere `instr_pc & 3` antes do fetch e entra na excecao com `ExcCode=04h`,
`BadVaddr = instr_pc` (§ BadVaddr (L813) do mesmo arquivo diz que so erro de endereco atualiza
esse registrador) e `EPC = instr_pc`, sem marcar delay slot: o delay slot do salto ja executou, o
fetch do alvo nao e delay slot. O teste afirma as quatro coisas por valor.

Para isso o corpo de entrada em excecao virou `enter_exception(...)`, chamado tanto pelo fetch
quanto pelo caminho antigo de `pending_exception`. O comportamento do caminho antigo nao muda:
os dois limpam `branch_target`, resolvem o load pendente e escolhem o vetor por `ExcCode`.

**O que fica aberto, com numero,** do mesmo relatorio do Amidog: 4.312 erros em codificacoes de
branch (`b_0xNN_f`/`b_0xNN_b`, apelidos de `bltz`/`bgez`/`bltzal`/`bgezal`) e ~590 em load delay
slot encadeado (`nop_lX_lY_d`). Viraram os itens 10.92 e 10.93 — agora com oraculo de hardware
para medir cada um.
