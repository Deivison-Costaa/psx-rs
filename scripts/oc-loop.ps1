# Roda N iteracoes do trabalhador em sequencia. Sem -AutoMerge, N e forcado a 1: a iteracao
# seguinte precisa da anterior mergeada (STATUS/ROADMAP atualizados na main), e o merge exige
# a revisao adversarial do orquestrador. -AutoMerge espera checks verdes e mergeia SEM
# revisao previa - use apenas com o orquestrador revisando a posteriori, e comece com N=1:
# um loop desatendido com protocolo ruim produz N PRs ruins em vez de 1 (licao do gb-rs).
#
# Regime vigente (decisao do usuario, 2026-07-27): revisao POR PR - rodar SEM -AutoMerge.
#
# Antes de entregar o PR ao revisor, o loop aplica na branch as remediacoes DETERMINISTICAS
# dos desvios que o trabalhador repetiu (iters 0009, 0010, 0011): checkbox do ROADMAP nao
# marcado, metricas pendentes nao incorporadas, e formatacao/lint reprovados.
#
# A terceira remediacao NAO cobre "CI com toolchain mais novo que o local" - `clippy --fix`
# so enxerga o que o clippy local enxerga, e foi assim que a 0016 abriu PR vermelha. Essa
# classe de falha deixou de existir na 0017c, com o toolchain pinado em rust-toolchain.toml.
#
# O que e semantico (escopo do handoff, correcao do hardware) continua sendo trabalho do
# revisor - e a 0017 mostrou que e ali que mora o defeito caro.
param(
    [int]$N = 1,
    [switch]$AutoMerge,
    [string]$Model = "opencode-go/gpt-5.6-luna",
    # Esforco de raciocinio repassado a TODA rodada do encadeamento. Sem o repasse, as rodadas
    # caem no default do oc-iter e a corrida inteira roda num esforco que ninguem escolheu —
    # a mesma classe do "default que nunca foi uma decisao" das iteracoes 0009..0017.
    [string]$Variant = "max",
    # Instrucao permanente passada a TODA iteracao do encadeamento. Sem ela cada rodada usa
    # a "Proxima tarefa" do STATUS.md e nada olha para tras; com ela, a rodada N comeca
    # revisando o PR da rodada N-1 e consertando o que achar. Substitui a revisao externa.
    [string]$TaskFile = "",
    # Nao para o encadeamento quando UMA rodada falha. Uma rodada ruim vira insumo da
    # seguinte, que tem instrucao de achar e consertar o defeito da anterior.
    [switch]$ContinuarAposFalha
)
$ErrorActionPreference = "Stop"

if (-not $AutoMerge -and $N -gt 1) {
    Write-Warning "Sem -AutoMerge o loop nao consegue encadear iteracoes; usando N=1."
    $N = 1
}

# Espera o PR ficar mergeavel, perguntando ao GitHub em vez de deduzir dos runs. Duas
# tentativas anteriores erraram: `gh pr checks` responde com os runs do commit ANTERIOR logo
# apos um push (merge da 0013 recusado com BLOCKED), e olhar check-runs por SHA tambem falha
# porque runs do mesmo SHA aparecem em ondas - na 0014 um segundo run de `check` registrou
# depois que o primeiro ja tinha terminado, e a janela entre eles pareceu verde.
# mergeStateStatus e a resposta autoritativa: CLEAN = a protecao de branch libera o merge.
function Wait-Checks([string]$pr) {
    foreach ($t in 1..40) {
        $sha = (gh pr view $pr --json headRefOid --jq .headRefOid 2>&1).Trim()
        if ($LASTEXITCODE -ne 0) { Start-Sleep 15; continue }

        $runs = gh api "repos/{owner}/{repo}/commits/$sha/check-runs" `
            --jq '[.check_runs[] | .conclusion // "PENDENTE"] | join(",")' 2>&1
        if ($LASTEXITCODE -eq 0) {
            if ($runs -match "PENDENTE" -or [string]::IsNullOrWhiteSpace($runs)) {
                Start-Sleep 15
                continue
            }
            if ($runs -match "failure|timed_out|cancelled") { return "falha" }
        }

        $st = (gh pr view $pr --json mergeStateStatus --jq .mergeStateStatus 2>&1)
        if ($LASTEXITCODE -eq 0) {
            switch -Regex ($st) {
                'CLEAN|UNSTABLE' { return "verde" }
                'DIRTY|BEHIND'   { return "falha" }
            }
        }
        Start-Sleep 15
    }
    return "timeout"
}

foreach ($i in 1..$N) {
    $temAberto = (Select-String -Path ROADMAP.md -Pattern '^\s*-\s\[ \]' -Quiet) -or
                 (Select-String -Path docs/achados.md -Pattern '^\s*-\s\[ \]' -Quiet)
    if (-not $temAberto) {
        Write-Host "[oc-loop] ROADMAP e achados sem itens abertos - fim."
        break
    }
    Write-Host "[oc-loop] iteracao $i de $N"
    if ($TaskFile) {
        pwsh -NoProfile -File scripts/oc-iter.ps1 -Model $Model -Variant $Variant -TaskFile $TaskFile
    } else {
        pwsh -NoProfile -File scripts/oc-iter.ps1 -Model $Model -Variant $Variant
    }
    if ($LASTEXITCODE -ne 0) {
        if ($ContinuarAposFalha) {
            Write-Host "[oc-loop] iteracao $i falhou - seguindo (a proxima recebe o estado como insumo)."
            git reset --hard HEAD 2>&1 | Out-Null
            git checkout main 2>&1 | Out-Null
            # `git reset --hard` nao remove arquivo NAO rastreado, e oc-iter.ps1 recusa arvore
            # suja contando os `??` do porcelain. Sem esta parada, uma rodada que morre deixando
            # um .mut sem commit faz TODAS as seguintes falharem em segundos na mesma checagem:
            # na noite2 foram 19 rodadas identicas queimadas assim. Nao usar `git clean` aqui —
            # apagar nao rastreado sem olhar ja destruiu quatro commits na iteracao 0038.
            $sujo = (git status --porcelain) -join "`n"
            if ($sujo) {
                Write-Host "[oc-loop] arvore AINDA suja depois do reset - parando em vez de repetir a mesma falha 19x."
                Write-Host "[oc-loop] resolva a mao (commite numa branch ou mova para fora) e relance:"
                Write-Host $sujo
                break
            }
            continue
        }
        Write-Host "[oc-loop] iteracao falhou - parando o loop."
        break
    }

    $pr = (gh pr list --state open --limit 1 --json number --jq '.[0].number')
    if (-not $pr) { Write-Host "[oc-loop] nenhum PR aberto - parando."; break }
    $info = gh pr view $pr --json headRefName,title | ConvertFrom-Json
    git checkout $info.headRefName | Out-Null
    git pull --ff-only | Out-Null

    $item = if ($info.title -match '\(ROADMAP (\d+\.\d+[a-z]?)\)') { $Matches[1] } else { "" }
    $iterN = if ($info.headRefName -match 'iter/(\d{4}[a-z]?)') { $Matches[1] } else { "" }

    # Remediacao 1: checkbox do item no ROADMAP (esquecido nas iters 0009, 0010 e 0011).
    # CUIDADO (achado da revisao da 0012): isso confia no titulo do PR, nao na entrega - a
    # 1.3 foi marcada sem os shifts. Conferir completude do item e do revisor, sempre.
    if ($item) {
        # O item pode morar na escada (ROADMAP.md) ou nos achados (docs/achados.md) desde a
        # iteracao 0181. Procurar so no primeiro deixava a remediacao silenciosamente inerte
        # para todo defeito achado por medicao, que e a maioria das rodadas.
        $alvo = "^- \[ \] " + [regex]::Escape($item) + " "
        foreach ($arq in @('ROADMAP.md', 'docs/achados.md')) {
            $linhas = Get-Content $arq
            $novas = $linhas | ForEach-Object {
                if ($_ -match $alvo) {
                    ($_ -replace '^- \[ \]', '- [x]') + $(if ($iterN) { " (iter $iterN)" } else { "" })
                } else { $_ }
            }
            if (Compare-Object $linhas $novas) {
                Set-Content $arq $novas
                git add $arq
                git commit -m "docs(roadmap): marca $item (iter $iterN) - auto-remediacao do loop" | Out-Null
                Write-Host "[oc-loop] checkbox $item marcado em $arq (trabalhador esqueceu)."
            }
        }
    }

    # Remediacao 2: metricas pendentes nao incorporadas (passo 8 do SKILL; a guarda
    # metrics_freshness.rs reprova a CI quando isso acontece - iter 0011).
    if (Test-Path logs/metrics-pending.csv) {
        Get-Content logs/metrics-pending.csv | Add-Content docs/metricas.csv
        Remove-Item logs/metrics-pending.csv
        git add docs/metricas.csv
        git commit -m "docs(metricas): incorpora linhas pendentes - auto-remediacao do loop" | Out-Null
        Write-Host "[oc-loop] metricas pendentes incorporadas ao docs/metricas.csv."
    }
    if ((git status --porcelain | Measure-Object).Count -eq 0) { git push | Out-Null }

    $estado = Wait-Checks $pr

    # Remediacao 3: check vermelho por fmt/lint (a CI usa toolchain mais nova que a local:
    # unnecessary_sort_by reprovou a iter 0010 depois de passar na maquina do trabalhador).
    if ($estado -eq "falha") {
        Write-Host "[oc-loop] check vermelho - tentando fmt + clippy --fix na branch."
        cargo fmt --all
        cargo clippy --fix --allow-dirty --allow-staged --all-targets 2>&1 | Out-Null
        cargo fmt --all
        if ((git status --porcelain | Measure-Object).Count -ne 0) {
            cargo clippy --all-targets -- -D warnings; $lintOk = ($LASTEXITCODE -eq 0)
            cargo test --all; $testOk = ($LASTEXITCODE -eq 0)
            if ($lintOk -and $testOk) {
                git add -A
                git commit -m "fix(lint): fmt + clippy --fix - auto-remediacao do loop" | Out-Null
                git push | Out-Null
                $estado = Wait-Checks $pr
            } else {
                git checkout -- .
                Write-Host "[oc-loop] auto-remediacao nao ficou verde local - descartada; falha e do revisor."
            }
        } else {
            Write-Host "[oc-loop] fmt/clippy sem mudanca - a falha nao e mecanica."
        }
    }

    git checkout main | Out-Null
    if ($estado -ne "verde") {
        Write-Host "[oc-loop] PR #$pr com checks $estado - parando; decisao sobe para o orquestrador."
        break
    }

    if ($AutoMerge) {
        gh pr merge $pr --merge
        $merged = gh pr view $pr --json state --jq .state 2>&1
        if ($merged -eq "MERGED") {
            git pull --ff-only | Out-Null
            Write-Host "[oc-loop] PR #$pr mergeado SEM revisao previa - revisar a posteriori."
        } else {
            Write-Host "[oc-loop] PR #$pr falhou ao mergear (estado: $merged) - parando."
            break
        }
    } else {
        Write-Host "[oc-loop] PR #$pr verde, aguardando REVISAO ADVERSARIAL (docs/prompts/review.md). NAO mergeado."
    }
}
