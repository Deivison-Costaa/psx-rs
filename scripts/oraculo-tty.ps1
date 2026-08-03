# Oraculo de TTY (ROADMAP 10.23): roda cada suite de tests/exes/ps1-tests/**/ que tem
# um gabarito `psx.log` (TTY capturado em hardware real pelo projeto ps1-tests) ao
# lado do EXE, e classifica o TTY do psx-cli contra esse gabarito. Mede; nao corrige.
#
# A classificacao pura mora em scripts/lib/tty-veredito.ps1 (testada com casos
# sinteticos em crates/psx-core/tests/oraculo_tty.rs, sem depender de BIOS/EXE).
param(
    [string]$ExeRoot = "tests/exes/ps1-tests",
    [string]$BiosPath = "bios/SCPH1001.BIN",
    [string]$OutFile = "logs/oraculo-tty.csv",
    [long]$MaxSteps = 800000000,
    [int]$TimeoutSec = 180
)
$ErrorActionPreference = "Stop"

. "$PSScriptRoot/lib/tty-veredito.ps1"

$OutDir = Split-Path $OutFile
if ($OutDir) {
    New-Item -ItemType Directory -Force $OutDir | Out-Null
}

if (-not (Test-Path $ExeRoot)) {
    Write-Host "oraculo-tty: 0 suites -- '$ExeRoot' nao existe (sem material para medir)"
    if (-not (Test-Path $OutFile)) {
        Set-Content $OutFile "ts,commit,suite,exe,status,detalhe"
    }
    exit 0
}

$ts = Get-Date -Format "yyyy-MM-ddTHH:mm:sszzz"
$commit = (git rev-parse --short HEAD).Trim()
$haveBios = Test-Path $BiosPath

$logs = @(Get-ChildItem -Path $ExeRoot -Recurse -Filter "psx.log" | Sort-Object FullName)
$rows = @()

function Suite-Relativa($dir) {
    (Resolve-Path -Relative $dir) -replace '\\', '/' -replace '^\./', ''
}

if (-not $haveBios) {
    foreach ($log in $logs) {
        $suite = Suite-Relativa $log.Directory
        $rows += "$ts,$commit,$suite,,sem-bios,"
    }
    if (-not (Test-Path $OutFile)) {
        Set-Content $OutFile "ts,commit,suite,exe,status,detalhe"
    }
    Add-Content $OutFile $rows
    Write-Host "oraculo-tty: 0 de $($logs.Count) suites medidas de verdade -- BIOS ausente em '$BiosPath', todas 'sem-bios' -> $OutFile"
    exit 0
}

if (Test-Path "target/release/psx-cli.exe") {
    $cliBin = (Resolve-Path "target/release/psx-cli.exe").Path
} elseif (Test-Path "target/release/psx-cli") {
    $cliBin = (Resolve-Path "target/release/psx-cli").Path
} else {
    Write-Error "psx-cli binario nao encontrado em target/release/ -- rode 'cargo build --release -p psx-cli'"
    exit 1
}

foreach ($log in $logs) {
    $dir = $log.Directory
    $suite = Suite-Relativa $dir

    $exeFile = $null
    foreach ($cand in (Get-ChildItem $dir -File | Where-Object { $_.Extension -in ".exe", ".psexe" })) {
        $stream = [System.IO.File]::OpenRead($cand.FullName)
        $buffer = New-Object byte[] 8
        $read = $stream.Read($buffer, 0, 8)
        $stream.Close()
        if ($read -eq 8 -and [System.Text.Encoding]::ASCII.GetString($buffer) -eq "PS-X EXE") {
            $exeFile = $cand
            break
        }
    }

    if (-not $exeFile) {
        $rows += "$ts,$commit,$suite,,sem-exe,"
        Write-Warning "oraculo-tty: $suite -- nenhum PS-EXE (magic 'PS-X EXE') ao lado do psx.log"
        continue
    }

    $stdoutPath = [System.IO.Path]::GetTempFileName()
    $stderrPath = [System.IO.Path]::GetTempFileName()
    try {
        $argString = "--bios `"$BiosPath`" --exe `"$($exeFile.FullName)`" --max-steps $MaxSteps"
        $proc = Start-Process -FilePath $cliBin -ArgumentList $argString -NoNewWindow -PassThru `
            -RedirectStandardOutput $stdoutPath -RedirectStandardError $stderrPath
        $finished = $proc.WaitForExit($TimeoutSec * 1000)

        if (-not $finished) {
            $proc.Kill()
            $rows += "$ts,$commit,$suite,$($exeFile.Name),timeout,>${TimeoutSec}s"
            Write-Warning "oraculo-tty: $suite -- estourou o timeout de ${TimeoutSec}s em --max-steps $MaxSteps"
            continue
        }

        $real = Get-Content $stdoutPath -Raw -ErrorAction SilentlyContinue
        $gabarito = Get-Content $log.FullName -Raw -ErrorAction SilentlyContinue

        $veredito = Get-TtyVeredito -Real $real -Gabarito $gabarito
        $rows += "$ts,$commit,$suite,$($exeFile.Name),$($veredito.Status),$($veredito.Detalhe)"
    } finally {
        Remove-Item $stdoutPath, $stderrPath -ErrorAction SilentlyContinue
    }
}

if (-not (Test-Path $OutFile)) {
    Set-Content $OutFile "ts,commit,suite,exe,status,detalhe"
}
Add-Content $OutFile $rows

$total = $rows.Count
$identico = @($rows | Where-Object { $_ -match ',identico,' }).Count
$difere = @($rows | Where-Object { $_ -match ',difere,' }).Count
$semSaida = @($rows | Where-Object { $_ -match ',sem-saida,' }).Count
$outros = $total - $identico - $difere - $semSaida
Write-Host "oraculo-tty: $identico identico, $difere difere, $semSaida sem-saida, $outros outros, de $total suites com gabarito (commit $commit, bios=$haveBios) -> $OutFile"
