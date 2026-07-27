# Roda as suites de EXEs de teste no psx-cli headless e acumula o placar em logs/scoreboard.csv.
# Enquanto o runner nao existe (item 1.11), registra cada EXE como "sem-runner" — o placar
# nasce honesto: 0 passando porque nada roda ainda, nao porque nada foi medido.
# A serie historica e publicada pela CI na branch scoreboard-data (item 1.12), nunca na main.
$ErrorActionPreference = "Stop"

$ExeRoot = "tests/exes"
$OutDir = "logs"
$OutFile = "$OutDir/scoreboard.csv"

if (-not (Test-Path $ExeRoot)) {
    Write-Error "tests/exes/ nao existe — rode scripts/fetch-test-exes.ps1 antes."
}
New-Item -ItemType Directory -Force $OutDir | Out-Null

$ts = Get-Date -Format "yyyy-MM-ddTHH:mm:sszzz"
$commit = (git rev-parse --short HEAD).Trim()
$rows = foreach ($exe in Get-ChildItem $ExeRoot -Recurse -Include *.exe, *.psexe | Sort-Object FullName) {
    $suite = (Resolve-Path -Relative $exe.Directory) -replace '\\', '/' -replace '^\./tests/exes/', ''
    "$ts,$commit,$suite,$($exe.Name),sem-runner,"
}

if (-not (Test-Path $OutFile)) {
    Set-Content $OutFile "ts,commit,suite,exe,status,ciclos"
}
Add-Content $OutFile $rows
$total = @($rows).Count
$pass = @($rows | Where-Object { $_ -match ",pass," }).Count
Write-Host "scoreboard: $pass/$total passando (commit $commit) -> $OutFile"
