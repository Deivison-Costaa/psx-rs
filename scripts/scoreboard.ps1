# Roda as suites de EXEs de teste no psx-cli headless e acumula o placar em logs/scoreboard.csv.
$ErrorActionPreference = "Stop"

$ExeRoot = "tests/exes"
$OutDir = "logs"
$OutFile = "$OutDir/scoreboard.csv"
$BiosPath = "bios/SCPH1001.BIN"
$TimeoutSec = 120

if (-not (Test-Path $ExeRoot)) {
    New-Item -ItemType Directory -Force $OutDir | Out-Null
    Write-Host "scoreboard: 0/0 — tests/exes/ nao existe (sem EXEs para avaliar)"
    if (-not (Test-Path $OutFile)) {
        Set-Content $OutFile "ts,commit,suite,exe,status,detalhe"
    }
    exit 0
}
New-Item -ItemType Directory -Force $OutDir | Out-Null

try {
    cargo build --release -p psx-cli 2>&1 | Out-Null
    if ($LASTEXITCODE -ne 0) {
        throw "build failed"
    }
} catch {
    Write-Warning "cargo build --release -p psx-cli falhou; usando binario existente (se disponivel)"
}

if (Test-Path "target/release/psx-cli.exe") {
    $cliBin = (Resolve-Path "target/release/psx-cli.exe").Path
} elseif (Test-Path "target/release/psx-cli") {
    $cliBin = (Resolve-Path "target/release/psx-cli").Path
} else {
    Write-Error "psx-cli binario nao encontrado em target/release/"
    exit 1
}

$haveBios = Test-Path $BiosPath

$ts = Get-Date -Format "yyyy-MM-ddTHH:mm:sszzz"
$commit = (git rev-parse --short HEAD).Trim()

$rows = @()

$candidateFiles = Get-ChildItem $ExeRoot -Include *.exe, *.psexe -Recurse | Sort-Object FullName

foreach ($candidate in $candidateFiles) {
    $suite = (Resolve-Path -Relative $candidate.Directory) -replace '\\', '/' -replace '^\./tests/exes/', ''

    if (-not $haveBios) {
        $rows += "$ts,$commit,$suite,$($candidate.Name),sem-bios,"
        continue
    }

    try {
        $stream = [System.IO.File]::OpenRead($candidate.FullName)
        $buffer = New-Object byte[] 8
        $read = $stream.Read($buffer, 0, 8)
        $stream.Close()
        if ($read -lt 8) {
            $rows += "$ts,$commit,$suite,$($candidate.Name),host-bin,"
            continue
        }
        $magic = [System.Text.Encoding]::ASCII.GetString($buffer)
        if ($magic -ne "PS-X EXE") {
            $rows += "$ts,$commit,$suite,$($candidate.Name),host-bin,"
            continue
        }
    } catch {
        $rows += "$ts,$commit,$suite,$($candidate.Name),host-bin,"
        continue
    }

    try {
        $argString = "--bios `"$BiosPath`" --exe `"$($candidate.FullName)`""
        $proc = Start-Process -FilePath $cliBin -ArgumentList $argString -NoNewWindow -PassThru -RedirectStandardOutput "logs/tmp_stdout.txt" -RedirectStandardError "logs/tmp_stderr.txt"
        $finished = $proc.WaitForExit($TimeoutSec * 1000)

        if (-not $finished) {
            $proc.Kill()
            $rows += "$ts,$commit,$suite,$($candidate.Name),timeout,"
            continue
        }

        if ($proc.ExitCode -ne 0) {
            $rows += "$ts,$commit,$suite,$($candidate.Name),fail-erro,"
            continue
        }

        $stdout = Get-Content "logs/tmp_stdout.txt" -Raw -ErrorAction SilentlyContinue
        $veredictLines = @()
        if ($stdout) {
            $veredictLines = @($stdout -split "`n" | Where-Object { $_ -match '^(pass|fail) - ' } | Sort-Object | Get-Unique)
        }
        $passCount = @($veredictLines | Where-Object { $_ -match '^pass - ' }).Count
        $failCount = @($veredictLines | Where-Object { $_ -match '^fail - ' }).Count

        if ($passCount -gt 0 -or $failCount -gt 0) {
            $status = if ($failCount -gt 0) { 'fail' } else { 'pass' }
            $detalhe = "${passCount}p/${failCount}f"
            $rows += "$ts,$commit,$suite,$($candidate.Name),$status,$detalhe"
        } else {
            $stderr = Get-Content "logs/tmp_stderr.txt" -Raw -ErrorAction SilentlyContinue
            if ($stderr -match "Runner: (\d+) passos, TTY: (\d+) bytes") {
                $ttyBytes = [int]$Matches[2]
                if ($ttyBytes -gt 0) {
                    $rows += "$ts,$commit,$suite,$($candidate.Name),tty,"
                } else {
                    $rows += "$ts,$commit,$suite,$($candidate.Name),sem-saida,"
                }
            } else {
                if ($stdout -and $stdout.Length -gt 0) {
                    $rows += "$ts,$commit,$suite,$($candidate.Name),tty,"
                } else {
                    $rows += "$ts,$commit,$suite,$($candidate.Name),sem-saida,"
                }
            }
        }
    } catch {
        $rows += "$ts,$commit,$suite,$($candidate.Name),erro,"
    }
}

Remove-Item "logs/tmp_stdout.txt" -ErrorAction SilentlyContinue
Remove-Item "logs/tmp_stderr.txt" -ErrorAction SilentlyContinue

if (-not (Test-Path $OutFile)) {
    Set-Content $OutFile "ts,commit,suite,exe,status,detalhe"
}
Add-Content $OutFile $rows
$total = @($rows).Count
$tty = @($rows | Where-Object { $_ -match ",tty," }).Count
Write-Host "scoreboard: $tty/$total produziram saida (criterio: TTY nao vazio; veredito real fica para o 1.13) (commit $commit, bios=$haveBios) -> $OutFile"
