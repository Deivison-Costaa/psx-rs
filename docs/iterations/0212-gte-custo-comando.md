<!-- Custo, tokens e duração NÃO entram aqui: são medidos por scripts/oc-iter.ps1 em
     docs/metricas.csv, casados por head_antes/head_depois. -->

# 0212 — gte-custo-comando

- **Data:** 2026-08-06
- **Item do roadmap:** 0212.1 — Degrau 4 da escada de timing de CPU/barramento
- **Objetivo:** `Gte::command_cycles(func)` — tabela pura de custo por comando GTE, sem
  chamador ainda (o Degrau 5 liga isso na CPU).

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| psx-spx | § cabeçalhos de comando COP2 (L481-642, um por comando) | docs/reference/07-gte.md |

## Erros de primeira tentativa

Nenhum — os 22 valores foram lidos direto dos cabeçalhos de seção (`grep -n "^#### COP2"`),
não copiados do braço de dispatch de `execute_command`, exatamente para evitar a armadilha
de CC/CDP (mesmo braço Rust `color_color`, custos diferentes na spec: 11 e 13).

## Bateria de mutação

Placar da bateria: 6/6 mutantes mortos, 2/2 controles verdes, 0 equivalente —
docs/mutantes/0212-gte-custo-comando.mut

m1/m2/m4 trocam o custo de um comando pelo de outro (RTPT↔RTPS, NCDT↔NCCT, AVSZ4↔AVSZ3), m3
é a armadilha CC/CDP, m5 quebra a máscara do opcode (perde o bit 5), m6 muda o custo do
catch-all de comando não documentado — todos mortos pelos 22 testes de valor exato.

## Placar antes → depois

Workspace: **1289 → 1313** testes (24 novos em `gte_custo_comando.rs`).

## Revisão cruzada (orquestrador)

Rodada de autoria do orquestrador (mesma exceção de executor da 0193/0209-0211). Os 22
testes de comando falhavam por erro de compilação antes do fix (`command_cycles` não
existia) e passam depois; `cpu_load_timing.rs` (invariante 17) e `gte_registers.rs`
reexecutados sem mudança — a função é pura e não tem chamador ainda, então não pode ter
afetado nada que já passava.

## Decisões e notas

LZCS/LZCR (`07-gte.md` L586, "? Cycles") não entram na tabela: são registradores de dados
(cop2r30/31), escritos/lidos via MTC2/MFC2 comuns, não comandos COP2 com opcode próprio — não
aparecem no `match` de `Gte::execute_command` e não têm custo de comando pra tabelar.

Degrau 5 (próximo): o stall do GTE ligado na CPU (`07-gte.md` L112-115 — ler registrador GTE
ou emitir novo comando antes do anterior terminar trava a CPU). Reusa o mesmo modelo
`busy_until = emissão + 1 + custo` do Degrau 3 (mult/div).
