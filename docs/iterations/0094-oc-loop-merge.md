# 0094 — oc-loop-merge

- **Data:** 2026-07-30
- **Item do roadmap:** 10.37
- **Objetivo:** o `oc-loop.ps1` deixar de anunciar merge que não aconteceu — esperar a conclusão dos
  checks do commit atual antes de consultar `mergeStateStatus`, e verificar o estado do PR depois do
  `gh pr merge`.

## Revisão do PR anterior

Revisão do PR #108 (iter 0093): sem achados novos. Os nove padrões conferidos, com o que foi olhado
em cada um: teste que não mede (o handler instalado é observável por três testes novos e pelas
suítes — `gpu/lines` saiu de 0 para 522 091 pixels na VRAM, medido pelo orquestrador); parâmetro não
consumido (o handler não recebe parâmetro); regra de borda (sem rasterização); campo de bit (o
`SR=0x1001` foi conferido bit a bit: IEc no 0 e IM[2] no 12); panic ou laço ilimitado (o handler faz
`RFE` e retorna); citação de spec (`cop0r13 - CAUSE`, conferida); escopo transbordado (nenhum);
portão que não mede (a nota 4 do doc é o oposto disso — foi ler o fonte do teste antes de concluir);
manifesto arquivado (nenhum).

## Spec consultada

Nenhuma seção de spec de hardware. O item é ferramenta de orquestração, e o comportamento
autoritativo é o da API do GitHub, observado ao vivo: `GraphQL: 2 of 2 required status checks are
expected. (mergePullRequest)`, em `logs/loop-noite8.err.log`.

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que o comportamento real é | Como foi pego |
|---|---|---|---|---|
| 1 | processo | Que `mergeStateStatus` fosse resposta autoritativa suficiente, por não sofrer da defasagem que o comentário do próprio script atribui a `gh pr checks` | Ele também responde o estado do commit ANTERIOR logo após um push, e a remediação 2 do loop empurra um commit de métricas em TODA rodada | Os PRs #106 e #107 anunciados como mergeados no log, ambos ABERTOS. Medido pelo orquestrador, que mergeou os dois à mão |
| 2 | Rust/teste | Que `body.find("check-runs")` provasse que o caminho da API está certo | `check-runs-old` **contém** `check-runs`, então o mutante que renomeia o caminho sobrevive. É o mesmo defeito de substring do item 10.16, agora no meu próprio teste | Bateria em 2/5. Corrigido asserindo `commits/$sha/check-runs"` com a aspa de fechamento |
| 3 | Rust/teste | Que afirmar a presença da palavra `PENDENTE` cobrisse a guarda que faz o loop esperar | A string continua no `jq` (`.conclusion // "PENDENTE"`) mesmo quando a comparação `-match "PENDENTE"` é removida — o mutante sobreviveu com a palavra ainda no arquivo | Bateria em 4/5 depois do primeiro fortalecimento. Corrigido asserindo a comparação, não a string |

## Bateria de mutação

Placar da bateria: 5/5 mutantes mortos, 2/2 controles verdes, 0 equivalente — docs/mutantes/0094-oc-loop-merge.mut

| Registro | Rótulo | Testes que pegaram, conforme o `.resultado` |
|---|---|---|
| m1 | `check-runs` renomeado para `check-runs-old` | `wait_checks_consulta_check_runs_antes_de_mergestatestatus` |
| m2 | guarda do `PENDENTE` nunca casa, o loop segue com checks rodando | `wait_checks_espera_enquanto_algum_check_esta_pendente` |
| m3 | `gh pr merge` sem verificação de `MERGED` | `gh_pr_merge_verifica_estado_merged_apos_o_merge` |
| m4 | `MERGED` substituído por `OK` | `gh_pr_merge_verifica_estado_merged_apos_o_merge` |
| m5 | bloco de verificação removido (`gh pr view` ausente) | `gh_pr_merge_verifica_estado_merged_apos_o_merge` |
| c1 | comentário acrescentado antes do laço | sobreviveu |
| c2 | `Start-Sleep 15` para 20 dentro da guarda | sobreviveu |

As atribuições foram lidas do `.resultado` gerado pela máquina, não preenchidas por inspeção.

## Placar antes → depois

Workspace: **685** → **688** testes (+3: `ci_oc_loop`).

Não há suíte de hardware envolvida. O efeito medível é operacional: antes, dois PRs consecutivos
(#106 e #107) foram anunciados como mergeados estando abertos, e o loop seguiu para a rodada
seguinte partindo de uma `main` sem o item anterior.

## Revisão cruzada (orquestrador)

Iteração **começada pelo trabalhador e terminada pelo orquestrador**, e isso é dado de processo. A
rodada 4 da noite8 escreveu o teste e o conserto — na ordem certa, teste primeiro — e morreu deixando
o manifesto sem commit. A guarda de árvore suja do próprio `oc-loop.ps1` funcionou: imprimiu o
arquivo e parou, em vez de queimar as rodadas seguintes na mesma falha.

O que o trabalhador acertou sozinho: o diagnóstico dos dois defeitos, a inversão da ordem em
`Wait-Checks` com `PENDENTE` como sentinela, e a verificação por releitura do estado com `break`.

O que o orquestrador fez: reparou três âncoras do manifesto que não casavam com o script (m1 e c2 por
indentação, m2 por recorte), trocou o m2 por um mutante de uma linha que reintroduz o defeito real,
rodou a bateria, viu **2/5**, e fortaleceu o teste em duas rodadas até 5/5.

O 2/5 é o dado que importa: o conserto estava certo e o teste que o acompanhava só pegava dois dos
cinco jeitos plausíveis de quebrá-lo. Sem a bateria, entraria verde.

## Decisões e notas

1. **O alvo do manifesto é `scripts/oc-loop.ps1`, não um arquivo sob `crates/*/src/`.** A regra
   existe porque mutar o teste é a versão trapaceável do exercício, mas aqui não há `src/` a mutar:
   o artefato é um script de orquestração. Mesma tensão estrutural já registrada quando o alvo foi
   um arquivo de suporte de teste, e ela merece decisão própria do projeto.
2. **O mutante m2 original mutava um bloco multi-linha com linha em branco no meio e a âncora não
   casava**, mesmo extraída do arquivo. Em vez de reverter o parser, troquei por um mutante de uma
   linha — que além de casar é semanticamente melhor: desliga a guarda e reintroduz exatamente o
   defeito que o item conserta. Âncora curta é mais robusta a refactor.
3. **O loop precisava ser relançado com o script já consertado.** Enquanto ele roda, o `pwsh` já tem
   o script antigo em memória: consertar sem relançar não muda nada. Por isso esta iteração foi
   fechada antes do relançamento, com o loop parado.
