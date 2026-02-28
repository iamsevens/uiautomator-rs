param(
    [string]$OutputRoot = "internal/testlogs/api-coverage",
    [switch]$FailOnUncovered
)

$ErrorActionPreference = "Stop"
Set-StrictMode -Version Latest

$utf8NoBom = New-Object System.Text.UTF8Encoding($false)
[Console]::InputEncoding = $utf8NoBom
[Console]::OutputEncoding = $utf8NoBom
$OutputEncoding = $utf8NoBom

$repoRoot = Split-Path -Parent $PSScriptRoot
Set-Location $repoRoot

function Get-BraceDelta {
    param([string]$Line)
    $openCount = ([regex]::Matches($Line, "\{")).Count
    $closeCount = ([regex]::Matches($Line, "\}")).Count
    return ($openCount - $closeCount)
}

function Get-ExportedTypeNames {
    param([string]$LibRsPath)

    $exported = New-Object System.Collections.Generic.HashSet[string]
    $lines = Get-Content -Path $LibRsPath -Encoding utf8

    foreach ($line in $lines) {
        $trimmed = $line.Trim()
        if ($trimmed -match '^pub\s+use\s+[^;]+\{(?<items>[^}]+)\}\s*;') {
            $items = $matches.items -split ","
            foreach ($rawItem in $items) {
                $item = $rawItem.Trim()
                if (-not $item) {
                    continue
                }

                if ($item -match '^(?<name>[A-Za-z_][A-Za-z0-9_]*)\s+as\s+[A-Za-z_][A-Za-z0-9_]*$') {
                    [void]$exported.Add($matches.name)
                }
                elseif ($item -match '^[A-Za-z_][A-Za-z0-9_]*$') {
                    [void]$exported.Add($item)
                }
            }
            continue
        }

        if ($trimmed -match '^pub\s+use\s+[^:]+::(?<item>[A-Za-z_][A-Za-z0-9_]*)\s*;') {
            [void]$exported.Add($matches.item)
        }
    }

    return @($exported | Sort-Object)
}

function Get-PublicApiEntries {
    param(
        [string]$SourceDir,
        [string[]]$ExportedTypes
    )

    $exportedTypeSet = New-Object System.Collections.Generic.HashSet[string]
    foreach ($typeName in $ExportedTypes) {
        [void]$exportedTypeSet.Add($typeName)
    }

    $entries = New-Object System.Collections.Generic.List[object]
    $files = Get-ChildItem -Path $SourceDir -Filter *.rs -File | Sort-Object Name

    foreach ($file in $files) {
        $module = [System.IO.Path]::GetFileNameWithoutExtension($file.Name)
        $lines = Get-Content -Path $file.FullName -Encoding utf8

        $currentImplType = $null
        $currentImplTypeExported = $false
        $implBraceDepth = 0
        $awaitingImplBody = $false

        for ($i = 0; $i -lt $lines.Count; $i++) {
            $line = $lines[$i]
            $lineNumber = $i + 1
            $lineTrimmed = $line.Trim()
            $implSetThisLine = $false

            if ($null -eq $currentImplType) {
                if ($lineTrimmed -match '^impl(?:<[^>]*>)?\s+(?<type>[A-Za-z_][A-Za-z0-9_]*)(?:<[^>]*>)?') {
                    $candidateType = $matches.type
                    $currentImplType = $candidateType
                    $currentImplTypeExported = $exportedTypeSet.Contains($candidateType)
                    $awaitingImplBody = ($line -notmatch '\{')
                    $implBraceDepth = if ($awaitingImplBody) { 0 } else { Get-BraceDelta -Line $line }
                    $implSetThisLine = $true
                }
            }

            if ($lineTrimmed -match '^pub\s+(?:async\s+)?fn\s+(?<method>[A-Za-z_][A-Za-z0-9_]*)\s*\(') {
                $methodName = $matches.method

                if ($null -ne $currentImplType -and $currentImplTypeExported -and -not $awaitingImplBody) {
                    $entries.Add([PSCustomObject]@{
                            ApiId      = "$currentImplType::$methodName"
                            Type       = $currentImplType
                            Method     = $methodName
                            Module     = $module
                            SourceFile = $file.FullName.Replace($repoRoot + "\", "")
                            SourceLine = $lineNumber
                            Signature  = $lineTrimmed
                        })
                }
                elseif ($module -eq "lib") {
                    $entries.Add([PSCustomObject]@{
                            ApiId      = "lib::$methodName"
                            Type       = ""
                            Method     = $methodName
                            Module     = $module
                            SourceFile = $file.FullName.Replace($repoRoot + "\", "")
                            SourceLine = $lineNumber
                            Signature  = $lineTrimmed
                        })
                }
            }

            if ($null -ne $currentImplType) {
                if ($awaitingImplBody) {
                    if (-not $implSetThisLine -and $line -match '\{') {
                        $awaitingImplBody = $false
                        $implBraceDepth += Get-BraceDelta -Line $line
                    }
                }
                else {
                    if (-not $implSetThisLine) {
                        $implBraceDepth += Get-BraceDelta -Line $line
                    }

                    if ($implBraceDepth -le 0) {
                        $currentImplType = $null
                        $currentImplTypeExported = $false
                        $implBraceDepth = 0
                        $awaitingImplBody = $false
                    }
                }
            }
        }
    }

    $uniqueById = @{}
    foreach ($entry in $entries) {
        $key = "$($entry.SourceFile):$($entry.SourceLine):$($entry.ApiId)"
        $uniqueById[$key] = $entry
    }

    return @($uniqueById.Values | Sort-Object ApiId, SourceFile, SourceLine)
}

function Get-TestCasesFromFile {
    param([string]$FilePath)

    $raw = Get-Content -Path $FilePath -Encoding utf8 -Raw
    $pattern = '(?ms)#\[(?:tokio::)?test[^\]]*\](?:\s*#\[[^\]]+\])*?\s*(?:pub\s+)?(?:async\s+)?fn\s+([A-Za-z_][A-Za-z0-9_]*)\s*\('
    $matches = [regex]::Matches($raw, $pattern)

    $cases = New-Object System.Collections.Generic.List[object]
    for ($idx = 0; $idx -lt $matches.Count; $idx++) {
        $current = $matches[$idx]
        $testName = $current.Groups[1].Value
        $start = $current.Index
        $end = if ($idx + 1 -lt $matches.Count) { $matches[$idx + 1].Index } else { $raw.Length }
        $body = $raw.Substring($start, $end - $start)

        $cases.Add([PSCustomObject]@{
                Name = $testName
                File = $FilePath.Replace($repoRoot + "\", "")
                Body = $body
            })
    }

    return ,$cases
}

function Get-TestCases {
    param([string[]]$Roots)

    $allCases = New-Object System.Collections.Generic.List[object]

    foreach ($root in $Roots) {
        $absRoot = Join-Path $repoRoot $root
        if (-not (Test-Path $absRoot)) {
            continue
        }

        $files = Get-ChildItem -Path $absRoot -Recurse -File -Filter *.rs | Sort-Object FullName
        foreach ($file in $files) {
            $cases = Get-TestCasesFromFile -FilePath $file.FullName
            foreach ($case in $cases) {
                $allCases.Add($case)
            }
        }
    }

    return ,$allCases
}

function Find-TestMatchesForApi {
    param(
        [PSCustomObject]$ApiEntry,
        [System.Collections.Generic.List[object]]$TestCases
    )

    $methodName = [regex]::Escape($ApiEntry.Method)
    $patterns = New-Object System.Collections.Generic.List[string]

    if ($ApiEntry.Type) {
        $typeName = [regex]::Escape($ApiEntry.Type)

        if ($ApiEntry.Method -eq "new") {
            $patterns.Add("\b$typeName\s*::\s*$methodName\s*\(")
        }
        else {
            $patterns.Add("\b$typeName\s*::\s*$methodName\s*\(")
            $patterns.Add("\.\s*$methodName\s*\(")
        }
    }
    else {
        $patterns.Add("\b(?:uiautomator::)?$methodName\s*\(")
    }

    $matched = New-Object System.Collections.Generic.HashSet[string]
    foreach ($case in $TestCases) {
        foreach ($pattern in $patterns) {
            if ($case.Body -match $pattern) {
                [void]$matched.Add("$($case.File)::$($case.Name)")
                break
            }
        }
    }

    return ,@($matched | Sort-Object)
}

function Write-MarkdownReport {
    param(
        [string]$Path,
        [object]$Summary,
        [object[]]$Rows
    )

    $coveredPercent = if ($Summary.total_apis -gt 0) {
        [Math]::Round(($Summary.covered_apis * 100.0 / $Summary.total_apis), 2)
    }
    else {
        0
    }

    $builder = New-Object System.Text.StringBuilder
    [void]$builder.AppendLine("# API Coverage Mapping Report")
    [void]$builder.AppendLine("")
    [void]$builder.AppendLine("- Generated At: $($Summary.generated_at)")
    [void]$builder.AppendLine("- Total APIs: $($Summary.total_apis)")
    [void]$builder.AppendLine("- Covered APIs: $($Summary.covered_apis)")
    [void]$builder.AppendLine("- Uncovered APIs: $($Summary.uncovered_apis)")
    [void]$builder.AppendLine("- Coverage: $coveredPercent%")
    [void]$builder.AppendLine("")
    [void]$builder.AppendLine("## Uncovered APIs")
    [void]$builder.AppendLine("")

    $uncoveredRows = @($Rows | Where-Object { -not $_.covered } | Sort-Object api_id)
    if ($uncoveredRows.Count -eq 0) {
        [void]$builder.AppendLine("All APIs are mapped to at least one test case.")
    }
    else {
        foreach ($row in $uncoveredRows) {
            [void]$builder.AppendLine("- ``$($row.api_id)`` (`$($row.source_file):$($row.source_line)`) ")
        }
    }

    [void]$builder.AppendLine("")
    [void]$builder.AppendLine("## API -> Test Mapping")
    [void]$builder.AppendLine("")
    [void]$builder.AppendLine("| API | Covered | Tests | Source |")
    [void]$builder.AppendLine("| --- | --- | --- | --- |")

    foreach ($row in ($Rows | Sort-Object api_id)) {
        $testsText = if ($row.covered) { ($row.tests -join "<br/>") } else { "-" }
        $coveredText = if ($row.covered) { "yes" } else { "no" }
        $sourceText = "$($row.source_file):$($row.source_line)"
        [void]$builder.AppendLine("| `$($row.api_id)` | $coveredText | $testsText | `$sourceText` |")
    }

    $builder.ToString() | Set-Content -Path $Path -Encoding utf8
}

$runId = Get-Date -Format "yyyyMMdd-HHmmss"
$outputDir = Join-Path $repoRoot (Join-Path $OutputRoot $runId)
New-Item -ItemType Directory -Force -Path $outputDir | Out-Null

$libRsPath = Join-Path $repoRoot "uiautomator/src/lib.rs"
$exportedTypes = Get-ExportedTypeNames -LibRsPath $libRsPath
$apiEntries = Get-PublicApiEntries -SourceDir (Join-Path $repoRoot "uiautomator/src") -ExportedTypes $exportedTypes

$testCases = Get-TestCases -Roots @(
    "uiautomator/tests",
    "uiautomator/src",
    "uiautomator-cli/tests"
)

$rows = New-Object System.Collections.Generic.List[object]
foreach ($apiEntry in $apiEntries) {
    $matches = @(
        Find-TestMatchesForApi -ApiEntry $apiEntry -TestCases $testCases
    )
    $rows.Add([PSCustomObject]@{
            api_id      = $apiEntry.ApiId
            type        = $apiEntry.Type
            method      = $apiEntry.Method
            module      = $apiEntry.Module
            source_file = $apiEntry.SourceFile
            source_line = $apiEntry.SourceLine
            signature   = $apiEntry.Signature
            covered     = ($matches.Count -gt 0)
            tests       = $matches
            test_count  = $matches.Count
        })
}

$totalApis = $rows.Count
$coveredApis = @($rows | Where-Object { $_.covered }).Count
$uncoveredApis = $totalApis - $coveredApis
$coveragePercent = if ($totalApis -gt 0) { [Math]::Round(($coveredApis * 100.0 / $totalApis), 2) } else { 0 }

$summary = [ordered]@{
    schema_version   = 1
    generated_at     = (Get-Date).ToString("o")
    output_dir       = $outputDir
    total_apis       = $totalApis
    covered_apis     = $coveredApis
    uncovered_apis   = $uncoveredApis
    coverage_percent = $coveragePercent
    exported_types   = $exportedTypes
    api_mapping      = $rows.ToArray()
}

$jsonPath = Join-Path $outputDir "api-coverage.json"
$mdPath = Join-Path $outputDir "api-coverage.md"

$summary | ConvertTo-Json -Depth 10 | Set-Content -Path $jsonPath -Encoding utf8
Write-MarkdownReport -Path $mdPath -Summary $summary -Rows $rows.ToArray()

Write-Host "API coverage mapping generated."
Write-Host "json: $jsonPath"
Write-Host "markdown: $mdPath"
Write-Host ("coverage: {0}% ({1}/{2})" -f $coveragePercent, $coveredApis, $totalApis)
Write-Host ("uncovered apis: {0}" -f $uncoveredApis)

if ($FailOnUncovered -and $uncoveredApis -gt 0) {
    throw "API coverage mapping found $uncoveredApis uncovered public APIs."
}
