# Classifica a saida do TTY do psx-cli contra o gabarito de hardware real (psx.log
# do projeto ps1-tests). Puro: nenhuma funcao aqui faz I/O -- so recebe texto e
# devolve veredito. Compartilhado por scripts/oraculo-tty.ps1 (execucao real) e por
# crates/psx-core/tests/oraculo_tty.rs (casos sinteticos, sem BIOS nem EXE).
#
# Classificacao (ROADMAP 10.23):
#   identico       -- todas as linhas batem apos normalizar fim de linha.
#   difere         -- K de M linhas nao batem (M = maior das duas contagens).
#   sem-saida      -- TTY vazio (nao roda o diff).
#   sem-alinhamento-- nenhuma linha nossa casa com o gabarito (item 10.98).
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

# So remove o prefixo quando TODAS as linhas nao-vazias o tem: alguns psx.log do ps1-tests
# prefixam '% ' (ex.: cpu/cop, 19 de 19 linhas), a maioria nao. Remover por palpite mascararia
# divergencia real.
function Remove-PrefixoUniforme {
    param([string[]]$Linhas)
    $naoVazias = @($Linhas | Where-Object { $_ -ne '' })
    if ($naoVazias.Count -eq 0) { return $Linhas }
    if (@($naoVazias | Where-Object { -not $_.StartsWith('% ') }).Count -gt 0) { return $Linhas }
    , @($Linhas | ForEach-Object { if ($_ -eq '') { $_ } else { $_.Substring(2) } })
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
    $linhasGabarito = Remove-PrefixoUniforme $linhasGabarito

    # Com kernel real (0170) o nosso TTY traz o banner de boot da BIOS, que o gabarito nao tem.
    # Alinha na primeira linha do gabarito que aparece na nossa saida; o que vem antes e
    # preambulo. As linhas do gabarito puladas ate a ancora contam como divergencia.
    $ancoraReal = -1
    $ancoraGab = -1
    for ($g = 0; $g -lt $linhasGabarito.Count -and $ancoraReal -lt 0; $g++) {
        $idx = [Array]::IndexOf($linhasReais, $linhasGabarito[$g])
        if ($idx -ge 0) { $ancoraReal = $idx; $ancoraGab = $g }
    }
    if ($ancoraReal -lt 0) {
        return [PSCustomObject]@{ Status = 'sem-alinhamento'; Detalhe = "0/$($linhasGabarito.Count)" }
    }
    $puladasNoGabarito = $ancoraGab
    $linhasReais = @($linhasReais[$ancoraReal..($linhasReais.Count - 1)])
    $linhasGabarito = @($linhasGabarito[$ancoraGab..($linhasGabarito.Count - 1)])

    $total = [Math]::Max($linhasReais.Count, $linhasGabarito.Count) + $puladasNoGabarito
    $diferentes = $puladasNoGabarito
    for ($i = 0; $i -lt ($total - $puladasNoGabarito); $i++) {
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
