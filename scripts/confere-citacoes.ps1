param(
    [switch]$Quiet
)

$ErrorActionPreference = 'Stop'
$projectRoot = Split-Path -Parent $PSScriptRoot

Push-Location $projectRoot
try {
    $cargoArgs = @('test', '-p', 'psx-core', '--test', 'spec_citations', '--', '--nocapture')
    if ($Quiet) {
        $cargoArgs += '--quiet'
    }
    & cargo $cargoArgs
    $exitCode = $LASTEXITCODE
    if ($exitCode -ne 0) {
        Write-Host ""
        Write-Host "Citacoes de spec com erro. Corrija antes de commitar." -ForegroundColor Red
    }
    exit $exitCode
}
finally {
    Pop-Location
}
