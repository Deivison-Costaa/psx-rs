# 0099 — ra-crash-diagnostico

- **Data:** 2026-07-30
- **Item do roadmap:** 4.4f (diagnostico, sem implementacao de correcao)
- **Objetivo:** Diagnosticar a corrupcao de $ra=3 no boot da BIOS, registrando itens 4.4f e 10.39 no ROADMAP.

## Revisão do PR anterior

Revisao do PR #113 (iter 0098 — oc-iter travamento):

1. TESTE QUE NAO MEDE — Os 4 testes de `ci_oc_iter.rs` verificam presenca de tokens no script. O teste exige a string exata `$ultimoAvanco = Get-Date`; mudanca equivalente reprovaria. Limitacao conhecida na nota 5 do doc da 0098. Sem falso negativo.
2. PARAMETRO NAO CONSUMIDO — Nao aplicavel.
3. REGRA DE BORDA TROCADA — Nao aplicavel.
4. CAMPO DE BIT LIDO ERRADO — Nao aplicavel.
5. PANIC/LACO ILIMITADO — Nao aplicavel.
6. CITACAO DE SPEC — `spec_citations` verde.
7. ESCOPO TRANSBORDADO — Implementa exatamente 10.38.
8. PORTAO QUE NAO MEDE — Bateria 6/6 mortos, 2/2 controles.
9. NAO ARQUIVE MANIFESTO — Nao houve.

Revisao do PR anterior: sem achados.

## Spec consultada

Nenhuma secao implementada. Diagnostico apenas:
- `docs/reference/02-cpu.md` § Caution - Load Delay (L251), § exception opcodes (L489)

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que o comportamento real e | Como foi pego |
|---|---|---|---|---|
| 1 | hipotese | Que pular o handler A0/B0 apos o hook resolveria a corrupcao | Pular o handler impediu o boot (I_MASK nunca mudou de 0x0000). O handler A0 e essencial para inicializacao | Teste `bios_escreve_i_mask_durante_boot` falhou |
| 2 | hipotese | Que range check 0x1F801000-0x1F801023 no mirror era correto | So EXP1_BASE (0x1F801000) desativa o mirror | Teste `mirror_nao_desativado_por_outros_registradores` falhou |
| 3 | processo | Que main nao mudaria durante a sessao | Commits 8c456aa e 348e852 foram mergeados em main, avancando em 2 commits | `git log` mostrou main a frente |
| 4 | processo | Que alteracoes sobreviveriam ao checkout | Alteracoes no index sobreviveram; arquivos foram perdidos 3x | `git status` mostrava sujeira nao minha |

## Diagnostico

Instrumentacao temporaria no CPU (`ra_write_log` em `set_reg`) e Bus (watchpoint em `write32`). Teste com BIOS real rodando 30M passos.

1. **Passo da corrupcao:** 26 595 827.
2. **Endereco:** `lw $ra, 0x2C($sp)` le de 0x801FFB84 ($sp = 0x801FFB58).
3. **Valor antigo:** 0x8004A4C8 (retorno valido em KSEG0).
4. **Watchpoint:** Escreve valores 0-15 repetindo a cada 144 ciclos. O write do valor 3 ocorre ~26 454 488.
5. **Estado:** HI=0x00000003, $fp=0x801FFF00, SR=0x00000404.

**Hipotese provavel:** Handler de interrupcao/evento reutiliza $sp do contexto interrompido ($sp+0x2C) como contador, sobrescrevendo $ra salvo.

## Placar antes → depois

**690** testes (inalterado — apenas ROADMAP e doc).

## Decisões e notas

1. Correcao NAO implementada. Item 4.4f permanece aberto.
2. Commit 8c456aa (espelhamento BIOS) mergeado em main durante a sessao. Nao resolve a corrupcao.
3. Proximo passo: instrumentar handler de interrupcao para verificar troca de pilha.
4. Item 10.39 registrado: fetch desalinhado deve levantar AdEL (excode 4).
