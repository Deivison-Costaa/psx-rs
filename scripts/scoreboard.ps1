# Roda as suites de EXEs de teste no psx-cli headless e acumula o placar em logs/scoreboard.csv.
$ErrorActionPreference = "Stop"

$ExeRoot = "tests/exes"
$OutDir = "logs"
$OutFile = "$OutDir/scoreboard.csv"
$BiosPath = "bios/SCPH1001.BIN"
$TimeoutSec = 120

if (-not (Test-Path $ExeRoot)) {
    Write-Error "tests/exes/ nao existe — rode scripts/fetch-test-exes.ps1 antes."
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

$exeFiles = Get-ChildItem $ExeRoot -Recurse -Include *.exe, *.psexe | Sort-Object FullName

foreach ($exe in $exeFiles) {
    $suite = (Resolve-Path -Relative $exe.Directory) -replace '\\', '/' -replace '^\./tests/exes/', ''

    if (-not $haveBios) {
        $rows += "$ts,$commit,$suite,$($exe.Name),sem-bios,"
        continue
    }

    try {
        $argString = "--bios `"$BiosPath`" --exe `"$($exe.FullName)`""
        $proc = Start-Process -FilePath $cliBin -ArgumentList $argString -NoNewWindow -PassThru -RedirectStandardOutput "logs/tmp_stdout.txt" -RedirectStandardError "logs/tmp_stderr.txt"
        $finished = $proc.WaitForExit($TimeoutSec * 1000)

        if (-not $finished) {
            $proc.Kill()
            $rows += "$ts,$commit,$suite,$($exe.Name),timeout,"
            continue
        }

        if ($proc.ExitCode -ne 0) {
            $rows += "$ts,$commit,$suite,$($exe.Name),fail-erro,"
            continue
        }

        $stderr = Get-Content "logs/tmp_stderr.txt" -Raw -ErrorAction SilentlyContinue
        if ($stderr -match "Runner: (\d+) passos, TTY: (\d+) bytes") {
            $ttyBytes = [int]$Matches[2]
            if ($ttyBytes -gt 0) {
                $rows += "$ts,$commit,$suite,$($exe.Name),pass,"
            } else {
                $rows += "$ts,$commit,$suite,$($exe.Name),fail,"
            }
        } else {
            $stdout = Get-Content "logs/tmp_stdout.txt" -Raw -ErrorAction SilentlyContinue
            if ($stdout -and $stdout.Length -gt 0) {
                $rows += "$ts,$commit,$suite,$($exe.Name),pass,"
            } else {
                $rows += "$ts,$commit,$suite,$($exe.Name),fail,"
            }
        }
    } catch {
        $rows += "$ts,$commit,$suite,$($exe.Name),erro,"
    }
}

Remove-Item "logs/tmp_stdout.txt" -ErrorAction SilentlyContinue
Remove-Item "logs/tmp_stderr.txt" -ErrorAction SilentlyContinue

if (-not (Test-Path $OutFile)) {
    Set-Content $OutFile "ts,commit,suite,exe,status,ciclos"
}
Add-Content $OutFile $rows
$total = @($rows).Count
$pass = @($rows | Where-Object { $_ -match ",pass," }).Count
Write-Host "scoreboard: $pass/$total passando (commit $commit, bios=$haveBios) -> $OutFile"
