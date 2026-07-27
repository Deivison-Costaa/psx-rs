# Dispara UMA iteração do trabalhador (opencode/DeepSeek) e appenda a métrica em
# docs/metricas.csv. Usa `opencode serve` + `--attach` (bug de sessão headless do
# `opencode run` direto no Windows, issue opencode#28407).
# O trabalhador abre o PR e PARA; revisão e merge são do orquestrador (SKILL passo 9).
param(
    [string]$Model = "deepseek/deepseek-chat",
    [string]$TaskOverride = "",
    [int]$TimeoutMin = 45,
    [int]$Port = 4096
)
$ErrorActionPreference = "Stop"

if ((git status --porcelain | Measure-Object).Count -ne 0) {
    Write-Error "Arvore suja - commit ou descarte antes de iterar."
}
git checkout main | Out-Null
git pull --ff-only | Out-Null
$headAntes = (git rev-parse --short HEAD).Trim()

$up = Test-Connection -TargetName localhost -TcpPort $Port -Quiet -TimeoutSeconds 2
if (-not $up) {
    Start-Process opencode -ArgumentList "serve", "--port", $Port -WindowStyle Hidden
    foreach ($i in 1..30) {
        if (Test-Connection -TargetName localhost -TcpPort $Port -Quiet -TimeoutSeconds 2) { break }
        Start-Sleep 1
    }
}

$task = if ($TaskOverride) { $TaskOverride } else { "a secao 'Proxima tarefa' do STATUS.md" }
$prompt = "Voce e o trabalhador do projeto psx-rs. Execute EXATAMENTE UMA iteracao seguindo " +
    "o protocolo em .claude/skills/iterate/SKILL.md (leia-o primeiro, inteiro). Tarefa: $task. " +
    "Ao abrir o PR, PARE - nao faca merge, nao comece outro item."

New-Item -ItemType Directory -Force logs | Out-Null
$ts = Get-Date -Format "yyyy-MM-ddTHH:mm:sszzz"
$stamp = Get-Date -Format "yyyyMMdd-HHmmss"
$outFile = "logs/oc-iter-$stamp.json"
$errFile = "logs/oc-iter-$stamp.err"

$sw = [System.Diagnostics.Stopwatch]::StartNew()
$p = Start-Process opencode -ArgumentList "run", "--attach", "http://localhost:$Port",
    "-m", $Model, "--format", "json", "`"$prompt`"" -NoNewWindow -PassThru `
    -RedirectStandardOutput $outFile -RedirectStandardError $errFile
$done = $p.WaitForExit($TimeoutMin * 60 * 1000)
if (-not $done) {
    Stop-Process -Id $p.Id -Force -Confirm:$false
    $resultado = "falha:timeout"
} elseif ($p.ExitCode -ne 0) {
    $resultado = "falha:exit-$($p.ExitCode)"
} else {
    $resultado = "ok"
}
$sw.Stop()

# Parser calibrado no smoke test (iter 0008b) sobre o JSON real do opencode 1.18.3:
# ultimo objeto "tokens" acumulado e ultimo "cost"; steps = eventos step_finish.
$raw = if (Test-Path $outFile) { Get-Content $outFile -Raw } else { "" }
$steps = ([regex]::Matches($raw, '"type"\s*:\s*"step[._-]?finish"')).Count
$cost = [regex]::Matches($raw, '"cost"\s*:\s*([0-9.eE+-]+)') | Select-Object -Last 1
$tin = [regex]::Matches($raw, '"input"\s*:\s*(\d+)') | Select-Object -Last 1
$tout = [regex]::Matches($raw, '"output"\s*:\s*(\d+)') | Select-Object -Last 1
$costV = if ($cost) { $cost.Groups[1].Value } else { "" }
$tinV = if ($tin) { $tin.Groups[1].Value } else { "" }
$toutV = if ($tout) { $tout.Groups[1].Value } else { "" }

$iter = Get-ChildItem docs/iterations -Filter "*.md" |
    Where-Object { $_.Name -match '^(\d{4})' } |
    ForEach-Object { $Matches[1] } | Sort-Object | Select-Object -Last 1
$headDepois = (git rev-parse --short HEAD).Trim()

Add-Content docs/metricas.csv `
    "$ts,$iter,$resultado,$costV,$tinV,$toutV,$steps,$($sw.ElapsedMilliseconds),$headAntes,$headDepois,$Model,trabalhador"
Write-Host "[oc-iter] $resultado iter=$iter custo=$costV tokens=$tinV/$toutV steps=$steps $([int]($sw.Elapsed.TotalMinutes))min -> $outFile"
Write-Host "[oc-iter] proximo passo do ORQUESTRADOR: revisar o PR (docs/prompts/review.md), commitar a linha de metricas na branch do PR e mergear."
if ($resultado -ne "ok") { exit 1 }
