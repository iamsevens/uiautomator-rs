[CmdletBinding()]
param(
    [string[]]$Crates = @("uiautomator"),
    [string]$OutputRoot = "internal/testlogs/unit-coverage"
)

$ErrorActionPreference = "Stop"
[Console]::OutputEncoding = [System.Text.Encoding]::UTF8

function Get-RepoRoot {
    return (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
}

function Invoke-UnitCoverage {
    param(
        [string]$RepoRoot,
        [string]$CrateName,
        [string]$RunDir
    )

    $crateDir = Join-Path $RepoRoot $CrateName
    if (-not (Test-Path $crateDir)) {
        throw "Crate directory not found: $crateDir"
    }

    $jsonPath = Join-Path $RunDir "$CrateName.unit-coverage.json"
    $logPath = Join-Path $RunDir "$CrateName.unit-coverage.log"

    Write-Host ""
    Write-Host "==> Collecting unit coverage for $CrateName"

    Push-Location $crateDir
    try {
        $stdoutPath = Join-Path $RunDir "$CrateName.unit-coverage.stdout.log"
        $stderrPath = Join-Path $RunDir "$CrateName.unit-coverage.stderr.log"

        $process = Start-Process `
            -FilePath "cargo" `
            -ArgumentList @("llvm-cov", "--lib", "--json", "--summary-only", "--output-path", $jsonPath) `
            -WorkingDirectory $crateDir `
            -NoNewWindow `
            -Wait `
            -PassThru `
            -RedirectStandardOutput $stdoutPath `
            -RedirectStandardError $stderrPath

        $combinedLog = @()
        if (Test-Path $stdoutPath) {
            $combinedLog += Get-Content $stdoutPath
        }
        if (Test-Path $stderrPath) {
            $combinedLog += Get-Content $stderrPath
        }
        $combinedLog | Set-Content -Path $logPath -Encoding UTF8

        if ($process.ExitCode -ne 0) {
            throw "Failed to collect unit coverage for $CrateName. See log: $logPath"
        }

        $json = Get-Content $jsonPath -Raw | ConvertFrom-Json
        if (-not $json.data -or $json.data.Count -eq 0) {
            throw "Coverage JSON did not contain data for $CrateName"
        }

        $payload = $json.data[0]
        $totals = $payload.totals
        [PSCustomObject]@{
            crate = $CrateName
            generated_at = (Get-Date).ToString("s")
            manifest_path = $json.cargo_llvm_cov.manifest_path
            files = $payload.files.Count
            lines_percent = [Math]::Round([double]$totals.lines.percent, 2)
            lines_covered = [int]$totals.lines.covered
            lines_total = [int]$totals.lines.count
            functions_percent = [Math]::Round([double]$totals.functions.percent, 2)
            functions_covered = [int]$totals.functions.covered
            functions_total = [int]$totals.functions.count
            regions_percent = [Math]::Round([double]$totals.regions.percent, 2)
            regions_covered = [int]$totals.regions.covered
            regions_total = [int]$totals.regions.count
            top_low_files = @(
                $payload.files |
                    Sort-Object { [double]$_.summary.lines.percent }, filename |
                    Select-Object -First 5 |
                    ForEach-Object {
                        [PSCustomObject]@{
                            filename = $_.filename
                            lines_percent = [Math]::Round([double]$_.summary.lines.percent, 2)
                            lines_covered = [int]$_.summary.lines.covered
                            lines_total = [int]$_.summary.lines.count
                        }
                    }
            )
        }
    }
    finally {
        Pop-Location
    }
}

$repoRoot = Get-RepoRoot
$runId = Get-Date -Format "yyyyMMdd-HHmmss"
$runDir = Join-Path $repoRoot (Join-Path $OutputRoot $runId)
$latestJson = Join-Path $repoRoot (Join-Path $OutputRoot "latest-summary.json")
$latestMd = Join-Path $repoRoot (Join-Path $OutputRoot "latest-summary.md")
$summaryJson = Join-Path $runDir "unit-coverage-summary.json"
$summaryMd = Join-Path $runDir "unit-coverage-summary.md"

New-Item -ItemType Directory -Force -Path $runDir | Out-Null

$results = @()
foreach ($crate in $Crates) {
    $results += Invoke-UnitCoverage -RepoRoot $repoRoot -CrateName $crate -RunDir $runDir
}

$summary = [PSCustomObject]@{
    generated_at = (Get-Date).ToString("s")
    run_id = $runId
    crates = $results
}

$summary | ConvertTo-Json -Depth 6 | Set-Content -Path $summaryJson -Encoding UTF8
$summary | ConvertTo-Json -Depth 6 | Set-Content -Path $latestJson -Encoding UTF8

$builder = New-Object System.Text.StringBuilder
[void]$builder.AppendLine("# Unit Coverage Summary")
[void]$builder.AppendLine("")
[void]$builder.AppendLine("- Generated at: $($summary.generated_at)")
[void]$builder.AppendLine("- Run id: $runId")
[void]$builder.AppendLine("")

foreach ($crate in $results) {
    [void]$builder.AppendLine("## $($crate.crate)")
    [void]$builder.AppendLine("")
    [void]$builder.AppendLine("- Lines: $($crate.lines_percent)% ($($crate.lines_covered)/$($crate.lines_total))")
    [void]$builder.AppendLine("- Functions: $($crate.functions_percent)% ($($crate.functions_covered)/$($crate.functions_total))")
    [void]$builder.AppendLine("- Regions: $($crate.regions_percent)% ($($crate.regions_covered)/$($crate.regions_total))")
    [void]$builder.AppendLine("- Files analyzed: $($crate.files)")
    [void]$builder.AppendLine("")
    [void]$builder.AppendLine("### Lowest Line Coverage Files")
    [void]$builder.AppendLine("")
    foreach ($file in $crate.top_low_files) {
        [void]$builder.AppendLine("- `$($file.filename)`: $($file.lines_percent)% ($($file.lines_covered)/$($file.lines_total))")
    }
    [void]$builder.AppendLine("")
}

$markdown = $builder.ToString()
$markdown | Set-Content -Path $summaryMd -Encoding UTF8
$markdown | Set-Content -Path $latestMd -Encoding UTF8

Write-Host ""
Write-Host "Unit coverage summary generated."
Write-Host "json: $summaryJson"
Write-Host "markdown: $summaryMd"
foreach ($crate in $results) {
    Write-Host ("{0}: lines {1}% ({2}/{3}), functions {4}% ({5}/{6}), regions {7}% ({8}/{9})" -f `
        $crate.crate, `
        $crate.lines_percent, `
        $crate.lines_covered, `
        $crate.lines_total, `
        $crate.functions_percent, `
        $crate.functions_covered, `
        $crate.functions_total, `
        $crate.regions_percent, `
        $crate.regions_covered, `
        $crate.regions_total)
}
