# 0151 — hook-flow

- **Data:** 2026-08-02
- **Item do roadmap:** 10.75
- **Objetivo:** medir a entrada do hook `0x801B8E60` e localizar o desvio que impede o caminho do contador.

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| psx-spx | § B(19h) - HookEntryInt(addr) (L1480-L1482) | `docs/reference/13-kernel-bios.md` |
| psx-spx | § Priority Chains (L1490-L1502) | `docs/reference/13-kernel-bios.md` |

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | custo de sondagem | Ler o opcode em todos os 500 M passos seria aceitável para montar o traço | A spec não autoriza custo de instrumentação; o dado necessário era apenas o trecho observado | A revisão da sonda encontrou `read32` fora do bloco das 20 ativações; movi a leitura para os 40 PCs capturados |
| 2 | identificação | O contador `hook entries: 1029` representava somente o hook do jogo | Há dois `B(19h)` instalados: o kernel usa `0x8005A1D8` e o jogo usa `0x801B8E60` | Filtrar por `hook[0]` produziu 13 entradas antes do primeiro spin e mais 7 depois; o total agregado não foi usado como evidência |
| 3 | cobertura | Parar no primeiro spin bastaria para as 20 ativações pedidas | A medição pedia as primeiras 20 ativações, não apenas as anteriores ao spin | A primeira execução capturou 13; a sonda continuou após `166378016` e capturou 20 sem alterar o emulador |
| 4 | passagem vazia | O novo teste poderia manter o placar antigo porque só acrescenta um diagnóstico | O portão compara o número do workspace com o `STATUS.md` | `status_handoff` falhou com 884 contra 885; o handoff foi atualizado antes do portão final |

## Bateria de mutação

Bateria de mutação: não se aplica — diagnóstico puro; nenhuma linha de produção foi alterada, somente teste, documentação e sonda descartável.

## Placar antes -> depois

- Antes: `884` testes registrados em `STATUS.md`.
- Depois: `885` testes; o novo `rayman_hook_flow.rs` passou sem BIOS ou disco.
- `cargo fmt --all -- --check`: verde.
- `cargo clippy --all-targets -- -D warnings`: verde.
- `cargo test --all --no-fail-fast`: verde após atualizar o placar do `STATUS.md`.

## Resultado da medição

A sonda foi executada com o disco `../roms/extraido/Rayman (USA) DADOS.cue`, sem alterar o
emulador. O primeiro spin ocorreu no passo `166378016`. As 20 entradas filtradas por
`hook[0] == 0x801B8E60` foram:

| # | passo | r2 | I_STAT | I_MASK |
|---:|---:|---:|---:|---:|
| 0 | 164112358 | `0x00000001` | `0x0001` | `0x000D` |
| 1 | 164125921 | `0x00000001` | `0x0008` | `0x000D` |
| 2 | 164127523 | `0x00000001` | `0x0008` | `0x000D` |
| 3 | 164157984 | `0x00000001` | `0x0000` | `0x000D` |
| 4 | 164455141 | `0x00000001` | `0x0000` | `0x000D` |
| 5 | 164754634 | `0x00000001` | `0x0000` | `0x008D` |
| 6 | 165054097 | `0x00000001` | `0x0000` | `0x000D` |
| 7 | 165354348 | `0x00000001` | `0x0000` | `0x008D` |
| 8 | 165653783 | `0x00000001` | `0x0000` | `0x000D` |
| 9 | 165988085 | `0x00000001` | `0x0000` | `0x008D` |
| 10 | 166322103 | `0x00000001` | `0x0000` | `0x000D` |
| 11 | 166364458 | `0x00000001` | `0x0004` | `0x000D` |
| 12 | 166374939 | `0x00000001` | `0x0004` | `0x000D` |
| 13 | 166576521 | `0x00000001` | `0x0000` | `0x000D` |
| 14 | 166823870 | `0x00000001` | `0x0000` | `0x000D` |
| 15 | 167072224 | `0x00000001` | `0x0000` | `0x000D` |
| 16 | 167319602 | `0x00000001` | `0x0000` | `0x000D` |
| 17 | 167567957 | `0x00000001` | `0x0000` | `0x000D` |
| 18 | 167816312 | `0x00000001` | `0x0000` | `0x000D` |
| 19 | 168063689 | `0x00000001` | `0x0000` | `0x000D` |

H1 está refutada: `r2` foi `1` em todas as 20 entradas, exatamente o valor descrito pela
spec para `HookEntryInt` (docs/reference/13-kernel-bios.md L1480-L1482). `I_MASK` também manteve
o bit 0 em todas as entradas.

O trecho observado contém:

- `0x801B8EA0: 0x1040003C`, `beq v0,$zero,0x801B8F94`. Nos casos com `I_STAT=0`, o valor
  calculado em `v0` era zero; o branch tomou o alvo `0x801B8F94` após o delay slot `0x801B8EA4`.
- Nos casos com `I_STAT=1`, `8` ou `4`, `0x801B8EA0` não tomou o branch. A checagem seguinte
  `0x801B8F0C: 0x1060000D` separou VBlank (`I_STAT=1`, segue) de outras causas (`8` ou `4`,
  alvo `0x801B8F44`).

Portanto, o PC concreto do primeiro desvio que exclui o caminho do incremento é
`0x801B8EA0`, sob a condição `I_STAT & I_MASK == 0`; o alvo do desvio é `0x801B8F94`.
Para uma causa não-VBlank com status não-zero, o desvio equivalente da checagem do bit 0 é
`0x801B8F0C` para `0x801B8F44`. H2 está confirmada como condição de fluxo; não há conserto
de hardware nesta iteração.

A sonda também observou o ciclo que teria o `sw` `0xAC22F2CC` em `0x801B8C50`, mas o valor
observado no contador permaneceu zero antes e depois e nenhum watchpoint de store ao endereço
`0x801DF2CC` apareceu. A presença de um PC candidato não foi tratada como prova de incremento.

## Revisão cruzada (orquestrador)

Pendente da revisão adversarial; esta iteração não altera timers, IRQ, vetor ou a produção.

## Decisões e notas

- A sonda ficou restrita ao teste descartável `vsync_timeout_diag.rs` e foi removida antes do commit de documentação.
- O teste permanente é um oráculo puro de branch MIPS com os 20 valores medidos; não faz `skip` por ambiente.
- A cadeia de prioridade da BIOS permanece relevante: a spec diz que um handler que reconhece a IRQ pode executar `ReturnFromException` e pular prioridades inferiores (docs/reference/13-kernel-bios.md L1494-L1502). A medição desta rodada localiza o desvio dentro do hook, mas não atribui sozinha quem limpou `I_STAT`.
