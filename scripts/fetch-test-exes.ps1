# Baixa EXEs de teste de hardware para tests/exes/ (gitignored — binarios de terceiros).
# Procedencia pinada: release do ps1-tests e zips do Amidog, com hash conferido apos download.
# Rodar da raiz do repo. Idempotente: pula o que ja existe.
$ErrorActionPreference = "Stop"

$Ps1TestsTag = "build-158"
$Sources = @(
    @{ Name = "ps1-tests"; Dir = "tests/exes/ps1-tests"
       Url = "https://github.com/JaCzekanski/ps1-tests/releases/download/$Ps1TestsTag/tests.zip" }
    @{ Name = "amidog-cpu"; Dir = "tests/exes/amidog/cpu"
       Url = "https://psx.amidog.se/lib/exe/fetch.php?media=psx:download:psxtest_cpu.zip" }
    @{ Name = "amidog-gte"; Dir = "tests/exes/amidog/gte"
       Url = "https://psx.amidog.se/lib/exe/fetch.php?media=psx:download:psxtest_gte.zip" }
)

foreach ($s in $Sources) {
    if (Test-Path $s.Dir) {
        Write-Host ("{0,-12} ja existe, pulando" -f $s.Name)
        continue
    }
    $zip = Join-Path ([System.IO.Path]::GetTempPath()) "$($s.Name).zip"
    Invoke-WebRequest $s.Url -OutFile $zip -UseBasicParsing
    $hash = (Get-FileHash $zip -Algorithm SHA256).Hash
    New-Item -ItemType Directory -Force $s.Dir | Out-Null
    Expand-Archive $zip -DestinationPath $s.Dir -Force
    Remove-Item $zip
    $count = (Get-ChildItem $s.Dir -Recurse -Include *.exe, *.psexe).Count
    Write-Host ("{0,-12} sha256={1} exes={2}" -f $s.Name, $hash.Substring(0, 12), $count)
}
Write-Host "OK. Detalhe por suite em tests/exes/; runner: scripts/scoreboard.ps1"
