# Roda N iteracoes do trabalhador em sequencia. Sem -AutoMerge, N e forcado a 1: a iteracao
# seguinte precisa da anterior mergeada (STATUS/ROADMAP atualizados na main), e o merge exige
# a revisao adversarial do orquestrador. -AutoMerge espera checks verdes e mergeia SEM
# revisao previa - use apenas com o orquestrador revisando a posteriori, e comece com N=1:
# um loop desatendido com protocolo ruim produz N PRs ruins em vez de 1 (licao do gb-rs).
param(
    [int]$N = 1,
    [switch]$AutoMerge,
    [string]$Model = "deepseek/deepseek-chat"
)
$ErrorActionPreference = "Stop"

if (-not $AutoMerge -and $N -gt 1) {
    Write-Warning "Sem -AutoMerge o loop nao consegue encadear iteracoes; usando N=1."
    $N = 1
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
    if ($AutoMerge) {
        $pr = (gh pr list --state open --limit 1 --json number --jq '.[0].number')
        if (-not $pr) { Write-Host "[oc-loop] nenhum PR aberto - parando."; break }
        $ok = $false
        foreach ($t in 1..40) {
            $checks = gh pr checks $pr 2>&1
            if ($LASTEXITCODE -eq 0 -and $checks -notmatch "pending") { $ok = $true; break }
            if ($checks -match "fail") { break }
            Start-Sleep 15
        }
        if (-not $ok) { Write-Host "[oc-loop] checks nao verdes no PR #$pr - parando."; break }
        gh pr merge $pr --merge
        git checkout main | Out-Null
        git pull --ff-only | Out-Null
        Write-Host "[oc-loop] PR #$pr mergeado SEM revisao previa - revisar a posteriori."
    }
}
