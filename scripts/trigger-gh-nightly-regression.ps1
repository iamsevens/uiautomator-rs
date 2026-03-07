param(
    [string]$Repo = "iamsevens/uiautomator-rs",

    [string]$Ref = "main",

    [string]$GuiRunnerRoot = "D:\actions-runner-uiautomator-rs-gui",

    [string]$TargetsJson = "",

    [int]$StepTimeoutMinutes = 45,

    [int]$PollIntervalSeconds = 15,

    [int]$MaxWaitMinutes = 360,

    [switch]$AllowConcurrentRun
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

. (Join-Path $PSScriptRoot "github-runner-common.ps1")

if ($StepTimeoutMinutes -lt 1) {
    throw "StepTimeoutMinutes must be >= 1"
}
if ($PollIntervalSeconds -lt 5) {
    throw "PollIntervalSeconds must be >= 5"
}
if ($MaxWaitMinutes -lt 1) {
    throw "MaxWaitMinutes must be >= 1"
}

$workflowName = "Nightly Device Regression"
$refBranch = if ($Ref.StartsWith("refs/heads/")) { $Ref.Substring("refs/heads/".Length) } else { $Ref }
$utf8NoBom = [System.Text.UTF8Encoding]::new($false)

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

function Resolve-WorkflowId {
    $all = Invoke-GhJson -Arguments @(
        "workflow", "list",
        "--repo", $Repo,
        "--all",
        "--json", "id,name,state"
    )

    $workflow = $all | Where-Object { $_.name -eq $workflowName } | Select-Object -First 1
    if ($null -eq $workflow) {
        throw "workflow '$workflowName' not found in '$Repo'"
    }
    if ($workflow.state -ne "active") {
        throw "workflow '$workflowName' is not active (state=$($workflow.state))"
    }

    return [string]$workflow.id
}

function Ensure-NoActiveRun {
    if ($AllowConcurrentRun) {
        return
    }

    $runs = Invoke-GhJson -Arguments @(
        "run", "list",
        "--repo", $Repo,
        "--workflow", $workflowName,
        "--limit", "30",
        "--json", "databaseId,status,url,event,headBranch"
    )

    $active = $runs |
        Where-Object {
            $_.event -eq "workflow_dispatch" -and
            $_.headBranch -eq $refBranch -and
            $_.status -ne "completed"
        } |
        Select-Object -First 1

    if ($active) {
        throw "workflow '$workflowName' already has active run: $($active.url)"
    }
}

function Dispatch-Workflow {
    param(
        [Parameter(Mandatory = $true)]
        [string]$WorkflowId
    )

    $payload = [ordered]@{
        ref    = $Ref
        inputs = [ordered]@{
            step_timeout_minutes = [string]$StepTimeoutMinutes
        }
    }

    if (-not [string]::IsNullOrWhiteSpace($TargetsJson)) {
        $payload.inputs["targets_json"] = $TargetsJson
    }

    $payloadJson = $payload | ConvertTo-Json -Depth 20 -Compress
    $tempFile = Join-Path $env:TEMP ("gh-dispatch-nightly-{0}.json" -f [Guid]::NewGuid().ToString("N"))
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
        [datetime]$NotBeforeUtc
    )

    $deadline = [datetime]::UtcNow.AddMinutes(5)
    while ([datetime]::UtcNow -lt $deadline) {
        Start-Sleep -Seconds 3
        $runs = Invoke-GhJson -Arguments @(
            "run", "list",
            "--repo", $Repo,
            "--workflow", $workflowName,
            "--limit", "30",
            "--json", "databaseId,status,conclusion,url,createdAt,event,headBranch"
        )

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

    throw "cannot discover new '$workflowName' run within 5 minutes"
}

function Wait-RunCompletion {
    param(
        [Parameter(Mandatory = $true)]
        [string]$RunId
    )

    $deadline = (Get-Date).AddMinutes($MaxWaitMinutes)
    while ((Get-Date) -lt $deadline) {
        $run = Invoke-GhJson -Arguments @(
            "run", "view", $RunId,
            "--repo", $Repo,
            "--json", "status,conclusion,url,headSha"
        )

        Write-Host ("[{0}] nightly: status={1} conclusion={2}" -f (Get-Date -Format "HH:mm:ss"), $run.status, $run.conclusion)
        if ($run.status -eq "completed") {
            return $run
        }
        Start-Sleep -Seconds $PollIntervalSeconds
    }

    throw "nightly run timed out after $MaxWaitMinutes minutes"
}

Invoke-Gh -Arguments @("auth", "status") | Out-Null
Assert-GitHubRunnerSingleListener -RunnerRoot $GuiRunnerRoot | Out-Null
Wait-GitHubRunnerReady -RunnerRoot $GuiRunnerRoot | Out-Null
$workflowId = Resolve-WorkflowId
Ensure-NoActiveRun

$dispatchedAt = [datetime]::UtcNow
Dispatch-Workflow -WorkflowId $workflowId

$run = Wait-NewRun -NotBeforeUtc $dispatchedAt
Write-Host ("nightly run: {0}" -f $run.url)

$final = Wait-RunCompletion -RunId ([string]$run.databaseId)

$summary = [ordered]@{
    repo            = $Repo
    ref             = $Ref
    workflow        = $workflowName
    run_id          = [string]$run.databaseId
    run_url         = $final.url
    status          = $final.status
    conclusion      = $final.conclusion
    head_sha        = $final.headSha
    step_timeout    = $StepTimeoutMinutes
    targets_json    = $TargetsJson
}

if ($final.conclusion -ne "success") {
    Write-Host ""
    Write-Host "Failed step logs:"
    Invoke-Gh -Arguments @("run", "view", [string]$run.databaseId, "--repo", $Repo, "--log-failed") | Out-Host
    throw "nightly run failed: $($final.url)"
}

Write-Host ""
Write-Host "Nightly run completed successfully."
$summary | ConvertTo-Json -Depth 10
