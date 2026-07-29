# Script de bateria de mutação do psx-rs.
# Aplica cada registro de um manifesto .mut, roda o teste e registra o placar.
# Parametros:
#   -Iter NNNN       roda um manifesto especifico (ex.: -Iter 0038)
#   -Alterados       roda apenas manifestos tocados pelo diff do PR
#   -MaxMutantes N   limita o numero de mutantes executados (debug)
$ErrorActionPreference = "Stop"

$RepoRoot = (git rev-parse --show-toplevel).Trim()
Push-Location $RepoRoot

# ---------- helpers ----------

function Die {
    param([string]$Msg)
    Write-Error "mutantes.ps1: $Msg"
    Pop-Location
    exit 1
}

# ---------- parametros ----------

$Iter = $null
$Alterados = $false
$MaxMutantes = 0

$argsArr = $args
for ($i = 0; $i -lt $argsArr.Count; $i++) {
    switch ($argsArr[$i]) {
        "-Iter" {
            if ($i + 1 -lt $argsArr.Count) {
                $Iter = $argsArr[++$i]
            } else { Die "faltou o valor de -Iter" }
        }
        "-Alterados" { $Alterados = $true }
        "-MaxMutantes" {
            if ($i + 1 -lt $argsArr.Count) {
                $MaxMutantes = [int]$argsArr[++$i]
            } else { Die "faltou o valor de -MaxMutantes" }
        }
        default {
            if ($argsArr[$i] -notmatch '^-') {
                Write-Warning "parametro posicional ignorado: $($argsArr[$i])"
            } else {
                Die "parametro desconhecido: $($argsArr[$i])"
            }
        }
    }
}

if (-not $Iter -and -not $Alterados) {
    Die "use -Iter NNNN ou -Alterados"
}
if ($Iter -and $Alterados) {
    Die "-Iter e -Alterados sao mutuamente exclusivos"
}

# ---------- documentos ----------

$MutantesDir = Join-Path $RepoRoot "docs/mutantes"
$LogsDir = Join-Path $RepoRoot "logs"
New-Item -ItemType Directory -Force $LogsDir | Out-Null

$SentinelPath = Join-Path $LogsDir "mutantes-em-andamento.txt"

# ---------- restauracao camada 1: recusar se arvore suja ----------

$statusPorcelain = (git status --porcelain) -join "`n"
if ($statusPorcelain.Trim().Length -gt 0) {
    Die @"
arvore suja antes de comecar. Restaure com git checkout -- ... ou faça commit.
git status --porcelain:
$statusPorcelain
"@
}

# ---------- restauracao camada 4: sentinela ----------

if (Test-Path $SentinelPath) {
    $files = Get-Content $SentinelPath -Raw
    Die @"
SENTINELA ENCONTRADA: $SentinelPath.
A execucao anterior foi interrompida (kill/Ctrl-C) com mutacoes aplicadas.
Restaure a arvore manualmente com:

    git checkout -- $($files -replace "`n", " ") 

Depois delete $SentinelPath.
"@
}

# ---------- descobrir manifestos ----------

$manifestPaths = @()
if ($Iter) {
    $pattern = Join-Path $MutantesDir "$Iter-*.mut"
    $candidates = @(Get-ChildItem $pattern -ErrorAction SilentlyContinue)
    if ($candidates.Count -eq 0) {
        Die "nenhum manifesto encontrado para iteracao $Iter em $MutantesDir"
    }
    $manifestPaths = @($candidates | ForEach-Object { $_.FullName })
} elseif ($Alterados) {
    try {
        $baseRef = (git symbolic-ref refs/remotes/origin/HEAD 2>$null) -replace '.*/', ''
        if (-not $baseRef) { $baseRef = "main" }
        $baseRef = "origin/$baseRef"
    } catch {
        $baseRef = "origin/main"
    }
    $diffFiles = @(git diff --name-only "$baseRef...HEAD" -- "$MutantesDir/" 2>$null)
    if ($diffFiles.Count -eq 0) {
        $diffFiles = @(git diff --name-only HEAD~1 -- "$MutantesDir/" 2>$null)
    }
    foreach ($f in $diffFiles) {
        $full = Join-Path $RepoRoot $f
        if ((Test-Path $full) -and $f -match '\.mut$') {
            $manifestPaths += $full
        }
    }
    if ($manifestPaths.Count -eq 0) {
        Write-Host "mutantes: nenhum manifesto alterado neste PR. Nada a rodar."
        Pop-Location
        exit 0
    }
}

# ---------- parse de manifesto ----------

function Parse-Manifest {
    param([string]$Path, [string]$RelPath)

    $lines = Get-Content $Path
    $state = "header"
    $header = @{}
    $records = @()
    $rec = $null
    $inDe = $false; $inPara = $false
    $deLines = @(); $paraLines = @()

    $i = 0
    while ($i -lt $lines.Count) {
        $line = $lines[$i].TrimEnd()
        $i++

        if ($line -eq "" -or $line.StartsWith("#")) { continue }

        if ($inPara) {
            if ($line -eq "@@FIM") {
                $rec.edits += @{ de = ($deLines -join "`n"); para = ($paraLines -join "`n"); ocorrencias = $pendingOcorrencias }
                $inPara = $false; $deLines = @(); $paraLines = @()
                $pendingOcorrencias = "1"
                continue
            }
            if ($line -eq "@@DE" -or $line -eq "@@PARA") {
                Die "$RelPath : sentinela '$line' dentro de @@PARA sem @@FIM"
            }
            $paraLines += $lines[$i - 1]
            continue
        }

        if ($inDe) {
            if ($line -eq "@@PARA") {
                $inDe = $false; $inPara = $true; $paraLines = @()
                continue
            }
            if ($line -eq "@@FIM") { Die "$RelPath : @@FIM sem @@PARA" }
            if ($line -eq "@@DE") { Die "$RelPath : @@DE aninhado (sem @@FIM anterior)" }
            $deLines += $lines[$i - 1]
            continue
        }

        if ($line -eq "@@DE") {
            if (-not $rec) { Die "$RelPath : @@DE antes de iniciar um registro" }
            $inDe = $true; $deLines = @()
            continue
        }
        if ($line -eq "@@PARA" -or $line -eq "@@FIM") {
            Die "$RelPath : $($line) sem @@DE anterior"
        }

        $colon = $line.IndexOf(":")
        if ($colon -le 0) { Die "$RelPath : linha sem chave:valor nem sentinela: '$line'" }
        $key = $line.Substring(0, $colon).Trim()
        $value = $line.Substring($colon + 1).Trim()

        switch ($key) {
            "formato"   { $header.formato = $value }
            "iteracao"  { $header.iteracao = $value }
            "item"      { $header.item = $value }
            "alvo"      { $header.alvo = $value }
            "teste"     { $header.teste = $value }
            "arquivada" { $header.arquivada = $value }
            "mutante" {
                if ($rec) { $records += $rec }
                $rec = @{ kind = "mutante"; id = $value; rotulo = ""; arquivo = $null; teste = $null;
                           justificativa = $null; edits = @() }
                $pendingOcorrencias = "1"
            }
            "controle" {
                if ($rec) { $records += $rec }
                $rec = @{ kind = "controle"; id = $value; rotulo = ""; arquivo = $null; teste = $null;
                           justificativa = $null; edits = @() }
                $pendingOcorrencias = "1"
            }
            "equivalente" {
                if ($rec) { $records += $rec }
                $rec = @{ kind = "equivalente"; id = $value; rotulo = ""; arquivo = $null; teste = $null;
                           justificativa = $null; edits = @() }
                $pendingOcorrencias = "1"
            }
            "rotulo"          { if ($rec) { $rec.rotulo = $value } }
            "arquivo"         { if ($rec) { $rec.arquivo = $value } }
            "teste"           { if ($rec) { $rec.teste = $value } }
            "justificativa"   { if ($rec) { $rec.justificativa = $value } }
            "ocorrencias"     { $pendingOcorrencias = $value }
            default           { Write-Warning "$RelPath : chave desconhecida: '$key'" }
        }
    }
    if ($rec) { $records += $rec }

    if ($header.formato -ne "1") { Die "$RelPath : formato deve ser 1 (encontrado $($header.formato))" }
    if (-not $header.teste) { Die "$RelPath : teste ausente no cabecalho" }
    if (-not $header.alvo) { Die "$RelPath : alvo ausente no cabecalho" }

    return @{ header = $header; records = $records }
}

# ---------- aplicar edicoes ----------

function Apply-Edits {
    param([string]$FilePath, [array]$Edits)

    $raw = (Get-Content $FilePath -Raw)
    if (-not $raw.EndsWith("`n")) { $raw += "`n" }
    $current = $raw

    foreach ($edit in $Edits) {
        $de = $edit.de
        $para = $edit.para
        $oc = $edit.ocorrencias

        $needle = "`n" + $de + "`n"

        $matches = @()
        $pos = 0
        while (($m = $current.IndexOf($needle, $pos, [StringComparison]::Ordinal)) -ge 0) {
            $matches += $m
            $pos = $m + 1
        }

        $expected = if ($oc -eq "todas") { -1 } else { [int]$oc }
        $actual = $matches.Count

        if ($expected -eq -1) {
            if ($actual -eq 0) {
                Die "edicao '@@DE' nao encontrada nenhuma vez (ocorrencias: todas)"
            }
        } else {
            if ($actual -ne $expected) {
                Die "edicao '@@DE' encontrada $actual vez(es), esperado $expected"
            }
        }

        $replacement = "`n" + $para + "`n"
        $newContent = ""
        $lastEnd = 0
        foreach ($m in $matches) {
            $newContent += $current.Substring($lastEnd, $m - $lastEnd + 1)
            $newContent += $para + "`n"
            $lastEnd = $m + $needle.Length
        }
        $newContent += $current.Substring($lastEnd)
        $current = $newContent
    }

    $current = $current.TrimEnd("`n") + "`n"
    [System.IO.File]::WriteAllText($FilePath, $current, [System.Text.Encoding]::UTF8)
}

# ---------- rodar cargo test ----------

function Invoke-CargoTest {
    param([string]$TestName)

    $sw = [System.Diagnostics.Stopwatch]::StartNew()
    $outputFile = Join-Path $LogsDir "mutantes-stdout.txt"
    $errFile = Join-Path $LogsDir "mutantes-stderr.txt"

    try {
        cargo test -p psx-core --test $TestName 2>&1 | Out-File -FilePath $outputFile -Encoding utf8
        $exit = $LASTEXITCODE
    } catch {
        $exit = 1
        "Exception: $_" | Out-File -FilePath $outputFile -Encoding utf8 -Append
    }

    $sw.Stop()
    $output = Get-Content $outputFile -Raw -ErrorAction SilentlyContinue
    if (-not $output) { $output = "" }

    $seconds = [math]::Round($sw.Elapsed.TotalSeconds, 1)

    $isCompileError = ($output -match 'error\[E' -or $output -match 'error: could not compile')
    $testFailed = ($output -match 'test result: FAILED')

    $status = switch ($true) {
        ($exit -eq 0) { "sobreviveu" }
        $isCompileError { "erro-manifesto" }
        $testFailed { "morreu" }
        default { "erro" }
    }

    $testNames = @()
    if ($testFailed -and $output) {
        if ($output -match 'failures:\r?\n((?:    [a-zA-Z_]\w+.+\r?\n)+)\r?\ntest result: FAILED') {
            $namesBlock = $Matches[1]
            foreach ($n in ($namesBlock -split "`n")) {
                $name = $n.Trim()
                if ($name -and $name -notmatch '----') {
                    $testNames += $name
                }
            }
        }
    }

    return @{
        exitCode = $exit
        status = $status
        seconds = $seconds
        testNames = ($testNames -join ';')
        output = $output
    }
}

# ---------- main ----------

$allResults = @()
$overallPassed = $true

foreach ($manifestPath in $manifestPaths) {
    $relPath = (Resolve-Path -Relative $manifestPath) -replace '\\', '/'
    $stem = [System.IO.Path]::GetFileNameWithoutExtension($manifestPath)

    $resultadoFile = Join-Path $MutantesDir "$stem.resultado"
    $logFile = Join-Path $LogsDir "mutantes-$stem.txt"

    $commit = (git rev-parse HEAD).Trim()
    $ts = (Get-Date).ToString("yyyy-MM-ddTHH:mm:sszzz")

    $mf = Parse-Manifest -Path $manifestPath -RelPath $relPath
    $hdr = $mf.header
    $recordsList = $mf.records

    Write-Host ""
    Write-Host "=== $relPath ==="
    Write-Host "    registro(s): $($recordsList.Count) | alvo: $($hdr.alvo) | teste: $($hdr.teste)"
    Write-Host ""

    if ($hdr.arquivada) {
        Write-Host "  ARQUIVADA - $($recordsList.Count) registro(s) NAO rodados. Motivo: $($hdr.arquivada)" -ForegroundColor Yellow
        Write-Host "  Cobertura perdida ate alguem reparar as ancoras." -ForegroundColor Yellow
        Write-Host ""
        continue
    }

    @(
        "# gerado por scripts/mutantes.ps1 — NAO editar a mao",
        "# commit: $commit",
        "# rodado_em: $ts"
    ) | Set-Content $resultadoFile
    "" | Set-Content $logFile

    $arquivosTocados = @()
    $recordsToRun = if ($MaxMutantes -gt 0 -and $MaxMutantes -lt $recordsList.Count) {
        $recordsList | Select-Object -First $MaxMutantes
    } else {
        $recordsList
    }

    $allPassed = $true
    $mutantesMorreram = 0
    $mutantesTotal = 0
    $controlesVerdes = 0
    $controlesTotal = 0
    $equivalentesTotal = 0

    foreach ($rec in $recordsToRun) {
        $arquivo = if ($rec.arquivo) { $rec.arquivo } else { $hdr.alvo }
        $teste = if ($rec.teste) { $rec.teste } else { $hdr.teste }
        $arquivoAbs = Join-Path $RepoRoot $arquivo

        if (-not (Test-Path $arquivoAbs)) {
            Write-Error "$($rec.kind) '$($rec.id)': arquivo '$arquivo' nao existe"
            $allPassed = $false
            continue
        }

        Write-Host -NoNewline "  $($rec.id) ($($rec.kind)) ... "

        $isControle = ($rec.kind -eq "controle")
        $isEquivalente = ($rec.kind -eq "equivalente")
        if ($isControle) {
            $controlesTotal++
        }
        if ($isEquivalente) {
            $equivalentesTotal++
        }
        if ($rec.kind -eq "mutante") { $mutantesTotal++ }

        $sentinelContent = ($arquivosTocados + $arquivo | Sort-Object -Unique) -join "`n"
        $sentinelContent | Set-Content $SentinelPath

        try {
            $arquivosTocados += $arquivo

            Apply-Edits -FilePath $arquivoAbs -Edits $rec.edits

            $result = Invoke-CargoTest -TestName $teste
            $status = $result.status
            $seconds = $result.seconds
            $testNames = $result.testNames

            if ($status -eq "erro-manifesto") {
                $cleanOutput = $result.output -replace '`n', ' '
                if ($cleanOutput.Length -gt 300) { $cleanOutput = $cleanOutput.Substring(0, 300) + "..." }
                Write-Host "ERRO DE MANIFESTO"
                Write-Host "    Saida do cargo (truncada): $cleanOutput"
                Write-Error "$($rec.id): erro de manifesto — mutante nao compila. Corrija o manifesto $stem.mut."
                $allPassed = $false

                $errCsvExpected = if ($rec.kind -eq "mutante") { "morreu" } else { "sobreviveu" }
                $csvLine = "$($rec.id),$($rec.kind),$errCsvExpected,erro-manifesto,$seconds,$testNames"
                Add-Content $resultadoFile $csvLine
                Add-Content $logFile "=== $($rec.id) ($($rec.kind)) ==="
                Add-Content $logFile "STATUS: erro-manifesto"
                Add-Content $logFile "SAIDA:`n$($result.output)"
                Add-Content $logFile ""
                throw "abort-this-record"
            }
        } finally {
            git checkout -- $arquivoAbs 2>$null
            if ($LASTEXITCODE -ne 0) {
                Write-Warning "git checkout falhou para $arquivoAbs — tentando git restore"
                git restore $arquivoAbs 2>$null
            }
        }

        $expected = if ($rec.kind -eq "mutante") { "morreu" } else { "sobreviveu" }

        switch ($rec.kind) {
            "mutante" {
                if ($status -eq "morreu") {
                    Write-Host "MORREU ($seconds s)"
                    $mutantesMorreram++
                } elseif ($status -eq "sobreviveu") {
                    Write-Host "SOBREVIVEU ($seconds s)"
                    $allPassed = $false
                } else {
                    Write-Host "$status ($seconds s)"
                    $allPassed = $false
                }
            }
            "controle" {
                if ($status -eq "sobreviveu") {
                    Write-Host "verde ($seconds s)"
                    $controlesVerdes++
                } else {
                    Write-Host "QUEBROU ($status, $seconds s)"
                    $allPassed = $false
                }
            }
            "equivalente" {
                if ($status -eq "sobreviveu") {
                    Write-Host "sobreviveu ($seconds s)"
                } elseif ($status -eq "morreu") {
                    Write-Host "MORREU — equivalente nao deveria morrer ($seconds s)"
                    Write-Error "$($rec.id): equivalente morreu. A justificativa estava errada ou o teste ficou mais forte."
                    $allPassed = $false
                } else {
                    Write-Host "$status ($seconds s)"
                    $allPassed = $false
                }
            }
        }

        $csvLine = "$($rec.id),$($rec.kind),$expected,$status,$seconds,$testNames"
        Add-Content $resultadoFile $csvLine

        $logEntry = @"
=== $($rec.id) ($($rec.kind)) ===
ROTULO: $($rec.rotulo)
STATUS: $status (esperado $expected)
SEGUNDOS: $seconds
SAIDA:
$($result.output)
"@
        Add-Content $logFile $logEntry
    }

    if (Test-Path $SentinelPath) {
        Remove-Item $SentinelPath -Force -ErrorAction SilentlyContinue
    }

    $linhaPlacar = "Placar da bateria: $mutantesMorreram/$mutantesTotal mutantes mortos, $controlesVerdes/$controlesTotal controles verdes, $equivalentesTotal equivalente — $relPath"
    Write-Host ""
    Write-Host $linhaPlacar
    Write-Host "resultado: $resultadoFile"
    Write-Host "log: $logFile"

    if (-not $allPassed) {
        $overallPassed = $false
    }
}

# ---------- restauracao camada 5: verificar arvore limpa ----------
$cleanStatus = $true
$allowedDirty = @()
foreach ($line in ($statusFinal -split "`n")) {
    $line = $line.Trim()
    if (-not $line) { continue }
    $file = $line.Substring(3).Trim()
    if ($file -match 'docs\\mutantes\\.*\.resultado$' -or $file -match 'docs/mutantes/.*\.resultado$') {
        continue
    }
    $cleanStatus = $false
    Write-Warning "arquivo modificado fora de docs/mutantes/*.resultado: $file ($line)"
}
if (-not $cleanStatus) {
    Die "arvore nao esta limpa apos a bateria. Restauracao falhou ou ha edicoes pendentes."
}

Pop-Location

if ($overallPassed) {
    exit 0
} else {
    exit 1
}
