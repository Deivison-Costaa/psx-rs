# Dispara UMA iteração do trabalhador (opencode/DeepSeek) e appenda a métrica em
# docs/metricas.csv. Usa `opencode serve` + `--attach` (bug de sessão headless do
# `opencode run` direto no Windows, issue opencode#28407).
# O trabalhador abre o PR e PARA; revisão e merge são do orquestrador (SKILL passo 9).
# Modelo padrao: v4-pro. As iteracoes 0009..0017 rodaram em deepseek-chat (geracao anterior);
# a troca foi decisao do usuario em 2026-07-27, no meio da segunda tentativa da 1.7. O par de
# comparacao barato passa a ser deepseek-v4-flash, nao mais chat/reasoner.
param(
    [string]$Model = "deepseek/deepseek-v4-pro",
    [string]$TaskOverride = "",
    [int]$TimeoutMin = 45,
    [int]$Port = 4096,
    # Rodada de CONTINUACAO de um item reprovado na revisao: fica na branch que ja existe e
    # troca o texto envoltorio, que senao manda "ao abrir o PR, PARE" — com o PR ja aberto,
    # o trabalhador le isso como "ja acabou" e devolve a rodada sem fazer nada. Aconteceu
    # duas vezes seguidas na iter 0038 (rodadas 4 e 5, US$ 0,056 e 4 min de parede).
    [string]$ContinueBranch = ""
)
$ErrorActionPreference = "Stop"

if ((git status --porcelain | Measure-Object).Count -ne 0) {
    Write-Error "Arvore suja - commit ou descarte antes de iterar."
}
if ($ContinueBranch) {
    git checkout $ContinueBranch | Out-Null
    git pull --ff-only | Out-Null
} else {
    git checkout main | Out-Null
    git pull --ff-only | Out-Null
}
$headAntes = (git rev-parse --short HEAD).Trim()

# Duas armadilhas do npm no Windows (smoke tests 0008b/1 e /2): o shim opencode.ps1 nao roda
# via Start-Process, e o shim .cmd degrada aspas — um "--version" DENTRO do prompt virou
# flag e o run imprimiu a versao e saiu com 0. Usar o .exe real, sem camada cmd.
$shim = (Get-Command opencode -ErrorAction SilentlyContinue)?.Source
$oc = if ($shim) {
    Join-Path (Split-Path $shim) "node_modules\opencode-ai\bin\opencode.exe"
} else { $null }
if (-not ($oc -and (Test-Path $oc))) { Write-Error "opencode.exe nao encontrado (npm i -g opencode-ai)" }

$up = Test-Connection -TargetName localhost -TcpPort $Port -Quiet -TimeoutSeconds 2
if (-not $up) {
    Start-Process $oc -ArgumentList "serve", "--port", $Port -WindowStyle Hidden
    foreach ($i in 1..30) {
        if (Test-Connection -TargetName localhost -TcpPort $Port -Quiet -TimeoutSeconds 2) { break }
        Start-Sleep 1
    }
}

$task = if ($TaskOverride) { $TaskOverride } else { "a secao 'Proxima tarefa' do STATUS.md" }
$prompt = if ($ContinueBranch) {
    "Voce e o trabalhador do projeto psx-rs. Esta e uma rodada de CONTINUACAO de um item que " +
    "JA foi reprovado na revisao adversarial, na branch $ContinueBranch, que JA existe e cujo " +
    "PR JA esta aberto. O PR estar aberto NAO significa que acabou: significa que voce vai " +
    "ACRESCENTAR commits a ele. Nao crie branch, nao recrie branch, nao abra PR, nao faca " +
    "merge, nao use --force. O protocolo em .claude/skills/iterate/SKILL.md vale para o " +
    "resto (passos 4 a 8); os passos 2, 9 e 11 nao se aplicam. Tarefa: $task. " +
    "Ao terminar, faca git push e PARE."
} else {
    "Voce e o trabalhador do projeto psx-rs. Execute EXATAMENTE UMA iteracao seguindo " +
    "o protocolo em .claude/skills/iterate/SKILL.md (leia-o primeiro, inteiro). Tarefa: $task. " +
    "Ao abrir o PR, PARE - nao faca merge, nao comece outro item."
}

New-Item -ItemType Directory -Force logs | Out-Null
$ts = Get-Date -Format "yyyy-MM-ddTHH:mm:sszzz"
$stamp = Get-Date -Format "yyyyMMdd-HHmmss"
$outFile = "logs/oc-iter-$stamp.json"
$errFile = "logs/oc-iter-$stamp.err"

# Start-Process -ArgumentList NAO reescapa: o prompt vira um argumento so porque esta entre
# aspas, e uma aspa DENTRO dele fecha a regiao citada. Dai qualquer token comecando com "-"
# que caia fora vira flag do CLI (o "->" de "remover o teto -> teste trava" derrubou a rodada
# de 28/07 11:52, com o opencode imprimindo o help). Escapar as aspas resolve.
$promptArg = '"' + ($prompt -replace '"', '\"') + '"'

$sw = [System.Diagnostics.Stopwatch]::StartNew()
$p = Start-Process $oc -ArgumentList "run", "--attach", "http://localhost:$Port",
    "-m", $Model, "--format", "json", $promptArg -NoNewWindow -PassThru `
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
# Exit 0 nao basta: no smoke 0008b/2 o CLI imprimiu a versao e saiu com 0 sem rodar nada.
if ($resultado -eq "ok" -and (Get-Item $outFile).Length -lt 1000) {
    $resultado = "falha:sem-execucao"
}

# Parser calibrado no smoke test 0008b sobre o JSON real do opencode 1.18.3: cada evento
# step_finish traz "cost" e "tokens"{input,output} DAQUELE step - os totais sao a SOMA.
# ("input" exclui cache read; steps conta so o evento "step_finish", nao o part "step-finish".)
# Arquivo vazio devolve $null no -Raw, e [regex]::Matches($null,...) lanca excecao ANTES do
# Add-Content la embaixo: a rodada de 28/07 11:52 falhou e nao deixou linha de metrica nenhuma.
# Justo a falha e o dado que mais interessa registrar aqui.
$raw = if (Test-Path $outFile) { Get-Content $outFile -Raw } else { "" }
if ($null -eq $raw) { $raw = "" }
$steps = ([regex]::Matches($raw, '"type":"step_finish"')).Count
$costV = ([regex]::Matches($raw, '"cost":([0-9.eE+-]+)') |
    ForEach-Object { [double]$_.Groups[1].Value } | Measure-Object -Sum).Sum
$costV = if ($costV) { [Math]::Round($costV, 4) } else { "" }
$tinV = ([regex]::Matches($raw, '"input":(\d+)') |
    ForEach-Object { [long]$_.Groups[1].Value } | Measure-Object -Sum).Sum
$toutV = ([regex]::Matches($raw, '"output":(\d+)') |
    ForEach-Object { [long]$_.Groups[1].Value } | Measure-Object -Sum).Sum

# $iter sai do ultimo doc em docs/iterations. Quando a rodada morre ANTES do passo 8 o doc
# nao existe e a linha herda o numero anterior — foi o caso do timeout da 0038, gravado como
# 0037. Continua valendo conferir o numero antes de commitar a metrica.
$iter = Get-ChildItem docs/iterations -Filter "*.md" |
    Where-Object { $_.Name -match '^(\d{4}[a-z]?)' } |
    ForEach-Object { $Matches[1] } | Sort-Object | Select-Object -Last 1
$headDepois = (git rev-parse --short HEAD).Trim()

# Vai para um pendente NAO rastreado (logs/ e gitignored): escrever direto no metricas.csv
# sujaria a arvore e conflitaria com o pull do modo auto-merge. O worker incorpora as linhas
# pendentes no commit docs(iter) da iteracao seguinte (SKILL passo 8); lag maximo 1 garantido
# por metrics_freshness.rs.
Add-Content logs/metrics-pending.csv `
    "$ts,$iter,$resultado,$costV,$tinV,$toutV,$steps,$($sw.ElapsedMilliseconds),$headAntes,$headDepois,$Model,trabalhador"
Write-Host "[oc-iter] $resultado iter=$iter custo=$costV tokens=$tinV/$toutV steps=$steps $([int]($sw.Elapsed.TotalMinutes))min -> $outFile"
Write-Host "[oc-iter] proximo passo do ORQUESTRADOR: revisar o PR (docs/prompts/review.md), commitar a linha de metricas na branch do PR e mergear."
if ($resultado -ne "ok") { exit 1 }
