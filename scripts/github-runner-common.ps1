Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

function Get-NormalizedRunnerPath {
    param(
        [Parameter(Mandatory = $true)]
        [string]$Path
    )

    return [System.IO.Path]::GetFullPath($Path).TrimEnd("\")
}

function Get-GitHubRunnerProcessInfo {
    param(
        [Parameter(Mandatory = $true)]
        [string]$RunnerRoot,

        [string[]]$Names = @("Runner.Listener.exe")
    )

    $normalizedRoot = Get-NormalizedRunnerPath -Path $RunnerRoot
    $normalizedPrefix = "{0}\" -f $normalizedRoot

    $processes = Get-CimInstance Win32_Process |
        Where-Object { $_.Name -in $Names }

    $matches = foreach ($process in $processes) {
        if ([string]::IsNullOrWhiteSpace($process.ExecutablePath)) {
            continue
        }

        $normalizedExecutablePath = Get-NormalizedRunnerPath -Path $process.ExecutablePath
        if (-not $normalizedExecutablePath.StartsWith($normalizedPrefix, [System.StringComparison]::OrdinalIgnoreCase)) {
            continue
        }

        [PSCustomObject]@{
            Name            = $process.Name
            ProcessId       = $process.ProcessId
            ParentProcessId = $process.ParentProcessId
            ExecutablePath  = $process.ExecutablePath
            CommandLine     = $process.CommandLine
            CreationDate    = $process.CreationDate
        }
    }

    return @($matches | Sort-Object CreationDate, ProcessId)
}

function Assert-GitHubRunnerSingleListener {
    param(
        [Parameter(Mandatory = $true)]
        [string]$RunnerRoot,

        [switch]$AllowZero
    )

    $listeners = @(Get-GitHubRunnerProcessInfo -RunnerRoot $RunnerRoot -Names @("Runner.Listener.exe"))

    if (($listeners.Count -eq 0) -and (-not $AllowZero)) {
        throw "no runner listener process found under $RunnerRoot"
    }

    if ($listeners.Count -gt 1) {
        $pidList = ($listeners | ForEach-Object { [string]$_.ProcessId }) -join ", "
        throw "multiple runner listener processes found under ${RunnerRoot}: $pidList"
    }

    return $listeners
}

function Wait-GitHubRunnerReady {
    param(
        [Parameter(Mandatory = $true)]
        [string]$RunnerRoot,

        [int]$TimeoutSeconds = 180,

        [int]$PollSeconds = 3
    )

    if ($TimeoutSeconds -lt 1) {
        throw "TimeoutSeconds must be >= 1"
    }
    if ($PollSeconds -lt 1) {
        throw "PollSeconds must be >= 1"
    }

    $deadline = (Get-Date).AddSeconds($TimeoutSeconds)
    $diagRoot = Join-Path $RunnerRoot "_diag"

    while ((Get-Date) -lt $deadline) {
        $listeners = @(Assert-GitHubRunnerSingleListener -RunnerRoot $RunnerRoot)
        $runnerLogs = Get-ChildItem -Path $diagRoot -Filter "Runner_*.log" -ErrorAction SilentlyContinue |
            Sort-Object LastWriteTime -Descending |
            Select-Object -First 3

        foreach ($runnerLog in $runnerLogs) {
            if (Select-String -Path $runnerLog.FullName -Pattern "Listening for Jobs" -SimpleMatch -Quiet -ErrorAction SilentlyContinue) {
                return $listeners
            }
        }

        Start-Sleep -Seconds $PollSeconds
    }

    throw "runner did not reach 'Listening for Jobs' state under $RunnerRoot within $TimeoutSeconds seconds"
}

function Stop-GitHubRunnerProcesses {
    param(
        [Parameter(Mandatory = $true)]
        [string]$RunnerRoot,

        [switch]$IncludeWorkers
    )

    $names = @("Runner.Listener.exe")
    if ($IncludeWorkers) {
        $names += "Runner.Worker.exe"
    }

    $targets = @(Get-GitHubRunnerProcessInfo -RunnerRoot $RunnerRoot -Names $names)
    foreach ($target in $targets) {
        Stop-Process -Id $target.ProcessId -Force -ErrorAction Stop
    }

    return $targets
}
