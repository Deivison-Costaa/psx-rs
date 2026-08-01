# Dispara UMA iteração do trabalhador (opencode/DeepSeek) e appenda a métrica em
# docs/metricas.csv. Usa `opencode serve` + `--attach` (bug de sessão headless do
# `opencode run` direto no Windows, issue opencode#28407).
# O trabalhador abre o PR e PARA; revisão e merge são do orquestrador (SKILL passo 9).
# Modelo padrao: gpt-5.6-luna pelo gateway opencode-go, esforco max. O provider `deepseek/` das
# iteracoes 0009..0138 nao esta autenticado na maquina de operacao desde a migracao para Linux
# (0139); o par de comparacao barato passa a ser opencode-go/deepseek-v4-pro.
param(
    [string]$Model = "opencode-go/gpt-5.6-luna",
    # Esforco de raciocinio da rodada (`opencode run --variant`). Parametro do SCRIPT, e nao
    # configuracao de maquina: o handoff da 0137 afirmava "reasoningEffort max ja configurado em
    # ~/.config/opencode" e o arquivo tinha so o $schema. Declarado aqui, o esforco viaja com o
    # repo. Vazio suprime a flag.
    [string]$Variant = "max",
    [string]$TaskOverride = "",
    # Parede da rodada. Era 45 min e matava rodada VIVA: 3 das 25 rodadas de 29-30/07 morreram
    # exatas em 2 700 023 ms com o JSON ainda crescendo. A rodada boa mediana leva 28 min, e
    # rodada morta de verdade agora cai em ~90 s pelo $TravamentoMin — a parede so precisa
    # cobrir a cauda lenta.
    [int]$TimeoutMin = 75,
    # Janela sem NENHUM byte novo no JSON da rodada que caracteriza provedor mudo. Ver o doc
    # da iteracao 0098 para as duas medicoes que fixaram o valor original (5 min).
    # Subiu para 25 na 0143: o PORTAO do passo 7 (`cargo test --all`) passou a levar 842 s
    # depois que a BIOS e o disco chegaram (0139) e os testes de emulacao deixaram de pular.
    # Durante o portao o trabalhador nao emite evento — o JSON para de crescer — e 5 min lia
    # esse silencio legitimo como provedor mudo: as DUAS rodadas da 0140 morreram assim.
    # O piso esta travado por ci_oc_iter.rs (MINUTOS_DO_PORTAO); se o portao encolher, o piso cai.
    [int]$TravamentoMin = 25,
    [int]$Port = 4096,
    # Rodada de CONTINUACAO de um item reprovado na revisao: fica na branch que ja existe e
    # troca o texto envoltorio, que senao manda "ao abrir o PR, PARE" — com o PR ja aberto,
    # o trabalhador le isso como "ja acabou" e devolve a rodada sem fazer nada. Aconteceu
    # duas vezes seguidas na iter 0038 (rodadas 4 e 5, US$ 0,056 e 4 min de parede).
    [string]$ContinueBranch = "",
    # Task longa vem de ARQUIVO, nao da linha de comando: prompt grande em -TaskOverride
    # esbarra nas armadilhas de quoting do Start-Process (ver comentario do $promptArg).
    # Lido com -Raw e no caminho ABSOLUTO — arquivo escrito pelo Bash em /tmp o PowerShell
    # le como C:\tmp, e uma rodada ja morreu no lancamento com zero tokens por isso.
    [string]$TaskFile = ""
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
# flag e o run imprimiu a versao e saiu com 0. Usar o .exe real, sem camada cmd. Fora do
# Windows nao ha shim: a instalacao e nativa e o caminho do PATH JA e o binario (0139).
$shim = (Get-Command opencode -ErrorAction SilentlyContinue)?.Source
$oc = if (-not $shim) {
    $null
} elseif ($IsWindows) {
    Join-Path (Split-Path $shim) "node_modules\opencode-ai\bin\opencode.exe"
} else {
    $shim
}
if (-not ($oc -and (Test-Path $oc))) { Write-Error "binario do opencode nao encontrado (Windows: npm i -g opencode-ai; Linux: instalacao nativa no PATH)" }

$up = Test-Connection -TargetName 127.0.0.1 -TcpPort $Port -Quiet -TimeoutSeconds 2
if (-not $up) {
    # WindowStyle lanca no PowerShell do Linux ("not supported for the cmdlet 'Start-Process'
    # on this edition"); com $ErrorActionPreference = "Stop" isso mata a rodada na subida do
    # daemon, antes do primeiro token. E enfeite de janela: vai por splat, so no Windows (0139).
    $serveArgs = @{ ArgumentList = @("serve", "--port", $Port) }
    if ($IsWindows) { $serveArgs["WindowStyle"] = "Hidden" }
    Start-Process $oc @serveArgs
    foreach ($i in 1..30) {
        if (Test-Connection -TargetName 127.0.0.1 -TcpPort $Port -Quiet -TimeoutSeconds 2) { break }
        Start-Sleep 1
    }
}

if ($TaskFile) {
    if (-not (Test-Path $TaskFile)) { Write-Error "TaskFile nao encontrado: $TaskFile" }
    $TaskOverride = Get-Content $TaskFile -Raw
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
# NAO tornar isto condicional a $IsWindows: medido na 0139, o pwsh no LINUX tambem achata o
# -ArgumentList e o .NET reparseia com regras de aspas do Windows. Sem o escape o prompt chega
# partido em 5 argumentos e o "--version" de dentro dele vira flag do CLI.
$promptArg = '"' + ($prompt -replace '"', '\"') + '"'

$sw = [System.Diagnostics.Stopwatch]::StartNew()
$argsRun = @("run", "--attach", "http://localhost:$Port", "-m", $Model)
if ($Variant) { $argsRun += @("--variant", $Variant) }
$argsRun += @("--format", "json", $promptArg)
$p = Start-Process $oc -ArgumentList $argsRun -NoNewWindow -PassThru `
    -RedirectStandardOutput $outFile -RedirectStandardError $errFile
# Esperar so pela parede confunde rodada lenta com rodada travada e paga 45 min pelas duas.
# O JSON cresce a cada evento enquanto o trabalhador vive, entao tamanho parado e provedor mudo.
$parede = (Get-Date).AddMinutes($TimeoutMin)
$ultimoTamanho = -1L
$ultimoAvanco = Get-Date
$resultado = $null
while (-not $p.HasExited) {
    Start-Sleep -Seconds 10
    $tamanho = if (Test-Path $outFile) { (Get-Item $outFile).Length } else { 0L }
    if ($tamanho -ne $ultimoTamanho) {
        $ultimoTamanho = $tamanho
        $ultimoAvanco = Get-Date
    }
    if (((Get-Date) - $ultimoAvanco).TotalMinutes -ge $TravamentoMin) {
        Stop-Process -Id $p.Id -Force -Confirm:$false
        $resultado = "falha:travamento"
        break
    }
    if ((Get-Date) -ge $parede) {
        Stop-Process -Id $p.Id -Force -Confirm:$false
        $resultado = "falha:timeout"
        break
    }
}
if ($null -eq $resultado) {
    $resultado = if ($p.ExitCode -ne 0) { "falha:exit-$($p.ExitCode)" } else { "ok" }
}

# Matar o cliente NAO encerra a sessao: ela vive no daemon `opencode serve` e segue commitando
# na arvore compartilhada. Ver o doc da iteracao 0101 para as sete escritas medidas depois da
# morte da rodada, incluindo tres commits direto na main.
function Stop-SessaoDoTrabalhador {
    param([int]$Porta)
    Get-Process -Name "opencode" -ErrorAction SilentlyContinue | ForEach-Object {
        Stop-Process -Id $_.Id -Force -Confirm:$false -ErrorAction SilentlyContinue
    }
    $limite = (Get-Date).AddSeconds(30)
    while ((Get-Date) -lt $limite) {
        if (-not (Test-Connection -TargetName 127.0.0.1 -TcpPort $Porta -Quiet -TimeoutSeconds 2)) {
            break
        }
        Start-Sleep -Seconds 1
    }
}

if ($resultado -like "falha:*") {
    Stop-SessaoDoTrabalhador -Porta $Port
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
