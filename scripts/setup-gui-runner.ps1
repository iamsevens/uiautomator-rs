param(
    [string]$Repo = "iamsevens/uiautomator-rs",

    [string]$RepoUrl = "https://github.com/iamsevens/uiautomator-rs",

    [string]$SourceRunnerRoot = "D:\actions-runner-uiautomator-rs",

    [string]$GuiRunnerRoot = "D:\actions-runner-uiautomator-rs-gui",

    [string]$TaskName = "uiautomator-rs-gui-runner",

    [string]$RunnerName = "",

    [string]$RunnerLabels = "gui",

    [switch]$Reconfigure,

    [switch]$StartTask
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

if ([string]::IsNullOrWhiteSpace($RunnerName)) {
    $RunnerName = "{0}-uiautomator-gui" -f $env:COMPUTERNAME
}

$zipPath = Join-Path $SourceRunnerRoot "actions-runner-win-x64-2.332.0.zip"
if (-not (Test-Path $zipPath)) {
    throw "runner package not found: $zipPath"
}

function Invoke-Gh {
    param([string[]]$Arguments)
    $output = & gh @Arguments 2>&1
    $exitCode = $LASTEXITCODE
    $text = ($output | ForEach-Object { $_.ToString() }) -join "`n"
    if ($exitCode -ne 0) {
        throw "gh command failed (exit=$exitCode): gh $($Arguments -join ' ')`n$text"
    }
    return $text.Trim()
}

function Expand-RunnerPackage {
    if (-not (Test-Path $GuiRunnerRoot)) {
        New-Item -ItemType Directory -Path $GuiRunnerRoot | Out-Null
    }

    $runnerDll = Join-Path $GuiRunnerRoot "bin\Runner.Listener.exe"
    if (-not (Test-Path $runnerDll)) {
        Expand-Archive -Path $zipPath -DestinationPath $GuiRunnerRoot -Force
    }
}

function Invoke-Config {
    param([string[]]$Arguments)

    $configCmd = Join-Path $GuiRunnerRoot "config.cmd"
    if (-not (Test-Path $configCmd)) {
        throw "config.cmd not found: $configCmd"
    }

    $output = & $configCmd @Arguments 2>&1
    $exitCode = $LASTEXITCODE
    $text = ($output | ForEach-Object { $_.ToString() }) -join "`n"
    if ($exitCode -ne 0) {
        throw "config.cmd failed (exit=$exitCode): $text"
    }
    return $text.Trim()
}

function Register-Task {
    $runCmd = Join-Path $GuiRunnerRoot "run.cmd"
    $action = New-ScheduledTaskAction -Execute "cmd.exe" -Argument "/c `"$runCmd`""
    $trigger = New-ScheduledTaskTrigger -AtLogOn -User ([System.Security.Principal.WindowsIdentity]::GetCurrent().Name)
    $settings = New-ScheduledTaskSettingsSet -AllowStartIfOnBatteries -ExecutionTimeLimit (New-TimeSpan -Hours 0) -MultipleInstances IgnoreNew
    $principal = New-ScheduledTaskPrincipal -UserId ([System.Security.Principal.WindowsIdentity]::GetCurrent().Name) -LogonType Interactive
    Register-ScheduledTask -TaskName $TaskName -Action $action -Trigger $trigger -Settings $settings -Principal $principal -Force | Out-Null
}

Invoke-Gh -Arguments @("auth", "status") | Out-Null
Expand-RunnerPackage

$runnerMarker = Join-Path $GuiRunnerRoot ".runner"
if ((Test-Path $runnerMarker) -and $Reconfigure) {
    $removeToken = Invoke-Gh -Arguments @("api", "-X", "POST", ("repos/{0}/actions/runners/remove-token" -f $Repo), "--jq", ".token")
    Invoke-Config -Arguments @("remove", "--token", $removeToken) | Out-Null
}

if ((-not (Test-Path $runnerMarker)) -or $Reconfigure) {
    $regToken = Invoke-Gh -Arguments @("api", "-X", "POST", ("repos/{0}/actions/runners/registration-token" -f $Repo), "--jq", ".token")
    Invoke-Config -Arguments @(
        "--unattended",
        "--url", $RepoUrl,
        "--token", $regToken,
        "--name", $RunnerName,
        "--labels", $RunnerLabels,
        "--work", "_work"
    ) | Out-Null
}

Register-Task

if ($StartTask) {
    Start-ScheduledTask -TaskName $TaskName
}

[ordered]@{
    repo = $Repo
    runner_root = $GuiRunnerRoot
    runner_name = $RunnerName
    task_name = $TaskName
    labels = $RunnerLabels
    start_task = [bool]$StartTask
    reconfigure = [bool]$Reconfigure
} | ConvertTo-Json -Depth 4
