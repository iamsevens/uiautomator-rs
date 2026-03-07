param(
    [string]$Repo = "iamsevens/uiautomator-rs",

    [string]$Ref = "main",

    [int]$PollIntervalSeconds = 15,

    [int]$MaxWaitMinutes = 120,

    [int]$GhRetryCount = 6,

    [int]$GhRetryDelaySeconds = 3,

    [switch]$AllowConcurrentRun
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

if ($PollIntervalSeconds -lt 5) {
    throw "PollIntervalSeconds must be >= 5"
}
if ($MaxWaitMinutes -lt 1) {
    throw "MaxWaitMinutes must be >= 1"
}
if ($GhRetryCount -lt 1) {
    throw "GhRetryCount must be >= 1"
}
if ($GhRetryDelaySeconds -lt 1) {
    throw "GhRetryDelaySeconds must be >= 1"
}

$refBranch = if ($Ref.StartsWith("refs/heads/")) { $Ref.Substring("refs/heads/".Length) } else { $Ref }

$workflowSequence = @(
    [PSCustomObject]@{
        Name     = "Release Check"
        Selector = "release-check.yml"
    },
    [PSCustomObject]@{
        Name     = "Publish Dry Run"
        Selector = "publish-dry-run.yml"
    }
)

function Test-TransientGhFailure {
    param([string]$Text)

    if ([string]::IsNullOrWhiteSpace($Text)) {
        return $false
    }

    return $Text -match 'HTTP 5\d\d' -or
    $Text -match 'timed out' -or
    $Text -match 'unexpected EOF' -or
    $Text -match 'connection (reset|closed)' -or
    $Text -match 'TLS' -or
    $Text -match 'temporary failure'
}

function Invoke-Gh {
    param(
        [Parameter(Mandatory = $true)]
        [string[]]$Arguments
    )

    for ($attempt = 1; $attempt -le $GhRetryCount; $attempt++) {
        $output = & gh @Arguments 2>&1
        $exitCode = $LASTEXITCODE
        $text = ($output | ForEach-Object { $_.ToString() }) -join "`n"
        if ($exitCode -eq 0) {
            return $text.Trim()
        }

        if (($attempt -lt $GhRetryCount) -and (Test-TransientGhFailure -Text $text)) {
            Write-Host ("[gh retry {0}/{1}] transient error: {2}" -f $attempt, $GhRetryCount, ($text -replace "`r?`n", " "))
            Start-Sleep -Seconds $GhRetryDelaySeconds
            continue
        }

        throw "gh command failed (exit=$exitCode): gh $($Arguments -join ' ')`n$text"
    }

    throw "gh command failed unexpectedly: gh $($Arguments -join ' ')"
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

function Ensure-NoActiveRun {
    param(
        [Parameter(Mandatory = $true)]
        [object]$Workflow
    )

    if ($AllowConcurrentRun) {
        return
    }

    $runs = Invoke-GhJson -Arguments @(
        "run", "list",
        "--repo", $Repo,
        "--workflow", $Workflow.Selector,
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
        throw ("workflow '{0}' already has active run: {1}" -f $Workflow.Name, $active.url)
    }
}

function Dispatch-Workflow {
    param(
        [Parameter(Mandatory = $true)]
        [object]$Workflow
    )

    Invoke-Gh -Arguments @(
        "workflow", "run", $Workflow.Selector,
        "--repo", $Repo,
        "--ref", $Ref
    ) | Out-Null
}

function Wait-NewRun {
    param(
        [Parameter(Mandatory = $true)]
        [object]$Workflow,
        [Parameter(Mandatory = $true)]
        [datetime]$NotBeforeUtc
    )

    $deadline = [datetime]::UtcNow.AddMinutes(5)
    while ([datetime]::UtcNow -lt $deadline) {
        Start-Sleep -Seconds 3
        $runs = Invoke-GhJson -Arguments @(
            "run", "list",
            "--repo", $Repo,
            "--workflow", $Workflow.Selector,
            "--limit", "30",
            "--json", "databaseId,status,conclusion,url,createdAt,event,headBranch,name"
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

    throw "cannot discover new run for '$($Workflow.Name)' within 5 minutes"
}

function Wait-RunCompletion {
    param(
        [Parameter(Mandatory = $true)]
        [object]$Workflow,
        [Parameter(Mandatory = $true)]
        [string]$RunId
    )

    $deadline = (Get-Date).AddMinutes($MaxWaitMinutes)
    while ((Get-Date) -lt $deadline) {
        $run = Invoke-GhJson -Arguments @(
            "run", "view", $RunId,
            "--repo", $Repo,
            "--json", "status,conclusion,url,headSha,updatedAt,jobs"
        )

        Write-Host ("[{0}] {1}: status={2} conclusion={3}" -f (Get-Date -Format "HH:mm:ss"), $Workflow.Name, $run.status, $run.conclusion)
        if ($run.status -eq "completed") {
            return $run
        }

        Start-Sleep -Seconds $PollIntervalSeconds
    }

    throw ("run timeout after {0} minutes: {1}" -f $MaxWaitMinutes, $Workflow.Name)
}

Invoke-Gh -Arguments @("auth", "status") | Out-Null

$results = New-Object System.Collections.Generic.List[object]

foreach ($workflow in $workflowSequence) {
    Write-Host ""
    Write-Host ("=== {0} ===" -f $workflow.Name)

    Ensure-NoActiveRun -Workflow $workflow
    $dispatchTime = [datetime]::UtcNow
    Dispatch-Workflow -Workflow $workflow
    $run = Wait-NewRun -Workflow $workflow -NotBeforeUtc $dispatchTime
    Write-Host ("{0} run: {1}" -f $workflow.Name, $run.url)

    $final = Wait-RunCompletion -Workflow $workflow -RunId ([string]$run.databaseId)
    $record = [ordered]@{
        workflow_name = $workflow.Name
        workflow_file = $workflow.Selector
        run_id        = [string]$run.databaseId
        run_url       = $final.url
        status        = $final.status
        conclusion    = $final.conclusion
        updated_at    = $final.updatedAt
        head_sha      = $final.headSha
    }
    $results.Add([PSCustomObject]$record)

    if ($final.conclusion -ne "success") {
        Write-Host ""
        Write-Host ("Failed step logs for {0}:" -f $workflow.Name)
        Invoke-Gh -Arguments @(
            "run", "view", [string]$run.databaseId,
            "--repo", $Repo,
            "--log-failed"
        ) | Out-Host
        throw ("workflow failed: {0}" -f $final.url)
    }
}

$summary = [ordered]@{
    repo       = $Repo
    ref        = $Ref
    started_at = (Get-Date).ToString("o")
    results    = $results.ToArray()
}

Write-Host ""
Write-Host "Release gate workflows completed successfully."
$summary | ConvertTo-Json -Depth 10
