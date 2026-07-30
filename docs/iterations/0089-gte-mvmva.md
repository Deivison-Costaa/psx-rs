# 0089 — gte-mvmva

- **Data:** 2026-07-30
- **Item do roadmap:** 5.4a
- **Objetivo:** Implementar comando GTE MVMVA com seleção de matriz (mx), vetor (v) e translação (cv).

## Revisão do PR anterior

PR #103 (iter 0088): **um achado**.

**Achado G1 — FLAG não é totalmente zerado no início do comando.** A máscara `&= 0x7FFF_F000` em `execute_command` preservava bits 31-12, mas `docs/reference/07-gte.md` L373-375 diz que *todos* os bits são resetados. Corrigido para `= 0`. Consertado nesta rodada.

Nove padrões conferidos:
1. Teste que não mede — 11 testes com valores golden da spec; sem round-trip ou assert_ne como única asserção
2. Parâmetro não consumido — GTE não tem FIFO de parâmetro como GPU; comandos leem registradores fixos
3. Regra de borda trocada — N/A (GTE)
4. Campo de bit lido errado — cmd, sf, lm extraídos corretamente; registradores mapeados via spec
5. Panic ou laço ilimitado — sem unwrap/expect/unsafe
6. Citação de spec — `confere-citacoes.ps1` verde (verificado nesta rodada)
7. Escopo transbordado — 5 comandos conforme item 5.3; sem funcionalidade extra
8. Portão — manifesto reparado (m4 `ocorrencias: 3→4`); `.resultado` rastreado
9. Manifesto arquivado — sem arquivamentos (apenas reparos de âncora)

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| psx-spx | GTE Command Encoding (L117) | `docs/reference/07-gte.md` |
| psx-spx | MVMVA Multiply matrix/vector/translation (L541) | `docs/reference/07-gte.md` |
| psx-spx | cop2r63 - FLAG - Returns any calculation errors (L336) | `docs/reference/07-gte.md` |
| psx-spx | Matrix Registers (L177) | `docs/reference/07-gte.md` |

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | aritmética | Cálculo MVMVA sem os 12 bits de fração: IR1 = TRX + RT11*VX | Matriz em 1.3.12: TRX é multiplicado por 0x1000h e o resultado é SAR(sf*12) | Testes sf=1 davam valores 1024× maiores que esperado |
| 2 | sign-extend | m33 lido como i32 direto para LLM e LCM (else branch) | L33 e LB3 são standalone S16: precisam de `as i16 as i32` como RT33 | Teste LLM deu IR errado (2×) antes da correção |
| 3 | âncora envelhecida | Manifesto 0088 m4 com 3 ocorrências de `let shift = sf * 12;` | MVMVA adicionou a 4ª ocorrência | `mutation_anchors.rs` reprovou; reparado com `ocorrencias: 4` e bateria re-executada |

## Bateria de mutação

Placar da bateria: 7/7 mutantes mortos, 2/2 controles verdes, 0 equivalente — docs/mutantes/0089-gte-mvmva.mut

| # | Tipo | Rótulo | Resultado |
|---|---|---|---|
| m1 | mutante | MVMVA ignora mx (sempre usa RT) | MORREU |
| m2 | mutante | MVMVA ignora v (sempre usa V0) | MORREU |
| m3 | mutante | MVMVA omite multiplicação de TR por 0x1000h | MORREU |
| m4 | mutante | MVMVA MAC1 inverte sinal do termo m11*vx | MORREU |
| m5 | mutante | MVMVA ignora lm (força lm=0) | MORREU |
| m6 | mutante | MVMVA raw2 usa tx em vez de ty | MORREU |
| m7 | mutante | MVMVA m33 sem sign-extend | MORREU |
| c1 | controle | renomeia tx → translacao_x | verde |
| c2 | controle | adiciona `let _ = 0` no inicio de mvmva | verde |

m7 sobreviveu na primeira execução (nenhum teste usava m33 negativo); teste `mvmva_lcm_m33_negativo_sign_extend` adicionado e bateria re-executada — m7 morreu.

## Placar antes → depois

Workspace: **662** → **669** testes (+7: gte_mvmva).

## Decisões e notas

1. **Item 5.4 dividido.** R4: uma micro-funcionalidade por iteração. MVMVA (5.4a) implementado nesta rodada; comandos de iluminação viram 5.4b (NCS/NCT/NCCS/NCCT), 5.4c (NCDS/NCDT/CC/CDP) e 5.4d (DCPL/DPCS/DPCT/INTPL).

2. **MVMVA mx=3 (garbage matrix) implementado.** Usa `-R*10h`, `+R*10h`, IR0, RT13, RT22 com a fórmula documentada em `docs/reference/07-gte.md` L561-564. Nenhum teste o exerce por falta de golden values.

3. **MVMVA cv=2 (FC/Bugged) implementado.** Aplica a redução documentada: MAC1=(Mx12*Vx2+Mx13*Vx3) sem translação. Nenhum teste o exerce.

4. **m33 de LLM e LCM usa `as i16 as i32`.** Corrigido durante desenvolvimento — todas as três matrizes (RT, LLM, LCM) têm o quinto registrador como standalone S16.
