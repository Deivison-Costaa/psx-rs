# Classifica a saida do TTY do psx-cli contra o gabarito de hardware real (psx.log
# do projeto ps1-tests). Puro: nenhuma funcao aqui faz I/O -- so recebe texto e
# devolve veredito. Compartilhado por scripts/oraculo-tty.ps1 (execucao real) e por
# crates/psx-core/tests/oraculo_tty.rs (casos sinteticos, sem BIOS nem EXE).
#
# Classificacao (ROADMAP 10.23):
#   identico       -- todas as linhas batem apos normalizar fim de linha.
#   difere         -- K de M linhas nao batem (M = maior das duas contagens).
#   sem-saida      -- TTY vazio (nao roda o diff).
#   sem-gabarito   -- psx.log ausente (defensivo; nao deveria ocorrer nas 21 suites
#                     que motivaram este arreio, mas a funcao nao deve confundir
#                     "gabarito ausente" com "0 linhas diferentes").

function ConvertTo-NormalizedLines {
    param([AllowEmptyString()][string]$Texto)
    if ([string]::IsNullOrEmpty($Texto)) {
        return @()
    }
    $semCrlf = ($Texto -replace "`r`n", "`n") -replace "`r", "`n"
    , ($semCrlf.TrimEnd("`n") -split "`n")
}

function Get-TtyVeredito {
    param(
        [AllowEmptyString()][string]$Real,
        # Sem tipo [string]: PowerShell converte $null em "" ao vincular um parametro
        # tipado como string, o que confundiria "gabarito ausente" com "gabarito vazio".
        $Gabarito
    )

    if ([string]::IsNullOrEmpty($Real)) {
        return [PSCustomObject]@{ Status = 'sem-saida'; Detalhe = '' }
    }

    if ($null -eq $Gabarito) {
        return [PSCustomObject]@{ Status = 'sem-gabarito'; Detalhe = '' }
    }

    $linhasReais = ConvertTo-NormalizedLines $Real
    $linhasGabarito = ConvertTo-NormalizedLines $Gabarito

    $total = [Math]::Max($linhasReais.Count, $linhasGabarito.Count)
    $diferentes = 0
    for ($i = 0; $i -lt $total; $i++) {
        $r = if ($i -lt $linhasReais.Count) { $linhasReais[$i] } else { $null }
        $g = if ($i -lt $linhasGabarito.Count) { $linhasGabarito[$i] } else { $null }
        if ($r -ne $g) { $diferentes++ }
    }

    if ($diferentes -eq 0) {
        [PSCustomObject]@{ Status = 'identico'; Detalhe = "0/$total" }
    } else {
        [PSCustomObject]@{ Status = 'difere'; Detalhe = "$diferentes/$total" }
    }
}
