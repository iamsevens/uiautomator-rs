param(
    [Parameter(Mandatory = $true)]
    [string]$Serial,

    [string]$Repo = "iamsevens/uiautomator-rs",

    [string]$Ref = "main",

    [string]$TargetName = "",

    [string]$ExpectedAbi = "",

    [int]$ExpectedAndroidMajor = 0,

    [int]$SmokeStepTimeoutMinutes = 20,

    [int]$MatrixStepTimeoutMinutes = 45,

    [int]$PollIntervalSeconds = 20,

    [int]$MaxWaitMinutes = 240
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

if ($PollIntervalSeconds -lt 5) {
    throw "PollIntervalSeconds must be >= 5"
}
if ($MaxWaitMinutes -lt 1) {
    throw "MaxWaitMinutes must be >= 1"
}

$utf8NoBom = [System.Text.UTF8Encoding]::new($false)
$refBranch = if ($Ref.StartsWith("refs/heads/")) { $Ref.Substring("refs/heads/".Length) } else { $Ref }

function Invoke-Gh {
    param(
        [Parameter(Mandatory = $true)]
        [string[]]$Arguments
    )

    $output = & gh @Arguments 2>&1
    $exitCode = $LASTEXITCODE
    $text = ($output | ForEach-Object { $_.ToString() }) -join "`n"

    if ($exitCode -ne 0) {
        throw "gh command failed (exit=$exitCode): gh $($Arguments -join ' ')`n$text"
    }

    return $text.Trim()
}

function Invoke-GhJson {
    param(
        [Parameter(Mandatory = $true)]
        [string[]]$Arguments
    )

    $text = Invoke-Gh -Arguments $Arguments
    if ([string]::IsNullOrWhiteSpace($text)) {
        return $null
    }

    try {
        return $text | ConvertFrom-Json
    }
    catch {
        throw "failed to parse JSON output from: gh $($Arguments -join ' ')`n$text"
    }
}

function Resolve-Workflow {
    param(
        [Parameter(Mandatory = $true)]
        [string]$WorkflowName
    )

    $all = Invoke-GhJson -Arguments @(
        "workflow", "list",
        "--repo", $Repo,
        "--all",
        "--json", "id,name,state,path"
    )

    $workflow = $all | Where-Object { $_.name -eq $WorkflowName } | Select-Object -First 1
    if ($null -eq $workflow) {
        throw "workflow '$WorkflowName' not found in '$Repo'"
    }
    if ($workflow.state -ne "active") {
        throw "workflow '$WorkflowName' is not active (state=$($workflow.state))"
    }

    return $workflow
}

function Dispatch-Workflow {
    param(
        [Parameter(Mandatory = $true)]
        [string]$WorkflowId,
        [Parameter(Mandatory = $true)]
        [hashtable]$Inputs
    )

    $payload = [ordered]@{
        ref    = $Ref
        inputs = [ordered]@{}
    }

    foreach ($key in $Inputs.Keys) {
        $payload.inputs[$key] = [string]$Inputs[$key]
    }

    $payloadJson = $payload | ConvertTo-Json -Compress -Depth 20
    $tempFile = Join-Path $env:TEMP ("gh-dispatch-{0}.json" -f [Guid]::NewGuid().ToString("N"))
    [System.IO.File]::WriteAllText($tempFile, $payloadJson, $utf8NoBom)

    try {
        Invoke-Gh -Arguments @(
            "api",
            ("repos/{0}/actions/workflows/{1}/dispatches" -f $Repo, $WorkflowId),
            "--method", "POST",
            "--input", $tempFile,
            "--silent"
        ) | Out-Null
    }
    finally {
        Remove-Item $tempFile -Force -ErrorAction SilentlyContinue
    }
}

function Wait-NewRun {
    param(
        [Parameter(Mandatory = $true)]
        [string]$WorkflowName,
        [Parameter(Mandatory = $true)]
        [datetime]$NotBeforeUtc,
        [int]$DiscoveryTimeoutSeconds = 180
    )

    $deadline = [datetime]::UtcNow.AddSeconds($DiscoveryTimeoutSeconds)
    while ([datetime]::UtcNow -lt $deadline) {
        $runs = Invoke-GhJson -Arguments @(
            "run", "list",
            "--repo", $Repo,
            "--workflow", $WorkflowName,
            "--limit", "20",
            "--json", "databaseId,status,conclusion,url,createdAt,event,headBranch"
        )

        if ($runs) {
            $candidate = $runs |
                Where-Object {
                    $_.event -eq "workflow_dispatch" -and
                    $_.headBranch -eq $refBranch -and
                    ([datetime]$_.createdAt).ToUniversalTime() -ge $NotBeforeUtc.AddSeconds(-5)
                } |
                Sort-Object { ([datetime]$_.createdAt).ToUniversalTime() } -Descending |
                Select-Object -First 1

            if ($candidate) {
                return $candidate
            }
        }

        Start-Sleep -Seconds 2
    }

    throw "unable to discover new run for workflow '$WorkflowName' within $DiscoveryTimeoutSeconds seconds"
}

function Wait-RunCompletion {
    param(
        [Parameter(Mandatory = $true)]
        [string]$RunId,
        [Parameter(Mandatory = $true)]
        [string]$Label
    )

    $deadline = (Get-Date).AddMinutes($MaxWaitMinutes)
    while ((Get-Date) -lt $deadline) {
        $run = Invoke-GhJson -Arguments @(
            "run", "view", $RunId,
            "--repo", $Repo,
            "--json", "status,conclusion,url"
        )

        Write-Host ("[{0}] {1}: status={2} conclusion={3}" -f (Get-Date -Format "HH:mm:ss"), $Label, $run.status, $run.conclusion)

        if ($run.status -eq "completed") {
            return $run
        }

        Start-Sleep -Seconds $PollIntervalSeconds
    }

    throw "$Label run timed out after $MaxWaitMinutes minutes"
}

try {
    Invoke-Gh -Arguments @("auth", "status") | Out-Null
}
catch {
    throw "gh auth is not ready. run: gh auth login"
}

if ([string]::IsNullOrWhiteSpace($TargetName)) {
    $TargetName = "target_{0}" -f ($Serial -replace "[^A-Za-z0-9._-]", "_")
}

$smokeWorkflow = Resolve-Workflow -WorkflowName "Install Smoke"
$matrixWorkflow = Resolve-Workflow -WorkflowName "Device Regression Matrix"

$target = [ordered]@{
    name   = $TargetName
    serial = $Serial
}

if (-not [string]::IsNullOrWhiteSpace($ExpectedAbi)) {
    $target.expected_abi = $ExpectedAbi
}
if ($ExpectedAndroidMajor -gt 0) {
    $target.expected_android_major = $ExpectedAndroidMajor
}

$targetJsonParts = @(
    ('"name":{0}' -f ($TargetName | ConvertTo-Json -Compress)),
    ('"serial":{0}' -f ($Serial | ConvertTo-Json -Compress))
)
if (-not [string]::IsNullOrWhiteSpace($ExpectedAbi)) {
    $targetJsonParts += ('"expected_abi":{0}' -f ($ExpectedAbi | ConvertTo-Json -Compress))
}
if ($ExpectedAndroidMajor -gt 0) {
    $targetJsonParts += ('"expected_android_major":{0}' -f $ExpectedAndroidMajor)
}
$targetsJson = "[{{{0}}}]" -f ($targetJsonParts -join ",")

Write-Host "Dispatching Install Smoke..."
$smokeDispatchedAt = [datetime]::UtcNow
Dispatch-Workflow -WorkflowId ([string]$smokeWorkflow.id) -Inputs @{
    serial              = $Serial
    target_name         = $TargetName
    step_timeout_minutes = [string]$SmokeStepTimeoutMinutes
}

$smokeRun = Wait-NewRun -WorkflowName $smokeWorkflow.name -NotBeforeUtc $smokeDispatchedAt
Write-Host ("Install Smoke run: {0}" -f $smokeRun.url)
$smokeFinal = Wait-RunCompletion -RunId ([string]$smokeRun.databaseId) -Label "Install Smoke"
if ($smokeFinal.conclusion -ne "success") {
    throw "Install Smoke failed: $($smokeFinal.url)"
}

Write-Host "Dispatching Device Regression Matrix..."
$matrixDispatchedAt = [datetime]::UtcNow
Dispatch-Workflow -WorkflowId ([string]$matrixWorkflow.id) -Inputs @{
    targets_json        = $targetsJson
    step_timeout_minutes = [string]$MatrixStepTimeoutMinutes
}

$matrixRun = Wait-NewRun -WorkflowName $matrixWorkflow.name -NotBeforeUtc $matrixDispatchedAt
Write-Host ("Device Regression Matrix run: {0}" -f $matrixRun.url)
$matrixFinal = Wait-RunCompletion -RunId ([string]$matrixRun.databaseId) -Label "Device Regression Matrix"
if ($matrixFinal.conclusion -ne "success") {
    throw "Device Regression Matrix failed: $($matrixFinal.url)"
}

$summary = [ordered]@{
    repo                 = $Repo
    ref                  = $Ref
    serial               = $Serial
    target_name          = $TargetName
    smoke_run_url        = $smokeFinal.url
    smoke_conclusion     = $smokeFinal.conclusion
    matrix_run_url       = $matrixFinal.url
    matrix_conclusion    = $matrixFinal.conclusion
    matrix_targets_json  = $targetsJson
}

Write-Host ""
Write-Host "All workflows completed successfully."
$summary | ConvertTo-Json -Depth 10
