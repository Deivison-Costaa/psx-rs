# 0017 — LWL/LWR/SWL/SWR — **REPROVADA** (PR #27 fechada sem merge)

- **Data:** 2026-07-27
- **Item do roadmap:** 1.7 (continua aberto)
- **Resultado:** rejeitada na revisão adversarial do orquestrador
- **Custo da tentativa:** US$ 0,0192 — 36.756 tokens de entrada, 28.170 de saída, 48 steps,
  5min17 (`deepseek/deepseek-chat`)
- **Artefato:** https://github.com/Deivison-Costaa/psx-rs/pull/27 (fechada, com a revisão
  nos comentários)

Este documento existe porque falha medida é dado do projeto. A iteração entregou CI verde,
20 testes passando, bateria de mutação com placar cheio — e a implementação estava errada
nos quatro opcodes.

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| psx-spx | Unaligned Load/Store + Unaligned Load/Store (Details) | docs/reference/02-cpu.md L235, L257 |

## Defeito 1 — vias de byte mascaradas em vez de deslocadas

A spec tabela a transferência por **endereço de byte**:

```
  lwl/swl [N*4+0]     transfer upper  8bit of Rt to/from [N*4+0]
```

Em little-endian, o byte em `N*4+0` é o **menos** significativo da palavra em `N*4`: depois
de `write32(0x1000, 0xAABBCCDD)`, `[0x1000]` vale `0xDD`. Logo `LWL` em `0x1000` põe `0xDD`
em `rt[31:24]`.

A implementação fazia `(old & 0x00FF_FFFF) | (mem_word & 0xFF00_0000)` — máscara sem
deslocamento, pegando o byte de `[N*4+3]`. Os 12 casos que não são "palavra inteira" estavam
errados; os quatro que transferem 32 bits acertavam por não haver via para deslocar.

## Defeito 2 — LWL e LWR não enxergavam um ao outro

A spec documenta o idioma imediatamente acima da tabela, e ele é a razão de os opcodes
existirem:

```
  lwl   r2,$0003(t0)   ;\no delay required between these
  lwr   r2,$0000(t0)   ;/(although both access r2)
```

Sonda do orquestrador com `[0..3] = DD CC BB AA`, `[4..7] = 44 33 22 11`, `t0 = 1` — a
palavra desalinhada em 1 é `0x44DDCCBB`. Resultado obtido: **`0x00CCBBAA`**. O `lwr` faz o
merge contra `self.reg(rt)`, que ainda é o valor antigo enquanto o resultado do `lwl` está
no load delay; o `lwl` é commitado depois e em seguida sobrescrito. A contribuição dele some
inteira.

## Por que os testes não pegaram

**Teste e implementação foram escritos pelo mesmo agente, na mesma sessão, a partir do mesmo
modelo mental errado.** O comentário do `lwl_offset_0_upper_8bits` é a paráfrase errada da
spec — "upper 8 bits of mem word" onde a spec diz *byte no endereço* — e a asserção
(`0xAAFF_FFFF`) apenas repete o que o código faz. A R5 (teste antes de implementar) foi
cumprida à risca e não protegeu: escrever o teste primeiro só protege contra o código
divergir do que o autor entendeu, nunca contra o autor ter entendido errado.

Agrava: `lwl_lwr_pair_different_regs` usa registradores **diferentes**, contornando
exatamente o caso que a spec documenta. E a bateria de mutação (7/7 "pegos") conferiu os
mutantes contra as mesmas expectativas erradas — placar cheio, valor zero.

## O que muda no protocolo

Para item cuja spec traz um **idioma canônico** (aqui, o par lwl/lwr reconstruindo uma
palavra desalinhada), o handoff passa a nomear um **teste de aceitação com valores
concretos**, derivado da spec pelo orquestrador e obrigatório no PR. Uma asserção ancorada
em bytes literais não pode ser satisfeita por um modelo mental errado — ao contrário de uma
asserção que o próprio autor deriva.

Vale registrar o que **o handoff anterior errou**: ele foi escrito por mim na revisão da
0016 e avisava que "o endereço define qual fragmento é transferido (tabelado na spec)" —
verdadeiro e inútil. Não nomeou a armadilha que importava (endereço de byte × valor da
palavra) nem exigiu o idioma da spec como teste. Metade do defeito nasce aí.

## Encaminhamento

Item 1.7 continua aberto, segunda tentativa a partir da main com handoff corrigido no
STATUS (as duas armadilhas nomeadas e o teste de aceitação obrigatório). A branch antiga
não é recuperada: os 20 testes teriam de ser reescritos de todo modo.
