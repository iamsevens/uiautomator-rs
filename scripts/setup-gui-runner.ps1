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

function Write-RunnerBootstrap {
    $startRunnerPs1 = Join-Path $GuiRunnerRoot "start-runner.ps1"
    $startRunnerCmd = Join-Path $GuiRunnerRoot "start-runner.cmd"

    $psContent = @'
param()

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$runnerRoot = Split-Path -Parent $MyInvocation.MyCommand.Path
$listenerPath = Join-Path $runnerRoot "bin\Runner.Listener.exe"
$runCmd = Join-Path $runnerRoot "run.cmd"
$envFile = Join-Path $runnerRoot ".env"

function Get-NormalizedPath {
    param([string]$Path)
    return [System.IO.Path]::GetFullPath($Path).TrimEnd("\")
}

function Get-ListenerProcess {
    param([string]$ExecutablePath)

    $normalizedExecutablePath = Get-NormalizedPath -Path $ExecutablePath
    Get-CimInstance Win32_Process |
        Where-Object { $_.Name -eq "Runner.Listener.exe" -and $_.ExecutablePath } |
        Where-Object { (Get-NormalizedPath -Path $_.ExecutablePath) -ieq $normalizedExecutablePath } |
        Sort-Object CreationDate, ProcessId
}

function Import-EnvFile {
    param([string]$Path)

    if (-not (Test-Path $Path)) {
        return
    }

    foreach ($line in Get-Content -Path $Path -Encoding utf8) {
        $trimmed = $line.Trim()
        if ([string]::IsNullOrWhiteSpace($trimmed) -or $trimmed.StartsWith("#")) {
            continue
        }

        $separatorIndex = $trimmed.IndexOf("=")
        if ($separatorIndex -lt 1) {
            continue
        }

        $name = $trimmed.Substring(0, $separatorIndex).Trim()
        $value = $trimmed.Substring($separatorIndex + 1)
        if ([string]::IsNullOrWhiteSpace($name)) {
            continue
        }

        Set-Item -Path ("Env:{0}" -f $name) -Value $value
    }
}

$listeners = @(Get-ListenerProcess -ExecutablePath $listenerPath)
if ($listeners.Count -gt 0) {
    $pidList = ($listeners | ForEach-Object { [string]$_.ProcessId }) -join ", "
    Write-Host "runner already active under $runnerRoot; refusing duplicate start (pid=$pidList)"
    exit 0
}

Import-EnvFile -Path $envFile
Set-Location $runnerRoot
& $runCmd
exit $LASTEXITCODE
'@

    $cmdContent = @'
@echo off
setlocal
set "PS_EXE=C:\Windows\System32\WindowsPowerShell\v1.0\powershell.exe"
"%PS_EXE%" -NoProfile -ExecutionPolicy Bypass -File "%~dp0start-runner.ps1" %*
exit /b %ERRORLEVEL%
'@

    Set-Content -Path $startRunnerPs1 -Value $psContent -Encoding utf8
    Set-Content -Path $startRunnerCmd -Value $cmdContent -Encoding ascii
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
    $runCmd = Join-Path $GuiRunnerRoot "start-runner.cmd"
    $action = New-ScheduledTaskAction -Execute "cmd.exe" -Argument "/c `"$runCmd`""
    $trigger = New-ScheduledTaskTrigger -AtLogOn -User ([System.Security.Principal.WindowsIdentity]::GetCurrent().Name)
    $settings = New-ScheduledTaskSettingsSet -AllowStartIfOnBatteries -ExecutionTimeLimit (New-TimeSpan -Hours 0) -MultipleInstances IgnoreNew
    $principal = New-ScheduledTaskPrincipal -UserId ([System.Security.Principal.WindowsIdentity]::GetCurrent().Name) -LogonType Interactive
    Register-ScheduledTask -TaskName $TaskName -Action $action -Trigger $trigger -Settings $settings -Principal $principal -Force | Out-Null
}

function Set-GitTransportDefaults {
    & git config --global http.version HTTP/1.1
    & git config --global http.postBuffer 524288000
    & git config --global core.compression 0
    & git config --global http.lowSpeedLimit 0
    & git config --global http.lowSpeedTime 999999
}

Invoke-Gh -Arguments @("auth", "status") | Out-Null
Expand-RunnerPackage
Write-RunnerBootstrap

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

Set-GitTransportDefaults
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
