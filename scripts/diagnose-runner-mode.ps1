param(
    [string]$RunnerRoot = "D:\actions-runner-uiautomator-rs",

    [string]$RunnerServicePattern = "actions.runner*uiautomator*",

    [string]$TaskName = "uiautomator-rs-gui-runner"
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

function Get-RunnerService {
    Get-CimInstance Win32_Service |
        Where-Object { $_.Name -like $RunnerServicePattern } |
        Select-Object -First 1
}

function Get-ProcessTable {
    param(
        [string[]]$Names
    )

    Get-Process -Name $Names -ErrorAction SilentlyContinue |
        Select-Object ProcessName, Id, SessionId, Path |
        Sort-Object ProcessName, Id
}

$service = Get-RunnerService
$task = Get-ScheduledTask -TaskName $TaskName -ErrorAction SilentlyContinue
$listener = Get-ProcessTable -Names @("Runner.Listener", "Runner.Worker")
$emulators = Get-ProcessTable -Names @("ldconsole", "ldplayer", "MuMuManager", "MuMuPlayer", "MuMuNxMain")

$report = [ordered]@{
    runner_root = $RunnerRoot
    current_user = [System.Security.Principal.WindowsIdentity]::GetCurrent().Name
    is_admin = ([Security.Principal.WindowsPrincipal] [Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
    service = if ($service) {
        [ordered]@{
            name = $service.Name
            state = $service.State
            start_name = $service.StartName
            path_name = $service.PathName
        }
    } else {
        $null
    }
    scheduled_task = if ($task) {
        [ordered]@{
            task_name = $task.TaskName
            state = $task.State.ToString()
            task_path = $task.TaskPath
        }
    } else {
        $null
    }
    runner_processes = @($listener | ForEach-Object {
        [ordered]@{
            name = $_.ProcessName
            pid = $_.Id
            session_id = $_.SessionId
            path = $_.Path
        }
    })
    emulator_processes = @($emulators | ForEach-Object {
        [ordered]@{
            name = $_.ProcessName
            pid = $_.Id
            session_id = $_.SessionId
            path = $_.Path
        }
    })
}

$report | ConvertTo-Json -Depth 6
