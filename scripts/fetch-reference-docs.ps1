# Baixa capítulos do psx-spx (markdown) em commit pinado e os grava em docs/reference/
# com um índice de seções no topo — o agente lê o índice e pula direto à seção (R8).
# Rodar da raiz do repo. Re-rodar é idempotente; para atualizar a spec, mude $Sha e commite.
$ErrorActionPreference = "Stop"

$Sha = "035c7654b01128bf851938f21640bbe3cb29e1e2"
$Base = "https://raw.githubusercontent.com/psx-spx/psx-spx.github.io/$Sha/docs"

$Chapters = [ordered]@{
    "01-memory-map.md"           = "memorymap.md"
    "02-cpu.md"                  = "cpuspecifications.md"
    "03-gpu.md"                  = "graphicsprocessingunitgpu.md"
    "04-dma.md"                  = "dmachannels.md"
    "05-timers.md"               = "timers.md"
    "06-cdrom.md"                = "cdromdrive.md"
    "07-gte.md"                  = "geometrytransformationenginegte.md"
    "08-spu.md"                  = "soundprocessingunitspu.md"
    "09-mdec.md"                 = "macroblockdecodermdec.md"
    "10-controllers-memcards.md" = "controllersandmemorycards.md"
    "11-interrupts.md"           = "interrupts.md"
    "12-memory-control.md"       = "memorycontrol.md"
    "13-kernel-bios.md"          = "kernelbios.md"
    "14-io-map.md"               = "iomap.md"
    "15-cdrom-format.md"         = "cdromformat.md"
}

$OutDir = "docs/reference"
New-Item -ItemType Directory -Force $OutDir | Out-Null

foreach ($entry in $Chapters.GetEnumerator()) {
    $local = $entry.Key
    $remote = $entry.Value
    $raw = (Invoke-WebRequest "$Base/$remote" -UseBasicParsing).Content
    $lines = $raw -split "`n"

    $index = [System.Collections.Generic.List[string]]::new()
    $headerSize = 0
    for ($i = 0; $i -lt $lines.Count; $i++) {
        if ($lines[$i] -match '^(#{1,5})\s+(.*)') {
            $depth = "  " * [Math]::Max(0, $Matches[1].Length - 4)
            $title = $Matches[2].Trim()
            $index.Add("$depth- L$($i + 1): $title")
        }
    }

    $head = @(
        "<!-- FONTE: psx-spx/psx-spx.github.io@$($Sha.Substring(0,7)) docs/$remote"
        "     Gerado por scripts/fetch-reference-docs.ps1 — NAO editar a mao."
        "     R8: leia o indice abaixo e abra SO a secao necessaria (offsets sao"
        "     relativos ao corpo, que comeca em CORPO:). -->"
        ""
        "## Indice de secoes"
        ""
    ) + $index + @("", "CORPO:", "")

    Set-Content -NoNewline -Path "$OutDir/$local" -Value (($head + $lines) -join "`n")
    Write-Host ("{0,-28} {1,8:n0} bytes, {2} secoes" -f $local, $raw.Length, $index.Count)
}

$readme = @(
    "# docs/reference — especificacoes de hardware"
    ""
    "Capitulos do psx-spx (nocash PSXSPX, https://psx-spx.consoledev.net) baixados de"
    "``psx-spx/psx-spx.github.io`` no commit ``$Sha``."
    "Gerados por ``scripts/fetch-reference-docs.ps1`` — para atualizar, mude o SHA no script,"
    "rode-o e commite a diferenca. Nao editar os arquivos a mao."
    ""
    "Cada arquivo comeca com um indice de secoes (``L<n>: titulo``, offset relativo a marca"
    "``CORPO:``). Leia o indice, depois SO a secao necessaria (R8 do CLAUDE.md)."
    ""
    "| Arquivo local | Capitulo original |"
    "|---|---|"
) + ($Chapters.GetEnumerator() | ForEach-Object { "| $($_.Key) | docs/$($_.Value) |" })
Set-Content -Path "$OutDir/README.md" -Value ($readme -join "`n")
Write-Host "OK: $($Chapters.Count) capitulos + README em $OutDir"
