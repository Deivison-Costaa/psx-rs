<!-- Custo, tokens e duração NÃO entram aqui: são medidos por scripts/oc-iter.ps1 em
     docs/metricas.csv, casados por head_antes/head_depois. -->

# 0166 — bcondz-codificacoes

- **Data:** 2026-08-02
- **Item do roadmap:** 10.92
- **Objetivo:** corrigir a decodificação dos 32 valores de `rt` do BcondZ (opcode 000001b),
  eliminando o no-op que os 28 valores fora da tabela oficial (bltz/bgez/bltzal/bgezal)
  causavam contra o oráculo Amidog.

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| psx-spx | § Opcode/Parameter Encoding (L193-196) — só tabela os 4 encodings canônicos | docs/reference/02-cpu.md |
| psx-spx | § jumps and branches (L469-470) — semântica de rs<0 (bltz) e rs>=0 (bgez) | docs/reference/02-cpu.md |
| psx-spx | § jumps and branches (L478) — link sempre ocorre; rs==$ra usa o valor antes do link | docs/reference/02-cpu.md |

A spec local — e o psx-spx upstream, checado nesta rodada (commit `035c7654`, o mesmo já
pinado em `scripts/fetch-reference-docs.ps1` continua sendo o mais recente a tocar
`cpuspecifications.md`) — **não documenta** o que os outros 28 valores de `rt` fazem. Não há
doc novo para baixar nem commitar: o oráculo desta rodada é o relatório do Amidog
(`psxtest_cpu`, família `b_0xNN`), conforme autorizado pela tarefa.

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | hipótese | Que o link (escrita em `$ra`) era decidido pelo bit4 solto de `rt` (`rt & 0x10 != 0`), replicando o `rt >= 16` que já existia no código antigo. | Não é assunto de spec: a tabela local só cobre `rt`=00h/01h/10h/11h (`02-cpu.md` L193-196). | Apliquei a hipótese e rodei o oráculo: os erros de `b_0xNN` caíram de 4.312 para 2.800, mas `b_0x12`..`b_0x1f` passaram a reprovar 100/100 na linha do `$ra` (`got 00000001 wanted 00000000`) — o hardware NÃO linka nesses valores, só em `rt`=10h/11h exatos. |
| 2 | processo | Que dava para usar `teste:` por registro no manifesto de mutação para desviar os mutantes de limite (`rs==0`) para `cpu_branches.rs`, mantendo o cabeçalho apontando para o teste novo. | Não é assunto de spec. | `scripts/mutantes.ps1` tem duas cláusulas `"teste"` no mesmo `switch` do PowerShell sem `break`; qualquer `teste:` de registro também reescreve o `header.teste`, valendo depois para os registros SEM override. Sintoma: `m1`-`m5` apareceram matados por testes de `cpu_branches.rs` mesmo com o cabeçalho apontando para `cpu_bcondz_codificacoes`, e `m3` sobreviveu porque o alvo real virou o arquivo errado. Reescrevi o manifesto sem overrides (acrescentei cobertura de limite ao próprio arquivo novo) em vez de mexer no script — fora do escopo do item 10.92. |

## Bateria de mutação

Placar da bateria: **7/7 mutantes mortos, 2/2 controles verdes, 0 equivalente** —
`docs/mutantes/0166-bcondz-codificacoes.resultado`.

- `m1` (reintroduz o `_ => return` original) — morto por `varredura_dos_32_valores_de_rt`.
- `m2` (inverte qual paridade de `rt` desvia com `rs` negativo) — morto por
  `varredura_dos_32_valores_de_rt` e mais três testes de `bgez`/`bltz`.
- `m3` (volta a linkar por `rt >= 16` em vez de `rt` exato) — morto por
  `varredura_dos_32_valores_de_rt`.
- `m4` (nunca linka, nem nos encodings canônicos) — morto por `varredura_dos_32_valores_de_rt`
  e mais dois testes de link.
- `m5` (linka exatamente nos encodings que não deveriam linkar) — morto por
  `varredura_dos_32_valores_de_rt` e mais dois testes de link.
- `m6` (fecha o limite de bgez excluindo `rs==0`) — morto por `zero_e_o_limite_entre_bltz_e_bgez`.
- `m7` (abre o limite de bltz incluindo `rs==0`) — morto por `zero_e_o_limite_entre_bltz_e_bgez`.
- `c1`/`c2` (reescritas equivalentes com `matches!`/`%`) — sobreviveram, como esperado.

Nenhum manifesto antigo tem âncora em `bcondz`; `mutation_anchors` não acusou envelhecimento.

## Placar antes → depois

- Amidog CPU (`psxtest_cpu`): `Result: 00000109` → `Result: 00000101`; erros `error @` da
  família `b_0xNN`: **4.312 → 0**.
- Workspace: **917 → 921 testes**.
- Portões: `cargo fmt --all`, `cargo fmt --all -- --check`, `cargo clippy --all-targets -- -D
  warnings` e `cargo test --all --no-fail-fast` — todos verdes (a única falha intermediária foi
  o placar do STATUS.md, corrigido neste mesmo commit de docs).

## Revisão cruzada (orquestrador)

Primeira rodada conduzida por `claude-sonnet-5` como trabalhador (as anteriores foram
`gpt-5.6-luna` ou o próprio orquestrador). Revisada antes do merge.

**Aprovado sem correção.** O que eu conferi, e por quê:

1. **A regra empírica bate com o hardware documentado fora deste repositório.** `rt`=10h/11h
   exatamente é o mesmo conjunto que `(rt AND 1Eh) == 10h`, isto é, bits 20..17 = 1000b — o
   critério que a invariante 3 já tinha registrado como palpite. O trabalhador chegou nele pelo
   oráculo, sem tê-lo lido pronto, e derrubou a própria primeira tentativa (`rt & 10h != 0`)
   quando o Amidog reprovou `b_0x12`..`b_0x1f`. Erro de primeira tentativa registrado no doc.
2. **A alteração em teste existente não afrouxa nada.** `bcondz_rt_fora_da_tabela_*` afirmava
   `pc == 0x8` com a mensagem "SUPOSICAO NAO VERIFICADA"; virou `pc == 0x14` medido. Trocar um
   marcador de dúvida por um valor de hardware é o desfecho previsto pela própria invariante 3,
   não um teste enfraquecido para passar.
3. **O teste distingue o caso que importa em `rs == $ra`.** Se a comparação usasse o valor
   pós-link (8), `bltzal` deixaria de desviar e `bgezal` passaria a desviar — o teste fixa os
   dois sentidos, então não é vacuoso.
4. **Nenhum mutante morreu de erro de compilação.** As sete linhas `error:` do log são o
   `test failed` do cargo; `error[E…]` não aparece nenhuma vez, e há 14 `panicked` de asserção.
   O `.resultado` versionado ainda registra qual teste matou cada mutante.
5. Portão reexecutado pelo orquestrador na árvore limpa: 921 testes, `fmt --check` e
   `clippy -D warnings` verdes; CI verde nos quatro jobs.

**Ressalva que fica aberta, e não deve virar comemoração:** o relatório de erro do Amidog está
vazio, mas `Result` é `00000101`, não `00000000`. Os bits 0x001 e 0x100 estavam ligados em todas
as medições, inclusive nas de 4.918 erros, e não caíram junto com nenhuma família — a hipótese é
que não sejam bits de falha, mas é hipótese. Não escrever em lugar nenhum que a CPU passa limpa
antes de decodificar esse valor.

## Decisões e notas

- Correção mínima em `bcondz`: `link = rt == 0x10 || rt == 0x11` (exato, não mais `rt >= 16`)
  e `cond = if rt & 0x01 == 0 { rs_val < 0 } else { rs_val >= 0 }` (bit0 sozinho, para os 32
  valores). O `rs_val` continua lido ANTES do link, preservando a regra de `rs`==`$ra`.
  Não mudei mais nada em `bcondz` (R4).
- Invariante 3 de `docs/invariantes.md` — que registrava a suposição de no-op e já apontava
  "bit16 decide a condição e o link vem de bits 20..17 = 1000b" como critério a testar
  primeiro — foi marcada como resolvida: o palpite batia quase certeiro (o link é por `rt`
  exato, não por padrão de bits).
- O teste antigo `bcondz_rt_fora_da_tabela_comportamento_assumido` (em `cpu_branches.rs`)
  fixava o no-op como suposição; renomeado para `bcondz_rt_fora_da_tabela_desvia_como_bltz` e
  reescrito para a asserção correta (`pc == 0x14`), já que a suposição documentada não é mais
  o comportamento do código.
- Achado de tooling fora de escopo: `scripts/mutantes.ps1` tem um bug de `switch` do
  PowerShell (duas cláusulas `"teste"` sem `break` fazem o override de registro vazar para o
  cabeçalho). Não corrigido nesta rodada — pertence a outro item. Registrado também na tabela
  de erros de primeira tentativa acima.
- Item 10.92 fechado nesta rodada: os 4.312 erros de branch mencionados no handoff de 0165 e
  no bloqueio do STATUS eram inteiramente desta família, e foram para zero.
