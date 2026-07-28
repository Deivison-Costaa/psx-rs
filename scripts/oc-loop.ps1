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
    [string]$Model = "deepseek/deepseek-v4-pro"
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
        $st = (gh pr view $pr --json mergeStateStatus --jq .mergeStateStatus 2>&1)
        if ($LASTEXITCODE -eq 0) {
            switch -Regex ($st) {
                'CLEAN|UNSTABLE' { return "verde" }
                'DIRTY|BEHIND'   { return "falha" }
            }
            $runs = gh api "repos/{owner}/{repo}/commits/$((gh pr view $pr --json headRefOid --jq .headRefOid).Trim())/check-runs" `
                --jq '[.check_runs[] | .conclusion // "-"] | join(" ")' 2>&1
            if ($LASTEXITCODE -eq 0 -and $runs -match "failure|timed_out|cancelled") { return "falha" }
        }
        Start-Sleep 15
    }
    return "timeout"
}

foreach ($i in 1..$N) {
    if (-not (Select-String -Path ROADMAP.md -Pattern '^\s*-\s\[ \]' -Quiet)) {
        Write-Host "[oc-loop] ROADMAP sem itens abertos - fim."
        break
    }
    Write-Host "[oc-loop] iteracao $i de $N"
    pwsh -NoProfile -File scripts/oc-iter.ps1 -Model $Model
    if ($LASTEXITCODE -ne 0) {
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
        $alvo = "^- \[ \] " + [regex]::Escape($item) + " "
        $linhas = Get-Content ROADMAP.md
        $novas = $linhas | ForEach-Object {
            if ($_ -match $alvo) {
                ($_ -replace '^- \[ \]', '- [x]') + $(if ($iterN) { " (iter $iterN)" } else { "" })
            } else { $_ }
        }
        if (Compare-Object $linhas $novas) {
            Set-Content ROADMAP.md $novas
            git add ROADMAP.md
            git commit -m "docs(roadmap): marca $item (iter $iterN) - auto-remediacao do loop" | Out-Null
            Write-Host "[oc-loop] checkbox $item marcado na branch (trabalhador esqueceu)."
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
        git pull --ff-only | Out-Null
        Write-Host "[oc-loop] PR #$pr mergeado SEM revisao previa - revisar a posteriori."
    } else {
        Write-Host "[oc-loop] PR #$pr verde, aguardando REVISAO ADVERSARIAL (docs/prompts/review.md). NAO mergeado."
    }
}
