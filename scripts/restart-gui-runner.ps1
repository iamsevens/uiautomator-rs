param(
    [string]$RunnerRoot = "D:\actions-runner-uiautomator-rs-gui",

    [string]$TaskName = "uiautomator-rs-gui-runner",

    [int]$StartupWaitSeconds = 8,

    [int]$ReadyTimeoutSeconds = 180,

    [switch]$UseScheduledTask
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

. (Join-Path $PSScriptRoot "github-runner-common.ps1")

if ($StartupWaitSeconds -lt 1) {
    throw "StartupWaitSeconds must be >= 1"
}
if ($ReadyTimeoutSeconds -lt 1) {
    throw "ReadyTimeoutSeconds must be >= 1"
}

$stopped = @(Stop-GitHubRunnerProcesses -RunnerRoot $RunnerRoot -IncludeWorkers)
$startRunnerCmd = Join-Path $RunnerRoot "start-runner.cmd"

if ($UseScheduledTask) {
    $task = Get-ScheduledTask -TaskName $TaskName -ErrorAction SilentlyContinue
    if ($null -eq $task) {
        throw "scheduled task not found: $TaskName"
    }
    Start-ScheduledTask -TaskName $TaskName
}
else {
    if (-not (Test-Path $startRunnerCmd)) {
        throw "runner start command not found: $startRunnerCmd"
    }

    Start-Process -FilePath "cmd.exe" -ArgumentList @("/c", "`"$startRunnerCmd`"") -WorkingDirectory $RunnerRoot | Out-Null
}

Start-Sleep -Seconds $StartupWaitSeconds
$listener = @(Wait-GitHubRunnerReady -RunnerRoot $RunnerRoot -TimeoutSeconds $ReadyTimeoutSeconds)

[ordered]@{
    runner_root = $RunnerRoot
    task_name = $TaskName
    stopped_process_ids = @($stopped | ForEach-Object { $_.ProcessId })
    listener_process_id = if ($listener.Count -eq 1) { $listener[0].ProcessId } else { $null }
    used_scheduled_task = [bool]$UseScheduledTask
} | ConvertTo-Json -Depth 4
