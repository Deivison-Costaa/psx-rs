<!-- Custo, tokens e duração NÃO entram aqui: são medidos por scripts/oc-iter.ps1 em
     docs/metricas.csv, casados por head_antes/head_depois. -->

# 0134 — fila-respostas

- **Data:** 2026-08-01
- **Item do roadmap:** 4.4ac
- **Objetivo:** 2ª resposta do CD-ROM enfileirada e entregue só depois do ack da 1ª, com
  atraso físico — fechar o retry infinito de Init medido na 0133.

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| psx-spx | § HINTSTS/fila de INTs, L333-337: a 2ª INT de resposta NÃO é entregue até a 1ª ser reconhecida | docs/reference/06-cdrom.md |
| psx-spx | § Second Response, L2066: tempos médios por comando (GetID ≈ 4A00h cycles) | docs/reference/06-cdrom.md |

## Mecanismo

O ack via HCLRCTL agora só **marca** `second_request`; o bus consome a marca
(`take_second_request`) e agenda o evento `CDROM_SECOND` para
`total_cycles + 0x4A00`; a entrega (`deliver_second_now`) acontece no scheduler e o
IRQ2 sobe na entrega — nunca no instante do ack. Sem ack, a 2ª resposta fica
enfileirada para sempre (teste `segunda_nao_atropela_a_primeira_sem_ack`).

**Dívida explícita (10.53 / item 4.4ad):** `0x4A00` é a média do *GetID* aplicada a TODO
comando. Init (spin-up, pode passar de 1s), Pause (~116×), Stop (~730×) e ReadTOC (~1786×)
têm tempos próprios na tabela da spec (06-cdrom.md L2066) — vira tabela por comando no motor de
respostas da Fase B do plano de saída.

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | oraculo-teste | a rodada anterior migrou as suítes para a nova semântica | 4 suítes esquecidas (regs, seek-pause, setor-mode2, dpcr-gate) ainda consagravam a entrega no instante do ack — 12 testes vermelhos | `cargo test --workspace` |
| 2 | timing | uma constante de atraso serve para todo comando | 06-cdrom.md L2066: cada comando tem tempo próprio; 4A00h é só o GetID | auditoria multi-agente do plano de saída (defeito nº 1 do ranking); registrada como dívida no 4.4ad |

## Bateria de mutação

Placar da bateria: 5/5 mutantes mortos, 2/2 controles verdes, 0 equivalente — docs/mutantes/0134-fila-respostas.mut

m1 ack não enfileira; m2 o defeito original de volta (entrega inline no ack); m3 bus nunca
agenda; m4 INT3 no lugar de INT2; m5 resposta sem stat. Todos mortos por `cdrom_fila_int`.
Manifestos 0065/0080 arquivados (âncoras envelhecidas pela reestruturação do handler do ack
e do bloco de eventos do bus).

## Placar antes → depois

847 → 850 (3 testes novos em `cdrom_fila_int`, commit 8e89475).

**Critério de sistema (boot 400M, BIOS+disco):** flag RAM 0x91C4 = **1** (era 0 travado na
0133); TTY sai do retry de Init e atravessa `KERNEL SETUP!` → `BOOTSTRAP LOADER Type C
Ver 2.1` → `boot file : cdrom:PSX.EXE;1`. A tela passou da licença: 30 sub-itens depois, o
4.4 saiu do muro.

## Revisão cruzada (orquestrador)

Iteração executada pelo orquestrador (emergência do plano de saída; papéis invertidos
aprovados pelo usuário em 01/08 — goldens do orquestrador, implementação do trabalhador
volta na Fase B). Achados da auditoria multi-agente incorporados: a constante única virou
dívida nomeada (4.4ad) em vez de silenciosa; o próximo bloqueio já está identificado ANTES
de tropeçar nele — ver Decisões.

## Decisões e notas

- **Evidência de que o próximo bloqueio é o avanço de seek** (defeito nº 2 do ranking da
  auditoria): `seek_min/sec/sect` só são escritos no Setloc (`cdrom.rs`); ReadN/ReadS
  reentregam o MESMO setor para sempre. No boot 400M o loader caiu no fallback
  `PSX.EXE` (SYSTEM.CNF ilegível → parse falha) e os PCs 367M–400M ficam num laço de
  espera do kernel (0xA0/0x5C4–0x5DC/0xBFC0D950). Golden dirigido entra na Fase B (B2).
- Este fechamento faz parte do plano de saída do buraco 4.4 (diagnóstico por 2 workflows,
  18 agentes; estudo de caso completo vai para docs/relatorio.md na Fase C).
- `.gitignore` passa a cobrir `crates/psx-cli/tests/bins/` (artefato gerado por
  `espera_tela_sce.rs`, sujava o `git status` de toda sessão).
