param(
    [string]$RunnerRoot = "D:\actions-runner-uiautomator-rs",

    [string]$RunnerServicePattern = "actions.runner*uiautomator*",

    [string]$TaskName = "uiautomator-rs-gui-runner",

    [string]$RunnerUser = [System.Security.Principal.WindowsIdentity]::GetCurrent().Name,

    [switch]$DisableService,

    [switch]$StartTask
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

function Test-IsAdmin {
    $principal = New-Object Security.Principal.WindowsPrincipal([Security.Principal.WindowsIdentity]::GetCurrent())
    return $principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
}

function Get-RunnerService {
    Get-CimInstance Win32_Service |
        Where-Object { $_.Name -like $RunnerServicePattern } |
        Select-Object -First 1
}

$startRunnerCmd = Join-Path $RunnerRoot "start-runner.cmd"
if (-not (Test-Path $startRunnerCmd)) {
    throw "runner entry not found: $startRunnerCmd"
}

$action = New-ScheduledTaskAction -Execute "cmd.exe" -Argument "/c `"$startRunnerCmd`""
$trigger = New-ScheduledTaskTrigger -AtLogOn -User $RunnerUser
$settings = New-ScheduledTaskSettingsSet -AllowStartIfOnBatteries -ExecutionTimeLimit (New-TimeSpan -Hours 0) -MultipleInstances IgnoreNew
$principal = New-ScheduledTaskPrincipal -UserId $RunnerUser -LogonType Interactive

Register-ScheduledTask -TaskName $TaskName -Action $action -Trigger $trigger -Settings $settings -Principal $principal -Force | Out-Null

$service = Get-RunnerService
$serviceResult = "unchanged"

if ($DisableService) {
    if (-not (Test-IsAdmin)) {
        throw "DisableService requires an elevated PowerShell session"
    }

    if ($service) {
        if ($service.State -eq "Running") {
            Stop-Service -Name $service.Name -Force
        }
        Set-Service -Name $service.Name -StartupType Disabled
        $serviceResult = "disabled"
    } else {
        $serviceResult = "service-not-found"
    }
}

if ($StartTask) {
    Start-ScheduledTask -TaskName $TaskName
}

[ordered]@{
    runner_root = $RunnerRoot
    task_name = $TaskName
    runner_user = $RunnerUser
    service_result = $serviceResult
    start_task = [bool]$StartTask
    disable_service = [bool]$DisableService
} | ConvertTo-Json -Depth 4
