param(
    [string]$OutputRoot = "internal/testlogs/docs",
    [double]$MinDocsPercent = 99.0,
    [double]$MinExamplesPercent = 55.0,
    [switch]$FailOnThreshold
)

$ErrorActionPreference = "Stop"
Set-StrictMode -Version Latest

$utf8NoBom = New-Object System.Text.UTF8Encoding($false)
[Console]::InputEncoding = $utf8NoBom
[Console]::OutputEncoding = $utf8NoBom
$OutputEncoding = $utf8NoBom
$PSDefaultParameterValues["Out-File:Encoding"] = "utf8"
$PSDefaultParameterValues["Set-Content:Encoding"] = "utf8"
$PSDefaultParameterValues["Add-Content:Encoding"] = "utf8"

$repoRoot = Split-Path -Parent $PSScriptRoot
Set-Location $repoRoot

$timestamp = Get-Date -Format "yyyyMMdd-HHmmss"
$outputDir = Join-Path $repoRoot (Join-Path $OutputRoot $timestamp)
New-Item -ItemType Directory -Force -Path $outputDir | Out-Null

$crateConfigs = @(
    @{ Name = "uiautomator"; Path = "uiautomator" },
    @{ Name = "uiautomator-cli"; Path = "uiautomator-cli" }
)

function Estimate-TotalItems {
    param(
        [int]$DocumentedCount,
        [double]$Percent
    )

    if ($Percent -le 0) {
        return 0
    }

    return [int][Math]::Round(($DocumentedCount * 100.0) / $Percent, 0)
}

function Parse-CoverageOutput {
    param([string[]]$Lines)

    $rowRegex = '^\|\s*(?<name>[^|]+?)\s*\|\s*(?<doc>\d+)\s*\|\s*(?<docp>\d+(?:\.\d+)?)%\s*\|\s*(?<ex>\d+)\s*\|\s*(?<exp>\d+(?:\.\d+)?)%\s*\|'
    $rows = New-Object System.Collections.Generic.List[object]
    $total = $null

    foreach ($line in $Lines) {
        if ($line -notmatch $rowRegex) {
            continue
        }

        $name = $matches.name.Trim()
        $docCount = [int]$matches.doc
        $docPercent = [double]$matches.docp
        $exampleCount = [int]$matches.ex
        $examplePercent = [double]$matches.exp

        if ($name -eq "Total") {
            $total = [PSCustomObject]@{
                documented_items = $docCount
                documented_percent = $docPercent
                example_items = $exampleCount
                example_percent = $examplePercent
            }
            continue
        }

        $rows.Add([PSCustomObject]@{
            name = $name
            documented_items = $docCount
            documented_percent = $docPercent
            example_items = $exampleCount
            example_percent = $examplePercent
            total_items_estimated = Estimate-TotalItems -DocumentedCount $docCount -Percent $docPercent
            example_total_estimated = Estimate-TotalItems -DocumentedCount $exampleCount -Percent $examplePercent
        })
    }

    if ($null -eq $total) {
        throw "Unable to parse coverage total row from rustdoc output."
    }

    return [PSCustomObject]@{
        total = $total
        rows = @($rows.ToArray())
    }
}

function Run-CrateCoverage {
    param(
        [string]$CrateName,
        [string]$CratePath
    )

    Write-Host ""
    Write-Host "==> Collecting docs coverage for $CrateName"

    Push-Location $CratePath
    try {
        $stdoutPath = Join-Path $outputDir "$CrateName.rustdoc.stdout.log"
        $stderrPath = Join-Path $outputDir "$CrateName.rustdoc.stderr.log"

        $process = Start-Process `
            -FilePath "cargo" `
            -ArgumentList @("+nightly", "rustdoc", "--lib", "--", "-Z", "unstable-options", "--show-coverage") `
            -NoNewWindow `
            -Wait `
            -PassThru `
            -RedirectStandardOutput $stdoutPath `
            -RedirectStandardError $stderrPath
        $exitCode = $process.ExitCode
    }
    finally {
        Pop-Location
    }

    $stdoutLines = if (Test-Path $stdoutPath) { Get-Content -Path $stdoutPath -Encoding utf8 } else { @() }
    $stderrLines = if (Test-Path $stderrPath) { Get-Content -Path $stderrPath -Encoding utf8 } else { @() }
    $lines = @($stdoutLines + $stderrLines | ForEach-Object { "$_" })
    $logPath = Join-Path $outputDir "$CrateName.coverage.log"
    $lines | Set-Content -Path $logPath -Encoding utf8

    if ($exitCode -ne 0) {
        throw "Failed to run rustdoc coverage for $CrateName. See log: $logPath"
    }

    $parsed = Parse-CoverageOutput -Lines $lines
    $total = $parsed.total

    $totalItemsEstimated = Estimate-TotalItems -DocumentedCount $total.documented_items -Percent $total.documented_percent
    $exampleTotalEstimated = Estimate-TotalItems -DocumentedCount $total.example_items -Percent $total.example_percent

    return [PSCustomObject]@{
        crate = $CrateName
        crate_path = $CratePath
        documented_items = $total.documented_items
        total_items_estimated = $totalItemsEstimated
        documented_percent = $total.documented_percent
        example_items = $total.example_items
        example_total_estimated = $exampleTotalEstimated
        example_percent = $total.example_percent
        log_path = $logPath
        by_file = $parsed.rows
    }
}

function Build-MarkdownReport {
    param(
        [string]$Path,
        [object]$Summary
    )

    $builder = New-Object System.Text.StringBuilder
    [void]$builder.AppendLine("# Docs Coverage Report")
    [void]$builder.AppendLine("")
    [void]$builder.AppendLine("- Generated at: $($Summary.generated_at)")
    [void]$builder.AppendLine("- Docs coverage: $($Summary.aggregate.documented_percent)% ($($Summary.aggregate.documented_items) / $($Summary.aggregate.total_items_estimated), estimated total)")
    [void]$builder.AppendLine("- Examples coverage: $($Summary.aggregate.example_percent)% ($($Summary.aggregate.example_items) / $($Summary.aggregate.example_total_estimated), estimated total)")
    [void]$builder.AppendLine("")
    [void]$builder.AppendLine("| Crate | Docs | Doc % | Examples | Ex % |")
    [void]$builder.AppendLine("| --- | ---: | ---: | ---: | ---: |")

    foreach ($crate in $Summary.crates) {
        [void]$builder.AppendLine(
            ("| {0} | {1}/{2} | {3}% | {4}/{5} | {6}% |" -f
                $crate.crate,
                $crate.documented_items,
                $crate.total_items_estimated,
                $crate.documented_percent,
                $crate.example_items,
                $crate.example_total_estimated,
                $crate.example_percent)
        )
    }

    [void]$builder.AppendLine("")
    [void]$builder.AppendLine("## Thresholds")
    [void]$builder.AppendLine("")
    [void]$builder.AppendLine("- Min docs percent: $($Summary.thresholds.min_docs_percent)%")
    [void]$builder.AppendLine("- Min examples percent: $($Summary.thresholds.min_examples_percent)%")
    $thresholdMode = if ($Summary.thresholds.fail_on_threshold) { "enforced" } else { "report-only" }
    [void]$builder.AppendLine("- Threshold mode: $thresholdMode")
    [void]$builder.AppendLine("")
    [void]$builder.AppendLine("## Logs")
    [void]$builder.AppendLine("")

    foreach ($crate in $Summary.crates) {
        [void]$builder.AppendLine(("- {0}: {1}" -f $crate.crate, $crate.log_path))
    }

    $builder.ToString() | Set-Content -Path $Path -Encoding utf8
}

$crateResults = New-Object System.Collections.Generic.List[object]
foreach ($crateConfig in $crateConfigs) {
    $result = Run-CrateCoverage -CrateName $crateConfig.Name -CratePath $crateConfig.Path
    $crateResults.Add($result)
}

$docCount = 0
$docTotalEstimated = 0
$exampleCount = 0
$exampleTotalEstimated = 0
foreach ($crate in $crateResults) {
    $docCount += [int]$crate.documented_items
    $docTotalEstimated += [int]$crate.total_items_estimated
    $exampleCount += [int]$crate.example_items
    $exampleTotalEstimated += [int]$crate.example_total_estimated
}

$aggregateDocPercent = if ($docTotalEstimated -gt 0) {
    [Math]::Round(($docCount * 100.0) / $docTotalEstimated, 2)
}
else {
    0
}

$aggregateExamplePercent = if ($exampleTotalEstimated -gt 0) {
    [Math]::Round(($exampleCount * 100.0) / $exampleTotalEstimated, 2)
}
else {
    0
}

$summary = [PSCustomObject]@{
    schema_version = 1
    generated_at = (Get-Date).ToString("o")
    output_dir = $outputDir
    thresholds = [PSCustomObject]@{
        min_docs_percent = $MinDocsPercent
        min_examples_percent = $MinExamplesPercent
        fail_on_threshold = [bool]$FailOnThreshold
    }
    aggregate = [PSCustomObject]@{
        documented_items = $docCount
        total_items_estimated = $docTotalEstimated
        documented_percent = $aggregateDocPercent
        example_items = $exampleCount
        example_total_estimated = $exampleTotalEstimated
        example_percent = $aggregateExamplePercent
    }
    crates = @($crateResults.ToArray())
}

$jsonPath = Join-Path $outputDir "docs-coverage-summary.json"
$mdPath = Join-Path $outputDir "docs-coverage-summary.md"

$summary | ConvertTo-Json -Depth 8 | Set-Content -Path $jsonPath -Encoding utf8
Build-MarkdownReport -Path $mdPath -Summary $summary

$latestJson = Join-Path (Join-Path $repoRoot $OutputRoot) "latest-summary.json"
$latestMd = Join-Path (Join-Path $repoRoot $OutputRoot) "latest-summary.md"
$summary | ConvertTo-Json -Depth 8 | Set-Content -Path $latestJson -Encoding utf8
Get-Content $mdPath -Encoding utf8 | Set-Content -Path $latestMd -Encoding utf8

Write-Host ""
Write-Host "Docs coverage summary generated."
Write-Host ("json: {0}" -f $jsonPath)
Write-Host ("markdown: {0}" -f $mdPath)
Write-Host ("aggregate docs: {0}% ({1}/{2})" -f $aggregateDocPercent, $docCount, $docTotalEstimated)
Write-Host ("aggregate examples: {0}% ({1}/{2})" -f $aggregateExamplePercent, $exampleCount, $exampleTotalEstimated)

if ($FailOnThreshold) {
    $thresholdErrors = New-Object System.Collections.Generic.List[string]
    if ($aggregateDocPercent -lt $MinDocsPercent) {
        $thresholdErrors.Add("docs coverage $aggregateDocPercent% is below threshold $MinDocsPercent%")
    }
    if ($aggregateExamplePercent -lt $MinExamplesPercent) {
        $thresholdErrors.Add("examples coverage $aggregateExamplePercent% is below threshold $MinExamplesPercent%")
    }

    if ($thresholdErrors.Count -gt 0) {
        throw ($thresholdErrors -join "; ")
    }
}
